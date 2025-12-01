//! LNMP Envelope wrapper for records with operational metadata

use lnmp_core::LnmpRecord;

use crate::metadata::EnvelopeMetadata;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Complete LNMP message with operational context
///
/// An envelope wraps an LNMP record with operational metadata
/// (timestamp, source, trace_id, sequence) without affecting
/// the record's semantic checksum or deterministic properties.
///
/// # Example
///
/// ```
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
/// use lnmp_envelope::{LnmpEnvelope, EnvelopeMetadata};
///
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
///
/// let mut metadata = EnvelopeMetadata::new();
/// metadata.timestamp = Some(1732373147000);
/// metadata.source = Some("auth-service".to_string());
///
/// let envelope = LnmpEnvelope::with_metadata(record, metadata);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LnmpEnvelope {
    /// The LNMP record (mandatory)
    pub record: LnmpRecord,

    /// Optional operational metadata
    pub metadata: EnvelopeMetadata,
}

impl LnmpEnvelope {
    /// Creates a new envelope with the given record and empty metadata
    pub fn new(record: LnmpRecord) -> Self {
        Self {
            record,
            metadata: EnvelopeMetadata::new(),
        }
    }

    /// Creates a new envelope with the given record and metadata
    pub fn with_metadata(record: LnmpRecord, metadata: EnvelopeMetadata) -> Self {
        Self { record, metadata }
    }

    /// Returns true if metadata is empty
    pub fn has_metadata(&self) -> bool {
        !self.metadata.is_empty()
    }

    /// Validates the envelope (record and metadata)
    pub fn validate(&self) -> crate::Result<()> {
        self.metadata.validate()?;
        Ok(())
    }
}

/// Fluent builder for constructing envelopes
///
/// # Example
///
/// ```
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
/// use lnmp_envelope::EnvelopeBuilder;
///
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
///
/// let envelope = EnvelopeBuilder::new(record)
///     .timestamp(1732373147000)
///     .source("auth-service")
///     .trace_id("abc-123-xyz")
///     .sequence(42)
///     .build();
/// ```
pub struct EnvelopeBuilder {
    record: LnmpRecord,
    metadata: EnvelopeMetadata,
}

impl EnvelopeBuilder {
    /// Creates a new builder with the given record
    pub fn new(record: LnmpRecord) -> Self {
        Self {
            record,
            metadata: EnvelopeMetadata::new(),
        }
    }

    /// Sets the timestamp (Unix epoch milliseconds, UTC)
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.metadata.timestamp = Some(ts);
        self
    }

    /// Sets the source identifier
    pub fn source(mut self, src: impl Into<String>) -> Self {
        self.metadata.source = Some(src.into());
        self
    }

    /// Sets the trace ID for distributed tracing
    pub fn trace_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.trace_id = Some(id.into());
        self
    }

    /// Sets the sequence number
    pub fn sequence(mut self, seq: u64) -> Self {
        self.metadata.sequence = Some(seq);
        self
    }

    /// Adds a label (key-value pair)
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.labels.insert(key.into(), value.into());
        self
    }

    /// Builds the envelope
    pub fn build(self) -> LnmpEnvelope {
        LnmpEnvelope::with_metadata(self.record, self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::{LnmpField, LnmpValue};

    fn sample_record() -> LnmpRecord {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record
    }

    #[test]
    fn test_new_envelope_has_no_metadata() {
        let envelope = LnmpEnvelope::new(sample_record());
        assert!(!envelope.has_metadata());
    }

    #[test]
    fn test_builder_basic() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1732373147000)
            .source("test-service")
            .build();

        assert!(envelope.has_metadata());
        assert_eq!(envelope.metadata.timestamp, Some(1732373147000));
        assert_eq!(envelope.metadata.source, Some("test-service".to_string()));
    }

    #[test]
    fn test_builder_all_fields() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1732373147000)
            .source("auth-service")
            .trace_id("abc-123-xyz")
            .sequence(42)
            .label("tenant", "acme")
            .label("env", "prod")
            .build();

        assert_eq!(envelope.metadata.timestamp, Some(1732373147000));
        assert_eq!(envelope.metadata.source, Some("auth-service".to_string()));
        assert_eq!(envelope.metadata.trace_id, Some("abc-123-xyz".to_string()));
        assert_eq!(envelope.metadata.sequence, Some(42));
        assert_eq!(envelope.metadata.labels.len(), 2);
    }

    #[test]
    fn test_validate_succeeds_for_valid_envelope() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1732373147000)
            .source("short")
            .build();

        assert!(envelope.validate().is_ok());
    }
}
