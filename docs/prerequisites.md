# Prerequisites

All samples target **ROS 2 Jazzy** on Linux.

## ROS 2 Jazzy

Install the desktop or base package:

```bash
# Follow official docs: https://docs.ros.org/en/jazzy/Installation.html
source /opt/ros/jazzy/setup.bash
```

Verify:

```bash
ros2 --version
```

## Sample 1 — Traditional DDS

- ROS 2 Jazzy with default RMW (Cyclone DDS)
- `colcon` build tools: `sudo apt install ros-jazzy-ros-base`

## Sample 2 — rmw_zenoh

Everything in Sample 1, plus:

### rmw_zenoh_cpp

Install from ROS packages (if available for Jazzy):

```bash
sudo apt install ros-jazzy-rmw-zenoh-cpp
```

Or build from source: [rmw_zenoh](https://github.com/ros2/rmw_zenoh).

### zenohd router

From repo root:

```bash
docker compose -f infra/docker-compose.yml up -d
```

Verify the router is listening on TCP **7447**.

### Runtime env

```bash
export RMW_IMPLEMENTATION=rmw_zenoh_cpp
export ZENOH_CONFIG_OVERRIDE='mode="client";connect/endpoints=["tcp/127.0.0.1:7447"]'
```

Or point at the bundled config:

```bash
export ZENOH_SESSION_CONFIG_URI="$(pwd)/samples/02-rmw-zenoh/configs/zenoh-client.json5"
```

## Sample 3 — DDS + Bridge

Everything in Sample 1, plus:

### Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### zenoh-bridge-ros2dds (1.9.x)

Quick install via APT (Debian/Ubuntu):

```bash
make install-zenoh-bridge
```

Or manually from [Eclipse Zenoh releases](https://download.eclipse.org/zenoh/zenoh-plugin-ros2dds/latest/).

Verify:

```bash
zenoh-bridge-ros2dds --version
```

### zenohd

Same Docker compose as Sample 2:

```bash
docker compose -f infra/docker-compose.yml up -d
```

### Version alignment

Keep these on **1.9.x**:

| Component | Location |
|-----------|----------|
| `eclipse/zenoh:1.9.0` | `infra/docker-compose.yml` |
| `zenoh-bridge-ros2dds` | system PATH |
| `zenoh` Rust crate | `samples/03-dds-zenoh-bridge/rust/Cargo.toml` |

Mismatch across versions causes subtle wire/discovery failures.

## Optional — Docker

Required for `zenohd` in samples 2 and 3. Docker Compose v2 recommended.

On **Docker Desktop (macOS/Windows)**, `network_mode: host` does not work the same as on Linux. Run `zenohd` natively or use a Linux host for LAN discovery tests.

## Build tools summary

| Tool | Samples |
|------|---------|
| `colcon` | 1, 2, 3 (C++ nodes) |
| `cargo` | 3 (Rust Zenoh clients) |
| `docker compose` | 2, 3 (zenohd) |
| `zenoh-bridge-ros2dds` | 3 |

## Troubleshooting

- **Nodes don't see each other (Sample 1):** check `ROS_DOMAIN_ID` matches on both terminals.
- **rmw_zenoh nodes silent (Sample 2):** confirm `zenohd` is up and `ZENOH_CONFIG_OVERRIDE` or `ZENOH_SESSION_CONFIG_URI` points to `tcp/127.0.0.1:7447`.
- **Bridge shows no traffic (Sample 3):** ensure bridge config `allow.publishers` includes `/demo/chatter` (default: `/.*`). Only one bridge per ROS role per host.
- **Rust sub sees nothing:** set `MAIN_SUB_ROUTER=tcp/127.0.0.1:7447` and confirm the pub-side bridge is running with `zenoh_bridge-as-pub-client.json5`.
