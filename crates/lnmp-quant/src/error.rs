use thiserror::Error;

/// Errors that can occur during quantization and dequantization operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum QuantError {
    /// Invalid or unsupported embedding dimension
    #[error("Invalid dimension: {0}")]
    InvalidDimension(String),

    /// Unsupported quantization scheme
    #[error("Invalid or unsupported quantization scheme: {0}")]
    InvalidScheme(String),

    /// Data corruption or format error
    #[error("Data corrupted or invalid format: {0}")]
    DataCorrupted(String),

    /// Quantization encoding failed
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    /// Dequantization decoding failed
    #[error("Decoding failed: {0}")]
    DecodingFailed(String),
}
