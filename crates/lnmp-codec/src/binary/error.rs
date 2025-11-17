//! Error types for LNMP binary format operations.

use crate::error::LnmpError;

/// Error type for binary format operations
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryError {
    /// Unsupported protocol version
    UnsupportedVersion {
        /// Version byte that was found
        found: u8,
        /// List of supported versions
        supported: Vec<u8>,
    },

    /// Invalid field identifier
    InvalidFID {
        /// The invalid FID value
        fid: u16,
        /// Reason why it's invalid
        reason: String,
    },

    /// Invalid type tag
    InvalidTypeTag {
        /// The invalid type tag byte
        tag: u8,
    },

    /// Invalid value data
    InvalidValue {
        /// Field ID where the error occurred
        field_id: u16,
        /// Type tag of the value
        type_tag: u8,
        /// Reason why the value is invalid
        reason: String,
    },

    /// Trailing data after frame
    TrailingData {
        /// Number of bytes remaining
        bytes_remaining: usize,
    },

    /// Canonical form violation
    CanonicalViolation {
        /// Description of the violation
        reason: String,
    },

    /// Insufficient data (unexpected end of input)
    UnexpectedEof {
        /// Number of bytes expected
        expected: usize,
        /// Number of bytes found
        found: usize,
    },

    /// Invalid VarInt encoding
    InvalidVarInt {
        /// Reason why the VarInt is invalid
        reason: String,
    },

    /// Invalid UTF-8 in string
    InvalidUtf8 {
        /// Field ID where the error occurred
        field_id: u16,
    },

    /// Conversion error from text format
    TextFormatError {
        /// The underlying text format error
        source: LnmpError,
    },

    /// Nesting depth exceeded (v0.5)
    NestingDepthExceeded {
        /// Current depth
        depth: usize,
        /// Maximum allowed depth
        max: usize,
    },

    /// Nested structure not supported (v0.4 compatibility)
    NestedStructureNotSupported,

    /// Record size exceeded (v0.5)
    RecordSizeExceeded {
        /// Current size in bytes
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Invalid nested structure (v0.5)
    InvalidNestedStructure {
        /// Reason why the structure is invalid
        reason: String,
    },
    /// Delta encoder/decoder related failure
    DeltaError {
        /// Reason describing the delta error
        reason: String,
    },
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryError::UnsupportedVersion { found, supported } => {
                write!(
                    f,
                    "Unsupported version 0x{:02X}, supported versions: {:?}",
                    found, supported
                )
            }
            BinaryError::InvalidFID { fid, reason } => {
                write!(f, "Invalid FID {}: {}", fid, reason)
            }
            BinaryError::InvalidTypeTag { tag } => {
                write!(f, "Invalid type tag: 0x{:02X}", tag)
            }
            BinaryError::InvalidValue {
                field_id,
                type_tag,
                reason,
            } => {
                write!(
                    f,
                    "Invalid value for field {} (type tag 0x{:02X}): {}",
                    field_id, type_tag, reason
                )
            }
            BinaryError::TrailingData { bytes_remaining } => {
                write!(f, "Trailing data: {} bytes remaining", bytes_remaining)
            }
            BinaryError::CanonicalViolation { reason } => {
                write!(f, "Canonical form violation: {}", reason)
            }
            BinaryError::UnexpectedEof { expected, found } => {
                write!(
                    f,
                    "Unexpected end of input: expected {} bytes, found {}",
                    expected, found
                )
            }
            BinaryError::InvalidVarInt { reason } => {
                write!(f, "Invalid VarInt: {}", reason)
            }
            BinaryError::InvalidUtf8 { field_id } => {
                write!(f, "Invalid UTF-8 in field {}", field_id)
            }
            BinaryError::TextFormatError { source } => {
                write!(f, "Text format error: {}", source)
            }
            BinaryError::NestingDepthExceeded { depth, max } => {
                write!(
                    f,
                    "Nesting depth exceeded: depth {} exceeds maximum {}",
                    depth, max
                )
            }
            BinaryError::NestedStructureNotSupported => {
                write!(f, "Nested structures not supported in v0.4 binary format")
            }
            BinaryError::RecordSizeExceeded { size, max } => {
                write!(
                    f,
                    "Record size exceeded: size {} bytes exceeds maximum {} bytes",
                    size, max
                )
            }
            BinaryError::InvalidNestedStructure { reason } => {
                write!(f, "Invalid nested structure: {}", reason)
            }
            BinaryError::DeltaError { reason } => {
                write!(f, "Delta error: {}", reason)
            }
        }
    }
}

impl std::error::Error for BinaryError {}

impl From<LnmpError> for BinaryError {
    fn from(err: LnmpError) -> Self {
        BinaryError::TextFormatError { source: err }
    }
}

impl From<crate::binary::delta::DeltaError> for BinaryError {
    fn from(err: crate::binary::delta::DeltaError) -> Self {
        BinaryError::DeltaError { reason: format!("{}", err) }
    }
}
