#!/usr/bin/env bash
# ROS 2 leg: zenohd + bridge + listener + talker (bounded demo window).
set -eo pipefail

DURATION="${DEMO_DURATION_SEC:-8}"
BRIDGE_CFG="${ZENOH_BRIDGE_CONFIG:-/ws/configs/zenoh_bridge-as-bidirectional-client.json5}"

set +u
source /opt/ros/jazzy/setup.bash
source /ws/cpp/install/setup.bash
export RMW_IMPLEMENTATION="${RMW_IMPLEMENTATION:-rmw_cyclonedds_cpp}"
export ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST
set -u

echo "(ros2 demo) ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-unset}  duration=${DURATION}s"
echo "(ros2 demo) bridge=${BRIDGE_CFG}  nodes=zenohd + bridge + talker + listener"

router_pid=""
bridge_pid=""
listener_pid=""

router_listening() {
  timeout 1 bash -c 'echo >/dev/tcp/127.0.0.1/7447' 2>/dev/null
}

start_router_if_needed() {
  if router_listening; then
    echo "(ros2 demo) reusing existing Zenoh router on tcp/127.0.0.1:7447"
    return
  fi
  echo "(ros2 demo) starting zenohd on tcp/127.0.0.1:7447"
  zenohd &
  router_pid=$!
  for _ in $(seq 1 30); do
    if router_listening; then
      return
    fi
    sleep 0.2
  done
  echo "(ros2 demo) zenohd did not become ready on :7447" >&2
  exit 1
}

cleanup() {
  kill "$listener_pid" 2>/dev/null || true
  wait "$listener_pid" 2>/dev/null || true
  kill "$bridge_pid" 2>/dev/null || true
  wait "$bridge_pid" 2>/dev/null || true
  if [[ -n "$router_pid" ]]; then
    kill "$router_pid" 2>/dev/null || true
    wait "$router_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

start_router_if_needed

echo "(ros2 demo) starting zenoh-bridge-ros2dds"
zenoh-bridge-ros2dds --no-multicast-scouting -c "$BRIDGE_CFG" &
bridge_pid=$!

sleep 1.5

ros2 run demo_nodes listener &
listener_pid=$!

sleep 1
timeout "${DURATION}" ros2 run demo_nodes talker || [[ $? -eq 124 ]]

# Keep zenohd + bridge alive while the rust container finishes pub/sub.
RUST_WAIT=$(printf '%.0f' "${RUST_WAIT_ROS_SEC:-4}")
GRACE=$(printf '%.0f' "${ROS_GRACE_SEC:-4}")
echo "(ros2 demo) holding bridge for $((RUST_WAIT + GRACE))s for rust leg …"
sleep $((RUST_WAIT + GRACE))
