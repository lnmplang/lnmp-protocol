//! Kafka transport bindings for LNMP-Net.
//!
//! This module provides helpers to map LNMP-Net metadata to/from Kafka headers,
//! complementing the existing lnmp-transport Kafka bindings.

use crate::error::{NetError, Result};
use crate::kind::MessageKind;
use crate::message::NetMessage;
use std::str::FromStr;

/// Kafka header name for LNMP-Net message kind.
pub const HEADER_KIND: &str = "lnmp.kind";

/// Kafka header name for LNMP-Net message priority.
pub const HEADER_PRIORITY: &str = "lnmp.priority";

/// Kafka header name for LNMP-Net message TTL.
pub const HEADER_TTL: &str = "lnmp.ttl";

/// Kafka header name for LNMP-Net message class.
pub const HEADER_CLASS: &str = "lnmp.class";

/// Adds LNMP-Net metadata to Kafka headers.
///
/// Returns a Vec of (key, value) string pairs suitable for Kafka record headers.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::kafka::net_to_kafka_headers;
///
/// let kafka_headers = net_to_kafka_headers(&msg);
/// // Add to Kafka ProducerRecord headers
/// ```
pub fn net_to_kafka_headers(msg: &NetMessage) -> Vec<(String, String)> {
    let mut headers = Vec::new();

    headers.push((HEADER_KIND.to_string(), msg.kind.to_string()));
    headers.push((HEADER_PRIORITY.to_string(), msg.priority.to_string()));
    headers.push((HEADER_TTL.to_string(), msg.ttl_ms.to_string()));

    if let Some(ref class) = msg.class {
        headers.push((HEADER_CLASS.to_string(), class.clone()));
    }

    headers
}

/// Extracts LNMP-Net metadata from Kafka headers.
///
/// Returns (kind, priority, ttl_ms, class) tuple. Missing headers use defaults.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::kafka::kafka_headers_to_net_meta;
///
/// let (kind, priority, ttl_ms, class) = kafka_headers_to_net_meta(&headers)?;
/// ```
pub fn kafka_headers_to_net_meta(
    headers: &[(String, String)],
) -> Result<(MessageKind, u8, u32, Option<String>)> {
    let headers_map: std::collections::HashMap<&str, &str> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // Parse kind (default to Event)
    let kind = if let Some(&kind_str) = headers_map.get(HEADER_KIND) {
        MessageKind::from_str(kind_str)
            .map_err(|e| NetError::Other(format!("Invalid kind: {}", e)))?
    } else {
        MessageKind::default()
    };

    // Parse priority (default to kind's default)
    let priority = if let Some(&priority_str) = headers_map.get(HEADER_PRIORITY) {
        priority_str
            .parse()
            .map_err(|_| NetError::InvalidPriority("Parse error".into()))?
    } else {
        kind.default_priority()
    };

    // Parse TTL (default to kind's default)
    let ttl_ms = if let Some(&ttl_str) = headers_map.get(HEADER_TTL) {
        ttl_str
            .parse()
            .map_err(|_| NetError::InvalidTTL("Parse error".into()))?
    } else {
        kind.default_ttl_ms()
    };

    // Parse class (optional)
    let class = headers_map.get(HEADER_CLASS).map(|s| s.to_string());

    Ok((kind, priority, ttl_ms, class))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NetMessageBuilder;
    use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
    use lnmp_envelope::EnvelopeBuilder;

    fn sample_record() -> LnmpRecord {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 42,
            value: LnmpValue::Int(100),
        });
        record
    }

    #[test]
    fn test_net_to_kafka_headers() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::State)
            .priority(150)
            .ttl_ms(10000)
            .class("health")
            .build();

        let headers = net_to_kafka_headers(&msg);

        assert!(headers.contains(&("lnmp.kind".to_string(), "State".to_string())));
        assert!(headers.contains(&("lnmp.priority".to_string(), "150".to_string())));
        assert!(headers.contains(&("lnmp.ttl".to_string(), "10000".to_string())));
        assert!(headers.contains(&("lnmp.class".to_string(), "health".to_string())));
    }

    #[test]
    fn test_net_to_kafka_headers_no_class() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(100)
            .ttl_ms(5000)
            .build();

        let headers = net_to_kafka_headers(&msg);

        assert_eq!(headers.len(), 3); // kind, priority, ttl only
        assert!(!headers.iter().any(|(k, _)| k == "lnmp.class"));
    }

    #[test]
    fn test_kafka_headers_to_net_meta() {
        let headers = vec![
            ("lnmp.kind".to_string(), "Alert".to_string()),
            ("lnmp.priority".to_string(), "255".to_string()),
            ("lnmp.ttl".to_string(), "1000".to_string()),
            ("lnmp.class".to_string(), "safety".to_string()),
        ];

        let (kind, priority, ttl_ms, class) = kafka_headers_to_net_meta(&headers).unwrap();

        assert_eq!(kind, MessageKind::Alert);
        assert_eq!(priority, 255);
        assert_eq!(ttl_ms, 1000);
        assert_eq!(class, Some("safety".to_string()));
    }

    #[test]
    fn test_kafka_headers_to_net_meta_minimal() {
        let headers = vec![]; // No headers

        let (kind, priority, ttl_ms, class) = kafka_headers_to_net_meta(&headers).unwrap();

        assert_eq!(kind, MessageKind::Event);
        assert_eq!(priority, 100);
        assert_eq!(ttl_ms, 5000);
        assert_eq!(class, None);
    }

    #[test]
    fn test_kafka_headers_roundtrip() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let original = NetMessageBuilder::new(envelope, MessageKind::Command)
            .priority(180)
            .ttl_ms(2500)
            .class("control")
            .build();

        // Encode
        let headers = net_to_kafka_headers(&original);

        // Decode
        let (kind, priority, ttl_ms, class) = kafka_headers_to_net_meta(&headers).unwrap();

        // Verify roundtrip
        assert_eq!(kind, MessageKind::Command);
        assert_eq!(priority, 180);
        assert_eq!(ttl_ms, 2500);
        assert_eq!(class, Some("control".to_string()));
    }
}
