//! Call a ROS 2 service over Zenoh (`cargo run --bin main_srv_client`).
//!
//! Config: `configs/master_srv-client.yaml` or `MASTER_SRV_CLIENT_YAML`.
//! Overrides: `MAIN_SRV_ROUTER`, `MAIN_SRV_ZENOH_JSON5`, `MAIN_SRV_KEYEXPR`,
//! `MAIN_SRV_TYPE`, `MAIN_SRV_TIMEOUT_MS`, `MAIN_SRV_SEQ`, `MAIN_SRV_CLIENT_ID`,
//! `MAIN_SRV_A`, `MAIN_SRV_B`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use serde_yaml::{Mapping, Value};
use zenoh::Wait;
use zenoh::bytes::{Encoding, ZBytes};
use topology_proximity_comm::{
    config::configs_dir,
    ResolvedSrvCall,
    wire::ros_srv_cdr::{cyclone_request_attachment, ros2_service_reply_summary, ros2_service_request_cdr},
    zenoh::sub_cli::{apply_router_connect_override, zenoh_config_for_subscriber_from_file},
};

fn resolve_zenoh_json5_path(cli_json5_hint: Option<&str>) -> PathBuf {
    let cfgs = configs_dir();
    let root = cfgs.parent().expect("configs dir").to_path_buf();
    match std::env::var("MAIN_SRV_ZENOH_JSON5")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(p) => {
            let path = Path::new(&p);
            if path.is_absolute() {
                path.to_path_buf()
            } else if path
                .components()
                .filter(|c| matches!(c, std::path::Component::Normal(_)))
                .count()
                <= 1
            {
                cfgs.join(path)
            } else {
                root.join(path)
            }
        }
        None => match cli_json5_hint.map(str::trim).filter(|s| !s.is_empty()) {
            Some(bn) => {
                let path = Path::new(bn);
                if path.is_absolute() {
                    path.to_path_buf()
                } else if path.components().filter(|c| matches!(c, std::path::Component::Normal(_))).count()
                    <= 1
                {
                    cfgs.join(path)
                } else {
                    root.join(path)
                }
            }
            None => cfgs.join("zenoh_agent-as-client.json5"),
        },
    }
}

fn parse_u64(primary: Option<u64>, env_name: &str, fallback: u64) -> u64 {
    std::env::var(env_name)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .or(primary)
        .unwrap_or(fallback)
}

fn merge_args_with_srv_env(mut args: Value) -> anyhow::Result<Value> {
    if matches!(args, Value::Null) {
        args = Value::Mapping(Mapping::new());
    }
    let Some(m) = args.as_mapping_mut() else {
        anyhow::bail!(
            "ros2_service_call.args must be a YAML mapping (like `{{a: 2}}`), got {:?}",
            args
        )
    };

    fn env_i64(name: &str) -> Option<i64> {
        std::env::var(name)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| s.trim().parse().ok())
    }

    if let Some(a) = env_i64("MAIN_SRV_A") {
        m.insert(
            Value::String("a".into()),
            Value::Number(serde_yaml::Number::from(a)),
        );
    }
    if let Some(b) = env_i64("MAIN_SRV_B") {
        m.insert(
            Value::String("b".into()),
            Value::Number(serde_yaml::Number::from(b)),
        );
    }

    Ok(args)
}

fn parse_client_id_hex_or_decimal(raw: Option<String>) -> u64 {
    let Some(s0) = raw else {
        return 0xd055_e727_6e40_0301_u64;
    };
    let t = s0.trim();
    if t.is_empty() {
        return 0xd055_e727_6e40_0301_u64;
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).unwrap_or(0xd055_e727_6e40_0301_u64);
    }
    t.parse().unwrap_or(0xd055_e727_6e40_0301_u64)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config ===
    let master_yaml_path = ResolvedSrvCall::master_srv_yaml_path();
    let srv = ResolvedSrvCall::load_master_default().with_context(|| {
        format!(
            "load `{}` — set MASTER_SRV_CLIENT_YAML or fix YAML",
            master_yaml_path.display()
        )
    })?;

    let zenoh_cfg = resolve_zenoh_json5_path(srv.zenoh_json5.as_deref());
    if !zenoh_cfg.is_file() {
        anyhow::bail!(
            "main_srv_client: Zenoh JSON5 not found at {} — set MAIN_SRV_ZENOH_JSON5 or srv_client.zenoh_json5",
            zenoh_cfg.display(),
        );
    }
    // ======

    // Resolve keys and args ===
    let service_type = std::env::var("MAIN_SRV_TYPE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| srv.service_type.clone());

    let ke = match std::env::var("MAIN_SRV_KEYEXPR")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
    {
        Some(k) => k,
        None => srv.zenoh_keyexpr_resolved()?,
    };

    if ke.is_empty() {
        anyhow::bail!("resolved Zenoh keyexpr is empty");
    }

    let args = merge_args_with_srv_env(srv.args.clone())?;

    let seq = parse_u64(srv.seq, "MAIN_SRV_SEQ", 1);

    let client_id = parse_client_id_hex_or_decimal(
        std::env::var("MAIN_SRV_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| srv.client_id.clone()),
    );

    let timeout_ms: u64 = std::env::var("MAIN_SRV_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .or(srv.timeout_ms)
        .unwrap_or(10_000)
        .max(100);
    // ======

    // Open Zenoh session ===
    let mut config = zenoh_config_for_subscriber_from_file(&zenoh_cfg)?;

    if let Ok(s) = std::env::var("MAIN_SRV_ROUTER") {
        let t = s.trim();
        if !t.is_empty() {
            apply_router_connect_override(&mut config, t)?;
            println!("(srv) connect → `{t}` (MAIN_SRV_ROUTER)");
        }
    } else if let Some(ref r) = srv.router {
        let t = r.trim();
        if !t.is_empty() {
            apply_router_connect_override(&mut config, t)?;
            println!("(srv) connect → `{t}` (YAML srv_client.router)");
        }
    }

    let session = zenoh::open(config)
        .await
        .map_err(|e| anyhow::anyhow!("zenoh open: {e}"))?;

    let n_r = session.info().routers_zid().wait().count();
    // ======

    // Send query ===
    let req = ros2_service_request_cdr(&service_type, &args).with_context(|| {
        format!(
            "encode CDR request for `{}` (check ros2_service_call.args)",
            master_yaml_path.display()
        )
    })?;

    println!(
        "(srv) master `{}`\n\
         (srv) ros2_service_call: name=`{}` type=`{}` zenoh=`{}`\n\
         (srv) ZID {} routers={} seq={} client_id=0x{:x} timeout_ms={}",
        master_yaml_path.display(),
        srv.ros_service_name.trim(),
        service_type,
        ke,
        session.zid(),
        n_r,
        seq,
        client_id,
        timeout_ms,
    );

    let attachment = cyclone_request_attachment(client_id, seq, true);

    let replies = session
        .get(&ke)
        .payload(ZBytes::from(req.clone()))
        .attachment(attachment)
        .encoding(Encoding::ZENOH_BYTES)
        .timeout(Duration::from_millis(timeout_ms))
        .await
        .map_err(|e| anyhow::anyhow!("session.get establish: {e}"))?;
    // ======

    // Collect replies ===
    let mut last_ok_blob: Vec<u8> = Vec::new();
    let mut decode_fail: Option<anyhow::Error> = None;
    let mut reply_errors = Vec::<String>::new();

    while let Ok(reply) = replies.recv_async().await {
        match reply.into_result() {
            Ok(sample) => {
                let blob = sample.payload().to_bytes().into_owned();
                match ros2_service_reply_summary(&service_type, &blob) {
                    Ok(_) => last_ok_blob = blob,
                    Err(e) => {
                        decode_fail.get_or_insert(e);
                    }
                }
            }
            Err(err) => reply_errors.push(format!("{err:?}")),
        }
    }

    println!(
        "[srv] request_cdr(hex)={}",
        req.iter().map(|x| format!("{x:02x}")).collect::<String>()
    );

    if let Some(last_err) = reply_errors.last() {
        println!(
            "(srv) note: saw {} ReplyError replies; last={last_err:?}",
            reply_errors.len(),
        );
    }

    if !last_ok_blob.is_empty() {
        let summary = ros2_service_reply_summary(&service_type, &last_ok_blob)
            .context("decode service reply payload")?;
        println!("[srv] {summary}");
        Ok(())
    } else if let Some(err) = decode_fail {
        Err(err.context("decode Zenoh reply payload (wrong service type?)"))
    } else if !reply_errors.is_empty() {
        anyhow::bail!(
            "only ReplyError payloads ({}) — check bridge / routing / DDS domain",
            reply_errors.len()
        )
    } else {
        anyhow::bail!(
            "no replies — timeout, routing, or bridge not serving Zenoh `{}`?",
            ke
        )
    }
    // ======
}
