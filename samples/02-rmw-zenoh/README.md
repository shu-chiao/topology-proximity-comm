# Sample 2 — Zenoh as RMW (`rmw_zenoh_cpp`)

Same C++ talker/listener as Sample 1, but ROS 2 uses **Zenoh** instead of DDS as the middleware. The ROS 2 API is unchanged — only the RMW layer differs.

## Demo contract

Same as Sample 1:

| Item | Value |
|------|-------|
| Topic | `/demo/chatter` |
| Type | `std_msgs/msg/String` |
| Rate | 1 Hz |
| QoS | reliable, keep-last depth 10 |

## Prerequisites

- **Docker** with Compose v2 — see [docs/prerequisites.md](../../docs/prerequisites.md)
- Linux recommended (`network_mode: host`)

The quick demo starts `rmw_zenohd` inside the container, or reuses an existing router on `tcp/127.0.0.1:7447`.

## Build image

From repo root:

```bash
docker compose -f samples/02-rmw-zenoh/docker-compose.yml build
```

## Quick demo (~6 s, rmw_zenohd + talker + listener in one container)

```bash
docker compose -f samples/02-rmw-zenoh/docker-compose.yml run --rm demo
```

Use an isolated domain to avoid stray traffic on your LAN:

```bash
ROS_DOMAIN_ID=42 docker compose -f samples/02-rmw-zenoh/docker-compose.yml run --rm demo
```

## Manual multi-container style (Linux, host network)

Terminal A — router:

```bash
docker compose -f samples/02-rmw-zenoh/docker-compose.yml --profile manual up zenohd
```

Terminal B — listener:

```bash
docker compose -f samples/02-rmw-zenoh/docker-compose.yml --profile manual run --rm listener
```

Terminal C — talker:

```bash
docker compose -f samples/02-rmw-zenoh/docker-compose.yml --profile manual run --rm talker
```

## From the notebook

```python
from demo_runner import build_sample2_docker, run_sample2_docker_demo

build_sample2_docker()
print(run_sample2_docker_demo(duration_sec=6))
```

## Differences vs Sample 1

| | Sample 1 (DDS) | Sample 2 (rmw_zenoh) |
|--|----------------|----------------------|
| Middleware | Cyclone DDS | Zenoh via `rmw_zenoh_cpp` |
| Router required | no | yes (`rmw_zenohd` on `:7447`) |
| ROS code changes | — | none |
| WAN-friendly | LAN multicast | yes (via Zenoh router) |

Both talker and listener **must** use `rmw_zenoh_cpp` and reach the same router.

## Next

Compare with [Sample 3 — DDS + Bridge](../03-dds-zenoh-bridge/README.md) (local DDS ROS nodes + Zenoh bridge for remote clients).
