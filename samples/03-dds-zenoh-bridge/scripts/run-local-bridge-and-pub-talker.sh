#!/usr/bin/env bash
# Local publisher path: Zenoh bridge (client → zenohd) + demo talker, with DDS env aligned.
# Usage: from repo root or sample dir —  ./scripts/run-local-bridge-and-pub-talker.sh
# Override before run, e.g.  ROS_DOMAIN_ID=5 ./bash/run-local-bridge-and-pub-talker.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${ZENOH_BRIDGE_CONFIG:-$ROOT/configs/zenoh_bridge-as-pub-client.json5}"

# --- DDS / ROS 2 env (override by exporting before this script) ---
# ROS_DOMAIN_ID: omit for ROS middleware default (commonly domain 0). Bridge and ros2 nodes must match.
unset ROS_LOCALHOST_ONLY || true

# Optional middleware default (only if unset — don’t override a user choice)
if [[ -z "${RMW_IMPLEMENTATION:-}" ]]; then
  export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp
fi

# ROS distro for setup.bash (override: ROS_DISTRO=humble ./bash/… )
: "${ROS_DISTRO:=jazzy}"
if [[ -f "/opt/ros/${ROS_DISTRO}/setup.bash" ]]; then
  # ROS setup scripts assume optional vars exist; bash `set -u` breaks them (e.g. AMENT_TRACE_SETUP_FILES).
  set +u
  # shellcheck source=/dev/null
  source "/opt/ros/${ROS_DISTRO}/setup.bash"
  set -u
else
  echo "warning: /opt/ros/${ROS_DISTRO}/setup.bash not found — ensure ros2 is on PATH" >&2
fi

if ! command -v ros2 &>/dev/null; then
  echo "error: ros2 not found (source your ROS install or set ROS_DISTRO)" >&2
  exit 1
fi
if ! command -v zenoh-bridge-ros2dds &>/dev/null; then
  echo "error: zenoh-bridge-ros2dds not on PATH (apt install or see docs/prerequisites.md)" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "error: bridge config not found: $CONFIG" >&2
  exit 1
fi

BRIDGE_PID=""
cleanup() {
  if [[ -n "$BRIDGE_PID" ]] && kill -0 "$BRIDGE_PID" 2>/dev/null; then
    echo "Stopping zenoh-bridge-ros2dds (PID $BRIDGE_PID) …" >&2
    kill "$BRIDGE_PID" 2>/dev/null || true
    wait "$BRIDGE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "Repo:      $ROOT"
echo "Bridge:    $CONFIG"
echo "ROS_DISTRO=$ROS_DISTRO  ROS_DOMAIN_ID=${ROS_DOMAIN_ID:-<unset → middleware default>}  RMW_IMPLEMENTATION=$RMW_IMPLEMENTATION"
echo "Starting zenoh-bridge-ros2dds (background) …"
zenoh-bridge-ros2dds --no-multicast-scouting -c "$CONFIG" &
BRIDGE_PID=$!

# Brief pause so the bridge attaches before talker spins up (best-effort).
sleep 1

CPP_INSTALL="$ROOT/cpp/install/setup.bash"
if [[ -f "$CPP_INSTALL" ]]; then
  set +u
  # shellcheck source=/dev/null
  source "$CPP_INSTALL"
  set -u
else
  echo "warning: $CPP_INSTALL not found — build demo_nodes first (see README.md)" >&2
fi

echo "Starting talker (foreground, Ctrl+C stops talker and kills bridge) …"
ros2 run demo_nodes talker
