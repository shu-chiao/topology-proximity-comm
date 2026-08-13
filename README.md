# topology-proximity-comm

Comparison samples for ROS 2 communication stacks on **Jazzy**:

| Sample | Stack | Directory |
|--------|-------|-----------|
| 1. Traditional DDS | Cyclone DDS (default RMW) | [`samples/01-traditional-dds/`](samples/01-traditional-dds/) |
| 2. Zenoh RMW | `rmw_zenoh_cpp` | [`samples/02-rmw-zenoh/`](samples/02-rmw-zenoh/) |
| 3. DDS + Zenoh Bridge | Local DDS + `zenoh-bridge-ros2dds` | [`samples/03-dds-zenoh-bridge/`](samples/03-dds-zenoh-bridge/) |

Each sample is **isolated** and runnable on its own. All use the same demo contract:

- **Topic:** `/demo/chatter`
- **Type:** `std_msgs/msg/String`
- **Nodes:** C++ `talker` + `listener` (sample 3 also includes Rust Zenoh clients)

## Quick start

Work through the samples in order:

1. [Traditional DDS](samples/01-traditional-dds/README.md) — baseline, no Zenoh (**Docker supported**)
2. [rmw_zenoh](samples/02-rmw-zenoh/README.md) — ROS 2 API, Zenoh middleware
3. [DDS + Bridge](samples/03-dds-zenoh-bridge/README.md) — hybrid local DDS + remote Zenoh

## Docs

- [Comparison guide](docs/comparison.md) — when to pick which approach
- [Prerequisites](docs/prerequisites.md) — Jazzy, Zenoh 1.9.x, bridge/rmw install

## Shared infrastructure

Samples 2 and 3 need a Zenoh router:

```bash
docker compose -f infra/docker-compose.yml up -d
```

## Layout

```text
samples/
  01-traditional-dds/   # C++ talker/listener on DDS
  02-rmw-zenoh/         # same nodes, rmw_zenoh_cpp + zenohd
  03-dds-zenoh-bridge/  # C++ on DDS + Rust Zenoh clients via bridge
infra/                  # shared zenohd compose file
docs/                   # comparison + prerequisites
archived/               # superseded monolith (reference only)
```

## See also

- [`notebooks/quick_run_samples.ipynb`](notebooks/quick_run_samples.ipynb) — build & run all three samples from Jupyter
- `make help` — common install and docker helpers
