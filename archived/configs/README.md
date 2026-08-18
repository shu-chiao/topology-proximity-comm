# `configs/` — naming

Paths resolve under **`archived/configs/`** (or absolute paths). Override files with env vars listed below.

## Env overrides

| Env var | Default file | Used by |
|---------|--------------|---------|
| **`EDGE_AGENT_YAML`** | `edge_agent.yaml` | `zenoh_agent` |
| **`EDGE_AGENT_SUB_YAML`** | `edge_agent-sub.yaml` | `main_sub` |
| **`EDGE_AGENT_PUB_YAML`** | `edge_agent-pub.yaml` | `main_pub` |
| **`MASTER_SRV_CLIENT_YAML`** | `master_srv-client.yaml` | `main_srv_client` |
| **`MASTER_ACTION_CLIENT_YAML`** | `master_action-client.yaml` | `main_action_client` |

Client binaries also accept **`MAIN_*`** overrides (router, keyexpr, timeouts, etc.) — see comments in each YAML file and the matching `src/bin/*.rs`.

---

## YAML (`*.yaml`)

```text
<who>-<task>.yaml
```

- **`<who>`** — e.g. **`edge_agent`** (agent + bridge) or **`master_srv`** / **`master_action`** (client presets). Multi-word **`who`** uses **`_`** (same idea as **`zenoh_agent`** in JSON5).
- **`<task>`** — sub-mode (**`sub`**, **`pub`**, **`srv-client`**, **`action-client`**). Omit **`-<task>`** for the default file (**`<who>.yaml`** only).

| File | Meaning |
|------|---------|
| **`edge_agent.yaml`** | **`cargo run --bin zenoh_agent`** (Zenoh + bridge spawn). |
| **`edge_agent-sub.yaml`** | **`main_sub`** / subscriber profile. |
| **`edge_agent-pub.yaml`** | **`main_pub`** / publisher profile. |
| **`master_srv-client.yaml`** | **`main_srv_client`** · ROS service over Zenoh. |
| **`master_action-client.yaml`** | **`main_action_client`** · ROS action over Zenoh. |

---

## JSON5 (`*.json5`)

```text
<who>-as-<task>-<role>.json5
```

- **`zenoh_agent`** — Rust **`zenoh`** session; **`zenoh_bridge`** — **`zenoh-bridge-ros2dds`** config (**`_`** keeps **`who`** one token).
- **`<task>`** — **`pub`**, **`sub`**, **`srv-server`**, **`action-server`**, **`local`**, … Agent “default” files drop the extra segment (**`-as-peer`** / **`-as-client`**).
- **`<role>`** — **`peer`** (listen / mesh) vs **`client`** (dial **`zenohd`**) vs **`router`**.

| File | who | task | Zenoh role |
|------|-----|------|------------|
| **`zenoh_agent-as-peer.json5`** | **`zenoh_agent`** | default | **`peer`** |
| **`zenoh_agent-as-client.json5`** | **`zenoh_agent`** | default | **`client`** |
| **`zenoh_bridge-as-local-peer.json5`** | **`zenoh_bridge`** | **`local`** (→ agent **`:7411`**) | **`peer`** |
| **`zenoh_bridge-as-pub-client.json5`** | **`zenoh_bridge`** | **`pub`** | **`client`** |
| **`zenoh_bridge-as-sub-client.json5`** | **`zenoh_bridge`** | **`sub`** | **`client`** |
| **`zenoh_bridge-as-srv-server-client.json5`** | **`zenoh_bridge`** | **`srv-server`** | **`client`** |
| **`zenoh_bridge-as-action-server-client.json5`** | **`zenoh_bridge`** | **`action-server`** | **`client`** |

---

## Quick lookup

```text
YAML:   <who>-<task>.yaml     (underscores in multi-word who, e.g. edge_agent, master_srv)
JSON5:  zenoh_agent-as-*.json5   |   zenoh_bridge-as-*.json5
```
