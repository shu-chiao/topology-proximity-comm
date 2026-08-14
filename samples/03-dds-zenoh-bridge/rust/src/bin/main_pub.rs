//! Zenoh publisher (`cargo run --bin main_pub`).
//!
//! Config: `EDGE_AGENT_PUB_YAML`, `MAIN_PUB_*`, `ROS_MSG_TYPE`.
use std::path::{Path, PathBuf};

use topology_proximity_comm::config::configs_dir;
use topology_proximity_comm::ResolvedEdgePub;
use topology_proximity_comm::zenoh::pub_cli::{self, PublisherCliArgs};

fn resolve_client_zenoh_json5() -> PathBuf {
    let cfgs = configs_dir();
    let root = cfgs.parent().expect("configs dir").to_path_buf();
    match std::env::var("MAIN_PUB_ZENOH_JSON5")
        .ok()
        .filter(|s| !s.is_empty())
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
        None => cfgs.join("zenoh_client-as-docker.json5"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config ===
    let pub_edge = ResolvedEdgePub::load_pub_default()?;
    let zenoh_cfg = resolve_client_zenoh_json5();
    if !zenoh_cfg.is_file() {
        anyhow::bail!(
            "main_pub: Zenoh JSON5 not found at {} — set MAIN_PUB_ZENOH_JSON5",
            zenoh_cfg.display(),
        );
    }
    // ======

    let mut args: PublisherCliArgs = pub_cli::publisher_cli_args(&pub_edge);
    args.config_path = zenoh_cfg;

    // Env overrides ===
    if let Ok(s) = std::env::var("MAIN_PUB_KEYEXPR") {
        let t = s.trim();
        if !t.is_empty() {
            args.keyexpr = t.to_string();
        }
    }
    if let Ok(s) = std::env::var("MAIN_PUB_PAYLOAD") {
        args.payload = s;
    }
    if let Ok(ms) = std::env::var("MAIN_PUB_PERIOD_MS") {
        if let Ok(ms) = ms.trim().parse::<u64>() {
            if ms > 0 {
                args.period = std::time::Duration::from_millis(ms);
            }
        }
    }
    if let Ok(s) = std::env::var("MAIN_PUB_ROUTER") {
        let t = s.trim();
        if !t.is_empty() {
            args.router_connect_override = Some(t.to_string());
        }
    }
    if let Ok(s) = std::env::var("ROS_MSG_TYPE") {
        let t = s.trim();
        if !t.is_empty() {
            args.ros_msg_type = t.to_string();
        }
    }

    println!(
        "(main_pub) key=`{}` period={:?}",
        args.keyexpr.trim(),
        args.period,
    );
    // ======

    // Publish ===
    pub_cli::run(args).await
    // ======
}
