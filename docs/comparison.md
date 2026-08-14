# Comparison — DDS vs rmw_zenoh vs Bridge

Three ways to connect ROS 2 systems across machines. All samples in this repo use the same `/demo/chatter` pub/sub demo so you can compare stacks, not message shapes.

## Summary table

| Criterion | Sample 1: Traditional DDS | Sample 2: rmw_zenoh | Sample 3: DDS + Bridge |
|-----------|---------------------------|---------------------|------------------------|
| ROS code changes | none | none | none on ROS side |
| Middleware | Cyclone DDS (default) | `rmw_zenoh_cpp` | DDS locally + Zenoh via bridge |
| Zenoh router (`zenohd`) | not required | required | required (for remote leg) |
| Mixed DDS + Zenoh nodes | n/a | no — all nodes use same RMW | yes — by design |
| WAN / cross-subnet | LAN DDS discovery | native Zenoh routing | bridge carries selected topics |
| Ops complexity | lowest | medium | highest (bridge process + configs) |
| Best for | LAN lab, baseline | Greenfield Zenoh deployments | Existing DDS fleet + Zenoh edge/cloud |

## Data flow

### Sample 1 — Traditional DDS

```text
talker ──DDS──► listener
```

Both nodes use the same DDS domain. Discovery is multicast-based (LAN).

### Sample 2 — rmw_zenoh

```text
talker ──rmw_zenoh──► zenohd ◄──rmw_zenoh── listener
```

ROS 2 API is unchanged. The RMW layer speaks Zenoh instead of DDS. Every node must use `rmw_zenoh_cpp` and reach the same router.

### Sample 3 — DDS + Bridge (bidirectional)

```text
talker ──DDS──► bridge ──Zenoh──► zenohd ──Zenoh──► main_sub
main_pub ──Zenoh──► zenohd ──Zenoh──► bridge ──DDS──► listener
```

ROS nodes stay on local DDS. The bridge forwards selected topics into Zenoh. Remote Rust clients can both subscribe (ROS → Rust) and publish (Rust → ROS) through the same router.

## When to pick which

**Pick Sample 1 (DDS)** when:

- All nodes are on the same LAN
- You want the simplest setup with no extra infrastructure
- You are learning ROS 2 basics

**Pick Sample 2 (rmw_zenoh)** when:

- You want Zenoh end-to-end for all ROS communication
- You can standardize on `rmw_zenoh_cpp` across every node
- You do not need to mix legacy DDS nodes with Zenoh nodes

**Pick Sample 3 (Bridge)** when:

- You already have a DDS-based ROS 2 fleet and cannot change RMW on every node
- Only some topics need to cross the WAN
- You want Rust or other Zenoh-native consumers alongside ROS

## Language notes

| Sample | C++ | Rust |
|--------|-----|------|
| 1 | ROS talker/listener | optional (`rclrs`, future) |
| 2 | ROS talker/listener | optional (`rclrs`, future) |
| 3 | ROS talker/listener (DDS side) | Zenoh pub/sub clients (primary remote leg) |

## Version alignment

Keep these on the same Zenoh release line (**1.9.x** in this repo):

- `zenohd` / `zenoh-bridge-ros2dds` in sample 3 `docker/ros2` image
- `rmw_zenoh_cpp` in sample 2 Docker image
- Rust `zenoh` crate (`samples/03-dds-zenoh-bridge/rust/`)

See [prerequisites.md](prerequisites.md) for Docker setup.
