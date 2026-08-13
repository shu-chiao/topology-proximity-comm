# Sample 3 — DDS + Zenoh Bridge

Hybrid setup: ROS 2 nodes stay on **local DDS**, while `zenoh-bridge-ros2dds` forwards traffic through **Zenoh** for remote subscribers (Rust) or other bridged hosts.

```text
C++ talker (DDS)  →  zenoh-bridge-ros2dds  →  zenohd  →  Rust main_sub
```

## Demo contract

| Item | Value |
|------|-------|
| ROS topic | `/demo/chatter` |
| Type | `std_msgs/msg/String` |
| Zenoh key | `demo/chatter` (bridge mapping) |

## Prerequisites

- ROS 2 **Jazzy**, Rust toolchain, `zenoh-bridge-ros2dds` **1.9.x**
- See [docs/prerequisites.md](../../docs/prerequisites.md)

Start the shared router from repo root:

```bash
docker compose -f infra/docker-compose.yml up -d
```

## Build

### C++ ROS nodes (local DDS side)

```bash
source /opt/ros/jazzy/setup.bash
cd samples/03-dds-zenoh-bridge/cpp
colcon build
source install/setup.bash
```

### Rust Zenoh clients

```bash
cd samples/03-dds-zenoh-bridge/rust
cargo build
```

## Run — pub/sub demo (manual)

**Terminal 1 — bridge** (ROS publish → Zenoh):

```bash
zenoh-bridge-ros2dds --no-multicast-scouting \
  -c samples/03-dds-zenoh-bridge/configs/zenoh_bridge-as-pub-client.json5
```

**Terminal 2 — C++ talker** (local DDS):

```bash
source /opt/ros/jazzy/setup.bash
source samples/03-dds-zenoh-bridge/cpp/install/setup.bash
ros2 run demo_nodes talker
```

**Terminal 3 — Rust subscriber** (remote Zenoh leg):

```bash
cd samples/03-dds-zenoh-bridge/rust
MAIN_SUB_ROUTER=tcp/127.0.0.1:7447 cargo run --bin main_sub
```

You should see sample lines for `demo/chatter` with `Hello N` payloads.

## Run — helper script

Publisher side (bridge + talker in one script):

```bash
./samples/03-dds-zenoh-bridge/scripts/run-local-bridge-and-pub-talker.sh
```

Subscriber side (bridge + C++ listener on the ROS leg):

```bash
./samples/03-dds-zenoh-bridge/scripts/run-local-bridge-and-sub-listener.sh
```

For a full cross-stack test: run the pub script in one terminal and `MAIN_SUB_ROUTER=tcp/127.0.0.1:7447 cargo run --bin main_sub` in another.

## Layout

```text
cpp/demo_nodes/     ROS talker + listener (local DDS)
rust/               Zenoh pub/sub/service/action clients
configs/            bridge JSON5 + agent YAML
scripts/            local bridge demo helpers
```

## Advanced (secondary demos)

The Rust crate also includes service and action clients (`main_srv_client`, `main_action_client`) and matching bridge configs. These target upstream ROS demos (e.g. `/add_two_ints`, turtlesim actions) — see `configs/master_*.yaml` and `scripts/run-local-bridge-and-*-server.sh`.

## Differences vs other samples

| | Sample 1 | Sample 2 | Sample 3 |
|--|----------|----------|----------|
| ROS middleware | DDS | rmw_zenoh | DDS (local) |
| Zenoh role | none | RMW | bridge + remote clients |
| Mixed stacks | no | no | yes |

## See also

- [configs/README.md](configs/README.md) — config file naming
- [Sample 1](../01-traditional-dds/README.md) — DDS baseline
- [Sample 2](../02-rmw-zenoh/README.md) — full Zenoh RMW
