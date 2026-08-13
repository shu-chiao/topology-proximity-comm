//! CDR encoding for std_msgs/msg/String (main_pub and bridge).

const DDS_CDR_LE_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

pub fn ros2_std_msgs_string_cdr_utf8(payload: &str) -> anyhow::Result<Vec<u8>> {
    if payload.as_bytes().iter().any(|&b| b == 0) {
        anyhow::bail!("std_msgs/msg/String: payload must not contain embedded NUL");
    }
    let inner_len: u32 = (payload.len() + 1)
        .try_into()
        .map_err(|_| anyhow::anyhow!("std_msgs/msg/String: payload too long"))?;
    let mut out = Vec::with_capacity(4 + 4 + payload.len() + 1);
    out.extend_from_slice(&DDS_CDR_LE_HEADER);
    out.extend_from_slice(&inner_len.to_le_bytes());
    out.extend_from_slice(payload.as_bytes());
    out.push(0);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn vectors() {
        assert_eq!(
            hex(&ros2_std_msgs_string_cdr_utf8("").unwrap()),
            "000100000100000000"
        );
        assert_eq!(
            hex(&ros2_std_msgs_string_cdr_utf8("Hello World").unwrap()),
            "000100000c00000048656c6c6f20576f726c6400"
        );
        assert!(ros2_std_msgs_string_cdr_utf8("a\0b").is_err());
    }
}
