//! Operational metadata for LNMP records

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Operational metadata fields for an LNMP envelope
///
/// All fields are optional to provide flexibility. Applications should
/// set fields based on their requirements:
/// - Timestamp: For temporal reasoning and freshness
/// - Source: For routing, multi-tenant, and trust scoring
/// - TraceID: For distributed tracing integration
/// - Sequence: For conflict resolution and ordering
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnvelopeMetadata {
    /// Event timestamp in milliseconds since Unix epoch (UTC)
    ///
    /// Recommended for all events. Used for:
    /// - Temporal ordering
    /// - Freshness/decay calculations
    /// - Event replay
    pub timestamp: Option<u64>,

    /// Source service/device/tenant identifier
    ///
    /// Examples: "auth-service", "sensor-12", "tenant-acme"
    ///
    /// Recommendation: Keep ≤ 64 characters
    pub source: Option<String>,

    /// Distributed tracing correlation ID
    ///
    /// Compatible with W3C Trace Context and OpenTelemetry.
    /// Can hold full traceparent or just trace-id portion.
    ///
    /// Recommendation: Keep ≤ 128 characters
    pub trace_id: Option<String>,

    /// Monotonically increasing sequence number
    ///
    /// Used for ordering and conflict resolution.
    /// Should increment for each version of the same entity.
    pub sequence: Option<u64>,

    /// Extensibility labels (reserved for future use)
    ///
    /// V1: Optional, implementations may ignore
    /// Future: tenant, environment, region, priority, etc.
    pub labels: HashMap<String, String>,
}

impl EnvelopeMetadata {
    /// Creates a new empty metadata instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are None/empty
    pub fn is_empty(&self) -> bool {
        self.timestamp.is_none()
            && self.source.is_none()
            && self.trace_id.is_none()
            && self.sequence.is_none()
            && self.labels.is_empty()
    }

    /// Validates metadata constraints
    ///
    /// Checks:
    /// - Source length ≤ 64 characters (warning threshold)
    /// - TraceID length ≤ 128 characters (warning threshold)
    pub fn validate(&self) -> crate::Result<()> {
        if let Some(ref source) = self.source {
            if source.len() > 256 {
                return Err(crate::EnvelopeError::StringTooLong(
                    "source".to_string(),
                    256,
                ));
            }
        }

        if let Some(ref trace_id) = self.trace_id {
            if trace_id.len() > 256 {
                return Err(crate::EnvelopeError::StringTooLong(
                    "trace_id".to_string(),
                    256,
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metadata_is_empty() {
        let meta = EnvelopeMetadata::new();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_metadata_with_timestamp_not_empty() {
        let mut meta = EnvelopeMetadata::new();
        meta.timestamp = Some(1732373147000);
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_validate_accepts_short_strings() {
        let mut meta = EnvelopeMetadata::new();
        meta.source = Some("short-service".to_string());
        meta.trace_id = Some("abc-123-xyz".to_string());
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_too_long_source() {
        let mut meta = EnvelopeMetadata::new();
        meta.source = Some("x".repeat(257));
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_too_long_trace_id() {
        let mut meta = EnvelopeMetadata::new();
        meta.trace_id = Some("y".repeat(257));
        assert!(meta.validate().is_err());
    }
}
