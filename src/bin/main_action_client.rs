//! Send a ROS 2 action goal over Zenoh (`cargo run --bin main_action_client`).
//!
//! Generic: Zenoh session, action keys, send_goal / get_result queries, feedback subscribe.
//! Demo default (`configs/master_action-client.yaml`): `/turtle1/rotate_absolute`
//! (`turtlesim/action/RotateAbsolute`). CDR encode/decode for that action only today.
//!
//! Config: `MASTER_ACTION_CLIENT_YAML`, `MAIN_ACTION_*`, `MAIN_ACTION_FEEDBACK` (`0` disables).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use serde_yaml::{Mapping, Value};
use zenoh::Wait;
use zenoh::bytes::{Encoding, ZBytes};
use topology_proximity_comm::{
    ResolvedActionCall,
    wire::{
        format_action_tag,
        ros_action_cdr::{
            action_get_result_request_cdr, parse_send_goal_response_cdr, ros2_action_feedback_summary,
            ros2_action_result_summary, ros2_action_send_goal_cdr, zenoh_action_service_key,
        },
        ros_srv_cdr::cyclone_request_attachment,
        ActionLog,
    },
    zenoh::sub_cli::{apply_router_connect_override, zenoh_config_for_subscriber_from_file},
};

fn resolve_zenoh_json5_path(cli_json5_hint: Option<&str>) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cfgs = root.join("configs");
    match std::env::var("MAIN_ACTION_ZENOH_JSON5")
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

fn parse_goal_id_from_env() -> anyhow::Result<Option<[u8; 16]>> {
    let Some(raw) = std::env::var("MAIN_ACTION_GOAL_ID")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };
    let hex = raw.strip_prefix("0x").unwrap_or(raw.as_str());
    if hex.len() != 32 {
        anyhow::bail!("MAIN_ACTION_GOAL_ID must be 32 hex chars (16 bytes), got `{raw}`");
    }
    let mut id = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        if i >= 16 {
            break;
        }
        let s = std::str::from_utf8(chunk).context("MAIN_ACTION_GOAL_ID hex")?;
        id[i] = u8::from_str_radix(s, 16).context("MAIN_ACTION_GOAL_ID hex")?;
    }
    Ok(Some(id))
}

fn new_goal_id(client_id: u64) -> [u8; 16] {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut id = [0u8; 16];
    id[..8].copy_from_slice(&nanos.to_le_bytes()[..8]);
    id[8..].copy_from_slice(&client_id.to_le_bytes());
    id
}

// turtlesim /turtle1/rotate_absolute (RotateAbsolute goal field) ===
fn merge_goal_with_env(mut goal: Value) -> anyhow::Result<Value> {
    if matches!(goal, Value::Null) {
        goal = Value::Mapping(Mapping::new());
    }
    let Some(m) = goal.as_mapping_mut() else {
        anyhow::bail!("action goal must be a YAML mapping, got {goal:?}");
    };
    if let Ok(s) = std::env::var("MAIN_ACTION_THETA") {
        if !s.trim().is_empty() {
            let v: f64 = s.trim().parse().context("MAIN_ACTION_THETA")?;
            m.insert(Value::from("theta"), Value::from(v));
        }
    }
    Ok(goal)
}
// ======

async fn zenoh_query_payload(
    session: &zenoh::Session,
    phase: ActionLog,
    keyexpr: &str,
    payload: Vec<u8>,
    attachment: ZBytes,
    timeout: Duration,
) -> anyhow::Result<Vec<u8>> {
    let tag = format_action_tag(phase, true);
    println!("{tag} query `{keyexpr}` …");

    let replies = session
        .get(keyexpr)
        .payload(ZBytes::from(payload))
        .attachment(attachment)
        .encoding(Encoding::ZENOH_BYTES)
        .timeout(timeout)
        .await
        .map_err(|e| anyhow::anyhow!("session.get `{keyexpr}`: {e}"))?;

    let mut last_ok = Vec::new();
    let mut reply_errors = Vec::new();
    while let Ok(reply) = replies.recv_async().await {
        match reply.into_result() {
            Ok(sample) => last_ok = sample.payload().to_bytes().into_owned(),
            Err(err) => reply_errors.push(format!("{err:?}")),
        }
    }
    if !last_ok.is_empty() {
        return Ok(last_ok);
    }
    if let Some(err) = reply_errors.last() {
        anyhow::bail!("ReplyError on `{keyexpr}`: {err}");
    }
    anyhow::bail!("no reply on `{keyexpr}` within timeout")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config ===
    let master_yaml_path = ResolvedActionCall::master_action_yaml_path();
    let action = ResolvedActionCall::load_master_default().with_context(|| {
        format!(
            "load `{}` — set MASTER_ACTION_CLIENT_YAML or fix YAML",
            master_yaml_path.display()
        )
    })?;

    let zenoh_cfg = resolve_zenoh_json5_path(action.zenoh_json5.as_deref());
    if !zenoh_cfg.is_file() {
        anyhow::bail!(
            "main_action_client: Zenoh JSON5 not found at {} — set MAIN_ACTION_ZENOH_JSON5",
            zenoh_cfg.display(),
        );
    }
    // ======

    // Resolve keys and goal ===
    let action_type = std::env::var("MAIN_ACTION_TYPE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| action.action_type.clone());

    let action_base_key = match std::env::var("MAIN_ACTION_KEYEXPR")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
    {
        Some(k) => k,
        None => action.zenoh_keyexpr_resolved()?,
    };
    if action_base_key.is_empty() {
        anyhow::bail!("resolved Zenoh action key is empty");
    }

    // turtlesim RotateAbsolute: merge `theta` from YAML / MAIN_ACTION_THETA ===
    let goal = merge_goal_with_env(action.goal.clone())?;
    // ======
    let seq = parse_u64(action.seq, "MAIN_ACTION_SEQ", 1);
    let client_id = parse_client_id_hex_or_decimal(
        std::env::var("MAIN_ACTION_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| action.client_id.clone()),
    );
    let goal_id = parse_goal_id_from_env()?.unwrap_or_else(|| new_goal_id(client_id));

    let send_goal_timeout_ms = std::env::var("MAIN_ACTION_SEND_GOAL_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .or(action.send_goal_timeout_ms)
        .unwrap_or(10_000)
        .max(100);
    let get_result_timeout_ms = std::env::var("MAIN_ACTION_GET_RESULT_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .or(action.get_result_timeout_ms)
        .unwrap_or(60_000)
        .max(100);

    let send_goal_key = zenoh_action_service_key(&action_base_key, "_action/send_goal");
    let get_result_key = zenoh_action_service_key(&action_base_key, "_action/get_result");
    let feedback_key = zenoh_action_service_key(&action_base_key, "_action/feedback");

    let feedback_enabled = !matches!(
        std::env::var("MAIN_ACTION_FEEDBACK").ok().as_deref(),
        Some("0") | Some("false") | Some("False") | Some("FALSE")
    );
    // ======

    // Open Zenoh session ===
    let mut config = zenoh_config_for_subscriber_from_file(&zenoh_cfg)?;
    if let Ok(s) = std::env::var("MAIN_ACTION_ROUTER") {
        let t = s.trim();
        if !t.is_empty() {
            apply_router_connect_override(&mut config, t)?;
            println!("(action) connect → `{t}` (MAIN_ACTION_ROUTER)");
        }
    } else if let Some(ref r) = action.router {
        let t = r.trim();
        if !t.is_empty() {
            apply_router_connect_override(&mut config, t)?;
            println!("(action) connect → `{t}` (YAML action_client.router)");
        }
    }

    let session = zenoh::open(config)
        .await
        .map_err(|e| anyhow::anyhow!("zenoh open: {e}"))?;

    let n_r = session.info().routers_zid().wait().count();

    println!(
        "(action) master `{}`\n\
         (action) name=`{}` type=`{}`\n\
         (action) goal=`{}` result=`{}` feedback=`{}`\n\
         (action) ZID {} routers={} seq={} client_id=0x{:x} goal_id={}",
        master_yaml_path.display(),
        action.ros_action_name.trim(),
        action_type,
        send_goal_key,
        get_result_key,
        feedback_key,
        session.zid(),
        n_r,
        seq,
        client_id,
        goal_id.iter().map(|b| format!("{b:02x}")).collect::<String>(),
    );
    // ======

    // Send goal ===
    // turtlesim/action/RotateAbsolute CDR (goal_id + theta) ===
    let send_goal_req = ros2_action_send_goal_cdr(&action_type, goal_id, &goal).with_context(|| {
        format!(
            "encode send_goal for `{action_type}` (check goal in {})",
            master_yaml_path.display()
        )
    })?;
    // ======

    let send_attachment = cyclone_request_attachment(client_id, seq, true);
    let send_reply = zenoh_query_payload(
        &session,
        ActionLog::Goal,
        &send_goal_key,
        send_goal_req,
        send_attachment,
        Duration::from_millis(send_goal_timeout_ms),
    )
    .await
    .context("send_goal query")?;

    let send_goal_resp =
        parse_send_goal_response_cdr(&send_reply).context("decode send_goal reply")?;
    println!(
        "{} accepted={} stamp={}.{}",
        format_action_tag(ActionLog::Goal, true),
        send_goal_resp.accepted,
        send_goal_resp.stamp_sec,
        send_goal_resp.stamp_nanosec
    );
    if !send_goal_resp.accepted {
        anyhow::bail!("action server rejected the goal");
    }
    // ======

    // Subscribe feedback ===
    let action_type_feedback = action_type.clone();
    if feedback_enabled {
        match session
            .declare_subscriber(&feedback_key)
            .callback(move |sample| {
                let key = sample.key_expr().to_string();
                let blob = sample.payload().to_bytes().into_owned();
                // turtlesim/action/RotateAbsolute: decode `remaining` rad ===
                match ros2_action_feedback_summary(&action_type_feedback, &blob) {
                    Ok(line) => println!(
                        "{} `{key}` {line}",
                        format_action_tag(ActionLog::Feedback, true)
                    ),
                    Err(e) => eprintln!(
                        "{} `{key}` skip: {e}",
                        format_action_tag(ActionLog::Feedback, false)
                    ),
                }
            })
            .background()
            .await
        {
            Ok(()) => println!(
                "{} listening on `{feedback_key}`",
                format_action_tag(ActionLog::Feedback, true)
            ),
            Err(e) => eprintln!(
                "{} subscribe `{feedback_key}` skipped: {e}",
                format_action_tag(ActionLog::Feedback, false)
            ),
        }
    }
    // ======

    // Get result ===
    let get_result_req = action_get_result_request_cdr(goal_id);
    let get_attachment = cyclone_request_attachment(client_id, seq + 1, true);
    let result_reply = zenoh_query_payload(
        &session,
        ActionLog::Result,
        &get_result_key,
        get_result_req,
        get_attachment,
        Duration::from_millis(get_result_timeout_ms),
    )
    .await
    .context("get_result query")?;

    // turtlesim/action/RotateAbsolute: decode status + delta ===
    let summary = ros2_action_result_summary(&action_type, &result_reply)
        .context("decode get_result reply")?;
    // ======
    println!("{} {summary}", format_action_tag(ActionLog::Result, true));
    // ======
    Ok(())
}
