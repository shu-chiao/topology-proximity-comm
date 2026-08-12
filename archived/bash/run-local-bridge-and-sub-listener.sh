#!/usr/bin/env bash
# **`zenoh_bridge-as-sub-client.json5`** + **`listener`** (**`/chatter`** demo); **`allow.subscribers: ["/.*"]`**. **`ZENOH_BRIDGE_CONFIG`** to narrow topics.
# Other terminal:  MAIN_PUB_ROUTER=tcp/127.0.0.1:7447 ROS_MSG_TYPE=std_msgs/msg/String MAIN_PUB_KEYEXPR=chatter cargo run --bin main_pub
#
# Forces **`ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST`** unless **`ZENOH_EDGE_AGENT_BRIDGE_SUBNET_DDS=1`**.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${ZENOH_BRIDGE_CONFIG:-$ROOT/configs/zenoh_bridge-as-sub-client.json5}"

unset ROS_LOCALHOST_ONLY || true
[[ "${ZENOH_EDGE_AGENT_BRIDGE_SUBNET_DDS:-0}" == "1" ]] || export ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST

if [[ -z "${RMW_IMPLEMENTATION:-}" ]]; then
  export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp
fi

: "${ROS_DISTRO:=jazzy}"
if [[ -f "/opt/ros/${ROS_DISTRO}/setup.bash" ]]; then
  set +u
  # shellcheck source=/dev/null
  source "/opt/ros/${ROS_DISTRO}/setup.bash"
  set -u
else
  echo "warning: /opt/ros/${ROS_DISTRO}/setup.bash not found" >&2
fi

command -v ros2 &>/dev/null || {
  echo "error: ros2 not on PATH" >&2
  exit 1
}
command -v zenoh-bridge-ros2dds &>/dev/null || {
  echo "error: zenoh-bridge-ros2dds not on PATH (see docs/ros2-bridge.md)" >&2
  exit 1
}
[[ -f "$CONFIG" ]] || {
  echo "error: bridge config missing: $CONFIG" >&2
  exit 1
}

cleanup() {
  if [[ -n "${BRIDGE_PID:-}" ]] && kill -0 "$BRIDGE_PID" 2>/dev/null; then
    kill "$BRIDGE_PID" 2>/dev/null || true
    wait "$BRIDGE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "(listener script) zenoh-bridge ← $CONFIG  |  ROS_AUTOMATIC_DISCOVERY_RANGE=${ROS_AUTOMATIC_DISCOVERY_RANGE:-unset}"
zenoh-bridge-ros2dds --no-multicast-scouting -c "$CONFIG" &
BRIDGE_PID=$!
sleep 1
ros2 run demo_nodes_cpp listener
