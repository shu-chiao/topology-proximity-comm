//! Spawn `zenoh-bridge-ros2dds` and watch local bridge peers on :7411.
//! See `docs/ros2-bridge.md` for install and config.

use crate::wire::{Watch, format_bridge_mode_tag, format_tag};
use serde::Deserialize;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PEER_WATCH_INTERVAL: Duration = Duration::from_secs(1);
const PEER_WATCH_LOG_COOLDOWN: Duration = Duration::from_secs(25);
/// Grace period before logging "no peer" after spawn.
const PEER_WATCH_STARTUP_GRACE: Duration = Duration::from_secs(5);

/// How the bridge connects to Zenoh (`bridge.mode` in edge_agent.yaml).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeMode {
    /// Connect to the local agent on tcp/127.0.0.1:7411.
    LocalPeer,
    /// Connect directly to zenohd.
    Client,
}

impl BridgeMode {
    fn config_basename(self) -> &'static str {
        match self {
            Self::LocalPeer => "zenoh_bridge-as-local-peer.json5",
            Self::Client => "zenoh_bridge-as-pub-client.json5",
        }
    }

    fn tag_label(self) -> &'static str {
        match self {
            Self::LocalPeer => "(Local peer)",
            Self::Client => "(Client)",
        }
    }
}

/// Bridge spawn options from `configs/edge_agent.yaml`.
#[derive(Debug, Clone)]
pub struct BridgeSpawnOptions {
    pub mode: BridgeMode,
    pub skip_spawn: bool,
    pub verbose_logs: bool,
    pub detach: bool,
    pub executable: String,
    pub log_host_dds_env: bool,
}

fn config_path_for_mode(mode: BridgeMode, configs: &PathBuf) -> PathBuf {
    configs.join(mode.config_basename())
}

/// ROS/DDS env vars logged at startup (for ros2 nodes on the host).
const HOST_DDS_ENV_KEYS: &[&str] = &[
    "ROS_DISTRO",
    "RMW_IMPLEMENTATION",
    "ROS_DOMAIN_ID",
    "ROS_AUTOMATIC_DISCOVERY_RANGE",
    "ROS_LOCALHOST_ONLY",
];

fn format_env_truncated(s: String) -> String {
    const MAX: usize = 140;
    if s.len() <= MAX {
        return s;
    }
    let n = s.len() - MAX;
    format!("{}… (+{n} bytes)", s.get(..MAX).unwrap_or(s.as_str()),)
}

fn log_host_ros_dds_context(log: bool) {
    if !log {
        return;
    }
    println!(
        "{} ros2 host env: (unset) = middleware default; bridge ignores RMW_IMPLEMENTATION (embedded Cyclone)",
        format_bridge_mode_tag("(DDS)", true)
    );
    for key in HOST_DDS_ENV_KEYS {
        let val = match std::env::var(key) {
            Ok(v) => format_env_truncated(v.replace('\n', " ")),
            Err(std::env::VarError::NotPresent) => "(unset)".into(),
            Err(_) => "(invalid UTF-8)".into(),
        };
        println!("    {key}={val}");
    }
}

/// Kills the bridge child process when dropped.
pub struct BridgeKillOnDrop(Option<std::process::Child>);

impl Drop for BridgeKillOnDrop {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let pid = child.id();
            println!("Stopping zenoh-bridge-ros2dds (PID {pid}) …");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Start the bridge subprocess unless `skip_spawn` is set.
/// Returns `(None, mode)` when skipped or detached.
pub fn spawn(opts: &BridgeSpawnOptions) -> anyhow::Result<(Option<BridgeKillOnDrop>, BridgeMode)> {
    let mode = opts.mode;

    log_host_ros_dds_context(opts.log_host_dds_env);

    if opts.skip_spawn {
        println!(
            "{} Zenoh bridge subprocess skipped (`configs/edge_agent.yaml` **`bridge.skip_spawn: true`**).",
            format_bridge_mode_tag(mode.tag_label(), true)
        );
        return Ok((None, mode));
    }

    let configs = crate::config::configs_dir();
    let exe = &opts.executable;
    let cfg_path = config_path_for_mode(mode, &configs);

    if !cfg_path.is_file() {
        anyhow::bail!(
            "bridge JSON5 for {:?} missing: {}",
            mode,
            cfg_path.display()
        );
    }

    println!(
        "{}Spawning: `{} --no-multicast-scouting -c \"{}\"` …",
        format_bridge_mode_tag(mode.tag_label(), true),
        exe,
        cfg_path.display()
    );

    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--no-multicast-scouting")
        .args(["-c", cfg_path.to_str().unwrap_or("-")]);

    // Clear ROS_LOCALHOST_ONLY so bridge JSON5 controls DDS discovery.
    cmd.env_remove("ROS_LOCALHOST_ONLY");

    if !opts.verbose_logs {
        cmd.env("RUST_LOG", "error");
    }

    if opts.detach {
        let mut child = cmd.spawn().map_err(|e| {
            anyhow::anyhow!(
                "failed to spawn {exe:?} ({e}); install **`zenoh-bridge-ros2dds`** or set **`bridge.executable`** in **`configs/edge_agent.yaml`**",
            )
        })?;
        let pid = child.id();
        println!(
            "zenoh-bridge-ros2dds detached PID {pid} (`configs/edge_agent.yaml` **`bridge.detach: true`**)."
        );
        std::thread::spawn(move || {
            if let Err(e) = child.wait() {
                eprintln!("zenoh-bridge-ros2dds wait: {e}");
            }
        });
        Ok((None, mode))
    } else {
        let child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn zenoh-bridge-ros2dds: {e}"))?;
        let pid = child.id();
        println!("zenoh-bridge-ros2dds started PID {pid} (killed automatically when agent exits)");
        Ok((Some(BridgeKillOnDrop(Some(child))), mode))
    }
}

/// Log when local peers connect or disconnect on :7411.
pub fn spawn_peer_watch(session: zenoh::Session) {
    tokio::spawn(async move {
        let watch_started = Instant::now();
        let mut interval = tokio::time::interval(PEER_WATCH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_peer_snap: Vec<String> = Vec::new();
        let mut last_peer_wait_log = Instant::now() - PEER_WATCH_LOG_COOLDOWN;

        loop {
            interval.tick().await;

            let mut peers_it = session.info().peers_zid().await;
            let mut peer_snap: Vec<String> = std::iter::from_fn(|| peers_it.next())
                .map(|z| z.to_string())
                .collect();
            peer_snap.sort();

            if peer_snap.is_empty() {
                if !last_peer_snap.is_empty() {
                    eprintln!(
                        "{} All Zenoh peers disconnected.",
                        format_tag(Watch::Peer, false)
                    );
                    last_peer_snap.clear();
                } else if watch_started.elapsed() >= PEER_WATCH_STARTUP_GRACE
                    && last_peer_wait_log.elapsed() >= PEER_WATCH_LOG_COOLDOWN
                {
                    eprintln!("{} No peer on :7411 yet", format_tag(Watch::Peer, false));
                    last_peer_wait_log = Instant::now();
                }
            } else if peer_snap != last_peer_snap {
                println!(
                    "{} Zenoh peer(s) connected locally:",
                    format_tag(Watch::Peer, true)
                );
                for id in &peer_snap {
                    println!("  {id}");
                }
                last_peer_snap = peer_snap;
                last_peer_wait_log = Instant::now();
            }
        }
    });
}
