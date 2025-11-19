#![allow(clippy::approx_constant)]

//! Error handling tests for LNMP binary format (v0.4)
//!
//! These tests verify that the binary decoder properly detects and reports
//! various error conditions including:
//! - Unsupported protocol versions
//! - Invalid type tags
//! - Malformed values (VarInt, UTF-8, truncated data)
//! - Trailing data detection
//! - Canonical form violations

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder, BinaryError, DecoderConfig};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// ============================================================================
// Task 9.1: Version Validation Tests
// Requirements: 4.7, 6.1
// ============================================================================

#[test]
fn test_unsupported_version_0x00() {
    let bytes = vec![0x00, 0x00, 0x00]; // Version 0x00
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnsupportedVersion { found, supported }) => {
            assert_eq!(found, 0x00);
            assert!(supported.contains(&0x04));
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_unsupported_version_0x03() {
    let bytes = vec![0x03, 0x00, 0x00]; // Version 0x03 (v0.3 doesn't have binary)
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnsupportedVersion { found, .. }) => {
            assert_eq!(found, 0x03);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_unsupported_version_0x05() {
    let bytes = vec![0x05, 0x00, 0x00]; // Version 0x05 (future version)
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnsupportedVersion { found, .. }) => {
            assert_eq!(found, 0x05);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_unsupported_version_0x99() {
    let bytes = vec![0x99, 0x00, 0x00]; // Invalid version
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnsupportedVersion { found, supported }) => {
            assert_eq!(found, 0x99);
            assert!(supported.contains(&0x04));
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_unsupported_version_0x_ff() {
    let bytes = vec![0xFF, 0x00, 0x00]; // Invalid version
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnsupportedVersion { found, .. }) => {
            assert_eq!(found, 0xFF);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_version_0x04_accepted() {
    let bytes = vec![0x04, 0x00, 0x00]; // Valid version 0x04, empty record
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_ok());
    let record = result.unwrap();
    assert_eq!(record.fields().len(), 0);
}

#[test]
fn test_version_0x04_with_data_accepted() {
    // Create a valid record with version 0x04
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Verify it starts with version 0x04
    assert_eq!(binary[0], 0x04);

    // Decode should succeed
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&binary);
    assert!(result.is_ok());
}

// ============================================================================
// Task 9.2: Type Tag Validation Tests
// Requirements: 6.2
// ============================================================================

#[test]
fn test_invalid_type_tag_0x00() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x00, // Invalid TAG = 0x00
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidTypeTag { tag }) => {
            assert_eq!(tag, 0x00);
        }
        _ => panic!("Expected InvalidTypeTag error"),
    }
}

#[test]
fn test_v0_5_type_tag_0x06_not_yet_implemented() {
    // v0.5 type tags are now valid but not yet implemented in the decoder
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x06, // TAG = 0x06 (NestedRecord - v0.5)
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            type_tag, reason, ..
        }) => {
            assert_eq!(type_tag, 0x06);
            assert!(
                reason.contains("not yet implemented"),
                "Expected 'not yet implemented' in error message"
            );
        }
        _ => panic!("Expected InvalidValue error for v0.5 type tag"),
    }
}

#[test]
fn test_v0_5_type_tag_0x07_not_yet_implemented() {
    // v0.5 type tags are now valid but not yet implemented in the decoder
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x07, // TAG = 0x07 (NestedArray - v0.5)
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            type_tag, reason, ..
        }) => {
            assert_eq!(type_tag, 0x07);
            assert!(
                reason.contains("not yet implemented"),
                "Expected 'not yet implemented' in error message"
            );
        }
        _ => panic!("Expected InvalidValue error for v0.5 type tag"),
    }
}

#[test]
fn test_invalid_type_tag_0x_ff() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0xFF, // Invalid TAG = 0xFF
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidTypeTag { tag }) => {
            assert_eq!(tag, 0xFF);
        }
        _ => panic!("Expected InvalidTypeTag error"),
    }
}

#[test]
fn test_valid_type_tag_0x01_int() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x01, // TAG = Int (0x01)
        0x2A, // VALUE = 42 (VarInt)
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_valid_type_tag_0x02_float() {
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x02, // TAG = Float (0x02)
    ];
    bytes.extend_from_slice(&3.14f64.to_le_bytes()); // VALUE = 3.14

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_valid_type_tag_0x03_bool() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x03, // TAG = Bool (0x03)
        0x01, // VALUE = true
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_valid_type_tag_0x04_string() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x04, // TAG = String (0x04)
        0x05, // LENGTH = 5
        b'h', b'e', b'l', b'l', b'o', // VALUE = "hello"
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_valid_type_tag_0x05_string_array() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x05, // TAG = StringArray (0x05)
        0x02, // COUNT = 2
        0x01, b'a', // "a"
        0x01, b'b', // "b"
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);
    assert!(result.is_ok());
}

// ============================================================================
// Task 9.3: Value Validation Tests
// Requirements: 6.3, 6.4
// ============================================================================

#[test]
fn test_malformed_varint_incomplete() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x01, // TAG = Int
        0x80, // VarInt with continuation bit but no more bytes
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    // Should be InvalidValue wrapping the VarInt error
    match result {
        Err(BinaryError::InvalidValue {
            field_id, reason, ..
        }) => {
            assert_eq!(field_id, 1);
            assert!(reason.contains("VarInt"));
        }
        _ => panic!("Expected InvalidValue error for malformed VarInt"),
    }
}

#[test]
fn test_malformed_varint_too_long() {
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x01, // TAG = Int
    ];
    // Add 11 bytes with continuation bits (exceeds max 10 bytes for i64)
    bytes.extend_from_slice(&[0x80; 11]);

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            field_id, reason, ..
        }) => {
            assert_eq!(field_id, 1);
            assert!(reason.contains("VarInt"));
        }
        _ => panic!("Expected InvalidValue error for VarInt too long"),
    }
}

#[test]
fn test_invalid_utf8_in_string() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x04, // TAG = String
        0x03, // LENGTH = 3
        0xFF, 0xFE, 0xFD, // Invalid UTF-8 bytes
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidUtf8 { field_id }) => {
            assert_eq!(field_id, 1);
        }
        _ => panic!("Expected InvalidUtf8 error"),
    }
}

#[test]
fn test_invalid_utf8_in_string_array() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x05, // TAG = StringArray
        0x02, // COUNT = 2
        0x01, b'a', // First string "a" (valid)
        0x02, 0xFF, 0xFE, // Second string with invalid UTF-8
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidUtf8 { field_id }) => {
            assert_eq!(field_id, 1);
        }
        _ => panic!("Expected InvalidUtf8 error"),
    }
}

#[test]
fn test_truncated_value_int() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x01, // TAG = Int
              // Missing VarInt value
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue { .. }) | Err(BinaryError::UnexpectedEof { .. }) => {
            // Either error is acceptable
        }
        _ => panic!("Expected InvalidValue or UnexpectedEof error"),
    }
}

#[test]
fn test_truncated_value_float() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x02, // TAG = Float
        0x00, 0x00, 0x00, // Only 3 bytes instead of 8
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnexpectedEof { expected, found }) => {
            assert!(expected > found);
        }
        _ => panic!("Expected UnexpectedEof error"),
    }
}

#[test]
fn test_truncated_value_bool() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x03, // TAG = Bool
              // Missing bool value
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnexpectedEof { expected, found }) => {
            assert!(expected > found);
        }
        _ => panic!("Expected UnexpectedEof error"),
    }
}

#[test]
fn test_truncated_value_string() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x04, // TAG = String
        0x0A, // LENGTH = 10
        b'h', b'e', b'l', b'l', b'o', // Only 5 bytes instead of 10
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::UnexpectedEof { expected, found }) => {
            assert!(expected > found);
        }
        _ => panic!("Expected UnexpectedEof error"),
    }
}

#[test]
fn test_truncated_value_string_array() {
    let bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x01, // ENTRY_COUNT = 1
        0x01, 0x00, // FID = 1
        0x05, // TAG = StringArray
        0x03, // COUNT = 3
        0x01, b'a', // First string
        0x01, b'b', // Second string
              // Missing third string
    ];

    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue { .. }) | Err(BinaryError::UnexpectedEof { .. }) => {
            // Either error is acceptable
        }
        _ => panic!("Expected InvalidValue or UnexpectedEof error"),
    }
}

// ============================================================================
// Task 9.4: Trailing Data Detection Tests
// Requirements: 4.8, 6.4
// ============================================================================

#[test]
fn test_trailing_data_detected_in_strict_mode() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let mut binary = encoder.encode(&record).unwrap();

    // Add trailing data
    binary.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    let config = DecoderConfig::new().with_strict_parsing(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&binary);

    assert!(result.is_err());
    match result {
        Err(BinaryError::TrailingData { bytes_remaining }) => {
            assert_eq!(bytes_remaining, 4);
        }
        _ => panic!("Expected TrailingData error"),
    }
}

#[test]
fn test_trailing_data_single_byte() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let encoder = BinaryEncoder::new();
    let mut binary = encoder.encode(&record).unwrap();

    // Add single trailing byte
    binary.push(0xFF);

    let config = DecoderConfig::new().with_strict_parsing(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&binary);

    assert!(result.is_err());
    match result {
        Err(BinaryError::TrailingData { bytes_remaining }) => {
            assert_eq!(bytes_remaining, 1);
        }
        _ => panic!("Expected TrailingData error"),
    }
}

#[test]
fn test_trailing_data_many_bytes() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("test".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let mut binary = encoder.encode(&record).unwrap();

    // Add many trailing bytes
    binary.extend_from_slice(&[0xFF; 100]);

    let config = DecoderConfig::new().with_strict_parsing(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&binary);

    assert!(result.is_err());
    match result {
        Err(BinaryError::TrailingData { bytes_remaining }) => {
            assert_eq!(bytes_remaining, 100);
        }
        _ => panic!("Expected TrailingData error"),
    }
}

#[test]
fn test_trailing_data_ignored_without_strict_mode() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let mut binary = encoder.encode(&record).unwrap();

    // Add trailing data
    binary.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    // Default decoder (strict_parsing = false)
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&binary);

    // Should succeed without strict parsing
    assert!(result.is_ok());
    let decoded = result.unwrap();
    assert_eq!(decoded.fields().len(), 1);
}

#[test]
fn test_no_trailing_data_in_strict_mode() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // No trailing data added
    let config = DecoderConfig::new().with_strict_parsing(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&binary);

    // Should succeed
    assert!(result.is_ok());
}

// ============================================================================
// Task 9.5: Canonical Violation Tests
// Requirements: 6.5
// ============================================================================

#[test]
fn test_unsorted_fields_detected_with_validation() {
    // Manually create binary with unsorted fields
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x02, // ENTRY_COUNT = 2
    ];

    // Entry 1: FID=10, Bool=true
    bytes.extend_from_slice(&[0x0A, 0x00]); // FID=10
    bytes.push(0x03); // TAG=Bool
    bytes.push(0x01); // VALUE=true

    // Entry 2: FID=5, Bool=false (out of order!)
    bytes.extend_from_slice(&[0x05, 0x00]); // FID=5
    bytes.push(0x03); // TAG=Bool
    bytes.push(0x00); // VALUE=false

    let config = DecoderConfig::new().with_validate_ordering(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::CanonicalViolation { reason }) => {
            assert!(
                reason.contains("not in ascending FID order")
                    || reason.contains("F5") && reason.contains("F10")
            );
        }
        _ => panic!("Expected CanonicalViolation error"),
    }
}

#[test]
fn test_unsorted_fields_ignored_without_validation() {
    // Manually create binary with unsorted fields
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x02, // ENTRY_COUNT = 2
    ];

    // Entry 1: FID=10, Bool=true
    bytes.extend_from_slice(&[0x0A, 0x00]); // FID=10
    bytes.push(0x03); // TAG=Bool
    bytes.push(0x01); // VALUE=true

    // Entry 2: FID=5, Bool=false (out of order!)
    bytes.extend_from_slice(&[0x05, 0x00]); // FID=5
    bytes.push(0x03); // TAG=Bool
    bytes.push(0x00); // VALUE=false

    // Default decoder (validate_ordering = false)
    let decoder = BinaryDecoder::new();
    let result = decoder.decode(&bytes);

    // Should succeed without validation
    assert!(result.is_ok());
}

#[test]
fn test_sorted_fields_accepted_with_validation() {
    // Create properly sorted binary
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x03, // ENTRY_COUNT = 3
    ];

    // Entry 1: FID=5
    bytes.extend_from_slice(&[0x05, 0x00]);
    bytes.push(0x03);
    bytes.push(0x01);

    // Entry 2: FID=10
    bytes.extend_from_slice(&[0x0A, 0x00]);
    bytes.push(0x03);
    bytes.push(0x00);

    // Entry 3: FID=20
    bytes.extend_from_slice(&[0x14, 0x00]);
    bytes.push(0x03);
    bytes.push(0x01);

    let config = DecoderConfig::new().with_validate_ordering(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&bytes);

    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_canonical_violation_with_encoder_output() {
    // The encoder always produces sorted output, so we need to manually
    // create unsorted binary to test the validation
    let mut bytes = vec![
        0x04, 0x00, // VERSION, FLAGS
        0x03, // ENTRY_COUNT = 3
    ];

    // FID=50
    bytes.extend_from_slice(&[0x32, 0x00]);
    bytes.push(0x01);
    bytes.push(0x01);

    // FID=10 (out of order)
    bytes.extend_from_slice(&[0x0A, 0x00]);
    bytes.push(0x01);
    bytes.push(0x02);

    // FID=30 (out of order)
    bytes.extend_from_slice(&[0x1E, 0x00]);
    bytes.push(0x01);
    bytes.push(0x03);

    let config = DecoderConfig::new().with_validate_ordering(true);
    let decoder = BinaryDecoder::with_config(config);
    let result = decoder.decode(&bytes);

    assert!(result.is_err());
    match result {
        Err(BinaryError::CanonicalViolation { .. }) => {
            // Expected
        }
        _ => panic!("Expected CanonicalViolation error"),
    }
}
