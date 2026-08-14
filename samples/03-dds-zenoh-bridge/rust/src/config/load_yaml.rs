//! Load `configs/*.yaml` for subscriber and publisher.

use crate::config::configs_dir;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Subscriber settings from `edge_agent-sub.yaml`.
#[derive(Debug, Clone)]
pub struct ResolvedSubConfig {
    pub discover: bool,
    pub keyexpr: String,
}

#[derive(Debug, Deserialize)]
struct SubYamlRoot {
    #[serde(default)]
    subscriber: SubscriberSection,
}

#[derive(Debug, Deserialize, Default)]
struct SubscriberSection {
    #[serde(default)]
    discover: bool,
    #[serde(default = "default_keyexpr")]
    keyexpr: String,
}

fn default_keyexpr() -> String {
    "demo/chatter".into()
}

impl ResolvedSubConfig {
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

    pub fn load_sub_default() -> anyhow::Result<Self> {
        Self::load_file(&Self::sub_default_yaml_path())
    }

    pub fn load_file(yaml_path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(yaml_path)
            .map_err(|e| anyhow::anyhow!("read subscriber YAML {}: {e}", yaml_path.display()))?;
        let parsed: SubYamlRoot = serde_yaml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parse subscriber YAML {}: {e}", yaml_path.display()))?;
        Ok(Self {
            discover: parsed.subscriber.discover,
            keyexpr: parsed.subscriber.keyexpr,
        })
    }
}

/// Publisher settings for `main_pub` (`edge_agent-demo-pub.yaml`).
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
    "demo/chatter".into()
}

fn default_pub_payload() -> String {
    "Hello from Rust".into()
}

fn default_pub_period_ms() -> u64 {
    1000
}

impl ResolvedEdgePub {
    /// Publisher YAML path: `EDGE_AGENT_PUB_YAML` or `configs/edge_agent-demo-pub.yaml`.
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
            Err(_) => configs.join("edge_agent-demo-pub.yaml"),
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
