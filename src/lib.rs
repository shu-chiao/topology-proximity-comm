//! Topology / proximity communication: Zenoh sessions, ROS bridge, wire formats.

pub mod config;
pub mod wire;
pub mod zenoh;

pub use config::load_yaml::{
    ResolvedActionCall, ResolvedEdgeAgent, ResolvedEdgePub, ResolvedSrvCall, ZenohTopology,
};
