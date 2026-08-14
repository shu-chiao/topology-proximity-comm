#!/usr/bin/env bash
# Run talker + listener inside the container for a bounded demo window.
set -eo pipefail

DURATION="${DEMO_DURATION_SEC:-6}"

set +u
source /opt/ros/jazzy/setup.bash
source /ws/cpp/install/setup.bash
export RCUTILS_CONSOLE_STDOUT_LINE_BUFFERED=1
set -u

echo "(docker demo) ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-unset}  duration=${DURATION}s"
echo "(docker demo) topic=/demo/chatter  nodes=talker + listener"

ros2 run demo_nodes listener &
listener_pid=$!

cleanup() {
  kill "$listener_pid" 2>/dev/null || true
  wait "$listener_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

sleep 1
timeout "${DURATION}" stdbuf -oL ros2 run demo_nodes talker || [[ $? -eq 124 ]]
