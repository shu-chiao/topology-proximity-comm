# Sample 2 — Zenoh as RMW (`rmw_zenoh_cpp`)

Same C++ talker/listener as Sample 1, but ROS 2 uses **Zenoh** instead of DDS as the middleware. The ROS 2 API is unchanged — only the RMW layer differs.

## Demo contract

Same as Sample 1:

| Item | Value |
|------|-------|
| Topic | `/demo/chatter` |
| Type | `std_msgs/msg/String` |

## Prerequisites

- ROS 2 **Jazzy** with `rmw_zenoh_cpp` — see [docs/prerequisites.md](../../docs/prerequisites.md)
- **zenohd** router running:

```bash
docker compose -f infra/docker-compose.yml up -d
```

## Build

```bash
source /opt/ros/jazzy/setup.bash
cd samples/02-rmw-zenoh/cpp
colcon build
source install/setup.bash
```

## Run

Set the Zenoh RMW and point at the local router:

```bash
export RMW_IMPLEMENTATION=rmw_zenoh_cpp
export ZENOH_CONFIG_OVERRIDE='mode="client";connect/endpoints=["tcp/127.0.0.1:7447"]'
```

Or use the bundled config file:

```bash
export RMW_IMPLEMENTATION=rmw_zenoh_cpp
export ZENOH_SESSION_CONFIG_URI="$(pwd)/samples/02-rmw-zenoh/configs/zenoh-client.json5"
```

Terminal A (talker):

```bash
source /opt/ros/jazzy/setup.bash
source samples/02-rmw-zenoh/cpp/install/setup.bash
ros2 run demo_nodes talker
```

Terminal B (listener):

```bash
source /opt/ros/jazzy/setup.bash
source samples/02-rmw-zenoh/cpp/install/setup.bash
ros2 run demo_nodes listener
```

## Differences vs Sample 1

| | Sample 1 (DDS) | Sample 2 (rmw_zenoh) |
|--|----------------|----------------------|
| Middleware | Cyclone DDS | Zenoh via `rmw_zenoh_cpp` |
| Router required | no | yes (`zenohd` on `:7447`) |
| ROS code changes | — | none |
| WAN-friendly | LAN multicast | yes (via Zenoh router) |

Both talker and listener **must** use `rmw_zenoh_cpp` and reach the same `zenohd`.

## Next

Compare with [Sample 3 — DDS + Bridge](../03-dds-zenoh-bridge/README.md) (local DDS ROS nodes + Zenoh bridge for remote clients).
