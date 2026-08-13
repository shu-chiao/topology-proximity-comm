//! Load `configs/*.yaml` for the agent, subscriber, publisher, and service client.

use crate::config::configs_dir;
use crate::zenoh::bridge::{BridgeMode, BridgeSpawnOptions};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Parsed edge agent config (Zenoh, subscriber, bridge).
#[derive(Debug, Clone)]
pub struct ResolvedEdgeAgent {
    /// `peer` listens on :7411; `wan_client` connects to zenohd only.
    pub zenoh_topology: ZenohTopology,
    pub zenoh_config_path: PathBuf,
    /// Run subscriber inside the agent session (`main_sub` only today).
    pub embed_subscriber: bool,
    /// Print discover lines for new topics.
    pub subscriber_discover: bool,
    pub subscriber_keyexpr: String,
    pub bridge: BridgeSpawnOptions,
}

#[derive(Debug, Deserialize)]
struct FileRoot {
    #[serde(default)]
    zenoh: ZenohSection,
    #[serde(default)]
    subscriber: SubscriberSection,
    #[serde(default)]
    bridge: BridgeSection,
}

/// Which Zenoh JSON5 file to use.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ZenohTopology {
    /// Peer mode: listen on :7411 for the local bridge.
    #[default]
    Peer,
    /// Client mode: connect to zenohd only.
    WanClient,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ZenohSection {
    topology: ZenohTopology,
    #[serde(default = "default_zenoh_config_file")]
    config_file: String,
    #[serde(default = "default_zenoh_client_config_file")]
    client_config_file: String,
}

impl Default for ZenohSection {
    fn default() -> Self {
        Self {
            topology: ZenohTopology::default(),
            config_file: default_zenoh_config_file(),
            client_config_file: default_zenoh_client_config_file(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct SubscriberSection {
    #[serde(default)]
    embed: bool,
    /// Enable discover output for new topics.
    #[serde(default)]
    discover: bool,
    #[serde(default = "default_keyexpr")]
    keyexpr: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct BridgeSection {
    mode: BridgeMode,
    skip_spawn: bool,
    verbose_logs: bool,
    detach: bool,
    #[serde(default = "default_bridge_exe")]
    executable: String,
    #[serde(default = "default_true")]
    log_host_dds_env: bool,
}

impl Default for BridgeSection {
    fn default() -> Self {
        Self {
            mode: BridgeMode::LocalPeer,
            skip_spawn: false,
            verbose_logs: false,
            detach: false,
            executable: default_bridge_exe(),
            log_host_dds_env: true,
        }
    }
}

fn default_zenoh_config_file() -> String {
    "zenoh_agent-as-peer.json5".into()
}

fn default_zenoh_client_config_file() -> String {
    "zenoh_agent-as-client.json5".into()
}

fn default_keyexpr() -> String {
    "chatter".into()
}

fn default_bridge_exe() -> String {
    "zenoh-bridge-ros2dds".into()
}

fn default_true() -> bool {
    true
}

fn resolve_under_configs(configs_dir: &Path, raw: &str) -> PathBuf {
    let p = Path::new(raw);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        configs_dir.join(p)
    }
}

impl ResolvedEdgeAgent {
    /// Default path: `configs/edge_agent.yaml`, or `EDGE_AGENT_YAML`.
    pub fn default_yaml_path() -> PathBuf {
        let configs = configs_dir();
        match std::env::var("EDGE_AGENT_YAML") {
            Ok(s) => {
                let s = s.trim();
                let p = Path::new(s);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    configs.join(p)
                }
            }
            Err(_) => configs.join("edge_agent.yaml"),
        }
    }

    /// Load the default edge agent YAML.
    pub fn load_default() -> anyhow::Result<Self> {
        Self::load_file(&Self::default_yaml_path())
    }

    /// Subscriber YAML path: `EDGE_AGENT_SUB_YAML` or `configs/edge_agent-sub.yaml`.
    pub fn sub_default_yaml_path() -> PathBuf {
        let configs = configs_dir();
        match std::env::var("EDGE_AGENT_SUB_YAML") {
            Ok(s) => {
                let s = s.trim();
                let p = Path::new(s);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    configs.join(p)
                }
            }
            Err(_) => configs.join("edge_agent-sub.yaml"),
        }
    }

    /// Load subscriber YAML for `main_sub`.
    pub fn load_sub_default() -> anyhow::Result<Self> {
        Self::load_file(&Self::sub_default_yaml_path())
    }

    pub fn load_file(yaml_path: &Path) -> anyhow::Result<Self> {
        let configs_dir = configs_dir();

        let raw = fs::read_to_string(yaml_path)
            .map_err(|e| anyhow::anyhow!("read edge agent YAML {}: {e}", yaml_path.display()))?;

        let parsed: FileRoot = serde_yaml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parse YAML {}: {e}", yaml_path.display()))?;

        let mut topology = parsed.zenoh.topology;
        if let Ok(env_t) = std::env::var("ZENOH_AGENT_TOPOLOGY") {
            match env_t.trim().to_lowercase().as_str() {
                "peer" => topology = ZenohTopology::Peer,
                "wan_client" => topology = ZenohTopology::WanClient,
                _ => {}
            }
        }

        let zenoh_config_path = match topology {
            ZenohTopology::Peer => resolve_under_configs(&configs_dir, &parsed.zenoh.config_file),
            ZenohTopology::WanClient => {
                resolve_under_configs(&configs_dir, &parsed.zenoh.client_config_file)
            }
        };
        Ok(Self {
            zenoh_topology: topology,
            zenoh_config_path,
            embed_subscriber: parsed.subscriber.embed,
            subscriber_discover: parsed.subscriber.discover,
            subscriber_keyexpr: parsed.subscriber.keyexpr,
            bridge: BridgeSpawnOptions {
                mode: parsed.bridge.mode,
                skip_spawn: parsed.bridge.skip_spawn,
                verbose_logs: parsed.bridge.verbose_logs,
                detach: parsed.bridge.detach,
                executable: parsed.bridge.executable,
                log_host_dds_env: parsed.bridge.log_host_dds_env,
            },
        })
    }
}

/// Publisher settings for `main_pub` (`edge_agent-pub.yaml`).
#[derive(Debug, Clone)]
pub struct ResolvedEdgePub {
    pub keyexpr: String,
    pub payload: String,
    pub period_ms: u64,
}

#[derive(Debug, Deserialize)]
struct PubYamlRoot {
    publisher: PubPublisherSection,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct PubPublisherSection {
    keyexpr: String,
    payload: String,
    period_ms: u64,
}

impl Default for PubPublisherSection {
    fn default() -> Self {
        Self {
            keyexpr: default_pub_keyexpr(),
            payload: default_pub_payload(),
            period_ms: default_pub_period_ms(),
        }
    }
}

fn default_pub_keyexpr() -> String {
    "demo/rust_pub".into()
}

fn default_pub_payload() -> String {
    "Hello zenoh".into()
}

fn default_pub_period_ms() -> u64 {
    1000
}

impl ResolvedEdgePub {
    /// Publisher YAML path: `EDGE_AGENT_PUB_YAML` or `configs/edge_agent-pub.yaml`.
    pub fn pub_default_yaml_path() -> PathBuf {
        let configs = configs_dir();
        match std::env::var("EDGE_AGENT_PUB_YAML") {
            Ok(s) => {
                let s = s.trim();
                let p = Path::new(s);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    configs.join(p)
                }
            }
            Err(_) => configs.join("edge_agent-pub.yaml"),
        }
    }

    pub fn load_pub_default() -> anyhow::Result<Self> {
        Self::load_pub_file(&Self::pub_default_yaml_path())
    }

    pub fn load_pub_file(yaml_path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(yaml_path)
            .map_err(|e| anyhow::anyhow!("read publisher YAML {}: {e}", yaml_path.display()))?;
        let parsed: PubYamlRoot = serde_yaml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parse publisher YAML {}: {e}", yaml_path.display()))?;
        Ok(Self {
            keyexpr: parsed.publisher.keyexpr,
            payload: parsed.publisher.payload,
            period_ms: parsed.publisher.period_ms,
        })
    }
}

/// Service call settings for `main_srv_client` (`master_srv-client.yaml`).
#[derive(Debug, Clone)]
pub struct ResolvedSrvCall {
    /// ROS service name, e.g. `/add_two_ints`.
    pub ros_service_name: String,
    pub service_type: String,
    pub args: serde_yaml::Value,
    /// Override Zenoh key (default: strip leading `/` from service name).
    pub zenoh_keyexpr: Option<String>,
    /// Zenoh JSON5 config override (`MAIN_SRV_ZENOH_JSON5`).
    pub zenoh_json5: Option<String>,
    pub router: Option<String>,
    pub timeout_ms: Option<u64>,
    pub seq: Option<u64>,
    pub client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MasterSrvYamlRoot {
    ros2_service_call: MasterRos2ServiceCallSection,
    #[serde(default)]
    srv_client: MasterSrvClientSection,
}

#[derive(Debug, Deserialize)]
struct MasterRos2ServiceCallSection {
    /// ROS service name; Zenoh key strips `/` unless `zenoh_keyexpr` is set.
    service_name: String,
    /// ROS type string, e.g. `example_interfaces/srv/AddTwoInts`.
    service_type: String,
    #[serde(default = "default_srv_args_mapping")]
    args: serde_yaml::Value,
    /// Explicit Zenoh key override.
    #[serde(default)]
    zenoh_keyexpr: Option<String>,
}

fn default_srv_args_mapping() -> serde_yaml::Value {
    serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct MasterSrvClientSection {
    #[serde(default)]
    zenoh_json5: Option<String>,
    #[serde(default)]
    router: Option<String>,
    #[serde(default)]
    timeout_ms: Option<u64>,
    #[serde(default)]
    seq: Option<u64>,
    #[serde(default)]
    client_id: Option<String>,
}

impl ResolvedSrvCall {
    /// Service client YAML path: `MASTER_SRV_CLIENT_YAML` or `configs/master_srv-client.yaml`.
    pub fn master_srv_yaml_path() -> PathBuf {
        let configs = configs_dir();
        match std::env::var("MASTER_SRV_CLIENT_YAML") {
            Ok(s) => {
                let s = s.trim();
                let p = Path::new(s);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    configs.join(p)
                }
            }
            Err(_) => configs.join("master_srv-client.yaml"),
        }
    }

    pub fn load_master_default() -> anyhow::Result<Self> {
        Self::load_master_file(&Self::master_srv_yaml_path())
    }

    pub fn load_master_file(yaml_path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(yaml_path).map_err(|e| {
            anyhow::anyhow!(
                "read master service-client YAML {}: {e}",
                yaml_path.display()
            )
        })?;
        let parsed: MasterSrvYamlRoot = serde_yaml::from_str(&raw).map_err(|e| {
            anyhow::anyhow!(
                "parse master service-client YAML {}: {e}",
                yaml_path.display()
            )
        })?;
        let c = parsed.ros2_service_call;
        if c.service_name.trim().is_empty() {
            anyhow::bail!(
                "{}: ros2_service_call.service_name must not be empty",
                yaml_path.display()
            );
        }
        if c.service_type.trim().is_empty() {
            anyhow::bail!(
                "{}: ros2_service_call.service_type must not be empty",
                yaml_path.display()
            );
        }
        let sc = parsed.srv_client;
        Ok(Self {
            ros_service_name: c.service_name,
            service_type: c.service_type,
            args: c.args,
            zenoh_keyexpr: c.zenoh_keyexpr,
            zenoh_json5: sc.zenoh_json5,
            router: sc.router,
            timeout_ms: sc.timeout_ms,
            seq: sc.seq,
            client_id: sc.client_id,
        })
    }

    pub fn zenoh_keyexpr_resolved(&self) -> anyhow::Result<String> {
        if let Some(ref k) = self.zenoh_keyexpr {
            let t = k.trim();
            if t.is_empty() {
                anyhow::bail!("zenoh_keyexpr is set but empty");
            }
            return Ok(t.to_owned());
        }
        let s = self.ros_service_name.trim();
        Ok(s.strip_prefix('/').unwrap_or(s).trim().to_owned())
    }
}

/// Action client settings for `main_action_client` (`master_action-client.yaml`).
#[derive(Debug, Clone)]
pub struct ResolvedActionCall {
    pub ros_action_name: String,
    pub action_type: String,
    pub goal: serde_yaml::Value,
    pub zenoh_keyexpr: Option<String>,
    pub zenoh_json5: Option<String>,
    pub router: Option<String>,
    pub send_goal_timeout_ms: Option<u64>,
    pub get_result_timeout_ms: Option<u64>,
    pub seq: Option<u64>,
    pub client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MasterActionYamlRoot {
    ros2_action_send_goal: MasterRos2ActionSendGoalSection,
    #[serde(default)]
    action_client: MasterActionClientSection,
}

#[derive(Debug, Deserialize)]
struct MasterRos2ActionSendGoalSection {
    action_name: String,
    action_type: String,
    #[serde(default = "default_action_goal_mapping")]
    goal: serde_yaml::Value,
    #[serde(default)]
    zenoh_keyexpr: Option<String>,
}

fn default_action_goal_mapping() -> serde_yaml::Value {
    serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct MasterActionClientSection {
    #[serde(default)]
    zenoh_json5: Option<String>,
    #[serde(default)]
    router: Option<String>,
    #[serde(default)]
    send_goal_timeout_ms: Option<u64>,
    #[serde(default)]
    get_result_timeout_ms: Option<u64>,
    #[serde(default)]
    seq: Option<u64>,
    #[serde(default)]
    client_id: Option<String>,
}

impl ResolvedActionCall {
    pub fn master_action_yaml_path() -> PathBuf {
        let configs = configs_dir();
        match std::env::var("MASTER_ACTION_CLIENT_YAML") {
            Ok(s) => {
                let s = s.trim();
                let p = Path::new(s);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    configs.join(p)
                }
            }
            Err(_) => configs.join("master_action-client.yaml"),
        }
    }

    pub fn load_master_default() -> anyhow::Result<Self> {
        Self::load_master_file(&Self::master_action_yaml_path())
    }

    pub fn load_master_file(yaml_path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(yaml_path).map_err(|e| {
            anyhow::anyhow!(
                "read master action-client YAML {}: {e}",
                yaml_path.display()
            )
        })?;
        let parsed: MasterActionYamlRoot = serde_yaml::from_str(&raw).map_err(|e| {
            anyhow::anyhow!(
                "parse master action-client YAML {}: {e}",
                yaml_path.display()
            )
        })?;
        let c = parsed.ros2_action_send_goal;
        if c.action_name.trim().is_empty() {
            anyhow::bail!(
                "{}: ros2_action_send_goal.action_name must not be empty",
                yaml_path.display()
            );
        }
        if c.action_type.trim().is_empty() {
            anyhow::bail!(
                "{}: ros2_action_send_goal.action_type must not be empty",
                yaml_path.display()
            );
        }
        let ac = parsed.action_client;
        Ok(Self {
            ros_action_name: c.action_name,
            action_type: c.action_type,
            goal: c.goal,
            zenoh_keyexpr: c.zenoh_keyexpr,
            zenoh_json5: ac.zenoh_json5,
            router: ac.router,
            send_goal_timeout_ms: ac.send_goal_timeout_ms,
            get_result_timeout_ms: ac.get_result_timeout_ms,
            seq: ac.seq,
            client_id: ac.client_id,
        })
    }

    pub fn zenoh_keyexpr_resolved(&self) -> anyhow::Result<String> {
        if let Some(ref k) = self.zenoh_keyexpr {
            let t = k.trim();
            if t.is_empty() {
                anyhow::bail!("zenoh_keyexpr is set but empty");
            }
            return Ok(t.to_owned());
        }
        let s = self.ros_action_name.trim();
        Ok(s.strip_prefix('/').unwrap_or(s).trim().to_owned())
    }
}
