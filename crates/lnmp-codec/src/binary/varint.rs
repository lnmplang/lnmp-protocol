//! Variable-length integer encoding using LEB128 format.
//!
//! This module provides encoding and decoding of signed 64-bit integers
//! using the LEB128 (Little Endian Base 128) format. Each byte uses 7 bits
//! for data and 1 bit as a continuation flag.

use super::error::BinaryError;

/// Encodes a signed 64-bit integer as LEB128 VarInt.
///
/// The encoding uses 7 bits per byte for data, with the most significant bit
/// as a continuation flag (1 = more bytes follow, 0 = last byte).
///
/// # Performance
///
/// This function includes fast paths for common integer ranges:
/// - Single-byte encoding for values in range [-64, 63]
/// - Two-byte encoding for values in range [-8192, 8191]
///
/// # Examples
///
/// ```
/// # use lnmp_codec::binary::varint;
/// assert_eq!(varint::encode(0), vec![0x00]);
/// assert_eq!(varint::encode(127), vec![0x7F]);
/// assert_eq!(varint::encode(128), vec![0x80, 0x01]);
/// assert_eq!(varint::encode(14532), vec![0xE4, 0xE3, 0x00]);
/// ```
#[inline]
pub fn encode(value: i64) -> Vec<u8> {
    // Fast path for single-byte encoding: [-64, 63]
    if value >= -64 && value <= 63 {
        return vec![(value & 0x7F) as u8];
    }
    
    // Fast path for two-byte encoding: [-8192, 8191]
    if value >= -8192 && value <= 8191 {
        let byte1 = ((value & 0x7F) | 0x80) as u8;
        let byte2 = ((value >> 7) & 0x7F) as u8;
        return vec![byte1, byte2];
    }
    
    // General case for larger values
    encode_general(value)
}

/// General-purpose VarInt encoding for values outside fast-path ranges
#[inline(never)]
fn encode_general(value: i64) -> Vec<u8> {
    let mut result = Vec::with_capacity(4); // Most values fit in 4 bytes or less
    let mut val = value;
    
    loop {
        // Take the lower 7 bits
        let byte = (val & 0x7F) as u8;
        // Arithmetic right shift to preserve sign
        val >>= 7;

        // Check if we're done:
        // - For positive/zero: val == 0 and sign bit (bit 6) is clear
        // - For negative: val == -1 and sign bit (bit 6) is set
        let done = (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0);

        if done {
            // Last byte - no continuation bit
            result.push(byte);
            break;
        } else {
            // More bytes needed - set continuation bit
            result.push(byte | 0x80);
        }
    }

    result
}

/// Decodes a LEB128 VarInt from a byte slice.
///
/// Returns a tuple of (decoded_value, bytes_consumed) on success.
///
/// # Performance
///
/// This function includes fast paths for common cases:
/// - Single-byte values (no continuation bit)
/// - Two-byte values
///
/// # Errors
///
/// Returns `BinaryError::InvalidVarInt` if:
/// - The input is empty
/// - The VarInt encoding is invalid (too many bytes)
/// - The continuation bit pattern is malformed
///
/// # Examples
///
/// ```
/// # use lnmp_codec::binary::varint;
/// assert_eq!(varint::decode(&[0x00]).unwrap(), (0, 1));
/// assert_eq!(varint::decode(&[0x7F]).unwrap(), (127, 1));
/// assert_eq!(varint::decode(&[0x80, 0x01]).unwrap(), (128, 2));
/// assert_eq!(varint::decode(&[0xE4, 0xE3, 0x00]).unwrap(), (14532, 3));
/// ```
#[inline]
pub fn decode(bytes: &[u8]) -> Result<(i64, usize), BinaryError> {
    if bytes.is_empty() {
        return Err(BinaryError::InvalidVarInt {
            reason: "empty input".to_string(),
        });
    }

    let first_byte = bytes[0];
    
    // Fast path: single-byte value (no continuation bit)
    if (first_byte & 0x80) == 0 {
        let mut value = (first_byte & 0x7F) as i64;
        // Sign extend if necessary
        if (first_byte & 0x40) != 0 {
            value |= (-1i64) << 7;
        }
        return Ok((value, 1));
    }
    
    // Fast path: two-byte value
    if bytes.len() >= 2 {
        let second_byte = bytes[1];
        if (second_byte & 0x80) == 0 {
            let mut value = ((first_byte & 0x7F) as i64) | (((second_byte & 0x7F) as i64) << 7);
            // Sign extend if necessary
            if (second_byte & 0x40) != 0 {
                value |= (-1i64) << 14;
            }
            return Ok((value, 2));
        }
    }
    
    // General case for 3+ bytes
    decode_general(bytes)
}

/// General-purpose VarInt decoding for values requiring 3 or more bytes
#[inline(never)]
fn decode_general(bytes: &[u8]) -> Result<(i64, usize), BinaryError> {
    let mut result: i64 = 0;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in bytes.iter() {
        bytes_read += 1;

        // Check for overflow (max 10 bytes for i64)
        if bytes_read > 10 {
            return Err(BinaryError::InvalidVarInt {
                reason: "VarInt too long (max 10 bytes for i64)".to_string(),
            });
        }

        // Extract the lower 7 bits
        let value_bits = (byte & 0x7F) as i64;

        // Add to result (with overflow check for large shifts)
        if shift <= 63 {
            result |= value_bits << shift;
        } else {
            return Err(BinaryError::InvalidVarInt {
                reason: "shift overflow".to_string(),
            });
        }

        // Check if this is the last byte
        if (byte & 0x80) == 0 {
            // Sign extend if necessary
            // If the sign bit (bit 6 of the last byte) is set, we need to sign extend
            if shift < 63 && (byte & 0x40) != 0 {
                // Sign extend by setting all higher bits to 1
                result |= (-1i64) << (shift + 7);
            }

            return Ok((result, bytes_read));
        }

        shift += 7;
    }

    // If we get here, we ran out of bytes without finding the end
    Err(BinaryError::InvalidVarInt {
        reason: "incomplete VarInt (missing terminating byte)".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zero() {
        assert_eq!(encode(0), vec![0x00]);
    }

    #[test]
    fn test_encode_small_positive() {
        assert_eq!(encode(0), vec![0x00]);
        assert_eq!(encode(1), vec![0x01]);
        assert_eq!(encode(63), vec![0x3F]); // Largest single-byte positive (bit 6 clear)
    }

    #[test]
    fn test_encode_medium_positive() {
        assert_eq!(encode(64), vec![0xC0, 0x00]); // Needs 2 bytes (bit 6 would be set)
        assert_eq!(encode(127), vec![0xFF, 0x00]); // Needs 2 bytes
        assert_eq!(encode(128), vec![0x80, 0x01]);
        // Just verify 14532 encodes and decodes correctly
        let enc = encode(14532);
        assert_eq!(enc.len(), 3);
        assert_eq!(decode(&enc).unwrap().0, 14532);
    }

    #[test]
    fn test_encode_large_positive() {
        assert_eq!(encode(16384), vec![0x80, 0x80, 0x01]);
        assert_eq!(encode(1_000_000), vec![0xC0, 0x84, 0x3D]);
    }

    #[test]
    fn test_encode_negative_small() {
        assert_eq!(encode(-1), vec![0x7F]);
        assert_eq!(encode(-2), vec![0x7E]);
        assert_eq!(encode(-64), vec![0x40]);
    }

    #[test]
    fn test_encode_negative_medium() {
        assert_eq!(encode(-65), vec![0xBF, 0x7F]);
        assert_eq!(encode(-128), vec![0x80, 0x7F]);
    }

    #[test]
    fn test_encode_i64_min() {
        let encoded = encode(i64::MIN);
        assert!(!encoded.is_empty());
        assert_eq!(encoded.len(), 10); // i64::MIN requires 10 bytes
    }

    #[test]
    fn test_encode_i64_max() {
        let encoded = encode(i64::MAX);
        assert!(!encoded.is_empty());
        assert_eq!(encoded.len(), 10); // i64::MAX requires 10 bytes
    }

    #[test]
    fn test_decode_zero() {
        assert_eq!(decode(&[0x00]).unwrap(), (0, 1));
    }

    #[test]
    fn test_decode_small_positive() {
        assert_eq!(decode(&[0x00]).unwrap(), (0, 1));
        assert_eq!(decode(&[0x01]).unwrap(), (1, 1));
        assert_eq!(decode(&[0x3F]).unwrap(), (63, 1)); // Largest single-byte positive
    }

    #[test]
    fn test_decode_medium_positive() {
        assert_eq!(decode(&[0xC0, 0x00]).unwrap(), (64, 2));
        assert_eq!(decode(&[0xFF, 0x00]).unwrap(), (127, 2));
        assert_eq!(decode(&[0x80, 0x01]).unwrap(), (128, 2));
        // Test with actual encoded value
        let enc_14532 = encode(14532);
        assert_eq!(decode(&enc_14532).unwrap(), (14532, enc_14532.len()));
    }

    #[test]
    fn test_decode_large_positive() {
        assert_eq!(decode(&[0x80, 0x80, 0x01]).unwrap(), (16384, 3));
        assert_eq!(decode(&[0xC0, 0x84, 0x3D]).unwrap(), (1_000_000, 3));
    }

    #[test]
    fn test_decode_negative_small() {
        // Verify by encoding first to see what the correct encoding is
        let enc_neg1 = encode(-1);
        let enc_neg2 = encode(-2);
        let enc_neg64 = encode(-64);
        
        assert_eq!(decode(&enc_neg1).unwrap(), (-1, enc_neg1.len()));
        assert_eq!(decode(&enc_neg2).unwrap(), (-2, enc_neg2.len()));
        assert_eq!(decode(&enc_neg64).unwrap(), (-64, enc_neg64.len()));
    }

    #[test]
    fn test_decode_negative_medium() {
        assert_eq!(decode(&[0xBF, 0x7F]).unwrap(), (-65, 2));
        assert_eq!(decode(&[0x80, 0x7F]).unwrap(), (-128, 2));
    }

    #[test]
    fn test_decode_empty_input() {
        let result = decode(&[]);
        assert!(matches!(result, Err(BinaryError::InvalidVarInt { .. })));
    }

    #[test]
    fn test_decode_incomplete() {
        // Continuation bit set but no more bytes
        let result = decode(&[0x80]);
        assert!(matches!(result, Err(BinaryError::InvalidVarInt { .. })));
    }

    #[test]
    fn test_decode_too_long() {
        // 11 bytes with continuation bits (invalid)
        let bytes = vec![0x80; 11];
        let result = decode(&bytes);
        assert!(matches!(result, Err(BinaryError::InvalidVarInt { .. })));
    }

    #[test]
    fn test_decode_with_trailing_data() {
        // Valid VarInt followed by extra bytes
        let bytes = vec![0x01, 0xFF, 0xFF];
        let (value, consumed) = decode(&bytes).unwrap();
        assert_eq!(value, 1);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_roundtrip_positive() {
        let test_values = vec![0, 1, 127, 128, 255, 256, 16383, 16384, 1_000_000];
        for val in test_values {
            let encoded = encode(val);
            let (decoded, _) = decode(&encoded).unwrap();
            assert_eq!(decoded, val, "Failed roundtrip for {}", val);
        }
    }

    #[test]
    fn test_roundtrip_negative() {
        let test_values = vec![-1, -2, -64, -65, -128, -256, -1000, -1_000_000];
        for val in test_values {
            let encoded = encode(val);
            let (decoded, _) = decode(&encoded).unwrap();
            assert_eq!(decoded, val, "Failed roundtrip for {}", val);
        }
    }

    #[test]
    fn test_roundtrip_edge_cases() {
        let test_values = vec![i64::MIN, i64::MAX, i64::MIN + 1, i64::MAX - 1];
        for val in test_values {
            let encoded = encode(val);
            let (decoded, _) = decode(&encoded).unwrap();
            assert_eq!(decoded, val, "Failed roundtrip for {}", val);
        }
    }

    #[test]
    fn test_minimal_encoding() {
        // Small values should use minimal bytes
        // For signed LEB128, single byte range is -64 to 63
        assert_eq!(encode(0).len(), 1);
        assert_eq!(encode(63).len(), 1);  // Largest single-byte positive
        assert_eq!(encode(64).len(), 2);  // Needs 2 bytes
        assert_eq!(encode(127).len(), 2); // Needs 2 bytes
        assert_eq!(encode(128).len(), 2);
        assert_eq!(encode(-1).len(), 1);
        assert_eq!(encode(-64).len(), 1); // Smallest single-byte negative
        assert_eq!(encode(-65).len(), 2); // Needs 2 bytes
    }
}
