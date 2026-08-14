#!/usr/bin/env bash
# Run rmw_zenohd + talker + listener inside the container for a bounded demo window.
set -eo pipefail

DURATION="${DEMO_DURATION_SEC:-6}"

set +u
source /opt/ros/jazzy/setup.bash
source /ws/cpp/install/setup.bash
export RMW_IMPLEMENTATION=rmw_zenoh_cpp
export ZENOH_SESSION_CONFIG_URI=/ws/configs/zenoh-client.json5
export ZENOH_CONFIG_OVERRIDE='transport/shared_memory/enabled=false'
export RUST_LOG="${RUST_LOG:-warn}"
export RCUTILS_CONSOLE_STDOUT_LINE_BUFFERED=1
set -u

echo "(docker demo) ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-unset}  duration=${DURATION}s"
echo "(docker demo) RMW=rmw_zenoh_cpp  topic=/demo/chatter  nodes=rmw_zenohd + talker + listener"

router_pid=""
listener_pid=""

router_listening() {
  timeout 1 bash -c 'echo >/dev/tcp/127.0.0.1/7447' 2>/dev/null
}

start_router_if_needed() {
  if router_listening; then
    echo "(docker demo) reusing existing Zenoh router on tcp/127.0.0.1:7447"
    return
  fi
  echo "(docker demo) starting rmw_zenohd on tcp/127.0.0.1:7447"
  ros2 run rmw_zenoh_cpp rmw_zenohd >/dev/null 2>&1 &
  router_pid=$!
  for _ in $(seq 1 30); do
    if router_listening; then
      return
    fi
    sleep 0.2
  done
  echo "(docker demo) rmw_zenohd did not become ready on :7447" >&2
  exit 1
}

cleanup() {
  kill "$listener_pid" 2>/dev/null || true
  wait "$listener_pid" 2>/dev/null || true
  if [[ -n "$router_pid" ]]; then
    kill "$router_pid" 2>/dev/null || true
    wait "$router_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

start_router_if_needed

sleep 0.5

stdbuf -oL ros2 run demo_nodes listener &
listener_pid=$!

sleep 1
timeout "${DURATION}" stdbuf -oL ros2 run demo_nodes talker || [[ $? -eq 124 ]]
