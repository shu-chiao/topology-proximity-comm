# Sample 3 configs

Four files used by the Docker demo:

| File | Used by |
|------|---------|
| `zenoh_bridge-as-bidirectional-client.json5` | ros2 container — `zenoh-bridge-ros2dds` |
| `zenoh_client-as-docker.json5` | rust container — `main_sub` + `main_pub` |
| `edge_agent-sub.yaml` | rust container — subscriber keyexpr / discover |
| `edge_agent-demo-pub.yaml` | rust container — publisher keyexpr / payload |

Override paths via env vars in `docker-compose.yml` or `docker/*/run-demo.sh`.
