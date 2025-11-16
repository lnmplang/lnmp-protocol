//! Binary protocol format for LNMP v0.4.
//!
//! This module provides efficient binary encoding and decoding for LNMP data,
//! enabling zero-copy, deterministic serialization suitable for transport protocols.
//!
//! The binary format maintains canonical guarantees at the binary level while
//! providing seamless interoperability with the text format (v0.3).
//!
//! # Overview
//!
//! LNMP v0.4 introduces a binary protocol that enables bidirectional conversion
//! between LNMP-Text (v0.3) and LNMP-Binary representations. The binary format is
//! designed for:
//!
//! - **Agent-to-Model Communication**: Efficient data transfer between AI agents and language models
//! - **Network Transport**: Compact wire format for distributed systems
//! - **Storage**: Space-efficient persistence of LNMP records
//! - **Interoperability**: Seamless conversion between text and binary formats
//!
//! # Binary Format Structure
//!
//! The binary format consists of a frame with the following structure:
//!
//! ```text
//! ┌─────────┬─────────┬─────────────┬──────────────────────┐
//! │ VERSION │  FLAGS  │ ENTRY_COUNT │      ENTRIES...      │
//! │ (1 byte)│(1 byte) │  (VarInt)   │     (variable)       │
//! └─────────┴─────────┴─────────────┴──────────────────────┘
//! ```
//!
//! Each entry contains:
//!
//! ```text
//! ┌──────────┬──────────┬──────────────────┐
//! │   FID    │  THTAG   │      VALUE       │
//! │ (2 bytes)│ (1 byte) │   (variable)     │
//! └──────────┴──────────┴──────────────────┘
//! ```
//!
//! # Supported Types
//!
//! - **Integer** (0x01): VarInt encoded signed 64-bit integers
//! - **Float** (0x02): IEEE 754 double-precision (8 bytes, little-endian)
//! - **Boolean** (0x03): Single byte (0x00 = false, 0x01 = true)
//! - **String** (0x04): Length-prefixed UTF-8 (length as VarInt + bytes)
//! - **String Array** (0x05): Count-prefixed array of length-prefixed strings
//!
//! # Basic Usage
//!
//! ## Encoding to Binary
//!
//! ```
//! use lnmp_codec::binary::BinaryEncoder;
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! // Create a record
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField {
//!     fid: 7,
//!     value: LnmpValue::Bool(true),
//! });
//! record.add_field(LnmpField {
//!     fid: 12,
//!     value: LnmpValue::Int(14532),
//! });
//!
//! // Encode to binary
//! let encoder = BinaryEncoder::new();
//! let binary = encoder.encode(&record).unwrap();
//! ```
//!
//! ## Decoding from Binary
//!
//! ```
//! use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! # let mut record = LnmpRecord::new();
//! # record.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });
//! # let encoder = BinaryEncoder::new();
//! # let binary = encoder.encode(&record).unwrap();
//! // Decode from binary
//! let decoder = BinaryDecoder::new();
//! let decoded_record = decoder.decode(&binary).unwrap();
//! ```
//!
//! ## Text to Binary Conversion
//!
//! ```
//! use lnmp_codec::binary::BinaryEncoder;
//!
//! // Convert text format directly to binary
//! let text = "F7=1;F12=14532;F23=[\"admin\",\"dev\"]";
//! let encoder = BinaryEncoder::new();
//! let binary = encoder.encode_text(text).unwrap();
//! ```
//!
//! ## Binary to Text Conversion
//!
//! ```
//! use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
//!
//! # let text = "F7=1;F12=14532";
//! # let encoder = BinaryEncoder::new();
//! # let binary = encoder.encode_text(text).unwrap();
//! // Convert binary format to canonical text
//! let decoder = BinaryDecoder::new();
//! let text = decoder.decode_to_text(&binary).unwrap();
//! // Output: "F7=1\nF12=14532" (canonical format)
//! ```
//!
//! # Round-Trip Conversion
//!
//! The binary format maintains canonical form guarantees, ensuring stable round-trip conversion:
//!
//! ```
//! use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
//!
//! let original_text = "F23=[\"admin\"];F7=1;F12=14532"; // Unsorted
//!
//! // Text → Binary → Text
//! let encoder = BinaryEncoder::new();
//! let binary = encoder.encode_text(original_text).unwrap();
//!
//! let decoder = BinaryDecoder::new();
//! let canonical_text = decoder.decode_to_text(&binary).unwrap();
//! // Output: "F7=1\nF12=14532\nF23=[admin]" (sorted by FID)
//!
//! // Multiple round-trips produce stable output
//! let binary2 = encoder.encode_text(&canonical_text).unwrap();
//! assert_eq!(binary, binary2);
//! ```
//!
//! # Configuration Options
//!
//! ## Encoder Configuration
//!
//! ```
//! use lnmp_codec::binary::{BinaryEncoder, EncoderConfig};
//!
//! let config = EncoderConfig::new()
//!     .with_validate_canonical(true)
//!     .with_sort_fields(true);
//!
//! let encoder = BinaryEncoder::with_config(config);
//! ```
//!
//! ## Decoder Configuration
//!
//! ```
//! use lnmp_codec::binary::{BinaryDecoder, DecoderConfig};
//!
//! let config = DecoderConfig::new()
//!     .with_validate_ordering(true)  // Enforce canonical field order
//!     .with_strict_parsing(true);    // Detect trailing data
//!
//! let decoder = BinaryDecoder::with_config(config);
//! ```
//!
//! # Error Handling
//!
//! ```
//! use lnmp_codec::binary::{BinaryDecoder, BinaryError};
//!
//! let invalid_binary = vec![0x99, 0x00, 0x00]; // Invalid version
//!
//! let decoder = BinaryDecoder::new();
//! match decoder.decode(&invalid_binary) {
//!     Ok(record) => println!("Success!"),
//!     Err(BinaryError::UnsupportedVersion { found, supported }) => {
//!         eprintln!("Unsupported version: 0x{:02X}", found);
//!     }
//!     Err(e) => eprintln!("Decode error: {}", e),
//! }
//! ```
//!
//! # Advanced Usage
//!
//! ## Working with All Value Types
//!
//! ```
//! use lnmp_codec::binary::BinaryEncoder;
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField {
//!     fid: 1,
//!     value: LnmpValue::Int(-42),
//! });
//! record.add_field(LnmpField {
//!     fid: 2,
//!     value: LnmpValue::Float(3.14159),
//! });
//! record.add_field(LnmpField {
//!     fid: 3,
//!     value: LnmpValue::Bool(false),
//! });
//! record.add_field(LnmpField {
//!     fid: 4,
//!     value: LnmpValue::String("hello\nworld".to_string()),
//! });
//! record.add_field(LnmpField {
//!     fid: 5,
//!     value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
//! });
//!
//! let encoder = BinaryEncoder::new();
//! let binary = encoder.encode(&record).unwrap();
//! ```
//!
//! ## Strict Validation
//!
//! ```
//! use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder, DecoderConfig};
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! # let mut record = LnmpRecord::new();
//! # record.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });
//! # let encoder = BinaryEncoder::new();
//! # let mut binary = encoder.encode(&record).unwrap();
//! // Add trailing data
//! binary.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
//!
//! // Strict decoder will detect trailing data
//! let config = DecoderConfig::new().with_strict_parsing(true);
//! let decoder = BinaryDecoder::with_config(config);
//!
//! match decoder.decode(&binary) {
//!     Err(e) => println!("Detected error: {}", e),
//!     Ok(_) => println!("Unexpected success"),
//! }
//! ```
//!
//! # Performance Characteristics
//!
//! - **Space Efficiency**: 30-50% size reduction compared to text format for typical records
//! - **Encoding Speed**: < 1μs per field for simple types
//! - **Decoding Speed**: < 1μs per field for simple types
//! - **Round-trip**: < 10μs for typical 10-field record
//!
//! # Canonical Form Guarantees
//!
//! The binary encoder ensures:
//! - Fields are sorted by FID in ascending order
//! - Minimal VarInt encoding (no unnecessary leading bytes)
//! - UTF-8 string validation
//! - Consistent float representation
//!
//! The binary decoder validates:
//! - Field ordering (in strict mode)
//! - No duplicate FIDs
//! - Valid type tags
//! - No trailing data (in strict mode)
//!
//! # Compatibility
//!
//! - **Version**: v0.4 binary format only
//! - **Text Format**: Fully compatible with v0.3 text format for supported types
//! - **Nested Structures**: Not supported in v0.4 (reserved for v0.5+)
//!
//! # See Also
//!
//! - [`BinaryEncoder`]: Converts text/records to binary format
//! - [`BinaryDecoder`]: Converts binary format to text/records
//! - [`BinaryError`]: Error types for binary operations
//! - [`EncoderConfig`]: Configuration for binary encoding
//! - [`DecoderConfig`]: Configuration for binary decoding

pub mod decoder;
pub mod delta;
pub mod encoder;
pub mod entry;
pub mod error;
pub mod frame;
pub mod negotiation;
pub mod nested_decoder;
pub mod nested_encoder;
pub mod streaming;
pub mod types;
pub mod varint;

pub use decoder::{BinaryDecoder, DecoderConfig};
pub use delta::{
    DeltaConfig, DeltaDecoder, DeltaEncoder, DeltaError, DeltaOp, DeltaOperation, DELTA_TAG,
};
pub use encoder::{BinaryEncoder, EncoderConfig};
pub use entry::BinaryEntry;
pub use error::BinaryError;
pub use frame::BinaryFrame;
pub use negotiation::{
    Capabilities, ErrorCode, FeatureFlags, NegotiationError, NegotiationMessage,
    NegotiationResponse, NegotiationSession, NegotiationState, SchemaNegotiator,
};
pub use nested_decoder::{BinaryNestedDecoder, NestedDecoderConfig};
pub use nested_encoder::{BinaryNestedEncoder, NestedEncoderConfig};
pub use streaming::{
    BackpressureController, FrameFlags, FrameType, StreamingConfig, StreamingDecoder,
    StreamingEncoder, StreamingError, StreamingEvent, StreamingFrame, StreamingState,
};
pub use types::{BinaryValue, TypeTag};
