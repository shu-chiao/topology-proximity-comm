#!/usr/bin/env bash
# Rust leg: main_sub + main_pub (bounded demo window).
set -eo pipefail

DURATION="${DEMO_DURATION_SEC:-8}"
WAIT_ROS_SEC="${RUST_WAIT_ROS_SEC:-4}"

export MAIN_SUB_ROUTER="${MAIN_SUB_ROUTER:-tcp/127.0.0.1:7447}"
export MAIN_PUB_ROUTER="${MAIN_PUB_ROUTER:-tcp/127.0.0.1:7447}"
export MAIN_PUB_KEYEXPR="${MAIN_PUB_KEYEXPR:-demo/chatter}"
export ROS_MSG_TYPE="${ROS_MSG_TYPE:-std_msgs/msg/String}"
export MAIN_PUB_PAYLOAD="${MAIN_PUB_PAYLOAD:-Hello from Rust}"
export MAIN_SUB_ZENOH_JSON5="${MAIN_SUB_ZENOH_JSON5:-/build/configs/zenoh_client-as-docker.json5}"
export MAIN_PUB_ZENOH_JSON5="${MAIN_PUB_ZENOH_JSON5:-/build/configs/zenoh_client-as-docker.json5}"
export EDGE_AGENT_SUB_YAML="${EDGE_AGENT_SUB_YAML:-/build/configs/edge_agent-sub.yaml}"
export EDGE_AGENT_PUB_YAML="${EDGE_AGENT_PUB_YAML:-/build/configs/edge_agent-demo-pub.yaml}"
export ZENOH_CONFIG_OVERRIDE="${ZENOH_CONFIG_OVERRIDE:-transport/shared_memory/enabled=false}"

echo "(rust demo) duration=${DURATION}s  wait_for_ros=${WAIT_ROS_SEC}s"
echo "(rust demo) sub key=${MAIN_SUB_ROUTER}  pub keyexpr=${MAIN_PUB_KEYEXPR}"

sub_pid=""

cleanup() {
  kill "$sub_pid" 2>/dev/null || true
  wait "$sub_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "(rust demo) waiting ${WAIT_ROS_SEC}s for ros2 zenohd + bridge …"
sleep "${WAIT_ROS_SEC}"

main_sub &
sub_pid=$!

sleep 1
timeout "${DURATION}" main_pub || [[ $? -eq 124 ]]

sleep 1
