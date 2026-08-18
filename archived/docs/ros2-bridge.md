# ROS 2 bridge

`zenoh-bridge-ros2dds` connects ROS 2 (DDS) and Zenoh. Use it so ROS nodes and this Rust agent can talk through Zenoh.

Keep **zenoh-bridge-ros2dds**, **zenohd**, and the Rust **zenoh** crate on the same release line (**1.9.x** in this repo).

More JSON5 and `allow` options: [zenoh-plugin-ros2dds](https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds).

---

## Install

| What | Command |
|------|---------|
| Install via APT (Debian/Ubuntu) | `make install-zenoh-bridge` |
| Check if installed | `make check-bridge` |
| Install plugin for `zenohd` only | `make install-plugin` |

Manual APT or ZIP downloads: [Eclipse Zenoh releases](https://download.eclipse.org/zenoh/zenoh-plugin-ros2dds/latest/) (pick **`zenoh-bridge-ros2dds-…zip`**).

After install, confirm with:

```bash
zenoh-bridge-ros2dds --version
```

---

## Use with this repo

### Bridge modes

Set **`bridge.mode`** in [configs/edge_agent.yaml](../configs/edge_agent.yaml):

| Mode | Config file | Connects to |
|------|-------------|-------------|
| `local_peer` (default) | [zenoh_bridge-as-local-peer.json5](../configs/zenoh_bridge-as-local-peer.json5) | Local agent on `tcp/127.0.0.1:7411` |
| `client` | [zenoh_bridge-as-pub-client.json5](../configs/zenoh_bridge-as-pub-client.json5) | `zenohd` on `tcp/127.0.0.1:7447` (publishers only) |

### Agent settings

| Setting | What it does |
|---------|--------------|
| `skip_spawn: true` | Do not start the bridge; run it yourself. |
| `verbose_logs: true` | Show bridge INFO/WARN logs (default is quieter). |
| `mode` | `local_peer` or `client` — picks the config file above. |
| `executable` | Path or name of the bridge binary (default: `zenoh-bridge-ros2dds`). |
| `detach: true` | Start the bridge and leave it running after the agent exits. |
| `log_host_dds_env: true` | Print ROS/DDS-related env vars at startup. |

With default settings, **`cargo run --bin zenoh_agent`** starts the agent and spawns:

```bash
zenoh-bridge-ros2dds --no-multicast-scouting -c configs/zenoh_bridge-as-local-peer.json5
```

---

## Run the bridge yourself

From **`archived/`**:

```bash
# Default: talk to the local agent
zenoh-bridge-ros2dds --no-multicast-scouting \
  -c configs/zenoh_bridge-as-local-peer.json5

# Talk directly to zenohd
zenoh-bridge-ros2dds --no-multicast-scouting \
  -c configs/zenoh_bridge-as-pub-client.json5
```

---

## Local demos

Scripts under **`bash/`** for manual tests without **`cargo run`**:

| Demo | Script | Config |
|------|--------|--------|
| ROS talker → Zenoh | [run-local-bridge-and-pub-talker.sh](../bash/run-local-bridge-and-pub-talker.sh) | [zenoh_bridge-as-pub-client.json5](../configs/zenoh_bridge-as-pub-client.json5) |
| Zenoh → ROS listener | [run-local-bridge-and-sub-listener.sh](../bash/run-local-bridge-and-sub-listener.sh) | [zenoh_bridge-as-sub-client.json5](../configs/zenoh_bridge-as-sub-client.json5) |
| ROS service → Zenoh | [run-local-bridge-and-srv-server.sh](../bash/run-local-bridge-and-srv-server.sh) | [zenoh_bridge-as-srv-server-client.json5](../configs/zenoh_bridge-as-srv-server-client.json5) |
| ROS action → Zenoh | [run-local-bridge-and-action-server.sh](../bash/run-local-bridge-and-action-server.sh) | [zenoh_bridge-as-action-server-client.json5](../configs/zenoh_bridge-as-action-server-client.json5) |

For the service and action demos, set **`ZENOH_BRIDGE_ROUTER=tcp/<host>:7447`** if `zenohd` is not on localhost.

---

## Troubleshooting

| Symptom | Things to check |
|---------|-----------------|
| `(discover)` shows nothing | Samples must actually flow; empty topics alone do not print. |
| `(sub) routers=0` | `zenohd` not reachable on TCP 7447, wrong router address, or firewall. |
| Bridge and talker do not see each other | Match **`ROS_DOMAIN_ID`** with bridge JSON5 **`domain`**; unset **`ROS_LOCALHOST_ONLY=1`**. |
| Publisher not routed | Bridge logs should show `Route Publisher (ROS:/chatter -> Zenoh:…)`. |
| Remote subscriber silent | Bridge **`connect.endpoints`** and subscriber router must point at the same **`zenohd`**. |

Wire format details (CDR, key names): [zenoh-bridge-ros2dds-wire-contract.md](zenoh-bridge-ros2dds-wire-contract.md).
