# topology-proximity-comm — module layout

## `src/zenoh/`

| File | Role |
|------|------|
| `agent.rs` | `zenoh_agent` binary: open session, spawn bridge, router/peer watches |
| `bridge.rs` | Spawn `zenoh-bridge-ros2dds`, peer watch on `:7411` |
| `cloud.rs` | Log zenohd connect/disconnect |
| `pub_cli.rs` | `main_pub` publish loop |
| `sub_cli.rs` | `main_sub` subscribe loop; shared config helpers |

## `src/config/load_yaml.rs`

Loads `configs/*.yaml` into `ResolvedEdgeAgent`, `ResolvedEdgePub`, `ResolvedSrvCall`, `ResolvedActionCall`.

## `src/wire/`

ROS 2 CDR encoding for Zenoh queries and sample attachments (`ros_*_cdr.rs`), console tags (`log_tags.rs`).

## Adding a new Zenoh client

1. Add YAML schema + resolver in `config/load_yaml.rs` if needed.
2. Add wire types in `wire/` if new CDR shapes.
3. Add handler logic in `zenoh/` or a new `src/bin/my_client.rs`.
4. Register binary in `Cargo.toml`.
5. Update `configs/README.md`.

Keep HTTP/gRPC/vision code out of this repo — it lives in **edge_agent**.
