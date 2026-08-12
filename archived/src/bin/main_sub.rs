//! Zenoh subscriber (`cargo run --bin main_sub`). No agent or bridge spawn.
//!
//! Config: `configs/edge_agent-sub.yaml` or `EDGE_AGENT_SUB_YAML`.
//! Router override: `MAIN_SUB_ROUTER=tcp/host:7447`.
//! Discover: set `subscriber.discover: true` in YAML.
//! Stale timeout: `MAIN_SUB_TOPIC_STALE_SEC` (default 30, 0 disables).

use std::path::{Path, PathBuf};
use std::time::Duration;

use topology_proximity_comm::zenoh::sub_cli;
use topology_proximity_comm::zenoh::sub_cli::WaitPolicy;
use topology_proximity_comm::ResolvedEdgeAgent;

fn resolve_client_zenoh_json5() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cfgs = root.join("configs");
    match std::env::var("MAIN_SUB_ZENOH_JSON5")
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
        None => cfgs.join("zenoh_agent-as-client.json5"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config ===
    let edge_yaml = ResolvedEdgeAgent::sub_default_yaml_path();
    let edge = ResolvedEdgeAgent::load_sub_default()?;
    let zenoh_cfg = resolve_client_zenoh_json5();
    if !zenoh_cfg.is_file() {
        anyhow::bail!(
            "main_sub: Zenoh JSON5 not found at {} — set MAIN_SUB_ZENOH_JSON5",
            zenoh_cfg.display(),
        );
    }
    // ======

    // Discover stale timeout ===
    let topic_stale_after = if edge.subscriber_discover {
        let sec: u64 = std::env::var("MAIN_SUB_TOPIC_STALE_SEC")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        if sec == 0 {
            None
        } else {
            Some(Duration::from_secs(sec))
        }
    } else {
        None
    };

    println!(
        "(main_sub) edge `{}` zenoh `{}` discover={} keyexpr=`{}` stale_after={:?}",
        edge_yaml.display(),
        zenoh_cfg.display(),
        edge.subscriber_discover,
        edge.subscriber_keyexpr,
        topic_stale_after,
    );
    // ======

    // Build args ===
    let mut args = sub_cli::subscriber_args(&edge, WaitPolicy::UntilCtrlC);
    args.config_path = zenoh_cfg;
    args.topic_stale_after = topic_stale_after;
    if let Ok(s) = std::env::var("MAIN_SUB_ROUTER") {
        let t = s.trim();
        if !t.is_empty() {
            args.router_connect_override = Some(t.to_string());
        }
    }
    // ======

    // Subscribe ===
    sub_cli::run(args).await
    // ======
}
