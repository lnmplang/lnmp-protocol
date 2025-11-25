//! Error types for LNMP-Net

use thiserror::Error;

/// Result type for LNMP-Net operations
pub type Result<T> = std::result::Result<T, NetError>;

/// Errors that can occur in LNMP-Net operations
#[derive(Debug, Error)]
pub enum NetError {
    /// Timestamp is required for TTL-based operations but is missing
    #[error("Missing timestamp in envelope metadata (required for TTL checks)")]
    MissingTimestamp,

    /// Invalid priority value
    #[error("Invalid priority value: {0}")]
    InvalidPriority(String),

    /// Invalid TTL value
    #[error("Invalid TTL value: {0}")]
    InvalidTTL(String),

    /// Envelope validation error
    #[error("Envelope error: {0}")]
    EnvelopeError(#[from] lnmp_envelope::EnvelopeError),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
