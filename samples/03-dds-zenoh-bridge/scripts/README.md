# scripts/ — local bridge demos

Helper scripts for Sample 3. Each starts `zenoh-bridge-ros2dds` in the background, then runs a ROS node.

| Script | Bridge config | ROS node |
|--------|---------------|----------|
| `run-local-bridge-and-pub-talker.sh` | `zenoh_bridge-as-pub-client.json5` | `demo_nodes talker` |
| `run-local-bridge-and-sub-listener.sh` | `zenoh_bridge-as-sub-client.json5` | `demo_nodes listener` |
| `run-local-bridge-and-srv-server.sh` | `zenoh_bridge-as-srv-server-client.json5` | upstream `demo_nodes_cpp` (advanced) |
| `run-local-bridge-and-action-server.sh` | action bridge config | turtlesim action (advanced) |

Build C++ nodes first:

```bash
cd samples/03-dds-zenoh-bridge/cpp && colcon build
```

Override bridge config: `ZENOH_BRIDGE_CONFIG=/path/to.json5 ./scripts/run-local-bridge-and-pub-talker.sh`
