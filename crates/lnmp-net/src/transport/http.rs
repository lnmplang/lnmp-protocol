//! HTTP transport bindings for LNMP-Net.
//!
//! This module provides helpers to map LNMP-Net metadata (kind, priority, ttl, class)
//! to/from HTTP headers, complementing the existing lnmp-transport HTTP bindings.

use crate::error::{NetError, Result};
use crate::kind::MessageKind;
use crate::message::NetMessage;

#[cfg(feature = "transport")]
use http::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

/// HTTP header name for LNMP-Net message kind.
pub const HEADER_KIND: &str = "X-LNMP-Kind";

/// HTTP header name for LNMP-Net message priority.
pub const HEADER_PRIORITY: &str = "X-LNMP-Priority";

/// HTTP header name for LNMP-Net message TTL.
pub const HEADER_TTL: &str = "X-LNMP-TTL";

/// HTTP header name for LNMP-Net message class.
pub const HEADER_CLASS: &str = "X-LNMP-Class";

/// Adds LNMP-Net metadata to HTTP headers.
///
/// This function complements `lnmp_transport::http::envelope_to_headers` by adding
/// network behavior metadata (kind, priority, TTL, class).
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::http::net_to_http_headers;
///
/// let net_headers = net_to_http_headers(&msg)?;
/// // Merge with envelope headers for complete message
/// ```
#[cfg(feature = "transport")]
pub fn net_to_http_headers(msg: &NetMessage) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    // Add message kind
    headers.insert(
        HeaderName::from_static("x-lnmp-kind"),
        HeaderValue::from_str(&msg.kind.to_string())
            .map_err(|e| NetError::Other(format!("Invalid kind header: {}", e)))?,
    );

    // Add priority
    headers.insert(
        HeaderName::from_static("x-lnmp-priority"),
        HeaderValue::from_str(&msg.priority.to_string())
            .map_err(|e| NetError::Other(format!("Invalid priority header: {}", e)))?,
    );

    // Add TTL
    headers.insert(
        HeaderName::from_static("x-lnmp-ttl"),
        HeaderValue::from_str(&msg.ttl_ms.to_string())
            .map_err(|e| NetError::Other(format!("Invalid ttl header: {}", e)))?,
    );

    // Add class if present
    if let Some(ref class) = msg.class {
        headers.insert(
            HeaderName::from_static("x-lnmp-class"),
            HeaderValue::from_str(class)
                .map_err(|e| NetError::Other(format!("Invalid class header: {}", e)))?,
        );
    }

    Ok(headers)
}

/// Extracts LNMP-Net metadata from HTTP headers.
///
/// Returns (kind, priority, ttl_ms, class) tuple. Missing headers result in defaults:
/// - kind: Event (default)
/// - priority: 100
/// - ttl_ms: 5000
/// - class: None
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_net::transport::http::http_headers_to_net_meta;
///
/// let (kind, priority, ttl_ms, class) = http_headers_to_net_meta(&headers)?;
/// ```
#[cfg(feature = "transport")]
pub fn http_headers_to_net_meta(
    headers: &HeaderMap,
) -> Result<(MessageKind, u8, u32, Option<String>)> {
    // Parse kind (default to Event if missing)
    let kind = if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-kind")) {
        let kind_str = val
            .to_str()
            .map_err(|_| NetError::Other("Invalid kind header value".into()))?;
        MessageKind::from_str(kind_str)
            .map_err(|e| NetError::Other(format!("Invalid kind: {}", e)))?
    } else {
        MessageKind::default()
    };

    // Parse priority (default to kind's default)
    let priority = if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-priority")) {
        let priority_str = val
            .to_str()
            .map_err(|_| NetError::Other("Invalid priority header value".into()))?;
        priority_str
            .parse()
            .map_err(|_| NetError::InvalidPriority("Parse error".into()))?
    } else {
        kind.default_priority()
    };

    // Parse TTL (default to kind's default)
    let ttl_ms = if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-ttl")) {
        let ttl_str = val
            .to_str()
            .map_err(|_| NetError::Other("Invalid ttl header value".into()))?;
        ttl_str
            .parse()
            .map_err(|_| NetError::InvalidTTL("Parse error".into()))?
    } else {
        kind.default_ttl_ms()
    };

    // Parse class (optional)
    let class = if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-class")) {
        let class_str = val
            .to_str()
            .map_err(|_| NetError::Other("Invalid class header value".into()))?;
        Some(class_str.to_string())
    } else {
        None
    };

    Ok((kind, priority, ttl_ms, class))
}

#[cfg(all(test, feature = "transport"))]
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
    fn test_net_to_http_headers_all_fields() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Alert)
            .priority(255)
            .ttl_ms(1000)
            .class("safety")
            .build();

        let headers = net_to_http_headers(&msg).unwrap();

        assert_eq!(
            headers.get("x-lnmp-kind").unwrap().to_str().unwrap(),
            "Alert"
        );
        assert_eq!(
            headers.get("x-lnmp-priority").unwrap().to_str().unwrap(),
            "255"
        );
        assert_eq!(headers.get("x-lnmp-ttl").unwrap().to_str().unwrap(), "1000");
        assert_eq!(
            headers.get("x-lnmp-class").unwrap().to_str().unwrap(),
            "safety"
        );
    }

    #[test]
    fn test_net_to_http_headers_no_class() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(100)
            .ttl_ms(5000)
            .build();

        let headers = net_to_http_headers(&msg).unwrap();

        assert_eq!(
            headers.get("x-lnmp-kind").unwrap().to_str().unwrap(),
            "Event"
        );
        assert!(headers.get("x-lnmp-class").is_none());
    }

    #[test]
    fn test_http_headers_to_net_meta_complete() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-lnmp-kind"),
            HeaderValue::from_static("Command"),
        );
        headers.insert(
            HeaderName::from_static("x-lnmp-priority"),
            HeaderValue::from_static("200"),
        );
        headers.insert(
            HeaderName::from_static("x-lnmp-ttl"),
            HeaderValue::from_static("3000"),
        );
        headers.insert(
            HeaderName::from_static("x-lnmp-class"),
            HeaderValue::from_static("traffic"),
        );

        let (kind, priority, ttl_ms, class) = http_headers_to_net_meta(&headers).unwrap();

        assert_eq!(kind, MessageKind::Command);
        assert_eq!(priority, 200);
        assert_eq!(ttl_ms, 3000);
        assert_eq!(class, Some("traffic".to_string()));
    }

    #[test]
    fn test_http_headers_to_net_meta_minimal() {
        let headers = HeaderMap::new(); // No LNMP-Net headers

        let (kind, priority, ttl_ms, class) = http_headers_to_net_meta(&headers).unwrap();

        // Should use defaults
        assert_eq!(kind, MessageKind::Event);
        assert_eq!(priority, 100); // Event default
        assert_eq!(ttl_ms, 5000); // Event default
        assert_eq!(class, None);
    }

    #[test]
    fn test_http_headers_roundtrip() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let original = NetMessageBuilder::new(envelope.clone(), MessageKind::Query)
            .priority(120)
            .ttl_ms(5000)
            .class("health")
            .build();

        // Encode
        let headers = net_to_http_headers(&original).unwrap();

        // Decode
        let (kind, priority, ttl_ms, class) = http_headers_to_net_meta(&headers).unwrap();

        // Verify roundtrip
        assert_eq!(kind, MessageKind::Query);
        assert_eq!(priority, 120);
        assert_eq!(ttl_ms, 5000);
        assert_eq!(class, Some("health".to_string()));
    }
}
