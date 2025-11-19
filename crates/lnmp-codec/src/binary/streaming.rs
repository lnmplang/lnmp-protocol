//! Streaming Frame Layer (SFL) for LNMP v0.5
//!
//! This module provides chunked transmission support for large LNMP payloads,
//! enabling streaming with backpressure control and integrity validation.

use super::error::BinaryError;

/// Configuration for streaming operations
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingConfig {
    /// Size of each chunk in bytes (default: 4096)
    pub chunk_size: usize,
    /// Enable compression for payloads
    pub enable_compression: bool,
    /// Enable checksum validation for chunks
    pub enable_checksums: bool,
}

impl StreamingConfig {
    /// Creates a new StreamingConfig with default values
    pub fn new() -> Self {
        Self {
            chunk_size: 4096, // 4KB default
            enable_compression: false,
            enable_checksums: true,
        }
    }

    /// Sets the chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Enables or disables compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Enables or disables checksums
    pub fn with_checksums(mut self, enabled: bool) -> Self {
        self.enable_checksums = enabled;
        self
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming state for encoder
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamingState {
    /// No stream active
    Idle,
    /// Stream in progress
    Streaming {
        /// Number of bytes sent so far
        bytes_sent: usize,
        /// Number of chunks sent so far
        chunks_sent: usize,
    },
    /// Stream completed successfully
    Complete,
    /// Stream failed with error
    Error(String),
}

/// Streaming encoder for chunked transmission
pub struct StreamingEncoder {
    config: StreamingConfig,
    state: StreamingState,
}

impl StreamingEncoder {
    /// Creates a new StreamingEncoder with default configuration
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::new(),
            state: StreamingState::Idle,
        }
    }

    /// Creates a new StreamingEncoder with custom configuration
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            state: StreamingState::Idle,
        }
    }

    /// Emits a BEGIN frame to start the stream
    pub fn begin_stream(&mut self) -> Result<Vec<u8>, StreamingError> {
        match &self.state {
            StreamingState::Idle | StreamingState::Complete | StreamingState::Error(_) => {
                self.state = StreamingState::Streaming {
                    bytes_sent: 0,
                    chunks_sent: 0,
                };
                let frame = StreamingFrame::begin();
                self.encode_frame(&frame)
            }
            StreamingState::Streaming { .. } => Err(StreamingError::UnexpectedFrame {
                expected: FrameType::Chunk,
                found: FrameType::Begin,
            }),
        }
    }

    /// Segments data and emits CHUNK frames
    pub fn write_chunk(&mut self, data: &[u8]) -> Result<Vec<u8>, StreamingError> {
        match &self.state {
            StreamingState::Streaming {
                bytes_sent,
                chunks_sent,
            } => {
                if data.len() > self.config.chunk_size {
                    return Err(StreamingError::ChunkSizeExceeded {
                        size: data.len(),
                        max: self.config.chunk_size,
                    });
                }

                let has_more = false; // Caller determines if more chunks follow
                let frame = StreamingFrame::chunk(data.to_vec(), has_more);

                // Update state
                self.state = StreamingState::Streaming {
                    bytes_sent: bytes_sent + data.len(),
                    chunks_sent: chunks_sent + 1,
                };

                self.encode_frame(&frame)
            }
            StreamingState::Idle => Err(StreamingError::StreamNotStarted),
            StreamingState::Complete => Err(StreamingError::StreamAlreadyComplete),
            StreamingState::Error(_msg) => Err(StreamingError::UnexpectedFrame {
                expected: FrameType::Chunk,
                found: FrameType::Error,
            }),
        }
    }

    /// Emits an END frame to complete the stream
    pub fn end_stream(&mut self) -> Result<Vec<u8>, StreamingError> {
        match &self.state {
            StreamingState::Streaming { .. } => {
                self.state = StreamingState::Complete;
                let frame = StreamingFrame::end();
                self.encode_frame(&frame)
            }
            StreamingState::Idle => Err(StreamingError::StreamNotStarted),
            StreamingState::Complete => Err(StreamingError::StreamAlreadyComplete),
            StreamingState::Error(_) => Err(StreamingError::UnexpectedFrame {
                expected: FrameType::End,
                found: FrameType::Error,
            }),
        }
    }

    /// Emits an ERROR frame
    pub fn error_frame(&mut self, error: &str) -> Result<Vec<u8>, StreamingError> {
        self.state = StreamingState::Error(error.to_string());
        let frame = StreamingFrame::error(error.to_string());
        self.encode_frame(&frame)
    }

    /// Returns the current streaming state
    pub fn state(&self) -> &StreamingState {
        &self.state
    }

    /// Encodes a frame to bytes
    fn encode_frame(&self, frame: &StreamingFrame) -> Result<Vec<u8>, StreamingError> {
        let mut bytes = Vec::new();

        // FRAME_ID (1 byte)
        bytes.push(frame.frame_type.to_u8());

        // FLAGS (1 byte)
        bytes.push(frame.flags.to_u8());

        // CHUNK_SIZE (VarInt)
        bytes.extend(super::varint::encode(frame.chunk_size as i64));

        // CHECKSUM (4 bytes) - only for CHUNK frames with checksums enabled
        if frame.frame_type == FrameType::Chunk && self.config.enable_checksums {
            bytes.extend(&frame.checksum.to_le_bytes());
        } else {
            bytes.extend(&[0u8; 4]);
        }

        // PAYLOAD (variable)
        bytes.extend(&frame.payload);

        Ok(bytes)
    }
}

impl Default for StreamingEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the streaming decoder
#[derive(Debug, Clone, PartialEq)]
pub enum StreamingEvent {
    /// Stream started (BEGIN frame received)
    StreamStarted,
    /// Chunk received
    ChunkReceived {
        /// Number of bytes in this chunk
        bytes: usize,
    },
    /// Stream completed successfully (END frame received)
    StreamComplete {
        /// Total bytes received
        total_bytes: usize,
    },
    /// Stream error (ERROR frame received)
    StreamError {
        /// Error message
        message: String,
    },
}

/// Streaming decoder for receiving chunked transmissions
pub struct StreamingDecoder {
    config: StreamingConfig,
    state: StreamingState,
    buffer: Vec<u8>,
}

impl StreamingDecoder {
    /// Creates a new StreamingDecoder with default configuration
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::new(),
            state: StreamingState::Idle,
            buffer: Vec::new(),
        }
    }

    /// Creates a new StreamingDecoder with custom configuration
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            state: StreamingState::Idle,
            buffer: Vec::new(),
        }
    }

    /// Processes an incoming frame
    pub fn feed_frame(&mut self, frame_bytes: &[u8]) -> Result<StreamingEvent, StreamingError> {
        let frame = self.decode_frame(frame_bytes)?;

        match frame.frame_type {
            FrameType::Begin => match &self.state {
                StreamingState::Idle | StreamingState::Complete | StreamingState::Error(_) => {
                    self.state = StreamingState::Streaming {
                        bytes_sent: 0,
                        chunks_sent: 0,
                    };
                    self.buffer.clear();
                    Ok(StreamingEvent::StreamStarted)
                }
                StreamingState::Streaming { .. } => Err(StreamingError::UnexpectedFrame {
                    expected: FrameType::Chunk,
                    found: FrameType::Begin,
                }),
            },
            FrameType::Chunk => {
                match &self.state {
                    StreamingState::Streaming {
                        bytes_sent,
                        chunks_sent,
                    } => {
                        // Validate checksum if enabled
                        if self.config.enable_checksums {
                            frame.validate_checksum()?;
                        }

                        // Append to buffer
                        let chunk_size = frame.payload.len();
                        self.buffer.extend(&frame.payload);

                        // Update state
                        self.state = StreamingState::Streaming {
                            bytes_sent: bytes_sent + chunk_size,
                            chunks_sent: chunks_sent + 1,
                        };

                        Ok(StreamingEvent::ChunkReceived { bytes: chunk_size })
                    }
                    StreamingState::Idle => Err(StreamingError::StreamNotStarted),
                    StreamingState::Complete => Err(StreamingError::StreamAlreadyComplete),
                    StreamingState::Error(_) => Err(StreamingError::UnexpectedFrame {
                        expected: FrameType::Chunk,
                        found: FrameType::Error,
                    }),
                }
            }
            FrameType::End => match &self.state {
                StreamingState::Streaming { bytes_sent, .. } => {
                    let total_bytes = *bytes_sent;
                    self.state = StreamingState::Complete;
                    Ok(StreamingEvent::StreamComplete { total_bytes })
                }
                StreamingState::Idle => Err(StreamingError::StreamNotStarted),
                StreamingState::Complete => Err(StreamingError::StreamAlreadyComplete),
                StreamingState::Error(_) => Err(StreamingError::UnexpectedFrame {
                    expected: FrameType::End,
                    found: FrameType::Error,
                }),
            },
            FrameType::Error => {
                let message = String::from_utf8_lossy(&frame.payload).to_string();
                self.state = StreamingState::Error(message.clone());
                Ok(StreamingEvent::StreamError { message })
            }
        }
    }

    /// Returns the complete payload if stream is complete
    pub fn get_complete_payload(&self) -> Option<&[u8]> {
        match &self.state {
            StreamingState::Complete => Some(&self.buffer),
            _ => None,
        }
    }

    /// Returns the current streaming state
    pub fn state(&self) -> &StreamingState {
        &self.state
    }

    /// Decodes a frame from bytes
    fn decode_frame(&self, bytes: &[u8]) -> Result<StreamingFrame, StreamingError> {
        if bytes.len() < 7 {
            return Err(StreamingError::BinaryError(BinaryError::UnexpectedEof {
                expected: 7,
                found: bytes.len(),
            }));
        }

        let mut pos = 0;

        // FRAME_ID (1 byte)
        let frame_type = FrameType::from_u8(bytes[pos])?;
        pos += 1;

        // FLAGS (1 byte)
        let flags = FrameFlags::from_u8(bytes[pos]);
        pos += 1;

        // CHUNK_SIZE (VarInt)
        let (chunk_size_i64, varint_len) = super::varint::decode(&bytes[pos..])?;
        let chunk_size = chunk_size_i64 as usize;
        pos += varint_len;

        // CHECKSUM (4 bytes)
        if pos + 4 > bytes.len() {
            return Err(StreamingError::BinaryError(BinaryError::UnexpectedEof {
                expected: pos + 4,
                found: bytes.len(),
            }));
        }
        let checksum =
            u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
        pos += 4;

        // PAYLOAD (variable)
        if pos + chunk_size > bytes.len() {
            return Err(StreamingError::BinaryError(BinaryError::UnexpectedEof {
                expected: pos + chunk_size,
                found: bytes.len(),
            }));
        }
        let payload = bytes[pos..pos + chunk_size].to_vec();

        Ok(StreamingFrame {
            frame_type,
            flags,
            chunk_size,
            checksum,
            payload,
        })
    }
}

impl Default for StreamingDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Backpressure controller for flow control
#[derive(Debug, Clone, PartialEq)]
pub struct BackpressureController {
    /// Maximum number of bytes that can be in flight
    window_size: usize,
    /// Current number of bytes in flight (sent but not acknowledged)
    bytes_in_flight: usize,
}

impl BackpressureController {
    /// Creates a new BackpressureController with default window size (64KB)
    pub fn new() -> Self {
        Self {
            window_size: 65536, // 64KB default
            bytes_in_flight: 0,
        }
    }

    /// Creates a new BackpressureController with custom window size
    pub fn with_window_size(window_size: usize) -> Self {
        Self {
            window_size,
            bytes_in_flight: 0,
        }
    }

    /// Checks if more data can be sent
    pub fn can_send(&self) -> bool {
        self.bytes_in_flight < self.window_size
    }

    /// Returns the number of bytes that can be sent
    pub fn available_window(&self) -> usize {
        self.window_size.saturating_sub(self.bytes_in_flight)
    }

    /// Records that a chunk has been sent
    pub fn on_chunk_sent(&mut self, size: usize) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_add(size);
    }

    /// Records that a chunk has been acknowledged
    pub fn on_chunk_acked(&mut self, size: usize) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(size);
    }

    /// Returns the current window size
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the current bytes in flight
    pub fn bytes_in_flight(&self) -> usize {
        self.bytes_in_flight
    }

    /// Resets the controller state
    pub fn reset(&mut self) {
        self.bytes_in_flight = 0;
    }
}

impl Default for BackpressureController {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame type identifiers for streaming protocol
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Begin frame - signals start of stream
    Begin = 0xA0,
    /// Chunk frame - contains data segment
    Chunk = 0xA1,
    /// End frame - signals completion of stream
    End = 0xA2,
    /// Error frame - signals error condition
    Error = 0xA3,
}

impl FrameType {
    /// Converts a byte to a FrameType
    pub fn from_u8(byte: u8) -> Result<Self, StreamingError> {
        match byte {
            0xA0 => Ok(FrameType::Begin),
            0xA1 => Ok(FrameType::Chunk),
            0xA2 => Ok(FrameType::End),
            0xA3 => Ok(FrameType::Error),
            _ => Err(StreamingError::InvalidFrameType { found: byte }),
        }
    }

    /// Converts the FrameType to a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Flags byte layout for streaming frames
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameFlags {
    /// Bit 0: HAS_MORE - indicates more chunks follow
    pub has_more: bool,
    /// Bit 1: COMPRESSED - indicates payload is compressed
    pub compressed: bool,
    // Bits 2-7: Reserved for future use
}

impl FrameFlags {
    /// Creates a new FrameFlags with default values
    pub fn new() -> Self {
        Self {
            has_more: false,
            compressed: false,
        }
    }

    /// Creates FrameFlags from a byte
    pub fn from_u8(byte: u8) -> Self {
        Self {
            has_more: (byte & 0x01) != 0,
            compressed: (byte & 0x02) != 0,
        }
    }

    /// Converts FrameFlags to a byte
    pub fn to_u8(self) -> u8 {
        let mut byte = 0u8;
        if self.has_more {
            byte |= 0x01;
        }
        if self.compressed {
            byte |= 0x02;
        }
        byte
    }
}

impl Default for FrameFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming frame structure
///
/// Frame layout:
/// ```text
/// ┌──────────┬──────────┬──────────────┬──────────┬─────────────┐
/// │ FRAME_ID │  FLAGS   │ CHUNK_SIZE   │ CHECKSUM │   PAYLOAD   │
/// │ (1 byte) │ (1 byte) │  (VarInt)    │ (4 bytes)│  (variable) │
/// └──────────┴──────────┴──────────────┴──────────┴─────────────┘
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingFrame {
    /// Frame type identifier
    pub frame_type: FrameType,
    /// Frame flags
    pub flags: FrameFlags,
    /// Size of the payload chunk (0 for BEGIN/END/ERROR frames)
    pub chunk_size: usize,
    /// XOR checksum of the payload (0 for BEGIN/END frames)
    pub checksum: u32,
    /// Frame payload data
    pub payload: Vec<u8>,
}

impl StreamingFrame {
    /// Creates a new BEGIN frame
    pub fn begin() -> Self {
        Self {
            frame_type: FrameType::Begin,
            flags: FrameFlags::new(),
            chunk_size: 0,
            checksum: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a new CHUNK frame with payload
    pub fn chunk(payload: Vec<u8>, has_more: bool) -> Self {
        let chunk_size = payload.len();
        let checksum = Self::compute_xor_checksum(&payload);
        let mut flags = FrameFlags::new();
        flags.has_more = has_more;

        Self {
            frame_type: FrameType::Chunk,
            flags,
            chunk_size,
            checksum,
            payload,
        }
    }

    /// Creates a new END frame
    pub fn end() -> Self {
        Self {
            frame_type: FrameType::End,
            flags: FrameFlags::new(),
            chunk_size: 0,
            checksum: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a new ERROR frame with error message
    pub fn error(message: String) -> Self {
        let payload = message.into_bytes();
        let chunk_size = payload.len();

        Self {
            frame_type: FrameType::Error,
            flags: FrameFlags::new(),
            chunk_size,
            checksum: 0,
            payload,
        }
    }

    /// Computes XOR checksum for payload data
    pub fn compute_xor_checksum(data: &[u8]) -> u32 {
        let mut checksum = 0u32;
        for chunk in data.chunks(4) {
            let mut word = 0u32;
            for (i, &byte) in chunk.iter().enumerate() {
                word |= (byte as u32) << (i * 8);
            }
            checksum ^= word;
        }
        checksum
    }

    /// Validates the checksum of this frame
    pub fn validate_checksum(&self) -> Result<(), StreamingError> {
        if self.frame_type == FrameType::Chunk {
            let computed = Self::compute_xor_checksum(&self.payload);
            if computed != self.checksum {
                return Err(StreamingError::ChecksumMismatch {
                    expected: self.checksum,
                    found: computed,
                });
            }
        }
        Ok(())
    }
}

/// Error types for streaming operations
#[derive(Debug, Clone, PartialEq)]
pub enum StreamingError {
    /// Invalid frame type byte
    InvalidFrameType {
        /// The invalid frame type byte
        found: u8,
    },

    /// Checksum mismatch in chunk frame
    ChecksumMismatch {
        /// Expected checksum value
        expected: u32,
        /// Computed checksum value
        found: u32,
    },

    /// Unexpected frame type in sequence
    UnexpectedFrame {
        /// Expected frame type
        expected: FrameType,
        /// Found frame type
        found: FrameType,
    },

    /// Stream not started (no BEGIN frame received)
    StreamNotStarted,

    /// Stream already complete (END frame already received)
    StreamAlreadyComplete,

    /// Chunk size exceeds maximum allowed
    ChunkSizeExceeded {
        /// Actual chunk size
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Binary encoding/decoding error
    BinaryError(BinaryError),
}

impl std::fmt::Display for StreamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingError::InvalidFrameType { found } => {
                write!(f, "Invalid frame type: 0x{:02X}", found)
            }
            StreamingError::ChecksumMismatch { expected, found } => {
                write!(
                    f,
                    "Checksum mismatch: expected 0x{:08X}, found 0x{:08X}",
                    expected, found
                )
            }
            StreamingError::UnexpectedFrame { expected, found } => {
                write!(
                    f,
                    "Unexpected frame: expected {:?}, found {:?}",
                    expected, found
                )
            }
            StreamingError::StreamNotStarted => {
                write!(f, "Stream not started: no BEGIN frame received")
            }
            StreamingError::StreamAlreadyComplete => {
                write!(f, "Stream already complete: END frame already received")
            }
            StreamingError::ChunkSizeExceeded { size, max } => {
                write!(
                    f,
                    "Chunk size exceeded: size {} bytes exceeds maximum {} bytes",
                    size, max
                )
            }
            StreamingError::BinaryError(err) => {
                write!(f, "Binary error: {}", err)
            }
        }
    }
}

impl std::error::Error for StreamingError {}

impl From<BinaryError> for StreamingError {
    fn from(err: BinaryError) -> Self {
        StreamingError::BinaryError(err)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::*;

    #[test]
    fn test_frame_type_from_u8() {
        assert_eq!(FrameType::from_u8(0xA0).unwrap(), FrameType::Begin);
        assert_eq!(FrameType::from_u8(0xA1).unwrap(), FrameType::Chunk);
        assert_eq!(FrameType::from_u8(0xA2).unwrap(), FrameType::End);
        assert_eq!(FrameType::from_u8(0xA3).unwrap(), FrameType::Error);
    }

    #[test]
    fn test_frame_type_from_u8_invalid() {
        assert!(FrameType::from_u8(0x00).is_err());
        assert!(FrameType::from_u8(0xFF).is_err());
        assert!(FrameType::from_u8(0xA4).is_err());
    }

    #[test]
    fn test_frame_type_to_u8() {
        assert_eq!(FrameType::Begin.to_u8(), 0xA0);
        assert_eq!(FrameType::Chunk.to_u8(), 0xA1);
        assert_eq!(FrameType::End.to_u8(), 0xA2);
        assert_eq!(FrameType::Error.to_u8(), 0xA3);
    }

    #[test]
    fn test_frame_type_round_trip() {
        let types = vec![
            FrameType::Begin,
            FrameType::Chunk,
            FrameType::End,
            FrameType::Error,
        ];

        for frame_type in types {
            let byte = frame_type.to_u8();
            let parsed = FrameType::from_u8(byte).unwrap();
            assert_eq!(parsed, frame_type);
        }
    }

    #[test]
    fn test_frame_flags_default() {
        let flags = FrameFlags::new();
        assert!(!flags.has_more);
        assert!(!flags.compressed);
    }

    #[test]
    fn test_frame_flags_from_u8() {
        let flags = FrameFlags::from_u8(0x00);
        assert!(!flags.has_more);
        assert!(!flags.compressed);

        let flags = FrameFlags::from_u8(0x01);
        assert!(flags.has_more);
        assert!(!flags.compressed);

        let flags = FrameFlags::from_u8(0x02);
        assert!(!flags.has_more);
        assert!(flags.compressed);

        let flags = FrameFlags::from_u8(0x03);
        assert!(flags.has_more);
        assert!(flags.compressed);
    }

    #[test]
    fn test_frame_flags_to_u8() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.to_u8(), 0x00);

        flags.has_more = true;
        assert_eq!(flags.to_u8(), 0x01);

        flags.has_more = false;
        flags.compressed = true;
        assert_eq!(flags.to_u8(), 0x02);

        flags.has_more = true;
        assert_eq!(flags.to_u8(), 0x03);
    }

    #[test]
    fn test_frame_flags_round_trip() {
        for byte in 0..=0xFF {
            let flags = FrameFlags::from_u8(byte);
            let back = flags.to_u8();
            // Only bits 0 and 1 are used, so mask the result
            assert_eq!(back, byte & 0x03);
        }
    }

    #[test]
    fn test_streaming_frame_begin() {
        let frame = StreamingFrame::begin();
        assert_eq!(frame.frame_type, FrameType::Begin);
        assert_eq!(frame.chunk_size, 0);
        assert_eq!(frame.checksum, 0);
        assert!(frame.payload.is_empty());
    }

    #[test]
    fn test_streaming_frame_chunk() {
        let payload = vec![1, 2, 3, 4, 5];
        let frame = StreamingFrame::chunk(payload.clone(), true);
        assert_eq!(frame.frame_type, FrameType::Chunk);
        assert_eq!(frame.chunk_size, 5);
        assert!(frame.flags.has_more);
        assert_eq!(frame.payload, payload);
        assert_ne!(frame.checksum, 0);
    }

    #[test]
    fn test_streaming_frame_end() {
        let frame = StreamingFrame::end();
        assert_eq!(frame.frame_type, FrameType::End);
        assert_eq!(frame.chunk_size, 0);
        assert_eq!(frame.checksum, 0);
        assert!(frame.payload.is_empty());
    }

    #[test]
    fn test_streaming_frame_error() {
        let message = "Test error".to_string();
        let frame = StreamingFrame::error(message.clone());
        assert_eq!(frame.frame_type, FrameType::Error);
        assert_eq!(frame.chunk_size, message.len());
        assert_eq!(frame.payload, message.as_bytes());
    }

    #[test]
    fn test_compute_xor_checksum_empty() {
        let checksum = StreamingFrame::compute_xor_checksum(&[]);
        assert_eq!(checksum, 0);
    }

    #[test]
    fn test_compute_xor_checksum_single_byte() {
        let checksum = StreamingFrame::compute_xor_checksum(&[0x42]);
        assert_eq!(checksum, 0x42);
    }

    #[test]
    fn test_compute_xor_checksum_four_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let checksum = StreamingFrame::compute_xor_checksum(&data);
        // 0x04030201 in little-endian
        assert_eq!(checksum, 0x04030201);
    }

    #[test]
    fn test_compute_xor_checksum_multiple_words() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let checksum = StreamingFrame::compute_xor_checksum(&data);
        // XOR of 0x04030201 and 0x08070605
        assert_eq!(checksum, 0x04030201 ^ 0x08070605);
    }

    #[test]
    fn test_compute_xor_checksum_partial_word() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let checksum = StreamingFrame::compute_xor_checksum(&data);
        // XOR of 0x04030201 and 0x00000005
        assert_eq!(checksum, 0x04030201 ^ 0x00000005);
    }

    #[test]
    fn test_validate_checksum_chunk_valid() {
        let payload = vec![1, 2, 3, 4, 5];
        let frame = StreamingFrame::chunk(payload, false);
        assert!(frame.validate_checksum().is_ok());
    }

    #[test]
    fn test_validate_checksum_chunk_invalid() {
        let payload = vec![1, 2, 3, 4, 5];
        let mut frame = StreamingFrame::chunk(payload, false);
        frame.checksum = 0xDEADBEEF; // Wrong checksum
        assert!(frame.validate_checksum().is_err());
    }

    #[test]
    fn test_validate_checksum_non_chunk() {
        let frame = StreamingFrame::begin();
        assert!(frame.validate_checksum().is_ok());

        let frame = StreamingFrame::end();
        assert!(frame.validate_checksum().is_ok());

        let frame = StreamingFrame::error("test".to_string());
        assert!(frame.validate_checksum().is_ok());
    }

    #[test]
    fn test_streaming_error_display() {
        let err = StreamingError::InvalidFrameType { found: 0xFF };
        assert!(format!("{}", err).contains("0xFF"));

        let err = StreamingError::ChecksumMismatch {
            expected: 0x1234,
            found: 0x5678,
        };
        assert!(format!("{}", err).contains("0x00001234"));
        assert!(format!("{}", err).contains("0x00005678"));

        let err = StreamingError::StreamNotStarted;
        assert!(format!("{}", err).contains("not started"));

        let err = StreamingError::StreamAlreadyComplete;
        assert!(format!("{}", err).contains("already complete"));
    }

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::new();
        assert_eq!(config.chunk_size, 4096);
        assert!(!config.enable_compression);
        assert!(config.enable_checksums);
    }

    #[test]
    fn test_streaming_config_builder() {
        let config = StreamingConfig::new()
            .with_chunk_size(8192)
            .with_compression(true)
            .with_checksums(false);

        assert_eq!(config.chunk_size, 8192);
        assert!(config.enable_compression);
        assert!(!config.enable_checksums);
    }

    #[test]
    fn test_streaming_config_default_trait() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 4096);
        assert!(!config.enable_compression);
        assert!(config.enable_checksums);
    }

    #[test]
    fn test_streaming_encoder_new() {
        let encoder = StreamingEncoder::new();
        assert_eq!(encoder.state, StreamingState::Idle);
    }

    #[test]
    fn test_streaming_encoder_begin_stream() {
        let mut encoder = StreamingEncoder::new();
        let result = encoder.begin_stream();
        assert!(result.is_ok());

        match encoder.state {
            StreamingState::Streaming {
                bytes_sent,
                chunks_sent,
            } => {
                assert_eq!(bytes_sent, 0);
                assert_eq!(chunks_sent, 0);
            }
            _ => panic!("Expected Streaming state"),
        }
    }

    #[test]
    fn test_streaming_encoder_write_chunk() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();

        let data = vec![1, 2, 3, 4, 5];
        let result = encoder.write_chunk(&data);
        assert!(result.is_ok());

        match encoder.state {
            StreamingState::Streaming {
                bytes_sent,
                chunks_sent,
            } => {
                assert_eq!(bytes_sent, 5);
                assert_eq!(chunks_sent, 1);
            }
            _ => panic!("Expected Streaming state"),
        }
    }

    #[test]
    fn test_streaming_encoder_write_chunk_without_begin() {
        let mut encoder = StreamingEncoder::new();
        let data = vec![1, 2, 3];
        let result = encoder.write_chunk(&data);
        assert!(matches!(result, Err(StreamingError::StreamNotStarted)));
    }

    #[test]
    fn test_streaming_encoder_write_chunk_exceeds_size() {
        let config = StreamingConfig::new().with_chunk_size(10);
        let mut encoder = StreamingEncoder::with_config(config);
        encoder.begin_stream().unwrap();

        let data = vec![0u8; 20]; // Exceeds chunk size
        let result = encoder.write_chunk(&data);
        assert!(matches!(
            result,
            Err(StreamingError::ChunkSizeExceeded { .. })
        ));
    }

    #[test]
    fn test_streaming_encoder_end_stream() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();

        let result = encoder.end_stream();
        assert!(result.is_ok());
        assert_eq!(encoder.state, StreamingState::Complete);
    }

    #[test]
    fn test_streaming_encoder_end_stream_without_begin() {
        let mut encoder = StreamingEncoder::new();
        let result = encoder.end_stream();
        assert!(matches!(result, Err(StreamingError::StreamNotStarted)));
    }

    #[test]
    fn test_streaming_encoder_error_frame() {
        let mut encoder = StreamingEncoder::new();
        let result = encoder.error_frame("Test error");
        assert!(result.is_ok());

        match encoder.state {
            StreamingState::Error(msg) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected Error state"),
        }
    }

    #[test]
    fn test_streaming_encoder_full_flow() {
        let mut encoder = StreamingEncoder::new();

        // Begin
        let begin_bytes = encoder.begin_stream().unwrap();
        assert!(!begin_bytes.is_empty());

        // Write chunks
        let chunk1 = vec![1, 2, 3];
        let chunk1_bytes = encoder.write_chunk(&chunk1).unwrap();
        assert!(!chunk1_bytes.is_empty());

        let chunk2 = vec![4, 5, 6];
        let chunk2_bytes = encoder.write_chunk(&chunk2).unwrap();
        assert!(!chunk2_bytes.is_empty());

        // End
        let end_bytes = encoder.end_stream().unwrap();
        assert!(!end_bytes.is_empty());

        assert_eq!(encoder.state, StreamingState::Complete);
    }

    #[test]
    fn test_streaming_encoder_encode_frame_begin() {
        let encoder = StreamingEncoder::new();
        let frame = StreamingFrame::begin();
        let bytes = encoder.encode_frame(&frame).unwrap();

        // Should have: FRAME_ID (1) + FLAGS (1) + CHUNK_SIZE (VarInt) + CHECKSUM (4)
        assert!(bytes.len() >= 7);
        assert_eq!(bytes[0], FrameType::Begin.to_u8());
    }

    #[test]
    fn test_streaming_encoder_encode_frame_chunk() {
        let encoder = StreamingEncoder::new();
        let payload = vec![1, 2, 3, 4, 5];
        let frame = StreamingFrame::chunk(payload.clone(), false);
        let bytes = encoder.encode_frame(&frame).unwrap();

        // Should have: FRAME_ID (1) + FLAGS (1) + CHUNK_SIZE (VarInt) + CHECKSUM (4) + PAYLOAD
        assert!(bytes.len() >= 7 + payload.len());
        assert_eq!(bytes[0], FrameType::Chunk.to_u8());
    }

    #[test]
    fn test_streaming_encoder_multiple_chunks() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();

        for i in 0..5 {
            let data = vec![i; 10];
            let result = encoder.write_chunk(&data);
            assert!(result.is_ok());
        }

        match encoder.state {
            StreamingState::Streaming {
                bytes_sent,
                chunks_sent,
            } => {
                assert_eq!(bytes_sent, 50);
                assert_eq!(chunks_sent, 5);
            }
            _ => panic!("Expected Streaming state"),
        }
    }

    #[test]
    fn test_streaming_decoder_new() {
        let decoder = StreamingDecoder::new();
        assert_eq!(decoder.state, StreamingState::Idle);
        assert!(decoder.buffer.is_empty());
    }

    #[test]
    fn test_streaming_decoder_feed_begin() {
        let mut encoder = StreamingEncoder::new();
        let begin_bytes = encoder.begin_stream().unwrap();

        let mut decoder = StreamingDecoder::new();
        let event = decoder.feed_frame(&begin_bytes).unwrap();

        assert_eq!(event, StreamingEvent::StreamStarted);
        match decoder.state {
            StreamingState::Streaming { .. } => {}
            _ => panic!("Expected Streaming state"),
        }
    }

    #[test]
    fn test_streaming_decoder_feed_chunk() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();

        let data = vec![1, 2, 3, 4, 5];
        let chunk_bytes = encoder.write_chunk(&data).unwrap();

        let mut decoder = StreamingDecoder::new();
        decoder
            .feed_frame(&encoder.encode_frame(&StreamingFrame::begin()).unwrap())
            .unwrap();

        let event = decoder.feed_frame(&chunk_bytes).unwrap();
        assert_eq!(event, StreamingEvent::ChunkReceived { bytes: 5 });
    }

    #[test]
    fn test_streaming_decoder_feed_end() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();
        let end_bytes = encoder.end_stream().unwrap();

        let mut decoder = StreamingDecoder::new();
        decoder
            .feed_frame(&encoder.encode_frame(&StreamingFrame::begin()).unwrap())
            .unwrap();

        let event = decoder.feed_frame(&end_bytes).unwrap();
        match event {
            StreamingEvent::StreamComplete { total_bytes } => {
                assert_eq!(total_bytes, 0);
            }
            _ => panic!("Expected StreamComplete event"),
        }
    }

    #[test]
    fn test_streaming_decoder_feed_error() {
        let mut encoder = StreamingEncoder::new();
        let error_bytes = encoder.error_frame("Test error").unwrap();

        let mut decoder = StreamingDecoder::new();
        let event = decoder.feed_frame(&error_bytes).unwrap();

        match event {
            StreamingEvent::StreamError { message } => {
                assert_eq!(message, "Test error");
            }
            _ => panic!("Expected StreamError event"),
        }
    }

    #[test]
    fn test_streaming_decoder_chunk_without_begin() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();
        let chunk_bytes = encoder.write_chunk(&[1, 2, 3]).unwrap();

        let mut decoder = StreamingDecoder::new();
        let result = decoder.feed_frame(&chunk_bytes);
        assert!(matches!(result, Err(StreamingError::StreamNotStarted)));
    }

    #[test]
    fn test_streaming_decoder_get_complete_payload() {
        let mut encoder = StreamingEncoder::new();
        encoder.begin_stream().unwrap();

        let data1 = vec![1, 2, 3];
        let data2 = vec![4, 5, 6];

        let chunk1_bytes = encoder.write_chunk(&data1).unwrap();
        let chunk2_bytes = encoder.write_chunk(&data2).unwrap();
        let end_bytes = encoder.end_stream().unwrap();

        let mut decoder = StreamingDecoder::new();
        decoder
            .feed_frame(&encoder.encode_frame(&StreamingFrame::begin()).unwrap())
            .unwrap();
        decoder.feed_frame(&chunk1_bytes).unwrap();
        decoder.feed_frame(&chunk2_bytes).unwrap();
        decoder.feed_frame(&end_bytes).unwrap();

        let payload = decoder.get_complete_payload().unwrap();
        assert_eq!(payload, &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_streaming_decoder_get_complete_payload_before_end() {
        let mut decoder = StreamingDecoder::new();
        assert!(decoder.get_complete_payload().is_none());

        let mut encoder = StreamingEncoder::new();
        let begin_bytes = encoder.begin_stream().unwrap();
        decoder.feed_frame(&begin_bytes).unwrap();

        assert!(decoder.get_complete_payload().is_none());
    }

    #[test]
    fn test_streaming_round_trip() {
        let mut encoder = StreamingEncoder::new();
        let mut decoder = StreamingDecoder::new();

        // Begin
        let begin_bytes = encoder.begin_stream().unwrap();
        let event = decoder.feed_frame(&begin_bytes).unwrap();
        assert_eq!(event, StreamingEvent::StreamStarted);

        // Chunks
        let data1 = vec![1, 2, 3, 4, 5];
        let chunk1_bytes = encoder.write_chunk(&data1).unwrap();
        let event = decoder.feed_frame(&chunk1_bytes).unwrap();
        assert_eq!(event, StreamingEvent::ChunkReceived { bytes: 5 });

        let data2 = vec![6, 7, 8];
        let chunk2_bytes = encoder.write_chunk(&data2).unwrap();
        let event = decoder.feed_frame(&chunk2_bytes).unwrap();
        assert_eq!(event, StreamingEvent::ChunkReceived { bytes: 3 });

        // End
        let end_bytes = encoder.end_stream().unwrap();
        let event = decoder.feed_frame(&end_bytes).unwrap();
        match event {
            StreamingEvent::StreamComplete { total_bytes } => {
                assert_eq!(total_bytes, 8);
            }
            _ => panic!("Expected StreamComplete event"),
        }

        // Verify payload
        let payload = decoder.get_complete_payload().unwrap();
        assert_eq!(payload, &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_streaming_decoder_checksum_validation() {
        let config = StreamingConfig::new().with_checksums(true);
        let mut encoder = StreamingEncoder::with_config(config.clone());
        let mut decoder = StreamingDecoder::with_config(config);

        encoder.begin_stream().unwrap();
        let data = vec![1, 2, 3, 4, 5];
        let mut chunk_bytes = encoder.write_chunk(&data).unwrap();

        // Corrupt the checksum
        if chunk_bytes.len() > 10 {
            chunk_bytes[5] ^= 0xFF;
        }

        decoder
            .feed_frame(&encoder.encode_frame(&StreamingFrame::begin()).unwrap())
            .unwrap();
        let result = decoder.feed_frame(&chunk_bytes);
        assert!(matches!(
            result,
            Err(StreamingError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_streaming_decoder_invalid_frame_bytes() {
        let mut decoder = StreamingDecoder::new();
        let invalid_bytes = vec![0xFF, 0x00]; // Too short
        let result = decoder.feed_frame(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_backpressure_controller_new() {
        let controller = BackpressureController::new();
        assert_eq!(controller.window_size(), 65536);
        assert_eq!(controller.bytes_in_flight(), 0);
        assert!(controller.can_send());
    }

    #[test]
    fn test_backpressure_controller_with_window_size() {
        let controller = BackpressureController::with_window_size(8192);
        assert_eq!(controller.window_size(), 8192);
        assert_eq!(controller.bytes_in_flight(), 0);
    }

    #[test]
    fn test_backpressure_controller_can_send() {
        let mut controller = BackpressureController::with_window_size(1000);
        assert!(controller.can_send());

        controller.on_chunk_sent(500);
        assert!(controller.can_send());

        controller.on_chunk_sent(500);
        assert!(!controller.can_send());
    }

    #[test]
    fn test_backpressure_controller_available_window() {
        let mut controller = BackpressureController::with_window_size(1000);
        assert_eq!(controller.available_window(), 1000);

        controller.on_chunk_sent(300);
        assert_eq!(controller.available_window(), 700);

        controller.on_chunk_sent(400);
        assert_eq!(controller.available_window(), 300);
    }

    #[test]
    fn test_backpressure_controller_on_chunk_sent() {
        let mut controller = BackpressureController::new();
        assert_eq!(controller.bytes_in_flight(), 0);

        controller.on_chunk_sent(100);
        assert_eq!(controller.bytes_in_flight(), 100);

        controller.on_chunk_sent(200);
        assert_eq!(controller.bytes_in_flight(), 300);
    }

    #[test]
    fn test_backpressure_controller_on_chunk_acked() {
        let mut controller = BackpressureController::new();
        controller.on_chunk_sent(500);
        assert_eq!(controller.bytes_in_flight(), 500);

        controller.on_chunk_acked(200);
        assert_eq!(controller.bytes_in_flight(), 300);

        controller.on_chunk_acked(300);
        assert_eq!(controller.bytes_in_flight(), 0);
    }

    #[test]
    fn test_backpressure_controller_reset() {
        let mut controller = BackpressureController::new();
        controller.on_chunk_sent(1000);
        assert_eq!(controller.bytes_in_flight(), 1000);

        controller.reset();
        assert_eq!(controller.bytes_in_flight(), 0);
        assert!(controller.can_send());
    }

    #[test]
    fn test_backpressure_controller_saturating_add() {
        let mut controller = BackpressureController::with_window_size(100);
        controller.on_chunk_sent(usize::MAX);
        // Should saturate, not overflow
        assert!(controller.bytes_in_flight() > 0);
    }

    #[test]
    fn test_backpressure_controller_saturating_sub() {
        let mut controller = BackpressureController::new();
        controller.on_chunk_sent(100);
        controller.on_chunk_acked(200); // Ack more than sent
                                        // Should saturate at 0, not underflow
        assert_eq!(controller.bytes_in_flight(), 0);
    }

    #[test]
    fn test_backpressure_controller_flow_control_scenario() {
        let mut controller = BackpressureController::with_window_size(10000);

        // Send chunks until window is full
        let chunk_size = 2000;
        let mut chunks_sent = 0;

        while controller.can_send() && controller.available_window() >= chunk_size {
            controller.on_chunk_sent(chunk_size);
            chunks_sent += 1;
        }

        assert_eq!(chunks_sent, 5); // 5 * 2000 = 10000
        assert!(!controller.can_send());

        // Acknowledge some chunks
        controller.on_chunk_acked(chunk_size);
        controller.on_chunk_acked(chunk_size);

        // Should be able to send again
        assert!(controller.can_send());
        assert_eq!(controller.available_window(), 4000);
    }

    #[test]
    fn test_backpressure_controller_default() {
        let controller = BackpressureController::default();
        assert_eq!(controller.window_size(), 65536);
        assert_eq!(controller.bytes_in_flight(), 0);
    }
}
