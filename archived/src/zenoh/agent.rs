//! Zenoh session, topology checks, bridge spawn, and router/peer watches (`zenoh_agent` binary).

use std::path::Path;

use crate::{ResolvedEdgeAgent, ZenohTopology};
use super::{bridge, cloud};

pub async fn run() -> anyhow::Result<()> {
    // Load config ===
    println!(
        "Edge agent YAML ({})…",
        ResolvedEdgeAgent::default_yaml_path().display()
    );
    let edge = ResolvedEdgeAgent::load_default()?;
    // ======

    // Validate topology ===
    if edge.zenoh_topology == ZenohTopology::WanClient
        && edge.bridge.mode == bridge::BridgeMode::LocalPeer
        && !edge.bridge.skip_spawn
    {
        anyhow::bail!(
            "Zenoh `wan_client` has no `:7411` listener, but **`bridge`** would spawn **`local_peer`** (dials **`127.0.0.1:7411`**).\n\
             → Set **`bridge.skip_spawn: true`** and run **`zenoh-bridge-ros2dds`** on the talker machine, **or**\n\
             → Set **`bridge.mode: client`** to spawn the bridge toward **`zenohd`** from *this* host."
        );
    }
    // ======

    // Open Zenoh session ===
    let cfg_path = Path::new(&edge.zenoh_config_path);
    let config = zenoh::Config::from_file(cfg_path).map_err(|e| {
        anyhow::anyhow!(
            "load Zenoh config {}: {e} — fix **`zenoh.topology`** / **`zenoh.config_file`** (**`configs/edge_agent.yaml`**) or JSON5 contents",
            cfg_path.display()
        )
    })?;

    println!("Opening session ({})…", cfg_path.display());
    let session = zenoh::open(config).await.map_err(|e| {
        anyhow::anyhow!(
            "zenoh open failed ({e}); is `zenohd` reachable (`docker compose up -d` in dev)?"
        )
    })?;

    println!("Zenoh session up. Agent ZID: {}", session.zid());
    // ======

    // Start bridge and watches ===
    cloud::spawn_router_watch(session.clone());

    let (_bridge_kill, bridge_mode) = bridge::spawn(&edge.bridge)?;
    if bridge_mode == bridge::BridgeMode::LocalPeer
        && !edge.bridge.skip_spawn
        && edge.zenoh_topology == ZenohTopology::Peer
    {
        bridge::spawn_peer_watch(session.clone());
    }
    // ======

    // Run until Ctrl+C ===
    println!(
        "Session alive; press Ctrl+C to stop (closes Zenoh; bridge subprocess is killed unless configs/edge_agent.yaml bridge.detach is true)."
    );
    tokio::signal::ctrl_c().await?;

    println!("Shutting down…");
    // ======
    Ok(())
}
