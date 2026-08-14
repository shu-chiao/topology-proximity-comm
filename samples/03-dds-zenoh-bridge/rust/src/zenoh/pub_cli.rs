//! Publisher loop for main_pub (UTF-8 or ROS String CDR).

use std::time::Duration;

use zenoh::bytes::ZBytes;

use crate::ResolvedEdgePub;
use crate::wire::ros_msg_cdr;
use super::sub_cli::{apply_router_connect_override, zenoh_config_for_subscriber_from_file};

#[derive(Debug, Clone, Copy)]
enum MainPubWire {
    Utf8Framed,
    Ros2StdMsgsString,
}

fn resolve_ros_msg_wire(ros_msg_type: &Option<String>) -> anyhow::Result<MainPubWire> {
    match ros_msg_type {
        None => Ok(MainPubWire::Utf8Framed),
        Some(t) => match t.trim() {
            "" => anyhow::bail!("ROS_MSG_TYPE is set but empty"),
            "std_msgs/msg/String" => Ok(MainPubWire::Ros2StdMsgsString),
            other => anyhow::bail!(
                "unsupported ROS_MSG_TYPE `{other}` (only `std_msgs/msg/String` is implemented)"
            ),
        },
    }
}

/// Settings for the publisher run loop.
#[derive(Debug, Clone)]
pub struct PublisherCliArgs {
    pub config_path: std::path::PathBuf,
    pub keyexpr: String,
    pub payload: String,
    pub period: Duration,
    pub router_connect_override: Option<String>,
    pub ros_msg_type: Option<String>,
}

pub fn publisher_cli_args(pub_edge: &ResolvedEdgePub) -> PublisherCliArgs {
    PublisherCliArgs {
        config_path: std::path::PathBuf::new(),
        keyexpr: pub_edge.keyexpr.clone(),
        payload: pub_edge.payload.clone(),
        period: Duration::from_millis(pub_edge.period_ms.max(1)),
        router_connect_override: None,
        ros_msg_type: None,
    }
}

/// Expand `{n}` in the payload template with a 0-based counter (matches ROS talker `Hello 0…`).
fn format_payload(template: &str, seq: u64) -> String {
    template.replace("{n}", &seq.saturating_sub(1).to_string())
}

pub async fn run(args: PublisherCliArgs) -> anyhow::Result<()> {
    let ke = args.keyexpr.trim();
    if ke.is_empty() {
        anyhow::bail!("publisher keyexpr is empty");
    }

    let wire = resolve_ros_msg_wire(&args.ros_msg_type)?;

    let mut config = zenoh_config_for_subscriber_from_file(&args.config_path)?;

    if let Some(ref ep_raw) = args.router_connect_override {
        let ep = ep_raw.trim();
        if !ep.is_empty() {
            apply_router_connect_override(&mut config, ep)?;
            println!("(pub) connect → `{ep}`");
        }
    }

    let session = zenoh::open(config)
        .await
        .map_err(|e| anyhow::anyhow!("zenoh open: {e}"))?;

    let publisher = session
        .declare_publisher(ke)
        .await
        .map_err(|e| anyhow::anyhow!("declare_publisher: {e}"))?;

    println!(
        "(pub) ZID {} key=`{}` every {:?} nbytes_stem={}{}",
        session.zid(),
        ke,
        args.period,
        args.payload.len(),
        args.ros_msg_type
            .as_deref()
            .map(|t| format!(" ROS_MSG_TYPE={t}"))
            .unwrap_or_default(),
    );

    let mut ticker = tokio::time::interval(args.period);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut seq: u64 = 0;

    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => break,
            _ = ticker.tick() => {
                seq += 1;
                let text = format_payload(&args.payload, seq);
                match wire {
                    MainPubWire::Ros2StdMsgsString => {
                        let body = ros_msg_cdr::ros2_std_msgs_string_cdr_utf8(&text)?;
                        let n = body.len();
                        publisher
                            .put(ZBytes::from(body))
                            .encoding(zenoh::bytes::Encoding::ZENOH_BYTES)
                            .await
                            .map_err(|e| anyhow::anyhow!("put: {e}"))?;
                        println!(
                            "[pub] put seq={seq} text='{text}' nbytes={n} encoding=zenoh/bytes key=`{ke}`"
                        );
                    }
                    MainPubWire::Utf8Framed => {
                        let line = format!("#{} {}\n", seq, text);
                        publisher
                            .put(line.as_str())
                            .await
                            .map_err(|e| anyhow::anyhow!("put: {e}"))?;
                        println!(
                            "[pub] put seq={seq} text='{text}' nbytes={} key=`{ke}`",
                            line.len(),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_payload;

    #[test]
    fn payload_template_numbered() {
        assert_eq!(
            format_payload("Hello {n} from Rust", 1),
            "Hello 0 from Rust"
        );
        assert_eq!(
            format_payload("Hello {n} from Rust", 4),
            "Hello 3 from Rust"
        );
        assert_eq!(format_payload("static", 2), "static");
    }
}
