//! CDR encoding for std_msgs/msg/String (main_pub / main_sub and bridge).

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

/// Decode `std_msgs/msg/String` CDR from zenoh-bridge-ros2dds (LE, 4-byte header).
pub fn ros2_std_msgs_string_cdr_decode(bytes: &[u8]) -> anyhow::Result<String> {
    if bytes.len() < 8 {
        anyhow::bail!("std_msgs/msg/String CDR: too short ({} bytes)", bytes.len());
    }
    if bytes[0..4] != DDS_CDR_LE_HEADER {
        anyhow::bail!("std_msgs/msg/String CDR: unexpected header {:?}", &bytes[0..4]);
    }
    let inner_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let end = 8usize
        .checked_add(inner_len)
        .ok_or_else(|| anyhow::anyhow!("std_msgs/msg/String CDR: length overflow"))?;
    if bytes.len() < end {
        anyhow::bail!(
            "std_msgs/msg/String CDR: truncated (need {end} bytes, got {})",
            bytes.len()
        );
    }
    let data = &bytes[8..end];
    if inner_len == 0 {
        return Ok(String::new());
    }
    if data.last() != Some(&0) {
        anyhow::bail!("std_msgs/msg/String CDR: missing NUL terminator");
    }
    std::str::from_utf8(&data[..inner_len - 1])
        .map(|s| s.to_owned())
        .map_err(|e| anyhow::anyhow!("std_msgs/msg/String CDR: invalid UTF-8: {e}"))
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

    #[test]
    fn roundtrip() {
        for s in ["", "Hello 0", "Hello from Rust", "Hello World"] {
            let cdr = ros2_std_msgs_string_cdr_utf8(s).unwrap();
            assert_eq!(ros2_std_msgs_string_cdr_decode(&cdr).unwrap(), s);
        }
    }
}
