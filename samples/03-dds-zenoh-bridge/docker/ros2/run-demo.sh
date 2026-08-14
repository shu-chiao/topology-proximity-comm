#!/usr/bin/env bash
# ROS 2 leg: zenohd + bridge, then phased talker → listener demo.
set -eo pipefail

PHASE1_SEC=$(printf '%.0f' "${PHASE1_SEC:-5}")
PHASE2_SEC=$(printf '%.0f' "${PHASE2_SEC:-4}")
RUST_WAIT=$(printf '%.0f' "${RUST_WAIT_ROS_SEC:-3}")
GRACE=$(printf '%.0f' "${ROS_GRACE_SEC:-2}")
BRIDGE_CFG="${ZENOH_BRIDGE_CONFIG:-/ws/configs/zenoh_bridge-as-bidirectional-client.json5}"

set +u
source /opt/ros/jazzy/setup.bash
source /ws/cpp/install/setup.bash
export RMW_IMPLEMENTATION="${RMW_IMPLEMENTATION:-rmw_cyclonedds_cpp}"
export ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST
export RUST_LOG="${RUST_LOG:-warn}"
export RCUTILS_CONSOLE_STDOUT_LINE_BUFFERED=1
set -u

echo "(ros2 demo) ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-unset}  phase1=${PHASE1_SEC}s  phase2=${PHASE2_SEC}s"
echo "(ros2 demo) bridge=${BRIDGE_CFG}  nodes=zenohd + bridge + talker (phase1) + listener (phase2)"

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
  zenohd >/dev/null 2>&1 &
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
zenoh-bridge-ros2dds --no-multicast-scouting -c "$BRIDGE_CFG" >/dev/null 2>&1 &
bridge_pid=$!

sleep 1.5

wait_after_bridge=$(python3 -c "print(max(0.0, float('${RUST_WAIT}') - 1.5))")
if awk "BEGIN {exit !(${wait_after_bridge} > 0)}"; then
  sleep "${wait_after_bridge}"
fi

echo "(ros2 demo) === phase 1: ROS talker → Rust sub (${PHASE1_SEC}s) ==="
timeout "${PHASE1_SEC}" stdbuf -oL ros2 run demo_nodes talker || [[ $? -eq 124 ]]
echo "(ros2 demo) --- phase 1 done (ROS talker) ---"

echo "(ros2 demo) === phase 2: Rust pub → ROS listener (${PHASE2_SEC}s) ==="
stdbuf -oL ros2 run demo_nodes listener &
listener_pid=$!
sleep 1
sleep "${PHASE2_SEC}"
echo "(ros2 demo) --- phase 2 done (ROS listener) ---"

echo "(ros2 demo) holding bridge for ${GRACE}s …"
sleep "${GRACE}"
