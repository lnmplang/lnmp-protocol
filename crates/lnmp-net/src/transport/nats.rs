//! NATS transport bindings for LNMP-Net.
//!
//! This module provides helpers to map LNMP-Net metadata to/from NATS headers,
//! complementing the existing lnmp-transport NATS bindings.

use crate::error::{NetError, Result};
use crate::kind::MessageKind;
use crate::message::NetMessage;
use std::collections::HashMap;
use std::str::FromStr;

/// NATS header name for LNMP-Net message kind.
pub const HEADER_KIND: &str = "lnmp-kind";

/// NATS header name for LNMP-Net message priority.
pub const HEADER_PRIORITY: &str = "lnmp-priority";

/// NATS header name for LNMP-Net message TTL.
pub const HEADER_TTL: &str = "lnmp-ttl";

/// NATS header name for LNMP-Net message class.
pub const HEADER_CLASS: &str = "lnmp-class";

/// Adds LNMP-Net metadata to NATS headers.
///
/// Returns a HashMap suitable for NATS message headers.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::nats::net_to_nats_headers;
///
/// let nats_headers = net_to_nats_headers(&msg);
/// // Add to NATS message
/// ```
pub fn net_to_nats_headers(msg: &NetMessage) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    headers.insert(HEADER_KIND.to_string(), msg.kind.to_string());
    headers.insert(HEADER_PRIORITY.to_string(), msg.priority.to_string());
    headers.insert(HEADER_TTL.to_string(), msg.ttl_ms.to_string());

    if let Some(ref class) = msg.class {
        headers.insert(HEADER_CLASS.to_string(), class.clone());
    }

    headers
}

/// Extracts LNMP-Net metadata from NATS headers.
///
/// Returns (kind, priority, ttl_ms, class) tuple. Missing headers use defaults.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::nats::nats_headers_to_net_meta;
///
/// let (kind, priority, ttl_ms, class) = nats_headers_to_net_meta(&headers)?;
/// ```
pub fn nats_headers_to_net_meta(
    headers: &HashMap<String, String>,
) -> Result<(MessageKind, u8, u32, Option<String>)> {
    // Parse kind (default to Event)
    let kind = if let Some(kind_str) = headers.get(HEADER_KIND) {
        MessageKind::from_str(kind_str)
            .map_err(|e| NetError::Other(format!("Invalid kind: {}", e)))?
    } else {
        MessageKind::default()
    };

    // Parse priority (default to kind's default)
    let priority = if let Some(priority_str) = headers.get(HEADER_PRIORITY) {
        priority_str
            .parse()
            .map_err(|_| NetError::InvalidPriority("Parse error".into()))?
    } else {
        kind.default_priority()
    };

    // Parse TTL (default to kind's default)
    let ttl_ms = if let Some(ttl_str) = headers.get(HEADER_TTL) {
        ttl_str
            .parse()
            .map_err(|_| NetError::InvalidTTL("Parse error".into()))?
    } else {
        kind.default_ttl_ms()
    };

    // Parse class (optional)
    let class = headers.get(HEADER_CLASS).cloned();

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
    fn test_net_to_nats_headers() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Query)
            .priority(120)
            .ttl_ms(5000)
            .class("analytics")
            .build();

        let headers = net_to_nats_headers(&msg);

        assert_eq!(headers.get("lnmp-kind"), Some(&"Query".to_string()));
        assert_eq!(headers.get("lnmp-priority"), Some(&"120".to_string()));
        assert_eq!(headers.get("lnmp-ttl"), Some(&"5000".to_string()));
        assert_eq!(headers.get("lnmp-class"), Some(&"analytics".to_string()));
    }

    #[test]
    fn test_net_to_nats_headers_no_class() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(100)
            .ttl_ms(5000)
            .build();

        let headers = net_to_nats_headers(&msg);

        assert_eq!(headers.len(), 3); // kind, priority, ttl only
        assert!(!headers.contains_key("lnmp-class"));
    }

    #[test]
    fn test_nats_headers_to_net_meta() {
        let mut headers = HashMap::new();
        headers.insert("lnmp-kind".to_string(), "State".to_string());
        headers.insert("lnmp-priority".to_string(), "110".to_string());
        headers.insert("lnmp-ttl".to_string(), "8000".to_string());
        headers.insert("lnmp-class".to_string(), "monitoring".to_string());

        let (kind, priority, ttl_ms, class) = nats_headers_to_net_meta(&headers).unwrap();

        assert_eq!(kind, MessageKind::State);
        assert_eq!(priority, 110);
        assert_eq!(ttl_ms, 8000);
        assert_eq!(class, Some("monitoring".to_string()));
    }

    #[test]
    fn test_nats_headers_to_net_meta_minimal() {
        let headers = HashMap::new(); // No headers

        let (kind, priority, ttl_ms, class) = nats_headers_to_net_meta(&headers).unwrap();

        assert_eq!(kind, MessageKind::Event);
        assert_eq!(priority, 100);
        assert_eq!(ttl_ms, 5000);
        assert_eq!(class, None);
    }

    #[test]
    fn test_nats_headers_roundtrip() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let original = NetMessageBuilder::new(envelope, MessageKind::Alert)
            .priority(255)
            .ttl_ms(1000)
            .class("critical")
            .build();

        // Encode
        let headers = net_to_nats_headers(&original);

        // Decode
        let (kind, priority, ttl_ms, class) = nats_headers_to_net_meta(&headers).unwrap();

        // Verify roundtrip
        assert_eq!(kind, MessageKind::Alert);
        assert_eq!(priority, 255);
        assert_eq!(ttl_ms, 1000);
        assert_eq!(class, Some("critical".to_string()));
    }
}
