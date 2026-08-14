# topology-proximity-comm

Comparison samples for ROS 2 communication stacks on **Jazzy** — all runnable via **Docker**:

| Sample | Stack | Directory |
|--------|-------|-----------|
| 1. Traditional DDS | Cyclone DDS (default RMW) | [`samples/01-traditional-dds/`](samples/01-traditional-dds/) |
| 2. Zenoh RMW | `rmw_zenoh_cpp` | [`samples/02-rmw-zenoh/`](samples/02-rmw-zenoh/) |
| 3. DDS + Zenoh Bridge | Local DDS + `zenoh-bridge-ros2dds` | [`samples/03-dds-zenoh-bridge/`](samples/03-dds-zenoh-bridge/) |

Each sample is **isolated** and uses the same demo contract:

- **Topic:** `/demo/chatter`
- **Type:** `std_msgs/msg/String`
- **Nodes:** C++ `talker` + `listener` (sample 3 also includes Rust Zenoh clients)

## Quick start

**Prerequisites:** Docker + Compose v2 on Linux — see [docs/prerequisites.md](docs/prerequisites.md).

Work through the samples in order:

1. [Traditional DDS](samples/01-traditional-dds/README.md) — 1 container
2. [rmw_zenoh](samples/02-rmw-zenoh/README.md) — 1 container (+ `rmw_zenohd`)
3. [DDS + Bridge](samples/03-dds-zenoh-bridge/README.md) — 2 containers (ros2 + rust)

Or use the notebook:

```bash
jupyter notebook notebooks/quick_run_samples.ipynb
```

## Docs

- [Comparison guide](docs/comparison.md) — when to pick which approach
- [Prerequisites](docs/prerequisites.md) — Docker setup

## Layout

```text
samples/
  01-traditional-dds/   # Docker: talker/listener on DDS
  02-rmw-zenoh/         # Docker: rmw_zenoh_cpp + rmw_zenohd
  03-dds-zenoh-bridge/  # Docker: ros2 + rust via bridge
notebooks/              # quick_run_samples.ipynb + demo_runner.py
docs/                   # comparison + prerequisites
archived/               # superseded monolith (reference only)
```

## See also

- [`notebooks/quick_run_samples.ipynb`](notebooks/quick_run_samples.ipynb) — build & run all three samples from Jupyter
