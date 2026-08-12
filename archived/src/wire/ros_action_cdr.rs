//! ROS 2 action CDR helpers for Zenoh queries (send_goal / get_result).

use serde_yaml::{Mapping, Value};

const DDS_CDR_LE_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

fn yaml_mapping<'a>(v: &'a Value) -> anyhow::Result<&'a Mapping> {
    v.as_mapping().ok_or_else(|| {
        anyhow::anyhow!("action goal: expected YAML mapping/object, got {:?}", v)
    })
}

fn yaml_f32(map: &Mapping, key: &str) -> anyhow::Result<f32> {
    let got = map
        .get(Value::String(key.into()))
        .ok_or_else(|| anyhow::anyhow!("action goal: missing `{}`", key))?;
    match got {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("`{}`: not a number", key))
            .map(|v| v as f32),
        _ => anyhow::bail!("`{}`: expected number, got {:?}", key, got),
    }
}

/// Encode `turtlesim/action/RotateAbsolute` send_goal request (goal_id + theta).
pub fn turtlesim_rotate_absolute_send_goal_cdr(goal_id: [u8; 16], theta: f32) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 16 + 4);
    out.extend_from_slice(&DDS_CDR_LE_HEADER);
    out.extend_from_slice(&goal_id);
    out.extend_from_slice(&theta.to_le_bytes());
    out
}

/// Encode get_result request (goal_id only).
pub fn action_get_result_request_cdr(goal_id: [u8; 16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 16);
    out.extend_from_slice(&DDS_CDR_LE_HEADER);
    out.extend_from_slice(&goal_id);
    out
}

pub struct SendGoalResponse {
    pub accepted: bool,
    pub stamp_sec: i32,
    pub stamp_nanosec: u32,
}

/// Decode send_goal response (accepted + stamp).
pub fn parse_send_goal_response_cdr(bytes: &[u8]) -> anyhow::Result<SendGoalResponse> {
    if bytes.len() < 16 {
        anyhow::bail!(
            "send_goal reply: expected at least 16 bytes, got {}",
            bytes.len()
        );
    }
    let accepted = u32::from_le_bytes(bytes[4..8].try_into()?) != 0;
    let stamp_sec = i32::from_le_bytes(bytes[8..12].try_into()?);
    let stamp_nanosec = u32::from_le_bytes(bytes[12..16].try_into()?);
    Ok(SendGoalResponse {
        accepted,
        stamp_sec,
        stamp_nanosec,
    })
}

pub struct GetResultResponse {
    pub status: i32,
    pub delta: f32,
}

/// Decode `RotateAbsolute` get_result response (status + delta).
pub fn turtlesim_rotate_absolute_get_result_cdr(bytes: &[u8]) -> anyhow::Result<GetResultResponse> {
    if bytes.len() < 12 {
        anyhow::bail!(
            "get_result reply: expected at least 12 bytes, got {}",
            bytes.len()
        );
    }
    Ok(GetResultResponse {
        status: i32::from_le_bytes(bytes[4..8].try_into()?),
        delta: f32::from_le_bytes(bytes[8..12].try_into()?),
    })
}

/// Decode `RotateAbsolute` feedback (remaining angle in radians).
///
/// Bridge may send either:
/// - 8 B: CDR header + `remaining` (plain `Feedback`)
/// - 24 B: CDR header + `goal_id` (16 B) + `remaining` (`*_FeedbackMessage`)
pub fn turtlesim_rotate_absolute_feedback_remaining_cdr(bytes: &[u8]) -> anyhow::Result<f32> {
    let remaining = if bytes.len() >= 24 && bytes.starts_with(&DDS_CDR_LE_HEADER) {
        f32::from_le_bytes(bytes[20..24].try_into()?)
    } else if bytes.len() >= 8 && bytes.starts_with(&DDS_CDR_LE_HEADER) {
        f32::from_le_bytes(bytes[4..8].try_into()?)
    } else if bytes.len() == 4 {
        f32::from_le_bytes(bytes.try_into()?)
    } else {
        anyhow::bail!(
            "feedback: expected 8 or 24 bytes (or 4 raw), got {} (hex {:02x?})",
            bytes.len(),
            &bytes[..bytes.len().min(16)]
        );
    };
    if !remaining.is_finite() || remaining.abs() > 100.0 {
        anyhow::bail!(
            "feedback remaining out of range ({remaining}); payload len={} hex {:02x?}",
            bytes.len(),
            &bytes[..bytes.len().min(24)]
        );
    }
    Ok(remaining)
}

/// One-line summary of action feedback for logging.
pub fn ros2_action_feedback_summary(action_type: &str, payload: &[u8]) -> anyhow::Result<String> {
    match action_type.trim() {
        "turtlesim/action/RotateAbsolute" => {
            let remaining = turtlesim_rotate_absolute_feedback_remaining_cdr(payload)?;
            Ok(format!("remaining={remaining:.4} rad"))
        }
        unknown => anyhow::bail!("unsupported ROS 2 action type for feedback decode: `{unknown}`"),
    }
}

/// Build send_goal CDR from action type and YAML goal fields.
pub fn ros2_action_send_goal_cdr(
    action_type: &str,
    goal_id: [u8; 16],
    goal: &Value,
) -> anyhow::Result<Vec<u8>> {
    match action_type.trim() {
        "turtlesim/action/RotateAbsolute" => {
            let m = yaml_mapping(goal)?;
            let theta = yaml_f32(m, "theta")?;
            Ok(turtlesim_rotate_absolute_send_goal_cdr(goal_id, theta))
        }
        unknown => anyhow::bail!(
            "unsupported ROS 2 action type `{unknown}` (only `turtlesim/action/RotateAbsolute` is implemented)"
        ),
    }
}

/// One-line summary of get_result payload for logging.
pub fn ros2_action_result_summary(action_type: &str, payload: &[u8]) -> anyhow::Result<String> {
    match action_type.trim() {
        "turtlesim/action/RotateAbsolute" => {
            let r = turtlesim_rotate_absolute_get_result_cdr(payload)?;
            Ok(format!(
                "turtlesim/action/RotateAbsolute: status={} delta={}",
                r.status, r.delta
            ))
        }
        unknown => anyhow::bail!("unsupported ROS 2 action type for decode: `{unknown}`"),
    }
}

/// Zenoh query key for an action sub-service (`send_goal`, `get_result`, …).
pub fn zenoh_action_service_key(action_name: &str, suffix: &str) -> String {
    let base = action_name
        .trim()
        .strip_prefix('/')
        .unwrap_or(action_name.trim());
    format!("{base}/{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::{Mapping, Value};

    const GOAL_ID: [u8; 16] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef,
    ];

    #[test]
    fn send_goal_matches_rclpy() {
        let req = turtlesim_rotate_absolute_send_goal_cdr(GOAL_ID, 1.57);
        assert_eq!(
            req.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "000100000123456789abcdef0123456789abcdefc3f5c83f"
        );
    }

    #[test]
    fn get_result_request_matches_rclpy() {
        let req = action_get_result_request_cdr(GOAL_ID);
        assert_eq!(
            req.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "000100000123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn parse_send_goal_response() {
        let bytes: [u8; 16] = [
            0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00,
        ];
        let r = parse_send_goal_response_cdr(&bytes).unwrap();
        assert!(r.accepted);
        assert_eq!(r.stamp_sec, 1);
        assert_eq!(r.stamp_nanosec, 2);
    }

    #[test]
    fn parse_get_result_response() {
        let bytes: [u8; 12] = [
            0x00, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f,
        ];
        let r = turtlesim_rotate_absolute_get_result_cdr(&bytes).unwrap();
        assert_eq!(r.status, 4);
        assert!((r.delta - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parse_feedback_response() {
        let bytes: [u8; 8] = [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f];
        let r = turtlesim_rotate_absolute_feedback_remaining_cdr(&bytes).unwrap();
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parse_feedback_message_with_goal_id() {
        let mut bytes = [0u8; 24];
        bytes[0..4].copy_from_slice(&DDS_CDR_LE_HEADER);
        bytes[4..20].copy_from_slice(&[
            0x1f, 0xa0, 0x5b, 0x6c, 0x06, 0xa5, 0xb3, 0x18, 0x01, 0x03, 0xe4, 0x76, 0x72, 0x5e,
            0x05, 0x0d,
        ]);
        bytes[20..24].copy_from_slice(&0.42f32.to_le_bytes());
        let r = turtlesim_rotate_absolute_feedback_remaining_cdr(&bytes).unwrap();
        assert!((r - 0.42).abs() < 1e-5);
    }

    #[test]
    fn dispatcher_rotate_absolute() {
        let mut m = Mapping::new();
        m.insert(Value::from("theta"), Value::from(1.57f64));
        let req = ros2_action_send_goal_cdr(
            "turtlesim/action/RotateAbsolute",
            GOAL_ID,
            &Value::Mapping(m),
        )
        .unwrap();
        assert_eq!(req.len(), 24);
    }
}
