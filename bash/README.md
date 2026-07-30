# Bash helpers

**`run-local-bridge-and-*`** scripts share: **`ROS_DISTRO`** (default **`jazzy`**), **`RMW_IMPLEMENTATION`** (**`rmw_cyclonedds_cpp`**), **`ROS_DOMAIN_ID`**, **`ZENOH_BRIDGE_CONFIG`**. Source ROS **`setup.bash`** under **`set +u`** (**`set -u`** breaks it).

## `run-local-bridge-and-pub-talker.sh`

Talker → Zenoh with **`configs/zenoh_bridge-as-pub-client.json5`**. Expects **`zenohd`** on **`tcp/127.0.0.1:7447`**.

```bash
./bash/run-local-bridge-and-pub-talker.sh
```

## `run-local-bridge-and-sub-listener.sh`

Zenoh → ROS (**`zenoh_bridge-as-sub-client.json5`**, **`allow.subscribers: ["/.*"]`**). Demo **`listener`** on **`/chatter`**. **`ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST`** unless **`ZENOH_EDGE_AGENT_BRIDGE_SUBNET_DDS=1`**.

```bash
./bash/run-local-bridge-and-sub-listener.sh
```

```bash
MAIN_PUB_ROUTER=tcp/127.0.0.1:7447 ROS_MSG_TYPE=std_msgs/msg/String MAIN_PUB_KEYEXPR=chatter MAIN_PUB_PAYLOAD="Hello World" cargo run --bin main_pub
```

## `run-local-bridge-and-srv-server.sh`

**`add_two_ints_server`** on **`/add_two_ints`** with **`configs/zenoh_bridge-as-srv-server-client.json5`**. Finds **`zenohd`** via LAN multicast unless **`ZENOH_BRIDGE_ROUTER=tcp/<host>:7447`**. DDS scope: **`ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST`** unless **`ZENOH_EDGE_AGENT_BRIDGE_SUBNET_DDS=1`**.

```bash
./bash/run-local-bridge-and-srv-server.sh
# ZENOH_BRIDGE_ROUTER=tcp/192.168.1.10:7447 ./bash/run-local-bridge-and-srv-server.sh
```

## `run-local-bridge-and-action-server.sh`

**`turtlesim`** action server on **`/turtle1/rotate_absolute`** (**`turtlesim/action/RotateAbsolute`**) with **`configs/zenoh_bridge-as-action-server-client.json5`**. Same router and DDS env as the service script above.

```bash
./bash/run-local-bridge-and-action-server.sh
# ZENOH_BRIDGE_ROUTER=tcp/192.168.1.10:7447 ./bash/run-local-bridge-and-action-server.sh
```

Test from another machine (after `zenohd` is up):

```bash
ros2 action send_goal /turtle1/rotate_absolute turtlesim/action/RotateAbsolute "{theta: 1.57}"
# or:
cargo run --bin main_action_client
```
