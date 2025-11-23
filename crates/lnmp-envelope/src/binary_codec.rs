//! Binary TLV (Type-Length-Value) codec for envelope metadata
//!
//! Encodes envelope metadata as TLV entries for the LNMP container
//! metadata extension block.
//!
//! ## TLV Format
//!
//! ```text
//! Type (1 byte) | Length (2 bytes, BE) | Value (Length bytes)
//! ```
//!
//! ## Type Codes
//!
//! - `0x10`: Timestamp (u64 big-endian)
//! - `0x11`: Source (UTF-8 string)
//! - `0x12`: TraceID (UTF-8 string)
//! - `0x13`: Sequence (u64 big-endian)
//! - `0x14`: Labels (reserved)
//!
//! ## Canonical Ordering
//!
//! TLV entries MUST appear in ascending type order for determinism.

use crate::{EnvelopeError, EnvelopeMetadata, Result};
use std::io::{Read, Write};

/// TLV type codes
pub mod tlv_type {
    /// Timestamp field (u64 big-endian, Unix epoch milliseconds)
    pub const TIMESTAMP: u8 = 0x10;
    /// Source identifier field (UTF-8 string)
    pub const SOURCE: u8 = 0x11;
    /// Trace ID field (UTF-8 string)
    pub const TRACE_ID: u8 = 0x12;
    /// Sequence number field ( u64 big-endian)
    pub const SEQUENCE: u8 = 0x13;
    /// Labels field (reserved for future use)
    pub const LABELS: u8 = 0x14;
}

/// Binary TLV encoder for envelope metadata
pub struct TlvEncoder;

impl TlvEncoder {
    /// Encodes metadata to TLV binary format
    ///
    /// Entries are written in canonical order:
    /// 1. Timestamp (0x10)
    /// 2. Source (0x11)
    /// 3. TraceID (0x12)
    /// 4. Sequence (0x13)
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_envelope::{EnvelopeMetadata, binary_codec::TlvEncoder};
    ///
    /// let mut metadata = EnvelopeMetadata::new();
    /// metadata.timestamp = Some(1732373147000);
    /// metadata.source = Some("auth-service".to_string());
    ///
    /// let bytes = TlvEncoder::encode(&metadata).unwrap();
    /// assert!(bytes.len() > 0);
    /// ```
    pub fn encode(metadata: &EnvelopeMetadata) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Canonical order: timestamp, source, trace_id, sequence

        if let Some(ts) = metadata.timestamp {
            Self::write_timestamp(&mut buf, ts)?;
        }

        if let Some(ref source) = metadata.source {
            Self::write_source(&mut buf, source)?;
        }

        if let Some(ref trace_id) = metadata.trace_id {
            Self::write_trace_id(&mut buf, trace_id)?;
        }

        if let Some(seq) = metadata.sequence {
            Self::write_sequence(&mut buf, seq)?;
        }

        // Labels reserved for future

        Ok(buf)
    }

    fn write_timestamp<W: Write>(w: &mut W, ts: u64) -> Result<()> {
        w.write_all(&[tlv_type::TIMESTAMP])?;
        w.write_all(&8u16.to_be_bytes())?;
        w.write_all(&ts.to_be_bytes())?;
        Ok(())
    }

    fn write_source<W: Write>(w: &mut W, source: &str) -> Result<()> {
        let bytes = source.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(EnvelopeError::StringTooLong(
                "source".to_string(),
                u16::MAX as usize,
            ));
        }

        w.write_all(&[tlv_type::SOURCE])?;
        w.write_all(&(bytes.len() as u16).to_be_bytes())?;
        w.write_all(bytes)?;
        Ok(())
    }

    fn write_trace_id<W: Write>(w: &mut W, trace_id: &str) -> Result<()> {
        let bytes = trace_id.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(EnvelopeError::StringTooLong(
                "trace_id".to_string(),
                u16::MAX as usize,
            ));
        }

        w.write_all(&[tlv_type::TRACE_ID])?;
        w.write_all(&(bytes.len() as u16).to_be_bytes())?;
        w.write_all(bytes)?;
        Ok(())
    }

    fn write_sequence<W: Write>(w: &mut W, seq: u64) -> Result<()> {
        w.write_all(&[tlv_type::SEQUENCE])?;
        w.write_all(&8u16.to_be_bytes())?;
        w.write_all(&seq.to_be_bytes())?;
        Ok(())
    }
}

/// Binary TLV decoder for envelope metadata
pub struct TlvDecoder;

impl TlvDecoder {
    /// Decodes metadata from TLV binary format
    ///
    /// Unknown TRV types are skipped gracefully for forward compatibility.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_envelope::binary_codec::{TlvEncoder, TlvDecoder};
    /// use lnmp_envelope::EnvelopeMetadata;
    ///
    /// let mut original = EnvelopeMetadata::new();
    /// original.timestamp = Some(1732373147000);
    /// original.source = Some("test".to_string());
    ///
    /// let bytes = TlvEncoder::encode(&original).unwrap();
    /// let decoded = TlvDecoder::decode(&bytes).unwrap();
    ///
    /// assert_eq!(original, decoded);
    /// ```
    pub fn decode(data: &[u8]) -> Result<EnvelopeMetadata> {
        let mut metadata = EnvelopeMetadata::new();
        let mut cursor = std::io::Cursor::new(data);
        let mut last_type: Option<u8> = None;

        while cursor.position() < data.len() as u64 {
            let tlv_type = Self::read_u8(&mut cursor)?;
            let length = Self::read_u16_be(&mut cursor)?;

            // Check canonical ordering
            if let Some(prev) = last_type {
                if tlv_type <= prev {
                    return Err(EnvelopeError::NonCanonicalOrder(tlv_type, prev));
                }
            }
            last_type = Some(tlv_type);

            match tlv_type {
                tlv_type::TIMESTAMP => {
                    // Check for duplicate
                    if metadata.timestamp.is_some() {
                        return Err(EnvelopeError::DuplicateTlvEntry(tlv_type));
                    }
                    metadata.timestamp = Some(Self::read_timestamp(&mut cursor, length)?);
                }
                tlv_type::SOURCE => {
                    // Check for duplicate
                    if metadata.source.is_some() {
                        return Err(EnvelopeError::DuplicateTlvEntry(tlv_type));
                    }
                    metadata.source = Some(Self::read_string(&mut cursor, length)?);
                }
                tlv_type::TRACE_ID => {
                    // Check for duplicate
                    if metadata.trace_id.is_some() {
                        return Err(EnvelopeError::DuplicateTlvEntry(tlv_type));
                    }
                    metadata.trace_id = Some(Self::read_string(&mut cursor, length)?);
                }
                tlv_type::SEQUENCE => {
                    // Check for duplicate
                    if metadata.sequence.is_some() {
                        return Err(EnvelopeError::DuplicateTlvEntry(tlv_type));
                    }
                    metadata.sequence = Some(Self::read_sequence(&mut cursor, length)?);
                }
                _ => {
                    // Unknown type - skip gracefully
                    Self::skip(&mut cursor, length as usize)?;
                }
            }
        }

        Ok(metadata)
    }

    fn read_u8<R: Read>(r: &mut R) -> Result<u8> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)
            .map_err(|_| EnvelopeError::UnexpectedEof(0))?;
        Ok(buf[0])
    }

    fn read_u16_be<R: Read>(r: &mut R) -> Result<u16> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)
            .map_err(|_| EnvelopeError::UnexpectedEof(0))?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u64_be<R: Read>(r: &mut R) -> Result<u64> {
        let mut buf = [0u8; 8];
        r.read_exact(&mut buf)
            .map_err(|_| EnvelopeError::UnexpectedEof(0))?;
        Ok(u64::from_be_bytes(buf))
    }

    fn read_timestamp<R: Read>(r: &mut R, length: u16) -> Result<u64> {
        if length != 8 {
            return Err(EnvelopeError::InvalidTlvLength(length as usize));
        }
        Self::read_u64_be(r)
    }

    fn read_sequence<R: Read>(r: &mut R, length: u16) -> Result<u64> {
        if length != 8 {
            return Err(EnvelopeError::InvalidTlvLength(length as usize));
        }
        Self::read_u64_be(r)
    }

    fn read_string<R: Read>(r: &mut R, length: u16) -> Result<String> {
        let mut buf = vec![0u8; length as usize];
        r.read_exact(&mut buf)
            .map_err(|_| EnvelopeError::UnexpectedEof(0))?;
        String::from_utf8(buf).map_err(|e| e.into())
    }

    fn skip<R: Read>(r: &mut R, length: usize) -> Result<()> {
        let mut buf = vec![0u8; length];
        r.read_exact(&mut buf)
            .map_err(|_| EnvelopeError::UnexpectedEof(0))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_timestamp_only() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.timestamp = Some(1732373147000);

        let bytes = TlvEncoder::encode(&metadata).unwrap();
        let decoded = TlvDecoder::decode(&bytes).unwrap();

        assert_eq!(metadata, decoded);
    }

    #[test]
    fn test_encode_decode_all_fields() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.timestamp = Some(1732373147000);
        metadata.source = Some("auth-service".to_string());
        metadata.trace_id = Some("abc-123-xyz".to_string());
        metadata.sequence = Some(42);

        let bytes = TlvEncoder::encode(&metadata).unwrap();
        let decoded = TlvDecoder::decode(&bytes).unwrap();

        assert_eq!(metadata, decoded);
    }

    #[test]
    fn test_encode_canonical_order() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.sequence = Some(42); // Set last
        metadata.timestamp = Some(123); // Set first

        let bytes = TlvEncoder::encode(&metadata).unwrap();

        // Check bytes start with timestamp type (0x10)
        assert_eq!(bytes[0], tlv_type::TIMESTAMP);
    }

    #[test]
    fn test_decode_rejects_duplicate() {
        let mut buf = Vec::new();

        // Write timestamp twice
        buf.write_all(&[tlv_type::TIMESTAMP]).unwrap();
        buf.write_all(&8u16.to_be_bytes()).unwrap();
        buf.write_all(&123u64.to_be_bytes()).unwrap();

        buf.write_all(&[tlv_type::TIMESTAMP]).unwrap();
        buf.write_all(&8u16.to_be_bytes()).unwrap();
        buf.write_all(&456u64.to_be_bytes()).unwrap();

        let result = TlvDecoder::decode(&buf);
        assert!(result.is_err());
        // Duplicate is caught as NonCanonicalOrder (type <= prev)
        assert!(matches!(
            result,
            Err(EnvelopeError::NonCanonicalOrder(_, _))
        ));
    }

    #[test]
    fn test_decode_rejects_non_canonical_order() {
        let mut buf = Vec::new();

        // Write source before timestamp (wrong order)
        buf.write_all(&[tlv_type::SOURCE]).unwrap();
        buf.write_all(&4u16.to_be_bytes()).unwrap();
        buf.write_all(b"test").unwrap();

        buf.write_all(&[tlv_type::TIMESTAMP]).unwrap();
        buf.write_all(&8u16.to_be_bytes()).unwrap();
        buf.write_all(&123u64.to_be_bytes()).unwrap();

        let result = TlvDecoder::decode(&buf);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(EnvelopeError::NonCanonicalOrder(_, _))
        ));
    }

    #[test]
    fn test_decode_skips_unknown_type() {
        let mut buf = Vec::new();

        // Write timestamp
        buf.write_all(&[tlv_type::TIMESTAMP]).unwrap();
        buf.write_all(&8u16.to_be_bytes()).unwrap();
        buf.write_all(&123u64.to_be_bytes()).unwrap();

        // Write unknown type (0xFF)
        buf.write_all(&[0xFF]).unwrap();
        buf.write_all(&4u16.to_be_bytes()).unwrap();
        buf.write_all(&[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();

        let decoded = TlvDecoder::decode(&buf).unwrap();
        assert_eq!(decoded.timestamp, Some(123));
        assert!(decoded.source.is_none());
    }
}
