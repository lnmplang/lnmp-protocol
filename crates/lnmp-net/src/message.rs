//! Core message structure for LNMP-Net

use lnmp_envelope::LnmpEnvelope;

use crate::error::{NetError, Result};
use crate::kind::MessageKind;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Network message with semantic classification and QoS metadata
///
/// Wraps an LNMP envelope (which contains the record + operational metadata)
/// with network behavior metadata: message kind, priority, TTL, and optional class.
///
/// # Examples
///
/// ```
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
/// use lnmp_envelope::EnvelopeBuilder;
/// use lnmp_net::{MessageKind, NetMessage};
///
/// // Create a record
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField { fid: 42, value: LnmpValue::Int(100) });
///
/// // Create envelope with timestamp
/// let envelope = EnvelopeBuilder::new(record)
///     .timestamp(1700000000000)
///     .source("sensor-01")
///     .build();
///
/// // Create network message
/// let msg = NetMessage::new(envelope, MessageKind::Event);
/// ```
///
/// Network message wrapping an LNMP envelope with network behavior metadata.
///
/// Combines LNMP record data (via envelope) with network-level information
/// (kind, priority, TTL, class) for intelligent routing and LLM integration.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(bound(deserialize = ""))
)]
pub struct NetMessage {
    /// LNMP envelope containing record and operational metadata
    pub envelope: LnmpEnvelope,

    /// Message semantic classification
    pub kind: MessageKind,

    /// Priority (0-255): 0-50 low, 51-200 normal, 201-255 critical
    pub priority: u8,

    /// Time-to-live in milliseconds
    pub ttl_ms: u32,

    /// Optional domain classification (e.g., "health", "safety", "traffic")
    pub class: Option<String>,
}

impl NetMessage {
    /// Creates a new network message with defaults from MessageKind
    ///
    /// Uses `kind.default_priority()` and `kind.default_ttl_ms()` for QoS fields.
    pub fn new(envelope: LnmpEnvelope, kind: MessageKind) -> Self {
        Self {
            envelope,
            kind,
            priority: kind.default_priority(),
            ttl_ms: kind.default_ttl_ms(),
            class: None,
        }
    }

    /// Creates a new network message with custom priority and TTL
    pub fn with_qos(envelope: LnmpEnvelope, kind: MessageKind, priority: u8, ttl_ms: u32) -> Self {
        Self {
            envelope,
            kind,
            priority,
            ttl_ms,
            class: None,
        }
    }

    /// Checks if the message has expired based on current time
    ///
    /// Returns `Err(NetError::MissingTimestamp)` if envelope has no timestamp.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current time in epoch milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_core::LnmpRecord;
    /// use lnmp_envelope::EnvelopeBuilder;
    /// use lnmp_net::{MessageKind, NetMessage};
    ///
    /// let envelope = EnvelopeBuilder::new(LnmpRecord::new())
    ///     .timestamp(1000)
    ///     .build();
    ///
    /// let msg = NetMessage::with_qos(envelope, MessageKind::Event, 100, 5000);
    ///
    /// assert!(!msg.is_expired(5000).unwrap()); // Age = 4000ms, TTL = 5000ms
    /// assert!(msg.is_expired(7000).unwrap());  // Age = 6000ms > 5000ms
    /// ```
    pub fn is_expired(&self, now_ms: u64) -> Result<bool> {
        let timestamp = self
            .envelope
            .metadata
            .timestamp
            .ok_or(NetError::MissingTimestamp)?;

        let age_ms = now_ms.saturating_sub(timestamp);
        Ok(age_ms > self.ttl_ms as u64)
    }

    /// Returns the age of the message in milliseconds
    ///
    /// Returns `None` if envelope has no timestamp.
    pub fn age_ms(&self, now_ms: u64) -> Option<u64> {
        self.envelope
            .metadata
            .timestamp
            .map(|ts| now_ms.saturating_sub(ts))
    }

    /// Returns the source identifier from envelope metadata
    pub fn source(&self) -> Option<&str> {
        self.envelope.metadata.source.as_deref()
    }

    /// Returns the trace ID from envelope metadata
    pub fn trace_id(&self) -> Option<&str> {
        self.envelope.metadata.trace_id.as_deref()
    }

    /// Returns the timestamp from envelope metadata
    pub fn timestamp(&self) -> Option<u64> {
        self.envelope.metadata.timestamp
    }

    /// Returns a reference to the underlying LNMP record
    pub fn record(&self) -> &lnmp_core::LnmpRecord {
        &self.envelope.record
    }

    /// Validates the message (envelope + QoS fields)
    pub fn validate(&self) -> Result<()> {
        self.envelope.validate()?;
        Ok(())
    }
}

/// Fluent builder for constructing network messages
///
/// # Examples
///
/// ```
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
/// use lnmp_envelope::EnvelopeBuilder;
/// use lnmp_net::{MessageKind, NetMessageBuilder};
///
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(42) });
///
/// let envelope = EnvelopeBuilder::new(record)
///     .timestamp(1700000000000)
///     .source("service-a")
///     .build();
///
/// let msg = NetMessageBuilder::new(envelope, MessageKind::Alert)
///     .priority(255)
///     .ttl_ms(1000)
///     .class("safety")
///     .build();
///
/// assert_eq!(msg.priority, 255);
/// assert_eq!(msg.class, Some("safety".to_string()));
/// ```
pub struct NetMessageBuilder {
    envelope: LnmpEnvelope,
    kind: MessageKind,
    priority: u8,
    ttl_ms: u32,
    class: Option<String>,
}

impl NetMessageBuilder {
    /// Creates a new builder with defaults from MessageKind
    pub fn new(envelope: LnmpEnvelope, kind: MessageKind) -> Self {
        Self {
            envelope,
            kind,
            priority: kind.default_priority(),
            ttl_ms: kind.default_ttl_ms(),
            class: None,
        }
    }

    /// Sets the priority (0-255)
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the TTL in milliseconds
    pub fn ttl_ms(mut self, ttl_ms: u32) -> Self {
        self.ttl_ms = ttl_ms;
        self
    }

    /// Sets the domain class
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Builds the NetMessage
    pub fn build(self) -> NetMessage {
        NetMessage {
            envelope: self.envelope,
            kind: self.kind,
            priority: self.priority,
            ttl_ms: self.ttl_ms,
            class: self.class,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn sample_envelope(timestamp: u64) -> LnmpEnvelope {
        EnvelopeBuilder::new(sample_record())
            .timestamp(timestamp)
            .source("test-node")
            .build()
    }

    #[test]
    fn test_new_message_uses_kind_defaults() {
        let envelope = sample_envelope(1000);
        let msg = NetMessage::new(envelope, MessageKind::Alert);

        assert_eq!(msg.kind, MessageKind::Alert);
        assert_eq!(msg.priority, 255); // Alert default
        assert_eq!(msg.ttl_ms, 1000); // Alert default
    }

    #[test]
    fn test_is_expired_fresh_message() {
        let envelope = sample_envelope(1000);
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 100, 5000);

        // Age = 4000ms, TTL = 5000ms -> not expired
        assert!(!msg.is_expired(5000).unwrap());
    }

    #[test]
    fn test_is_expired_old_message() {
        let envelope = sample_envelope(1000);
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 100, 5000);

        // Age = 6000ms, TTL = 5000ms -> expired
        assert!(msg.is_expired(7000).unwrap());
    }

    #[test]
    fn test_is_expired_missing_timestamp() {
        let envelope = LnmpEnvelope::new(sample_record());
        let msg = NetMessage::new(envelope, MessageKind::Event);

        assert!(msg.is_expired(5000).is_err());
    }

    #[test]
    fn test_age_ms() {
        let envelope = sample_envelope(1000);
        let msg = NetMessage::new(envelope, MessageKind::Event);

        assert_eq!(msg.age_ms(6000), Some(5000));
    }

    #[test]
    fn test_age_ms_missing_timestamp() {
        let envelope = LnmpEnvelope::new(sample_record());
        let msg = NetMessage::new(envelope, MessageKind::Event);

        assert_eq!(msg.age_ms(5000), None);
    }

    #[test]
    fn test_accessors() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .source("node-alpha")
            .trace_id("trace-123")
            .build();

        let msg = NetMessage::new(envelope, MessageKind::State);

        assert_eq!(msg.source(), Some("node-alpha"));
        assert_eq!(msg.trace_id(), Some("trace-123"));
        assert_eq!(msg.timestamp(), Some(1000));
    }

    #[test]
    fn test_builder_defaults() {
        let envelope = sample_envelope(1000);
        let msg = NetMessageBuilder::new(envelope, MessageKind::Command).build();

        assert_eq!(msg.priority, 150); // Command default
        assert_eq!(msg.ttl_ms, 2000); // Command default
        assert_eq!(msg.class, None);
    }

    #[test]
    fn test_builder_custom_values() {
        let envelope = sample_envelope(1000);
        let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(200)
            .ttl_ms(10000)
            .class("health")
            .build();

        assert_eq!(msg.priority, 200);
        assert_eq!(msg.ttl_ms, 10000);
        assert_eq!(msg.class, Some("health".to_string()));
    }

    #[test]
    fn test_validate() {
        let envelope = sample_envelope(1000);
        let msg = NetMessage::new(envelope, MessageKind::Event);

        assert!(msg.validate().is_ok());
    }
}
