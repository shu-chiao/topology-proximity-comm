# Wire format (Zenoh ↔ ROS 2)

How topic and service data is named and encoded when using **`zenoh-bridge-ros2dds`**. Use this when writing Rust publishers/subscribers that must match the bridge.

Upstream reference: [zenoh-plugin-ros2dds](https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds).

---

## Topic names

| ROS topic | Zenoh key (default `namespace: "/"`) |
|-----------|--------------------------------------|
| `/chatter` | `chatter` |

The leading `/` is dropped. With **`namespace: "/bot"`**, `/chatter` becomes **`bot/chatter`**.

DDS may still use names like **`rt/chatter`** internally.

---

## Messages (topics)

Typed ROS messages travel as **CDR** bytes.

Example — **`std_msgs/msg/String`**:

- CDR header: `00 01 00 00`
- Then string length, UTF-8 text, and a trailing `NUL`

Rust helpers: [src/utils/ros_msg_cdr.rs](../src/utils/ros_msg_cdr.rs).

### Encoding

The bridge often reports encoding **`0`**. This repo’s **`main_pub`** sends **`zenoh/bytes`** on the CDR path.

---

## Try it: publish to ROS

```bash
ROS_MSG_TYPE=std_msgs/msg/String MAIN_PUB_KEYEXPR=chatter cargo run --bin main_pub
```

Then run the listener demo:

```bash
bash/run-local-bridge-and-sub-listener.sh
```

Without **`ROS_MSG_TYPE`**, payloads stay plain UTF-8 text (`#seq …`).

Subscriber example: **`cargo run --bin main_sub`**.

---

## Services

Zenoh keys drop the leading `/` the same way as topics: **`/add_two_ints`** → **`add_two_ints`**.

| Part | Contents |
|------|----------|
| **Payload** | CDR request or response body (no Cyclone request header inside the payload) |
| **Attachment** | Request header (`rqh` + 16-byte GUID/seq + endian flag) |

Example — **`example_interfaces/srv/AddTwoInts`**:

- Request: two **`int64`** fields (`a`, `b`)
- Response: one **`int64`** field (`sum`)

Rust helpers: [src/utils/ros_srv_cdr.rs](../src/utils/ros_srv_cdr.rs).

Try the client:

```bash
cargo run --bin main_srv_client
```

Config: [configs/master_srv-client.yaml](../configs/master_srv-client.yaml) (override with **`MASTER_SRV_CLIENT_YAML`**).

---

## Actions

Zenoh keys drop the leading `/` like services: **`/turtle1/rotate_absolute`** → base **`turtle1/rotate_absolute`**.

| Step | Zenoh key |
|------|-----------|
| send_goal | `turtle1/rotate_absolute/_action/send_goal` |
| get_result | `turtle1/rotate_absolute/_action/get_result` |

Same as services: CDR payload + `rqh` attachment on each query.

Example — **`turtlesim/action/RotateAbsolute`**:

- Goal: **`theta`** (`float`)
- Feedback: **`remaining`** (`float`, radians left) — bridge may wrap with **`goal_id`** (24-byte CDR)
- Result: **`delta`** (`float`)

Rust helpers: [src/utils/ros_action_cdr.rs](../src/utils/ros_action_cdr.rs).

Try the client (with [run-local-bridge-and-action-server.sh](../bash/run-local-bridge-and-action-server.sh) on the edge):

```bash
cargo run --bin main_action_client
```

Config: [configs/master_action-client.yaml](../configs/master_action-client.yaml) (override with **`MASTER_ACTION_CLIENT_YAML`**).
