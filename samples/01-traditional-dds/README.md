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

- ROS 2 **Jazzy** on the host **or** Docker (see below)
- See [docs/prerequisites.md](../../docs/prerequisites.md)

---

## Option A — Docker (recommended)

No host ROS install required. Uses official `ros:jazzy-ros-base`.

### Build image

From repo root:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml build
```

### Quick demo (~6 s, talker + listener in one container)

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm demo
```

Use an isolated DDS domain to avoid stray traffic on your LAN:

```bash
ROS_DOMAIN_ID=42 docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm demo
```

### Manual two-container style (Linux, host network)

Terminal A:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm listener
```

Terminal B:

```bash
docker compose -f samples/01-traditional-dds/docker-compose.yml run --rm talker
```

Both services use `network_mode: host` so DDS discovery works on Linux.

### From the notebook

```python
from demo_runner import build_sample1_docker, run_sample1_docker_demo

build_sample1_docker()
print(run_sample1_docker_demo(duration_sec=6))
```

---

## Option B — Host ROS install

### Build

```bash
source /opt/ros/jazzy/setup.bash
cd samples/01-traditional-dds/cpp
colcon build
source install/setup.bash
```

### Run

Terminal A (talker):

```bash
source /opt/ros/jazzy/setup.bash
source samples/01-traditional-dds/cpp/install/setup.bash
ros2 run demo_nodes talker
```

Terminal B (listener):

```bash
source /opt/ros/jazzy/setup.bash
source samples/01-traditional-dds/cpp/install/setup.bash
ros2 run demo_nodes listener
```

The listener should print `I heard: 'Hello N'` once per second.

### Environment

Both nodes must share the same DDS domain and RMW:

```bash
export ROS_DOMAIN_ID=0          # default
export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp   # optional; Jazzy default
```

## Next

Compare with [Sample 2 — rmw_zenoh](../02-rmw-zenoh/README.md) (same nodes, Zenoh middleware).
