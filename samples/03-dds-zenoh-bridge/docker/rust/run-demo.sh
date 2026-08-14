#!/usr/bin/env bash
# Rust leg: phased sub (ROS→Rust) then pub (Rust→ROS).
set -eo pipefail

PHASE1_SEC=$(printf '%.0f' "${PHASE1_SEC:-5}")
PHASE2_SEC=$(printf '%.0f' "${PHASE2_SEC:-4}")
WAIT_ROS_SEC=$(printf '%.0f' "${RUST_WAIT_ROS_SEC:-3}")
SUB_SEC=$((PHASE1_SEC + 1))

export MAIN_SUB_ROUTER="${MAIN_SUB_ROUTER:-tcp/127.0.0.1:7447}"
export MAIN_PUB_ROUTER="${MAIN_PUB_ROUTER:-tcp/127.0.0.1:7447}"
export MAIN_PUB_KEYEXPR="${MAIN_PUB_KEYEXPR:-demo/chatter}"
export ROS_MSG_TYPE="${ROS_MSG_TYPE:-std_msgs/msg/String}"
# Default must use single quotes — `{n}` breaks bash brace expansion in double quotes.
if [[ -z "${MAIN_PUB_PAYLOAD:-}" ]]; then
  export MAIN_PUB_PAYLOAD='Hello {n} from Rust'
fi
export MAIN_SUB_ZENOH_JSON5="${MAIN_SUB_ZENOH_JSON5:-/build/configs/zenoh_client-as-docker.json5}"
export MAIN_PUB_ZENOH_JSON5="${MAIN_PUB_ZENOH_JSON5:-/build/configs/zenoh_client-as-docker.json5}"
export EDGE_AGENT_SUB_YAML="${EDGE_AGENT_SUB_YAML:-/build/configs/edge_agent-sub.yaml}"
export EDGE_AGENT_PUB_YAML="${EDGE_AGENT_PUB_YAML:-/build/configs/edge_agent-demo-pub.yaml}"
export ZENOH_CONFIG_OVERRIDE="${ZENOH_CONFIG_OVERRIDE:-transport/shared_memory/enabled=false}"

echo "(rust demo) phase1=${PHASE1_SEC}s  phase2=${PHASE2_SEC}s  wait_for_ros=${WAIT_ROS_SEC}s"
echo "(rust demo) sub router=${MAIN_SUB_ROUTER}  pub keyexpr=${MAIN_PUB_KEYEXPR}"

echo "(rust demo) waiting ${WAIT_ROS_SEC}s for ros2 zenohd + bridge …"
sleep "${WAIT_ROS_SEC}"

echo "(rust demo) === phase 1: ROS talker → Rust sub (${PHASE1_SEC}s) ==="
timeout "${SUB_SEC}" main_sub || [[ $? -eq 124 ]]
echo "(rust demo) --- phase 1 done (Rust sub) ---"

echo "(rust demo) === phase 2: Rust pub → ROS listener (${PHASE2_SEC}s) ==="
sleep 1
timeout "${PHASE2_SEC}" main_pub || [[ $? -eq 124 ]]
echo "(rust demo) --- phase 2 done (Rust pub) ---"

sleep 1
