#![warn(missing_docs)]
#![warn(clippy::all)]

//! # lnmp-envelope
//!
//! Operational metadata envelope for LNMP records.
//!
//! This crate provides a way to attach operational context (timestamp, source,
//! trace ID, sequence) to LNMP records without affecting their deterministic
//! properties or semantic checksums.
//!
//! ## Alignment with Industry Standards
//!
//! LNMP Envelope aligns with:
//! - **CloudEvents**: Similar context attributes (time, source, id)
//! - **Kafka Headers**: Record-level metadata separate from payload
//! - **W3C Trace Context**: Compatible trace ID format for distributed tracing
//! - **OpenTelemetry**: Seamless integration with telemetry spans
//!
//! ## Core Principles
//!
//! 1. **Determinism Preserved**: Envelope metadata does NOT affect `SemanticChecksum`
//! 2. **Zero Overhead**: Unused envelope features have no performance cost
//! 3. **Transport Agnostic**: Defined independently, bindings provided separately
//! 4. **Future Proof**: Extensible via labels, unknown fields skipped gracefully
//!
//! ## Quick Start
//!
//! ```
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//! use lnmp_envelope::EnvelopeBuilder;
//!
//! // Create a record
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
//!
//! // Wrap with envelope
//! let envelope = EnvelopeBuilder::new(record)
//!     .timestamp(1732373147000)
//!     .source("auth-service")
//!     .trace_id("abc-123-xyz")
//!     .sequence(42)
//!     .build();
//!
//! assert!(envelope.has_metadata());
//! ```
//!
//! ## Encoding Formats
//!
//! ### Binary (TLV)
//!
//! Envelope metadata is encoded as Type-Length-Value entries in the LNMP
//! container metadata extension block:
//!
//! ```text
//! Type: 0x10 (Timestamp) | Length: 8 | Value: u64 BE
//! Type: 0x11 (Source)    | Length: N | Value: UTF-8 string
//! Type: 0x12 (TraceID)   | Length: M | Value: UTF-8 string
//! Type: 0x13 (Sequence)  | Length: 8 | Value: u64 BE
//! ```
//!
//! ### Text (Header Comment)
//!
//! ```text
//! #ENVELOPE timestamp=1732373147000 source=auth-service trace_id="abc-123"
//! F12=14532
//! F7=1
//! ```
//!
//! ## Transport Bindings
//!
//! ### HTTP
//!
//! ```text
//! X-LNMP-Timestamp: 1732373147000
//! X-LNMP-Source: auth-service
//! X-LNMP-Trace-ID: abc-123-xyz
//! X-LNMP-Sequence: 42
//! ```
//!
//! ### Kafka
//!
//! ```text
//! Record headers:
//!   lnmp.timestamp: "1732373147000"
//!   lnmp.source: "auth-service"
//!   lnmp.trace_id: "abc-123-xyz"
//!   lnmp.sequence: "42"
//! ```
//!
//! ## Features
//!
//! - `serde`: Enable serde serialization support (optional)

pub mod binary_codec;
mod envelope;
mod error;
mod metadata;
pub mod text_codec;

pub use envelope::{EnvelopeBuilder, LnmpEnvelope};
pub use error::{EnvelopeError, Result};
pub use metadata::EnvelopeMetadata;

// Re-export for convenience
pub use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
