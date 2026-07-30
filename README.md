# topology-proximity-comm

Zenoh sessions, ROS 2 ↔ Zenoh bridge, and wire-format helpers for edge topology / proximity communication.

```text
ROS 2 (DDS)  →  zenoh-bridge-ros2dds  →  Zenoh  →  zenohd (cloud)  →  remote clients
                              ↑
                    zenoh_agent (peer + bridge spawn)
```

## Prerequisites

- Rust toolchain
- [Docker Compose](https://docs.docker.com/compose/) on Linux for local `zenohd` (`network_mode: host`)
- **`zenoh-bridge-ros2dds` 1.9.x** on `PATH` when `bridge.skip_spawn: false` — see [`docs/ros2-bridge.md`](docs/ros2-bridge.md)

Align **zenoh** (crate), **eclipse/zenoh**, and **zenoh-bridge-ros2dds** on **1.9**.

## Runnable entry points

| What | Command |
|------|---------|
| Zenoh agent (session + bridge) | `cargo run --bin zenoh_agent` |
| Subscriber | `cargo run --bin main_sub` |
| Publisher | `cargo run --bin main_pub` |
| ROS service over Zenoh | `cargo run --bin main_srv_client` |
| ROS action over Zenoh | `cargo run --bin main_action_client` |
| Local `zenohd` | `docker compose up -d` or `make docker-up` |

## Quick start

```bash
docker compose up -d
cargo run --bin zenoh_agent
```

## Layout

```text
src/
  zenoh/     session, bridge spawn, pub/sub CLI
  config/    YAML loaders (configs/*.yaml)
  wire/      ROS CDR + log tags for Zenoh queries
configs/     YAML + JSON5 for agent, bridge, clients
bash/        local bridge + ROS demo scripts
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for module roles and how to extend.

## See also

- [`configs/README.md`](configs/README.md) — YAML / JSON5 naming
- [`docs/ros2-bridge.md`](docs/ros2-bridge.md) — bridge install & usage
- [`docs/zenoh-bridge-ros2dds-wire-contract.md`](docs/zenoh-bridge-ros2dds-wire-contract.md) — wire format
- [`docs/dev.log`](docs/dev.log) — operational pitfalls & integration notes
