//! BinaryEntry unit tests for LNMP binary format (v0.4)
//!
//! These tests verify the BinaryEntry structure encoding and decoding:
//! - Entry encoding/decoding for each value type
//! - FID boundary values (0, 65535)
//! - Invalid type tags are rejected
//! - Malformed value data returns appropriate errors
//!
//! Requirements: 2.1, 2.2, 2.3, 2.4

use lnmp_codec::binary::error::BinaryError;
use lnmp_codec::binary::entry::BinaryEntry;
use lnmp_codec::binary::types::{BinaryValue, TypeTag};
use lnmp_codec::binary::varint;

// ============================================================================
// Entry Encoding/Decoding Tests for Each Value Type
// Requirements: 2.1, 2.2, 2.3, 2.4
// ============================================================================

#[test]
fn test_entry_encode_decode_int() {
    let test_values = vec![0, 1, -1, 42, -42, 127, 128, 14532, -14532, i64::MAX, i64::MIN];
    
    for val in test_values {
        let entry = BinaryEntry {
            fid: 10,
            tag: TypeTag::Int,
            value: BinaryValue::Int(val),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry, "Failed for int value: {}", val);
        assert_eq!(consumed, bytes.len());
    }
}

#[test]
fn test_entry_encode_decode_float() {
    let test_values = vec![
        0.0, 1.0, -1.0, 3.14, -3.14, 2.718, 
        f64::MIN, f64::MAX, f64::MIN_POSITIVE, f64::EPSILON,
        f64::INFINITY, f64::NEG_INFINITY,
    ];
    
    for val in test_values {
        let entry = BinaryEntry {
            fid: 20,
            tag: TypeTag::Float,
            value: BinaryValue::Float(val),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry, "Failed for float value: {}", val);
        assert_eq!(consumed, bytes.len());
    }
}

#[test]
fn test_entry_encode_decode_float_nan() {
    let entry = BinaryEntry {
        fid: 20,
        tag: TypeTag::Float,
        value: BinaryValue::Float(f64::NAN),
    };
    
    let bytes = entry.encode();
    let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
    
    // NaN != NaN, so check separately
    match decoded.value {
        BinaryValue::Float(f) => assert!(f.is_nan(), "NaN not preserved"),
        _ => panic!("Expected Float variant"),
    }
    assert_eq!(decoded.fid, entry.fid);
    assert_eq!(decoded.tag, entry.tag);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_entry_encode_decode_bool() {
    for val in [true, false] {
        let entry = BinaryEntry {
            fid: 30,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(val),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry, "Failed for bool value: {}", val);
        assert_eq!(consumed, bytes.len());
    }
}

#[test]
fn test_entry_encode_decode_string() {
    let test_strings = vec![
        "",
        "a",
        "hello",
        "Hello World",
        "emoji: 🎯",
        "日本語",
        "mixed: hello 世界 🌍",
        "newline\ntest",
        "tab\there",
        "quote\"test",
        "backslash\\test",
    ];
    
    for s in test_strings {
        let entry = BinaryEntry {
            fid: 40,
            tag: TypeTag::String,
            value: BinaryValue::String(s.to_string()),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry, "Failed for string: {}", s);
        assert_eq!(consumed, bytes.len());
    }
}

#[test]
fn test_entry_encode_decode_string_array() {
    let test_arrays: Vec<Vec<&str>> = vec![
        vec![],
        vec!["a"],
        vec!["admin", "dev"],
        vec!["one", "two", "three"],
        vec!["", "empty", ""],
        vec!["emoji 🎯", "unicode 世界"],
    ];
    
    for arr in test_arrays {
        let entry = BinaryEntry {
            fid: 50,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry, "Failed for string array: {:?}", arr);
        assert_eq!(consumed, bytes.len());
    }
}

// ============================================================================
// FID Boundary Value Tests
// Requirements: 2.1
// ============================================================================

#[test]
fn test_fid_minimum_value() {
    let entry = BinaryEntry {
        fid: 0,
        tag: TypeTag::Int,
        value: BinaryValue::Int(42),
    };
    
    let bytes = entry.encode();
    
    // Verify FID is encoded as little-endian
    assert_eq!(bytes[0], 0x00);
    assert_eq!(bytes[1], 0x00);
    
    let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
    assert_eq!(decoded.fid, 0);
    assert_eq!(decoded, entry);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_fid_maximum_value() {
    let entry = BinaryEntry {
        fid: 65535,
        tag: TypeTag::Int,
        value: BinaryValue::Int(42),
    };
    
    let bytes = entry.encode();
    
    // Verify FID is encoded as little-endian (65535 = 0xFFFF)
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xFF);
    
    let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
    assert_eq!(decoded.fid, 65535);
    assert_eq!(decoded, entry);
    assert_eq!(consumed, bytes.len());
}

#[test]
fn test_fid_various_values() {
    let test_fids = vec![0, 1, 7, 12, 23, 100, 255, 256, 1000, 32767, 32768, 65534, 65535];
    
    for fid in test_fids {
        let entry = BinaryEntry {
            fid,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        };
        
        let bytes = entry.encode();
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded.fid, fid, "Failed for FID: {}", fid);
    }
}

// ============================================================================
// Invalid Type Tag Tests
// Requirements: 2.2, 6.2
// ============================================================================

#[test]
fn test_invalid_type_tag_rejected() {
    // 0x06 and 0x07 are now valid in v0.5 (NestedRecord and NestedArray)
    // 0x08-0x0F are reserved but valid type tags
    // Only truly invalid tags should be tested here
    let invalid_tags = vec![0x00, 0x10, 0x20, 0x50, 0xFF];
    
    for tag in invalid_tags {
        let bytes = vec![
            0x01, 0x00,  // FID = 1
            tag,         // Invalid TAG
            0x00,        // Some data
        ];
        
        let result = BinaryEntry::decode(&bytes);
        assert!(
            matches!(result, Err(BinaryError::InvalidTypeTag { tag: t }) if t == tag),
            "Expected InvalidTypeTag error for tag 0x{:02X}, got: {:?}",
            tag,
            result
        );
    }
}

#[test]
fn test_v0_5_type_tags_not_yet_implemented() {
    // v0.5 type tags (0x06, 0x07) should be recognized but return an error
    // indicating they're not yet implemented in BinaryEntry
    let v0_5_tags = vec![0x06, 0x07];
    
    for tag in v0_5_tags {
        let bytes = vec![
            0x01, 0x00,  // FID = 1
            tag,         // v0.5 TAG
            0x00,        // Some data
        ];
        
        let result = BinaryEntry::decode(&bytes);
        assert!(
            matches!(result, Err(BinaryError::InvalidValue { type_tag: t, .. }) if t == tag),
            "Expected InvalidValue error for v0.5 tag 0x{:02X}, got: {:?}",
            tag,
            result
        );
    }
}

#[test]
fn test_reserved_type_tags_rejected() {
    // Reserved type tags (0x08-0x0F) should be recognized but return an error
    let reserved_tags = vec![0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];
    
    for tag in reserved_tags {
        let bytes = vec![
            0x01, 0x00,  // FID = 1
            tag,         // Reserved TAG
            0x00,        // Some data
        ];
        
        let result = BinaryEntry::decode(&bytes);
        match result {
            Err(BinaryError::InvalidValue { type_tag: t, reason, .. }) => {
                assert_eq!(t, tag, "Type tag mismatch");
                assert!(reason.contains("Reserved"), "Expected 'Reserved' in error message, got: {}", reason);
            }
            other => panic!("Expected InvalidValue error for reserved tag 0x{:02X}, got: {:?}", tag, other),
        }
    }
}

#[test]
fn test_valid_type_tags_accepted() {
    let valid_entries = vec![
        (0x01, BinaryValue::Int(42)),
        (0x02, BinaryValue::Float(3.14)),
        (0x03, BinaryValue::Bool(true)),
        (0x04, BinaryValue::String("test".to_string())),
        (0x05, BinaryValue::StringArray(vec!["a".to_string()])),
    ];
    
    for (tag_byte, value) in valid_entries {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::from_u8(tag_byte).unwrap(),
            value,
        };
        
        let bytes = entry.encode();
        assert_eq!(bytes[2], tag_byte, "Tag byte mismatch");
        
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        assert_eq!(decoded.tag.to_u8(), tag_byte);
    }
}

// ============================================================================
// Malformed Value Data Tests
// Requirements: 2.3, 2.4, 6.3, 6.4
// ============================================================================

#[test]
fn test_malformed_int_truncated_varint() {
    let bytes = vec![
        0x01, 0x00,  // FID = 1
        0x01,        // TAG = Int
        0x80,        // VarInt continuation byte without following byte
    ];
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::InvalidValue { .. })),
        "Expected InvalidValue error for truncated VarInt, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_float_insufficient_bytes() {
    let bytes = vec![
        0x01, 0x00,  // FID = 1
        0x02,        // TAG = Float
        0x00, 0x00, 0x00, 0x00,  // Only 4 bytes instead of 8
    ];
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error for truncated float, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_bool_invalid_value() {
    let invalid_bool_values = vec![0x02, 0x03, 0x10, 0xFF];
    
    for bool_val in invalid_bool_values {
        let bytes = vec![
            0x01, 0x00,  // FID = 1
            0x03,        // TAG = Bool
            bool_val,    // Invalid boolean value
        ];
        
        let result = BinaryEntry::decode(&bytes);
        assert!(
            matches!(result, Err(BinaryError::InvalidValue { .. })),
            "Expected InvalidValue error for bool value 0x{:02X}, got: {:?}",
            bool_val,
            result
        );
    }
}

#[test]
fn test_malformed_string_invalid_utf8() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x04,        // TAG = String
    ];
    bytes.extend_from_slice(&varint::encode(3));  // Length = 3
    bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::InvalidUtf8 { field_id: 1 })),
        "Expected InvalidUtf8 error, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_string_truncated_data() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x04,        // TAG = String
    ];
    bytes.extend_from_slice(&varint::encode(10));  // Length = 10
    bytes.extend_from_slice(b"short");  // Only 5 bytes
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_string_negative_length() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x04,        // TAG = String
    ];
    bytes.extend_from_slice(&varint::encode(-5));  // Negative length
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::InvalidValue { .. })),
        "Expected InvalidValue error for negative string length, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_string_array_negative_count() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x05,        // TAG = StringArray
    ];
    bytes.extend_from_slice(&varint::encode(-3));  // Negative count
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::InvalidValue { .. })),
        "Expected InvalidValue error for negative array count, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_string_array_truncated_string() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x05,        // TAG = StringArray
    ];
    bytes.extend_from_slice(&varint::encode(2));  // Count = 2
    bytes.extend_from_slice(&varint::encode(5));  // First string length = 5
    bytes.extend_from_slice(b"hello");  // First string
    bytes.extend_from_slice(&varint::encode(10));  // Second string length = 10
    bytes.extend_from_slice(b"short");  // Only 5 bytes instead of 10
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error, got: {:?}",
        result
    );
}

#[test]
fn test_malformed_string_array_invalid_utf8_in_element() {
    let mut bytes = vec![
        0x01, 0x00,  // FID = 1
        0x05,        // TAG = StringArray
    ];
    bytes.extend_from_slice(&varint::encode(2));  // Count = 2
    bytes.extend_from_slice(&varint::encode(5));  // First string length = 5
    bytes.extend_from_slice(b"hello");  // First string (valid)
    bytes.extend_from_slice(&varint::encode(3));  // Second string length = 3
    bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]);  // Invalid UTF-8
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::InvalidUtf8 { field_id: 1 })),
        "Expected InvalidUtf8 error, got: {:?}",
        result
    );
}

// ============================================================================
// Insufficient Data Tests
// Requirements: 6.4
// ============================================================================

#[test]
fn test_insufficient_data_no_fid() {
    let bytes = vec![0x01];  // Only 1 byte, need 2 for FID
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { expected: 2, found: 1 })),
        "Expected UnexpectedEof error, got: {:?}",
        result
    );
}

#[test]
fn test_insufficient_data_no_tag() {
    let bytes = vec![0x01, 0x00];  // FID but no TAG
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error, got: {:?}",
        result
    );
}

#[test]
fn test_insufficient_data_no_value() {
    let bytes = vec![0x01, 0x00, 0x03];  // FID + TAG (Bool) but no value
    
    let result = BinaryEntry::decode(&bytes);
    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error, got: {:?}",
        result
    );
}

// ============================================================================
// Trailing Data Handling Tests
// ============================================================================

#[test]
fn test_decode_with_trailing_data() {
    let entry = BinaryEntry {
        fid: 1,
        tag: TypeTag::Int,
        value: BinaryValue::Int(42),
    };
    
    let mut bytes = entry.encode();
    bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);  // Extra bytes
    
    let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
    
    assert_eq!(decoded, entry);
    assert_eq!(consumed, bytes.len() - 4);  // Should not consume trailing bytes
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_string_encoding() {
    let entry = BinaryEntry {
        fid: 1,
        tag: TypeTag::String,
        value: BinaryValue::String("".to_string()),
    };
    
    let bytes = entry.encode();
    let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
    
    assert_eq!(decoded, entry);
}

#[test]
fn test_empty_string_array_encoding() {
    let entry = BinaryEntry {
        fid: 1,
        tag: TypeTag::StringArray,
        value: BinaryValue::StringArray(vec![]),
    };
    
    let bytes = entry.encode();
    let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
    
    assert_eq!(decoded, entry);
}

#[test]
fn test_large_string_encoding() {
    let large_string = "a".repeat(10000);
    let entry = BinaryEntry {
        fid: 1,
        tag: TypeTag::String,
        value: BinaryValue::String(large_string.clone()),
    };
    
    let bytes = entry.encode();
    let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
    
    assert_eq!(decoded, entry);
}

#[test]
fn test_large_string_array_encoding() {
    let large_array: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let entry = BinaryEntry {
        fid: 1,
        tag: TypeTag::StringArray,
        value: BinaryValue::StringArray(large_array.clone()),
    };
    
    let bytes = entry.encode();
    let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
    
    assert_eq!(decoded, entry);
}
