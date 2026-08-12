#!/usr/bin/env bash
# zenoh_bridge-as-action-server-client.json5 + ros2 run turtlesim turtlesim_node
#   action: /turtle1/rotate_absolute  (turtlesim/action/RotateAbsolute)
# Router: multicast to zenohd, or ZENOH_BRIDGE_ROUTER=tcp/<host>:7447 for TCP-only.
# ZENOH_BRIDGE_CONFIG, ROS_DISTRO. ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST unless ZENOH_EDGE_AGENT_BRIDGE_SUBNET_DDS=1.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${ZENOH_BRIDGE_CONFIG:-$ROOT/configs/zenoh_bridge-as-action-server-client.json5}"

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

BRIDGE_EXTRA=()
if [[ -n "${ZENOH_BRIDGE_ROUTER:-}" ]]; then
  BRIDGE_EXTRA+=(--no-multicast-scouting --connect "${ZENOH_BRIDGE_ROUTER}")
  echo "(rotate_absolute script) zenoh-bridge ← $CONFIG  explicit ZENOH_BRIDGE_ROUTER=${ZENOH_BRIDGE_ROUTER}"
else
  echo "(rotate_absolute script) zenoh-bridge ← $CONFIG  (Zenoh multicast scouting)  ROS_AUTOMATIC_DISCOVERY_RANGE=${ROS_AUTOMATIC_DISCOVERY_RANGE:-unset}"
fi

zenoh-bridge-ros2dds "${BRIDGE_EXTRA[@]}" -c "$CONFIG" &
BRIDGE_PID=$!
sleep 1
ros2 run turtlesim turtlesim_node
