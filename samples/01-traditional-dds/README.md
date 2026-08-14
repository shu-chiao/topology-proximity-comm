# Sample 1 — Traditional ROS 2 DDS

Baseline ROS 2 pub/sub using the default RMW (**Cyclone DDS**). No Zenoh involved.

## Demo contract

| Item | Value |
|------|-------|
| Topic | `/demo/chatter` |
| Type | `std_msgs/msg/String` |
| Rate | 1 Hz |
| QoS | reliable, keep-last depth 10 |

## Prerequisites

- **Docker** with Compose v2 — see [docs/prerequisites.md](../../docs/prerequisites.md)
- Linux recommended (`network_mode: host` for DDS discovery)

## Build image

From repo root:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml build
```

## Quick demo (~6 s, talker + listener in one container)

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm demo
```

Use an isolated DDS domain to avoid stray traffic on your LAN:

```bash
ROS_DOMAIN_ID=42 docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm demo
```

## Manual two-container style (Linux, host network)

Terminal A:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml --profile manual run --rm listener
```

Terminal B:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml --profile manual run --rm talker
```

## From the notebook

```python
from demo_runner import build_sample1_docker, run_sample1_docker_demo

build_sample1_docker()
print(run_sample1_docker_demo(duration_sec=6))
```

## Next

Compare with [Sample 2 — rmw_zenoh](../02-rmw-zenoh/README.md) (same nodes, Zenoh middleware).
