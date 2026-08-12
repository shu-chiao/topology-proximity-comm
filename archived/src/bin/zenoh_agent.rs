//! Zenoh peer agent: session, topology checks, bridge spawn, router/peer watches.
//!
//! ```bash
//! cargo run --bin zenoh_agent
//! ```

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    topology_proximity_comm::zenoh::agent::run().await
}
