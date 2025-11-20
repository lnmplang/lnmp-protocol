//! Container-aware helpers that bridge `.lnmp` headers with codec entry points.

use std::{fmt, str};

use crate::{
    binary::{delta::DeltaApplyContext, BinaryDecoder, BinaryEncoder, BinaryError},
    Encoder, LnmpError, Parser,
};
use lnmp_core::{
    LnmpContainerError, LnmpContainerHeader, LnmpFileMode, LnmpRecord,
    LNMP_FLAG_CHECKSUM_REQUIRED, LNMP_FLAG_COMPRESSED, LNMP_FLAG_ENCRYPTED,
    LNMP_HEADER_SIZE,
};

/// Borrowed view over a `.lnmp` container.
#[derive(Debug, Clone, Copy)]
pub struct ContainerFrame<'a> {
    header: LnmpContainerHeader,
    metadata: &'a [u8],
    payload: &'a [u8],
}

/// Helper that builds `.lnmp` containers from parsed records or raw payloads.
#[derive(Debug, Clone)]
pub struct ContainerBuilder {
    header: LnmpContainerHeader,
    metadata: Vec<u8>,
    checksum_confirmed: bool,
    stream_meta: Option<StreamMetadata>,
    delta_meta: Option<DeltaMetadata>,
}

/// Decoded view over stream metadata (mode `0x03`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamMetadata {
    /// Preferred chunk size in bytes.
    pub chunk_size: u32,
    /// Checksum type (0 = none, 1 = XOR32, 2 = SC32).
    pub checksum_type: u8,
    /// Stream flags bitfield.
    pub flags: u8,
}

/// Decoded view over delta metadata (mode `0x04`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaMetadata {
    /// Base snapshot identifier.
    pub base_snapshot: u64,
    /// Delta algorithm identifier.
    pub algorithm: u8,
    /// Compression hint identifier.
    pub compression: u8,
}

impl ContainerBuilder {
    /// Starts a builder for the given mode.
    pub fn new(mode: LnmpFileMode) -> Self {
        Self {
            header: LnmpContainerHeader::new(mode),
            metadata: Vec::new(),
            checksum_confirmed: true,
            stream_meta: None,
            delta_meta: None,
        }
    }

    /// Overrides the header flags.
    pub fn with_flags(mut self, flags: u16) -> Self {
        self.header.flags = flags;
        self
    }

    /// Attaches metadata bytes that are written after the header.
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Result<Self, ContainerEncodeError> {
        self.header.metadata_len = Self::checked_metadata_len(metadata.len())?;
        self.metadata = metadata;
        Ok(self)
    }

    /// Attaches metadata from a borrowed buffer.
    pub fn with_metadata_bytes(self, metadata: &[u8]) -> Result<Self, ContainerEncodeError> {
        self.with_metadata(metadata.to_vec())
    }

    /// Indicates whether checksum hints are present when `checksum` flag is set.
    pub fn with_checksum_confirmation(mut self, confirmed: bool) -> Self {
        self.checksum_confirmed = confirmed;
        self
    }

    /// Returns the current header snapshot.
    pub const fn header(&self) -> LnmpContainerHeader {
        self.header
    }

    /// Wraps an existing payload slice with the configured header/metadata.
    pub fn wrap_payload(self, payload: &[u8]) -> Result<Vec<u8>, ContainerEncodeError> {
        self.wrap_payload_internal(payload)
    }

    /// Encodes a record according to the selected mode and wraps it in a container.
    pub fn encode_record(self, record: &LnmpRecord) -> Result<Vec<u8>, ContainerEncodeError> {
        self.validate_flags()?;
        self.validate_checksum_requirements()?;
        match self.header.mode {
            LnmpFileMode::Text => {
                let encoder = Encoder::new();
                let text = encoder.encode(record);
                self.wrap_payload_internal(text.as_bytes())
            }
            LnmpFileMode::Binary => {
                let encoder = BinaryEncoder::new();
                let binary = encoder
                    .encode(record)
                    .map_err(ContainerEncodeError::BinaryCodec)?;
                self.wrap_payload_internal(&binary)
            }
            mode => Err(ContainerEncodeError::UnsupportedMode(mode)),
        }
    }

    /// Attaches stream metadata and switches mode to stream.
    pub fn with_stream_metadata(
        mut self,
        meta: StreamMetadata,
    ) -> Result<Self, ContainerEncodeError> {
        self.header.mode = LnmpFileMode::Stream;
        self.stream_meta = Some(meta);
        Ok(self)
    }

    /// Attaches delta metadata and switches mode to delta.
    pub fn with_delta_metadata(
        mut self,
        meta: DeltaMetadata,
    ) -> Result<Self, ContainerEncodeError> {
        self.header.mode = LnmpFileMode::Delta;
        self.delta_meta = Some(meta);
        Ok(self)
    }

    fn wrap_payload_internal(mut self, payload: &[u8]) -> Result<Vec<u8>, ContainerEncodeError> {
        self.populate_auto_metadata()?;
        self.validate_flags()?;
        encode_validate_metadata_requirements(self.header.mode, self.metadata.len())?;
        encode_validate_metadata_semantics(self.header.mode, &self.metadata)?;
        let mut buffer =
            Vec::with_capacity(LNMP_HEADER_SIZE + self.metadata.len() + payload.len());
        buffer.extend_from_slice(&self.header.encode());
        buffer.extend_from_slice(&self.metadata);
        buffer.extend_from_slice(payload);
        Ok(buffer)
    }

    fn checked_metadata_len(len: usize) -> Result<u32, ContainerEncodeError> {
        u32::try_from(len).map_err(|_| ContainerEncodeError::MetadataTooLarge(len))
    }

    fn populate_auto_metadata(&mut self) -> Result<(), ContainerEncodeError> {
        if let Some(meta) = self.stream_meta {
            let mut buf = Vec::with_capacity(6);
            buf.extend_from_slice(&meta.chunk_size.to_be_bytes());
            buf.push(meta.checksum_type);
            buf.push(meta.flags);
            self.header.metadata_len = Self::checked_metadata_len(buf.len())?;
            self.metadata = buf;
        } else if let Some(meta) = self.delta_meta {
            let mut buf = Vec::with_capacity(10);
            buf.extend_from_slice(&meta.base_snapshot.to_be_bytes());
            buf.push(meta.algorithm);
            buf.push(meta.compression);
            self.header.metadata_len = Self::checked_metadata_len(buf.len())?;
            self.metadata = buf;
        } else if self.header.metadata_len as usize != self.metadata.len() {
            self.header.metadata_len = Self::checked_metadata_len(self.metadata.len())?;
        }
        Ok(())
    }

    fn validate_flags(&self) -> Result<(), ContainerEncodeError> {
        let flags = self.header.flags;
        // In v1 only the checksum flag is allowed.
        let reserved = flags & !LNMP_FLAG_CHECKSUM_REQUIRED;
        if reserved != 0 {
            return Err(ContainerEncodeError::ReservedFlags(reserved));
        }
        if flags & (LNMP_FLAG_COMPRESSED | LNMP_FLAG_ENCRYPTED) != 0 {
            return Err(ContainerEncodeError::UnsupportedFlags(
                flags & (LNMP_FLAG_COMPRESSED | LNMP_FLAG_ENCRYPTED),
            ));
        }
        Ok(())
    }

    fn validate_checksum_requirements(&self) -> Result<(), ContainerEncodeError> {
        if self.header.flags & LNMP_FLAG_CHECKSUM_REQUIRED == 0 {
            return Ok(());
        }
        if !self.checksum_confirmed {
            return Err(ContainerEncodeError::ChecksumFlagMissingHints);
        }
        Ok(())
    }
}

impl<'a> ContainerFrame<'a> {
    /// Parses a `.lnmp` container from raw bytes.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ContainerFrameError> {
        if bytes.len() < LNMP_HEADER_SIZE {
            return Err(ContainerFrameError::Header(
                LnmpContainerError::TruncatedHeader,
            ));
        }

        let header_bytes = &bytes[..LNMP_HEADER_SIZE];
        let header =
            LnmpContainerHeader::parse(header_bytes).map_err(ContainerFrameError::Header)?;

        let metadata_len = usize::try_from(header.metadata_len)
            .map_err(|_| ContainerFrameError::MetadataLengthOverflow(header.metadata_len))?;

        let available = bytes.len() - LNMP_HEADER_SIZE;
        if available < metadata_len {
            return Err(ContainerFrameError::TruncatedMetadata {
                expected: header.metadata_len,
                available,
            });
        }

        let metadata_start = LNMP_HEADER_SIZE;
        let metadata_end = metadata_start + metadata_len;
        let metadata = &bytes[metadata_start..metadata_end];
        let payload = &bytes[metadata_end..];

        validate_reserved_flags(header.flags)?;
        validate_metadata_requirements(header.mode, metadata_len)?;
        validate_metadata_semantics(header.mode, metadata)?;

        Ok(Self {
            header,
            metadata,
            payload,
        })
    }

    /// Header describing this container.
    pub const fn header(&self) -> LnmpContainerHeader {
        self.header
    }

    /// Metadata segment placed immediately after the header.
    pub fn metadata(&self) -> &'a [u8] {
        self.metadata
    }

    /// Raw payload region.
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    /// Builds a delta apply context from the metadata (if mode is Delta).
    pub fn delta_apply_context(&self) -> Option<DeltaApplyContext> {
        if self.header.mode != LnmpFileMode::Delta {
            return None;
        }
        let meta = parse_delta_metadata(self.metadata).ok()?;
        Some(delta_apply_context_from_metadata(&meta))
    }

    /// Returns a typed view over the payload bytes.
    pub fn body(&self) -> ContainerBody<'a> {
        match self.header.mode {
            LnmpFileMode::Text => ContainerBody::Text(self.payload),
            LnmpFileMode::Binary => ContainerBody::Binary(self.payload),
            LnmpFileMode::Stream => ContainerBody::Stream(self.payload),
            LnmpFileMode::Delta => ContainerBody::Delta(self.payload),
            LnmpFileMode::QuantumSafe => ContainerBody::QuantumSafe(self.payload),
        }
    }

    /// Decodes the payload into a [`LnmpRecord`] using mode-specific codecs.
    pub fn decode_record(&self) -> Result<LnmpRecord, ContainerDecodeError> {
        match self.header.mode {
            LnmpFileMode::Text => self.decode_text_record(),
            LnmpFileMode::Binary => self.decode_binary_record(),
            mode => Err(ContainerDecodeError::UnsupportedMode(mode)),
        }
    }

    /// Parses stream metadata if present (mode `0x03`).
    pub fn stream_metadata(&self) -> Option<Result<StreamMetadata, MetadataError>> {
        if self.header.mode != LnmpFileMode::Stream {
            return None;
        }
        Some(parse_stream_metadata(self.metadata))
    }

    /// Parses delta metadata if present (mode `0x04`).
    pub fn delta_metadata(&self) -> Option<Result<DeltaMetadata, MetadataError>> {
        if self.header.mode != LnmpFileMode::Delta {
            return None;
        }
        Some(parse_delta_metadata(self.metadata))
    }

    /// Canonicalizes the payload into LNMP text using [`Encoder`].
    pub fn decode_to_text(&self) -> Result<String, ContainerDecodeError> {
        let record = self.decode_record()?;
        let encoder = Encoder::new();
        Ok(encoder.encode(&record))
    }

    fn decode_text_record(&self) -> Result<LnmpRecord, ContainerDecodeError> {
        let text = str::from_utf8(self.payload).map_err(ContainerDecodeError::InvalidUtf8)?;
        let mut parser =
            Parser::new(text).map_err(ContainerDecodeError::TextCodec)?;
        parser
            .parse_record()
            .map_err(ContainerDecodeError::TextCodec)
    }

    fn decode_binary_record(&self) -> Result<LnmpRecord, ContainerDecodeError> {
        let decoder = BinaryDecoder::new();
        decoder
            .decode(self.payload)
            .map_err(ContainerDecodeError::BinaryCodec)
    }
}

/// High-level view over the payload region for each mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerBody<'a> {
    /// LNMP/Text payload (UTF-8).
    Text(&'a [u8]),
    /// LNMP/Binary payload.
    Binary(&'a [u8]),
    /// LNMP/Stream payload.
    Stream(&'a [u8]),
    /// LNMP/Delta payload.
    Delta(&'a [u8]),
    /// LNMP/Quantum-Safe payload.
    QuantumSafe(&'a [u8]),
}

/// Errors that can surface while parsing a `.lnmp` container frame.
#[derive(Debug)]
pub enum ContainerFrameError {
    /// Header level validation failed.
    Header(LnmpContainerError),
    /// Reserved flags were set in a v1 container.
    ReservedFlags(u16),
    /// Metadata length does not satisfy mode requirements.
    InvalidMetadataLength {
        /// Mode specified in the header.
        mode: LnmpFileMode,
        /// Expected metadata length for this mode.
        expected: usize,
        /// Actual metadata length from the header.
        actual: usize,
    },
    /// Metadata length exceeded available bytes.
    TruncatedMetadata {
        /// Metadata bytes declared in the header.
        expected: u32,
        /// Metadata bytes available in the frame.
        available: usize,
    },
    /// Metadata length cannot be represented on this platform.
    MetadataLengthOverflow(u32),
    /// Metadata field contains a value that is not allowed for this mode.
    InvalidMetadataValue {
        /// Mode specified in the header.
        mode: LnmpFileMode,
        /// Field name within the metadata.
        field: &'static str,
        /// Offending value.
        value: u8,
    },
}

impl fmt::Display for ContainerFrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerFrameError::Header(err) => write!(f, "{err}"),
            ContainerFrameError::ReservedFlags(flags) => {
                write!(f, "reserved flags set in v1 container: 0x{flags:04x}")
            }
            ContainerFrameError::InvalidMetadataLength {
                mode,
                expected,
                actual,
            } => write!(
                f,
                "mode {mode:?} requires {expected} metadata bytes but header declares {actual}"
            ),
            ContainerFrameError::TruncatedMetadata { expected, available } => {
                write!(
                    f,
                    "metadata expects {expected} bytes but only {available} are available"
                )
            }
            ContainerFrameError::MetadataLengthOverflow(len) => write!(
                f,
                "metadata length {len} cannot be represented on this platform"
            ),
            ContainerFrameError::InvalidMetadataValue { mode, field, value } => {
                write!(
                    f,
                    "mode {mode:?} metadata field {field} contains unsupported value 0x{value:02X}"
                )
            }
        }
    }
}

impl std::error::Error for ContainerFrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ContainerFrameError::Header(err) => Some(err),
            ContainerFrameError::ReservedFlags(_) => None,
            ContainerFrameError::InvalidMetadataLength { .. } => None,
            _ => None,
        }
    }
}

impl From<LnmpContainerError> for ContainerFrameError {
    fn from(value: LnmpContainerError) -> Self {
        Self::Header(value)
    }
}

fn validate_reserved_flags(flags: u16) -> Result<(), ContainerFrameError> {
    const ALLOWED: u16 = LNMP_FLAG_CHECKSUM_REQUIRED;
    let reserved = flags & !ALLOWED;
    if reserved != 0 {
        return Err(ContainerFrameError::ReservedFlags(reserved));
    }
    Ok(())
}

fn validate_metadata_requirements(
    mode: LnmpFileMode,
    metadata_len: usize,
) -> Result<(), ContainerFrameError> {
        match mode {
            LnmpFileMode::Stream => {
                if metadata_len != 6 {
                    return Err(ContainerFrameError::InvalidMetadataLength {
                        mode,
                    expected: 6,
                    actual: metadata_len,
                });
            }
        }
        LnmpFileMode::Delta => {
            if metadata_len != 10 {
                return Err(ContainerFrameError::InvalidMetadataLength {
                    mode,
                    expected: 10,
                    actual: metadata_len,
                });
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_metadata_semantics(
    mode: LnmpFileMode,
    metadata: &[u8],
) -> Result<(), ContainerFrameError> {
    match mode {
        LnmpFileMode::Delta => {
            if metadata.len() >= 9 {
                let algorithm = metadata[8];
                if algorithm != 0x00 && algorithm != 0x01 {
                    return Err(ContainerFrameError::InvalidMetadataValue {
                        mode,
                        field: "algorithm",
                        value: algorithm,
                    });
                }
            }
            if metadata.len() >= 10 {
                let compression = metadata[9];
                if compression != 0x00 && compression != 0x01 {
                    return Err(ContainerFrameError::InvalidMetadataValue {
                        mode,
                        field: "compression",
                        value: compression,
                    });
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn encode_validate_metadata_requirements(
    mode: LnmpFileMode,
    metadata_len: usize,
) -> Result<(), ContainerEncodeError> {
    match mode {
        LnmpFileMode::Stream => {
            if metadata_len != 6 {
                return Err(ContainerEncodeError::InvalidMetadataLength {
                    mode,
                    expected: 6,
                    actual: metadata_len,
                });
            }
        }
        LnmpFileMode::Delta => {
            if metadata_len != 10 {
                return Err(ContainerEncodeError::InvalidMetadataLength {
                    mode,
                    expected: 10,
                    actual: metadata_len,
                });
            }
        }
        _ => {}
    }
    Ok(())
}

fn encode_validate_metadata_semantics(
    mode: LnmpFileMode,
    metadata: &[u8],
) -> Result<(), ContainerEncodeError> {
    match mode {
        LnmpFileMode::Delta => {
            if metadata.len() >= 9 {
                let algorithm = metadata[8];
                if algorithm != 0x00 && algorithm != 0x01 {
                    return Err(ContainerEncodeError::InvalidMetadataValue {
                        mode,
                        field: "algorithm",
                        value: algorithm,
                    });
                }
            }
            if metadata.len() >= 10 {
                let compression = metadata[9];
                if compression != 0x00 && compression != 0x01 {
                    return Err(ContainerEncodeError::InvalidMetadataValue {
                        mode,
                        field: "compression",
                        value: compression,
                    });
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Errors that surface while decoding metadata blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// Metadata buffer is too short.
    Truncated {
        /// Expected number of bytes.
        expected: usize,
        /// Actual metadata length available.
        actual: usize,
    },
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::Truncated { expected, actual } => {
                write!(
                    f,
                    "metadata too short: expected at least {expected} bytes, found {actual}"
                )
            }
        }
    }
}

impl std::error::Error for MetadataError {}

/// Errors returned while decoding the payload content.
#[derive(Debug)]
pub enum ContainerDecodeError {
    /// Failure when parsing the container frame.
    Frame(ContainerFrameError),
    /// Payload contained invalid UTF-8 (text mode only).
    InvalidUtf8(str::Utf8Error),
    /// Text codec reported an error.
    TextCodec(LnmpError),
    /// Binary codec reported an error.
    BinaryCodec(BinaryError),
    /// Mode is not currently supported by the decoder.
    UnsupportedMode(LnmpFileMode),
}

impl fmt::Display for ContainerDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerDecodeError::Frame(err) => write!(f, "{err}"),
            ContainerDecodeError::InvalidUtf8(err) => write!(f, "invalid UTF-8: {err}"),
            ContainerDecodeError::TextCodec(err) => write!(f, "{err}"),
            ContainerDecodeError::BinaryCodec(err) => write!(f, "{err}"),
            ContainerDecodeError::UnsupportedMode(mode) => {
                write!(f, "mode {mode:?} is not supported yet")
            }
        }
    }
}

impl std::error::Error for ContainerDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ContainerDecodeError::Frame(err) => Some(err),
            ContainerDecodeError::InvalidUtf8(err) => Some(err),
            ContainerDecodeError::TextCodec(err) => Some(err),
            ContainerDecodeError::BinaryCodec(err) => Some(err),
            ContainerDecodeError::UnsupportedMode(_) => None,
        }
    }
}

impl From<ContainerFrameError> for ContainerDecodeError {
    fn from(value: ContainerFrameError) -> Self {
        Self::Frame(value)
    }
}

/// Errors produced while emitting `.lnmp` containers.
#[derive(Debug)]
pub enum ContainerEncodeError {
    /// Metadata payload cannot fit in the header field.
    MetadataTooLarge(usize),
    /// Binary encoder failed.
    BinaryCodec(BinaryError),
    /// Mode is not supported for encoding helpers.
    UnsupportedMode(LnmpFileMode),
    /// Requested flags require capabilities that are not available yet.
    UnsupportedFlags(u16),
    /// Reserved flags are set in a v1 container (only checksum is allowed).
    ReservedFlags(u16),
    /// Checksum flag set but record lacks checksum hints.
    ChecksumFlagMissingHints,
    /// Metadata length does not satisfy mode requirements.
    InvalidMetadataLength {
        /// Mode provided.
        mode: LnmpFileMode,
        /// Expected metadata length.
        expected: usize,
        /// Actual metadata length.
        actual: usize,
    },
    /// Metadata field contains a value that is not allowed for this mode.
    InvalidMetadataValue {
        /// Mode provided.
        mode: LnmpFileMode,
        /// Field name.
        field: &'static str,
        /// Offending value.
        value: u8,
    },
}

impl fmt::Display for ContainerEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerEncodeError::MetadataTooLarge(len) => {
                write!(f, "metadata of length {len} is too large for the container")
            }
            ContainerEncodeError::BinaryCodec(err) => write!(f, "{err}"),
            ContainerEncodeError::UnsupportedMode(mode) => {
                write!(f, "mode {mode:?} is not supported for encoding yet")
            }
        ContainerEncodeError::UnsupportedFlags(bits) => write!(
            f,
            "flags {bits:#06X} require compression/encryption which is not implemented"
        ),
        ContainerEncodeError::ReservedFlags(bits) => {
            write!(f, "reserved flags are not allowed in v1: {bits:#06X}")
        }
        ContainerEncodeError::ChecksumFlagMissingHints => write!(
            f,
            "checksum flag is set but no fields contain embedded checksum hints"
        ),
        ContainerEncodeError::InvalidMetadataLength {
            mode,
            expected,
            actual,
        } => write!(
            f,
            "mode {mode:?} requires {expected} metadata bytes but header declares {actual}"
        ),
        ContainerEncodeError::InvalidMetadataValue {
            mode,
            field,
            value,
        } => write!(
            f,
            "mode {mode:?} metadata field {field} contains unsupported value 0x{value:02X}"
        ),
    }
}
}

impl std::error::Error for ContainerEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ContainerEncodeError::BinaryCodec(err) => Some(err),
            _ => None,
        }
    }
}

/// Parses stream metadata bytes into a [`StreamMetadata`] structure.
pub fn parse_stream_metadata(metadata: &[u8]) -> Result<StreamMetadata, MetadataError> {
    if metadata.len() < 6 {
        return Err(MetadataError::Truncated {
            expected: 6,
            actual: metadata.len(),
        });
    }
    let chunk_size = u32::from_be_bytes([metadata[0], metadata[1], metadata[2], metadata[3]]);
    Ok(StreamMetadata {
        chunk_size,
        checksum_type: metadata[4],
        flags: metadata[5],
    })
}

/// Parses delta metadata bytes into a [`DeltaMetadata`] structure.
pub fn parse_delta_metadata(metadata: &[u8]) -> Result<DeltaMetadata, MetadataError> {
    if metadata.len() < 10 {
        return Err(MetadataError::Truncated {
            expected: 10,
            actual: metadata.len(),
        });
    }
    let base_snapshot = u64::from_be_bytes([
        metadata[0], metadata[1], metadata[2], metadata[3], metadata[4], metadata[5], metadata[6],
        metadata[7],
    ]);
    Ok(DeltaMetadata {
        base_snapshot,
        algorithm: metadata[8],
        compression: metadata[9],
    })
}

/// Constructs a delta apply context from decoded delta metadata.
pub fn delta_apply_context_from_metadata(meta: &DeltaMetadata) -> DeltaApplyContext {
    DeltaApplyContext {
        metadata_base: Some(meta.base_snapshot),
        required_base: None,
        algorithm: Some(meta.algorithm),
        compression: Some(meta.compression),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::BinaryEncoder;
    use lnmp_core::{LnmpField, LnmpValue, LNMP_FLAG_CHECKSUM_REQUIRED, LNMP_FLAG_COMPRESSED};

    fn build_container_bytes(
        mode: LnmpFileMode,
        metadata: &[u8],
        payload: &[u8],
    ) -> Vec<u8> {
        let mut header = LnmpContainerHeader::new(mode);
        header.metadata_len = metadata.len() as u32;
        let mut bytes = header.encode().to_vec();
        bytes.extend_from_slice(metadata);
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn parse_text_frame() {
        let payload = b"F7=1\nF12=14532\n";
        let bytes = build_container_bytes(LnmpFileMode::Text, &[], payload);
        let frame = ContainerFrame::parse(&bytes).unwrap();
        assert_eq!(frame.metadata(), b"");
        assert_eq!(frame.payload(), payload);
        assert_eq!(frame.body(), ContainerBody::Text(payload));
        assert_eq!(frame.header().mode, LnmpFileMode::Text);
    }

    #[test]
    fn decode_text_record() {
        let payload = b"F7=1\nF12=14532\n";
        let bytes = build_container_bytes(LnmpFileMode::Text, &[], payload);
        let frame = ContainerFrame::parse(&bytes).unwrap();
        let record = frame.decode_record().unwrap();
        assert_eq!(record.fields().len(), 2);
        let text = frame.decode_to_text().unwrap();
        assert!(text.contains("F7=1"));
        assert!(text.contains("F12=14532"));
    }

    #[test]
    fn decode_binary_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();
        let bytes = build_container_bytes(LnmpFileMode::Binary, &[], &binary);
        let frame = ContainerFrame::parse(&bytes).unwrap();
        let decoded = frame.decode_record().unwrap();
        assert_eq!(decoded.fields().len(), 2);
    }

    #[test]
    fn detect_truncated_metadata() {
        let mut header = LnmpContainerHeader::new(LnmpFileMode::Text);
        header.metadata_len = 4;
        let mut bytes = header.encode().to_vec();
        bytes.extend_from_slice(&[0xAA, 0xBB]);
        let err = ContainerFrame::parse(&bytes).unwrap_err();
        match err {
            ContainerFrameError::TruncatedMetadata { expected, available } => {
                assert_eq!(expected, 4);
                assert_eq!(available, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn builder_wraps_text_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        let builder = ContainerBuilder::new(LnmpFileMode::Text);
        let bytes = builder.encode_record(&record).unwrap();
        let frame = ContainerFrame::parse(&bytes).unwrap();
        assert_eq!(frame.header().mode, LnmpFileMode::Text);
    }

    #[test]
    fn builder_wraps_binary_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        let builder = ContainerBuilder::new(LnmpFileMode::Binary);
        let bytes = builder.encode_record(&record).unwrap();
        let frame = ContainerFrame::parse(&bytes).unwrap();
        assert_eq!(frame.header().mode, LnmpFileMode::Binary);
        assert!(!frame.payload().is_empty());
    }

    #[test]
    fn builder_rejects_compression_flag() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        let builder = ContainerBuilder::new(LnmpFileMode::Text).with_flags(LNMP_FLAG_COMPRESSED);
        let err = builder.encode_record(&record).unwrap_err();
        assert!(matches!(err, ContainerEncodeError::ReservedFlags(_)));
    }

    #[test]
    fn builder_rejects_reserved_flags() {
        let record = LnmpRecord::new();
        let builder = ContainerBuilder::new(LnmpFileMode::Text).with_flags(0x0002);
        let err = builder.encode_record(&record).unwrap_err();
        assert!(matches!(err, ContainerEncodeError::ReservedFlags(_)));
    }

    #[test]
    fn checksum_flag_requires_hint() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(10),
        });
        let builder = ContainerBuilder::new(LnmpFileMode::Text)
            .with_flags(LNMP_FLAG_CHECKSUM_REQUIRED)
            .with_checksum_confirmation(false);
        let err = builder.encode_record(&record).unwrap_err();
        assert!(matches!(
            err,
            ContainerEncodeError::ChecksumFlagMissingHints
        ));
    }

    #[test]
    fn builder_requires_stream_metadata_length() {
        let builder = ContainerBuilder::new(LnmpFileMode::Stream)
            .with_metadata(vec![])
            .unwrap();
        let err = builder.wrap_payload(b"payload").unwrap_err();
        assert!(matches!(
            err,
            ContainerEncodeError::InvalidMetadataLength {
                expected: 6,
                actual: 0,
                ..
            }
        ));
    }

    #[test]
    fn builder_requires_delta_metadata_length() {
        let builder = ContainerBuilder::new(LnmpFileMode::Delta)
            .with_metadata(vec![])
            .unwrap();
        let err = builder.wrap_payload(b"payload").unwrap_err();
        assert!(matches!(
            err,
            ContainerEncodeError::InvalidMetadataLength {
                expected: 10,
                actual: 0,
                ..
            }
        ));
    }

    #[test]
    fn builder_rejects_invalid_delta_algorithm() {
        let builder = ContainerBuilder::new(LnmpFileMode::Delta)
            .with_delta_metadata(DeltaMetadata {
                base_snapshot: 1,
                algorithm: 0xFF,
                compression: 0x00,
            })
            .unwrap();
        let err = builder.wrap_payload(b"payload").unwrap_err();
        assert!(matches!(
            err,
            ContainerEncodeError::InvalidMetadataValue {
                mode: LnmpFileMode::Delta,
                field: "algorithm",
                value: 0xFF
            }
        ));
    }

    #[test]
    fn builder_rejects_invalid_delta_compression() {
        let builder = ContainerBuilder::new(LnmpFileMode::Delta)
            .with_delta_metadata(DeltaMetadata {
                base_snapshot: 1,
                algorithm: 0x00,
                compression: 0xFF,
            })
            .unwrap();
        let err = builder.wrap_payload(b"payload").unwrap_err();
        assert!(matches!(
            err,
            ContainerEncodeError::InvalidMetadataValue {
                mode: LnmpFileMode::Delta,
                field: "compression",
                value: 0xFF
            }
        ));
    }

    #[test]
    fn parse_stream_metadata_bytes() {
        let bytes = [0x00, 0x00, 0x10, 0x00, 0x02, 0x03];
        let meta = parse_stream_metadata(&bytes).unwrap();
        assert_eq!(meta.chunk_size, 4096);
        assert_eq!(meta.checksum_type, 0x02);
        assert_eq!(meta.flags, 0x03);
    }

    #[test]
    fn parse_delta_metadata_bytes() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 5, 0x01, 0x00];
        let meta = parse_delta_metadata(&bytes).unwrap();
        assert_eq!(meta.base_snapshot, 5);
        assert_eq!(meta.algorithm, 0x01);
        assert_eq!(meta.compression, 0x00);
    }
}
