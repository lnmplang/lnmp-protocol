//! Error types for LNMP Envelope

use thiserror::Error;

/// Errors that can occur when working with envelopes
#[derive(Debug, Error, PartialEq)]
pub enum EnvelopeError {
    /// Invalid TLV type code
    #[error("Invalid TLV type code: {0:#x}")]
    InvalidTlvType(u8),

    /// Invalid TLV length
    #[error("Invalid TLV length: {0}")]
    InvalidTlvLength(usize),

    /// Unexpected end of data
    #[error("Unexpected end of data at offset {0}")]
    UnexpectedEof(usize),

    /// Invalid UTF-8 in string field
    #[error("Invalid UTF-8 in field: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    /// Invalid timestamp value
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(u64),

    /// Invalid sequence value
    #[error("Invalid sequence: {0}")]
    InvalidSequence(u64),

    /// String field exceeds maximum length
    #[error("String field '{0}' exceeds maximum length of {1} bytes")]
    StringTooLong(String, usize),

    /// Malformed envelope header in text format
    #[error("Malformed envelope header: {0}")]
    MalformedHeader(String),

    /// Duplicate TLV entry
    #[error("Duplicate TLV entry for type {0:#x}")]
    DuplicateTlvEntry(u8),

    /// TLV entries not in canonical order
    #[error("TLV entries not in canonical order: {0:#x} after {1:#x}")]
    NonCanonicalOrder(u8, u8),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for EnvelopeError {
    fn from(err: std::io::Error) -> Self {
        EnvelopeError::Io(err.to_string())
    }
}

/// Result type for envelope operations
pub type Result<T> = std::result::Result<T, EnvelopeError>;
