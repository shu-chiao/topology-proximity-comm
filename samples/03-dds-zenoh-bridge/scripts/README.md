# scripts/ — advanced reference (host only)

These shell scripts predate the Docker demo. **Use [../README.md](../README.md) Docker instructions for the main sample 3 walkthrough.**

They require a host ROS Jazzy install, `zenoh-bridge-ros2dds`, and a built `cpp/` workspace — not used by the notebook or `docker compose` flow.

| Script | Bridge config | ROS node |
|--------|---------------|----------|
| `run-local-bridge-and-pub-talker.sh` | `zenoh_bridge-as-pub-client.json5` | `demo_nodes talker` |
| `run-local-bridge-and-sub-listener.sh` | `zenoh_bridge-as-sub-client.json5` | `demo_nodes listener` |
| `run-local-bridge-and-srv-server.sh` | `zenoh_bridge-as-srv-server-client.json5` | upstream service demo (advanced) |
| `run-local-bridge-and-action-server.sh` | action bridge config | turtlesim action (advanced) |
