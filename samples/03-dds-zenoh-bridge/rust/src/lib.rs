//! Topology / proximity communication: Zenoh pub/sub clients and ROS wire formats.

pub mod config;
pub mod wire;
pub mod zenoh;

pub use config::load_yaml::{ResolvedEdgePub, ResolvedSubConfig};
