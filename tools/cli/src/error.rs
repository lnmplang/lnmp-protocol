use std::io;
use thiserror::Error;

/// CLI-specific error types
#[derive(Error, Debug)]
pub enum CliError {
    /// I/O error (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// LNMP codec error (parsing, encoding, decoding)
    #[error("LNMP codec error: {0}")]
    Codec(String),

    /// Binary codec error
    #[error("Binary codec error: {0}")]
    BinaryCodec(#[from] lnmp::codec::binary::BinaryError),

    /// Embedding error  
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Quantization error
    #[error("Quantization error: {0}")]
    Quant(#[from] lnmp::quant::QuantError),

    /// Spatial error
    #[error("Spatial error: {0}")]
    Spatial(#[from] lnmp::spatial::SpatialError),

    /// Transport protocol error
    #[error("Transport error: {0}")]
    Transport(#[from] lnmp::transport::TransportError),

    /// Serialization error (JSON, bincode, etc.)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid input provided by user
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

// Implement conversions for common error types
impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for CliError {
    fn from(err: bincode::Error) -> Self {
        CliError::Serialization(err.to_string())
    }
}

impl From<String> for CliError {
    fn from(msg: String) -> Self {
        CliError::Other(msg)
    }
}

impl From<&str> for CliError {
    fn from(msg: &str) -> Self {
        CliError::Other(msg.to_string())
    }
}

/// Type alias for CLI Result
pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CliError::InvalidInput("test error".to_string());
        assert_eq!(err.to_string(), "Invalid input: test error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cli_err: CliError = io_err.into();
        assert!(matches!(cli_err, CliError::Io(_)));
    }

    #[test]
    fn test_string_conversion() {
        let err: CliError = "test message".into();
        assert!(matches!(err, CliError::Other(_)));
    }
}
