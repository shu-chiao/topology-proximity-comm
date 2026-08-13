//! ROS 2 service CDR and request headers for Zenoh queries.

use serde_yaml::{Mapping, Value};

use zenoh::bytes::ZBytes;

const DDS_CDR_LE_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

const ATTACHMENT_KEY_REQUEST_HEADER: [u8; 3] = *b"rqh";

/// Build the request header attachment for a service call.
pub fn cyclone_request_attachment(client_id: u64, seq_num: u64, little_endian: bool) -> ZBytes {
    let mut header = [0u8; 16];
    if little_endian {
        header[..8].copy_from_slice(&client_id.to_le_bytes());
        header[8..].copy_from_slice(&seq_num.to_le_bytes());
    } else {
        header[..8].copy_from_slice(&client_id.to_be_bytes());
        header[8..].copy_from_slice(&seq_num.to_be_bytes());
    }
    let mut tail = [0u8; 17];
    tail[0..16].copy_from_slice(&header);
    tail[16] = little_endian as u8;
    let mut w = ZBytes::writer();
    w.append(ZBytes::from(ATTACHMENT_KEY_REQUEST_HEADER));
    w.append(ZBytes::from(tail));
    w.finish()
}

/// Encode AddTwoInts request (a, b) as CDR.
pub fn example_interfaces_add_two_ints_request_cdr_le(a: i64, b: i64) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 8 + 8);
    out.extend_from_slice(&DDS_CDR_LE_HEADER);
    out.extend_from_slice(&a.to_le_bytes());
    out.extend_from_slice(&b.to_le_bytes());
    out
}

/// Decode AddTwoInts response sum from CDR bytes.
pub fn example_interfaces_add_two_ints_response_sum_cdr_le(bytes: &[u8]) -> anyhow::Result<i64> {
    if bytes.len() < 12 {
        anyhow::bail!(
            "AddTwoInts reply: expected at least 12 bytes (CDR hdr + sum), got {}",
            bytes.len()
        );
    }
    if bytes[1] > 1 {
        anyhow::bail!(
            "AddTwoInts reply: unsupported CDR submessage (byte[1]={})",
            bytes[1]
        );
    }
    Ok(i64::from_le_bytes(bytes[4..12].try_into()?))
}

fn yaml_mapping<'a>(v: &'a Value) -> anyhow::Result<&'a Mapping> {
    v.as_mapping()
        .ok_or_else(|| anyhow::anyhow!("service call `args`: expected YAML mapping/object, got {:?}", v))
}

fn yaml_i64(map: &Mapping, key: &str) -> anyhow::Result<i64> {
    let got = map
        .get(Value::String(key.into()))
        .ok_or_else(|| anyhow::anyhow!("service args: missing `{}`", key))?;
    match got {
        Value::Number(n) => n.as_i64().ok_or_else(|| anyhow::anyhow!("`{}`: not integer", key)),
        _ => anyhow::bail!("`{}`: expected number, got {:?}", key, got),
    }
}

/// Build request CDR from service type and YAML args.
pub fn ros2_service_request_cdr(service_type: &str, args: &Value) -> anyhow::Result<Vec<u8>> {
    let ty = service_type.trim();
    match ty {
        "example_interfaces/srv/AddTwoInts" => {
            let m = yaml_mapping(args)?;
            let a = yaml_i64(m, "a")?;
            let b = yaml_i64(m, "b")?;
            Ok(example_interfaces_add_two_ints_request_cdr_le(a, b))
        }
        unknown => anyhow::bail!(
            "unsupported ROS 2 service `type`: `{}` (only `example_interfaces/srv/AddTwoInts` is implemented)",
            unknown
        ),
    }
}

/// One-line summary of a service response for logging.
pub fn ros2_service_reply_summary(service_type: &str, payload: &[u8]) -> anyhow::Result<String> {
    match service_type.trim() {
        "example_interfaces/srv/AddTwoInts" => {
            let sum = example_interfaces_add_two_ints_response_sum_cdr_le(payload)?;
            Ok(format!("example_interfaces/srv/AddTwoInts: sum={sum}"))
        }
        unknown => anyhow::bail!("unsupported ROS 2 service type for decode: `{}`", unknown),
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::{Mapping, Value};

    use super::*;

    #[test]
    fn add_two_ints_request_matches_rclpy() {
        let expected: [u8; 20] = [
            0x00, 0x01, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            example_interfaces_add_two_ints_request_cdr_le(2, 3),
            Vec::from(expected)
        );
    }

    #[test]
    fn add_two_ints_response_roundtrip() {
        let rep: [u8; 12] = [
            0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(example_interfaces_add_two_ints_response_sum_cdr_le(&rep).unwrap(), 5);
    }

    #[test]
    fn ros2_dispatcher_add_two_ints() {
        let mut m = Mapping::new();
        m.insert(Value::from("a"), Value::from(2i64));
        m.insert(Value::from("b"), Value::from(3i64));
        let req = ros2_service_request_cdr("example_interfaces/srv/AddTwoInts", &Value::Mapping(m))
            .unwrap();
        assert_eq!(req.len(), 20);

        let rep: [u8; 12] = [
            0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let line = ros2_service_reply_summary("example_interfaces/srv/AddTwoInts", &rep).unwrap();
        assert!(line.contains("sum=5"), "{line}");
    }

    #[test]
    fn attachment_prefix_and_flattened_len() {
        let z = cyclone_request_attachment(0x0102_0304_0506_0708, 9, true);
        let flat = z.to_bytes();
        assert_eq!(flat.len(), 3 + 17);
        assert_eq!(&flat[0..3], b"rqh");
    }
}
