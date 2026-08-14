# Prerequisites

All samples run via **Docker** — no host ROS, Rust, or bridge install required.

## Required

| Tool | Notes |
|------|-------|
| **Docker** | Engine 20+ recommended |
| **Docker Compose** | v2 (`docker compose`) |
| **Linux host** | `network_mode: host` for DDS/Zenoh discovery (samples 1–3) |

Verify:

```bash
docker --version
docker compose version
```

## Per sample

| Sample | Docker compose file | Containers |
|--------|---------------------|------------|
| 1 — Traditional DDS | `samples/01-traditional-dds/docker-compose.yml` | 1 (`demo`) |
| 2 — rmw_zenoh | `samples/02-rmw-zenoh/docker-compose.yml` | 1 (`demo`; includes `rmw_zenohd`) |
| 3 — DDS + Bridge | `samples/03-dds-zenoh-bridge/docker-compose.yml` | 2 (`ros2` + `rust`) |

Each Docker image builds its own dependencies (Jazzy, `rmw_zenoh_cpp`, `zenoh-bridge-ros2dds`, Rust binaries, etc.).

## Notebook (optional)

```bash
pip install -r notebooks/requirements.txt   # ipykernel only
jupyter notebook notebooks/quick_run_samples.ipynb
```

## Platform notes

On **Docker Desktop (macOS/Windows)**, `network_mode: host` does not behave like Linux. Use a Linux machine or VM for these demos.

First `docker compose build` may take several minutes (especially sample 3 Rust compile).

## Troubleshooting

- **Sample 1 — no messages:** check `ROS_DOMAIN_ID` matches if you override it; ensure nothing else is flooding the same domain on your LAN.
- **Sample 2 — router port in use:** stop other `zenohd` on `:7447`, or let the demo reuse an existing router.
- **Sample 3 — rust cannot connect:** ensure the **ros2** container started first and `zenohd` is listening on `tcp/127.0.0.1:7447`.
- **Sample 3 — build slow:** Rust first compile is cached in the Docker layer; subsequent runs are faster.

## Version alignment (inside Docker images)

Keep these on **Zenoh 1.9.x** when updating Dockerfiles:

| Component | Sample 3 ros2 image |
|-----------|---------------------|
| `zenohd` | Eclipse apt repo |
| `zenoh-bridge-ros2dds` | Eclipse apt repo |
| `zenoh` Rust crate | `samples/03-dds-zenoh-bridge/rust/Cargo.toml` |
