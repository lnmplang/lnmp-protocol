//! Definitions for the `.lnmp` container header.

use core::fmt;

/// ASCII `LNMP` magic number.
pub const LNMP_MAGIC: [u8; 4] = *b"LNMP";

/// First container version.
pub const LNMP_CONTAINER_VERSION_1: u8 = 1;

/// Header size (magic + version + mode + flags + metadata length).
pub const LNMP_HEADER_SIZE: usize = 12;

/// Payload must include checksums.
pub const LNMP_FLAG_CHECKSUM_REQUIRED: u16 = 0x0001;
/// Payload contains mode-specific compression.
pub const LNMP_FLAG_COMPRESSED: u16 = 0x0002;
/// Payload is encrypted.
pub const LNMP_FLAG_ENCRYPTED: u16 = 0x0004;
/// Payload contains quantum-safe signature data.
pub const LNMP_FLAG_QSIG: u16 = 0x0008;
/// Payload contains quantum-safe key-exchange metadata.
pub const LNMP_FLAG_QKEX: u16 = 0x0010;
/// Reserved for signaling a metadata extension block (TLV chain) after fixed metadata.
/// MUST be `0` in v1; a future version/flag bump is required to enable.
pub const LNMP_FLAG_EXT_META_BLOCK: u16 = 0x8000;

/// Supported `.lnmp` modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LnmpFileMode {
    /// LNMP/Text.
    Text = 0x01,
    /// LNMP/Binary.
    Binary = 0x02,
    /// LNMP/Stream.
    Stream = 0x03,
    /// LNMP/Delta.
    Delta = 0x04,
    /// LNMP/Quantum-Safe (reserved for future use).
    QuantumSafe = 0x05,
    /// LNMP/Embedding.
    Embedding = 0x06,
}

impl LnmpFileMode {
    /// Converts a raw byte into a mode.
    pub fn from_byte(value: u8) -> Result<Self, LnmpContainerError> {
        match value {
            0x01 => Ok(Self::Text),
            0x02 => Ok(Self::Binary),
            0x03 => Ok(Self::Stream),
            0x04 => Ok(Self::Delta),
            0x05 => Ok(Self::QuantumSafe),
            0x06 => Ok(Self::Embedding),
            other => Err(LnmpContainerError::UnknownMode(other)),
        }
    }

    /// Returns the mode identifier as a byte.
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Structured representation of the `.lnmp` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LnmpContainerHeader {
    /// Header version.
    pub version: u8,
    /// Container mode.
    pub mode: LnmpFileMode,
    /// Flags written in big-endian order.
    pub flags: u16,
    /// Length of metadata that follows the header (big-endian).
    pub metadata_len: u32,
}

impl LnmpContainerHeader {
    /// Creates a new header with default values.
    pub const fn new(mode: LnmpFileMode) -> Self {
        Self {
            version: LNMP_CONTAINER_VERSION_1,
            mode,
            flags: 0,
            metadata_len: 0,
        }
    }

    /// Parses bytes into a header.
    pub fn parse(bytes: &[u8]) -> Result<Self, LnmpContainerError> {
        if bytes.len() < LNMP_HEADER_SIZE {
            return Err(LnmpContainerError::TruncatedHeader);
        }

        if bytes[0..4] != LNMP_MAGIC {
            return Err(LnmpContainerError::InvalidMagic);
        }

        let version = bytes[4];
        if version != LNMP_CONTAINER_VERSION_1 {
            return Err(LnmpContainerError::UnsupportedVersion(version));
        }

        let mode = LnmpFileMode::from_byte(bytes[5])?;
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        let metadata_len = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        Ok(Self {
            version,
            mode,
            flags,
            metadata_len,
        })
    }

    /// Serializes the header into bytes.
    pub fn encode(&self) -> [u8; LNMP_HEADER_SIZE] {
        let mut buf = [0u8; LNMP_HEADER_SIZE];
        buf[0..4].copy_from_slice(&LNMP_MAGIC);
        buf[4] = self.version;
        buf[5] = self.mode.as_byte();
        buf[6..8].copy_from_slice(&self.flags.to_be_bytes());
        buf[8..12].copy_from_slice(&self.metadata_len.to_be_bytes());
        buf
    }
}

/// Errors that can occur while handling headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LnmpContainerError {
    /// Header is shorter than expected.
    TruncatedHeader,
    /// Magic bytes do not match `LNMP`.
    InvalidMagic,
    /// Container version is not supported.
    UnsupportedVersion(u8),
    /// Mode identifier is unknown.
    UnknownMode(u8),
}

impl fmt::Display for LnmpContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LnmpContainerError::TruncatedHeader => write!(f, "LNMP header is truncated"),
            LnmpContainerError::InvalidMagic => write!(f, "LNMP magic does not match"),
            LnmpContainerError::UnsupportedVersion(v) => {
                write!(f, "LNMP header version {v} is not supported")
            }
            LnmpContainerError::UnknownMode(mode) => {
                write!(f, "LNMP mode {mode:#04x} is not recognized")
            }
        }
    }
}

impl std::error::Error for LnmpContainerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_header() {
        let header = LnmpContainerHeader::new(LnmpFileMode::Binary);
        let encoded = header.encode();
        let parsed = LnmpContainerHeader::parse(&encoded).unwrap();
        assert_eq!(parsed.mode, LnmpFileMode::Binary);
        assert_eq!(parsed.version, LNMP_CONTAINER_VERSION_1);
        assert_eq!(parsed.flags, 0);
        assert_eq!(parsed.metadata_len, 0);
    }

    #[test]
    fn detect_invalid_magic() {
        let mut bytes = [0u8; LNMP_HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"FOO!");
        assert!(matches!(
            LnmpContainerHeader::parse(&bytes),
            Err(LnmpContainerError::InvalidMagic)
        ));
    }

    #[test]
    fn detect_unknown_mode() {
        let mut header = LnmpContainerHeader::new(LnmpFileMode::Text).encode();
        header[5] = 0xFF;
        assert!(matches!(
            LnmpContainerHeader::parse(&header),
            Err(LnmpContainerError::UnknownMode(0xFF))
        ));
    }

    #[test]
    fn detect_truncated_header() {
        assert!(matches!(
            LnmpContainerHeader::parse(&[0u8; 4]),
            Err(LnmpContainerError::TruncatedHeader)
        ));
    }

    #[test]
    fn test_embedding_mode() {
        let header = LnmpContainerHeader::new(LnmpFileMode::Embedding);
        let encoded = header.encode();
        let parsed = LnmpContainerHeader::parse(&encoded).unwrap();
        assert_eq!(parsed.mode, LnmpFileMode::Embedding);
    }
}
