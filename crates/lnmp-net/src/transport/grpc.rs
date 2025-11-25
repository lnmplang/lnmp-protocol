//! gRPC transport bindings for LNMP-Net.
//!
//! This module provides helpers to map LNMP-Net metadata to/from gRPC metadata,
//! complementing the existing lnmp-transport gRPC bindings.

use crate::error::{NetError, Result};
use crate::kind::MessageKind;
use crate::message::NetMessage;
use std::str::FromStr;

/// gRPC metadata key for LNMP-Net message kind.
/// Note: gRPC metadata keys must be lowercase.
pub const METADATA_KIND: &str = "lnmp-kind";

/// gRPC metadata key for LNMP-Net message priority.
pub const METADATA_PRIORITY: &str = "lnmp-priority";

/// gRPC metadata key for LNMP-Net message TTL.
pub const METADATA_TTL: &str = "lnmp-ttl";

/// gRPC metadata key for LNMP-Net message class.
pub const METADATA_CLASS: &str = "lnmp-class";

/// Adds LNMP-Net metadata to gRPC metadata.
///
/// Returns a Vec of (key, value) string pairs suitable for gRPC metadata.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::grpc::net_to_grpc_metadata;
///
/// let grpc_metadata = net_to_grpc_metadata(&msg);
/// // Add to gRPC request metadata
/// ```
pub fn net_to_grpc_metadata(msg: &NetMessage) -> Vec<(String, String)> {
    let mut metadata = Vec::new();

    metadata.push((METADATA_KIND.to_string(), msg.kind.to_string()));
    metadata.push((METADATA_PRIORITY.to_string(), msg.priority.to_string()));
    metadata.push((METADATA_TTL.to_string(), msg.ttl_ms.to_string()));

    if let Some(ref class) = msg.class {
        metadata.push((METADATA_CLASS.to_string(), class.clone()));
    }

    metadata
}

/// Extracts LNMP-Net metadata from gRPC metadata.
///
/// Returns (kind, priority, ttl_ms, class) tuple. Missing metadata uses defaults.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::grpc::grpc_metadata_to_net_meta;
///
/// let (kind, priority, ttl_ms, class) = grpc_metadata_to_net_meta(&metadata)?;
/// ```
pub fn grpc_metadata_to_net_meta(
    metadata: &[(String, String)],
) -> Result<(MessageKind, u8, u32, Option<String>)> {
    let metadata_map: std::collections::HashMap<&str, &str> = metadata
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // Parse kind (default to Event)
    let kind = if let Some(&kind_str) = metadata_map.get(METADATA_KIND) {
        MessageKind::from_str(kind_str)
            .map_err(|e| NetError::Other(format!("Invalid kind: {}", e)))?
    } else {
        MessageKind::default()
    };

    // Parse priority (default to kind's default)
    let priority = if let Some(&priority_str) = metadata_map.get(METADATA_PRIORITY) {
        priority_str
            .parse()
            .map_err(|_| NetError::InvalidPriority("Parse error".into()))?
    } else {
        kind.default_priority()
    };

    // Parse TTL (default to kind's default)
    let ttl_ms = if let Some(&ttl_str) = metadata_map.get(METADATA_TTL) {
        ttl_str
            .parse()
            .map_err(|_| NetError::InvalidTTL("Parse error".into()))?
    } else {
        kind.default_ttl_ms()
    };

    // Parse class (optional)
    let class = metadata_map.get(METADATA_CLASS).map(|s| s.to_string());

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
    fn test_net_to_grpc_metadata() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Command)
            .priority(180)
            .ttl_ms(2000)
            .class("automation")
            .build();

        let metadata = net_to_grpc_metadata(&msg);

        assert!(metadata.contains(&("lnmp-kind".to_string(), "Command".to_string())));
        assert!(metadata.contains(&("lnmp-priority".to_string(), "180".to_string())));
        assert!(metadata.contains(&("lnmp-ttl".to_string(), "2000".to_string())));
        assert!(metadata.contains(&("lnmp-class".to_string(), "automation".to_string())));
    }

    #[test]
    fn test_net_to_grpc_metadata_no_class() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(100)
            .ttl_ms(5000)
            .build();

        let metadata = net_to_grpc_metadata(&msg);

        assert_eq!(metadata.len(), 3); // kind, priority, ttl only
        assert!(!metadata.iter().any(|(k, _)| k == "lnmp-class"));
    }

    #[test]
    fn test_grpc_metadata_to_net_meta() {
        let metadata = vec![
            ("lnmp-kind".to_string(), "Query".to_string()),
            ("lnmp-priority".to_string(), "130".to_string()),
            ("lnmp-ttl".to_string(), "4000".to_string()),
            ("lnmp-class".to_string(), "search".to_string()),
        ];

        let (kind, priority, ttl_ms, class) = grpc_metadata_to_net_meta(&metadata).unwrap();

        assert_eq!(kind, MessageKind::Query);
        assert_eq!(priority, 130);
        assert_eq!(ttl_ms, 4000);
        assert_eq!(class, Some("search".to_string()));
    }

    #[test]
    fn test_grpc_metadata_to_net_meta_minimal() {
        let metadata = vec![]; // No metadata

        let (kind, priority, ttl_ms, class) = grpc_metadata_to_net_meta(&metadata).unwrap();

        assert_eq!(kind, MessageKind::Event);
        assert_eq!(priority, 100);
        assert_eq!(ttl_ms, 5000);
        assert_eq!(class, None);
    }

    #[test]
    fn test_grpc_metadata_roundtrip() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let original = NetMessageBuilder::new(envelope, MessageKind::State)
            .priority(95)
            .ttl_ms(15000)
            .class("telemetry")
            .build();

        // Encode
        let metadata = net_to_grpc_metadata(&original);

        // Decode
        let (kind, priority, ttl_ms, class) = grpc_metadata_to_net_meta(&metadata).unwrap();

        // Verify roundtrip
        assert_eq!(kind, MessageKind::State);
        assert_eq!(priority, 95);
        assert_eq!(ttl_ms, 15000);
        assert_eq!(class, Some("telemetry".to_string()));
    }
}
