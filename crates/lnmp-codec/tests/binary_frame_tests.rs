#![allow(clippy::approx_constant)]

//! BinaryFrame unit tests for LNMP binary format (v0.4)
//!
//! These tests verify the BinaryFrame structure encoding and decoding:
//! - Empty frame (zero entries) encoding/decoding
//! - Single entry frame encoding/decoding
//! - Multiple entry frame encoding/decoding
//! - Version validation rejects non-0x04 versions
//! - Flags byte is preserved
//!
//! Requirements: 3.1, 3.2, 3.3, 3.4, 4.1, 4.7

use lnmp_codec::binary::entry::BinaryEntry;
use lnmp_codec::binary::error::BinaryError;
use lnmp_codec::binary::frame::BinaryFrame;
use lnmp_codec::binary::types::{BinaryValue, TypeTag};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// ============================================================================
// Empty Frame Tests
// Requirements: 3.1, 3.2, 3.3, 3.4
// ============================================================================

#[test]
fn test_empty_frame_encoding() {
    let frame = BinaryFrame::new(vec![]);
    let bytes = frame.encode();

    // Verify structure: VERSION (1) + FLAGS (1) + ENTRY_COUNT (1 for 0)
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0x04, "VERSION should be 0x04");
    assert_eq!(bytes[1], 0x00, "FLAGS should be 0x00");
    assert_eq!(bytes[2], 0x00, "ENTRY_COUNT should be 0");
}

#[test]
fn test_empty_frame_decoding() {
    let frame = BinaryFrame::new(vec![]);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_empty_frame_roundtrip() {
    let frame = BinaryFrame::new(vec![]);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let bytes2 = decoded.encode();

    assert_eq!(bytes, bytes2, "Empty frame should roundtrip identically");
}

// ============================================================================
// Single Entry Frame Tests
// Requirements: 3.1, 3.2, 3.3, 3.4, 3.5
// ============================================================================

#[test]
fn test_single_entry_frame_int() {
    let entries = vec![BinaryEntry {
        fid: 7,
        tag: TypeTag::Int,
        value: BinaryValue::Int(42),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();

    assert_eq!(bytes[0], 0x04, "VERSION should be 0x04");
    assert_eq!(bytes[1], 0x00, "FLAGS should be 0x00");
    assert_eq!(bytes[2], 0x01, "ENTRY_COUNT should be 1");
    assert!(bytes.len() > 3, "Should have entry data");
}

#[test]
fn test_single_entry_frame_bool() {
    let entries = vec![BinaryEntry {
        fid: 7,
        tag: TypeTag::Bool,
        value: BinaryValue::Bool(true),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_single_entry_frame_string() {
    let entries = vec![BinaryEntry {
        fid: 1,
        tag: TypeTag::String,
        value: BinaryValue::String("hello".to_string()),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_single_entry_frame_float() {
    let entries = vec![BinaryEntry {
        fid: 2,
        tag: TypeTag::Float,
        value: BinaryValue::Float(3.14159),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_single_entry_frame_string_array() {
    let entries = vec![BinaryEntry {
        fid: 5,
        tag: TypeTag::StringArray,
        value: BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

// ============================================================================
// Multiple Entry Frame Tests
// Requirements: 3.1, 3.2, 3.3, 3.4, 3.5
// ============================================================================

#[test]
fn test_multiple_entries_frame_encoding() {
    let entries = vec![
        BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        },
        BinaryEntry {
            fid: 12,
            tag: TypeTag::Int,
            value: BinaryValue::Int(14532),
        },
        BinaryEntry {
            fid: 23,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        },
    ];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();

    assert_eq!(bytes[0], 0x04, "VERSION should be 0x04");
    assert_eq!(bytes[1], 0x00, "FLAGS should be 0x00");
    assert_eq!(bytes[2], 0x03, "ENTRY_COUNT should be 3");
}

#[test]
fn test_multiple_entries_frame_decoding() {
    let entries = vec![
        BinaryEntry {
            fid: 1,
            tag: TypeTag::Int,
            value: BinaryValue::Int(-42),
        },
        BinaryEntry {
            fid: 2,
            tag: TypeTag::Float,
            value: BinaryValue::Float(3.14),
        },
        BinaryEntry {
            fid: 3,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(false),
        },
    ];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_multiple_entries_all_types() {
    let entries = vec![
        BinaryEntry {
            fid: 1,
            tag: TypeTag::Int,
            value: BinaryValue::Int(100),
        },
        BinaryEntry {
            fid: 2,
            tag: TypeTag::Float,
            value: BinaryValue::Float(2.718),
        },
        BinaryEntry {
            fid: 3,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        },
        BinaryEntry {
            fid: 4,
            tag: TypeTag::String,
            value: BinaryValue::String("test".to_string()),
        },
        BinaryEntry {
            fid: 5,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        },
    ];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_multiple_entries_roundtrip() {
    let entries = vec![
        BinaryEntry {
            fid: 10,
            tag: TypeTag::String,
            value: BinaryValue::String("hello".to_string()),
        },
        BinaryEntry {
            fid: 20,
            tag: TypeTag::Int,
            value: BinaryValue::Int(999),
        },
    ];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let bytes2 = decoded.encode();

    assert_eq!(
        bytes, bytes2,
        "Multiple entry frame should roundtrip identically"
    );
}

#[test]
fn test_large_number_of_entries() {
    let mut entries = Vec::new();
    for i in 0..100 {
        entries.push(BinaryEntry {
            fid: i,
            tag: TypeTag::Int,
            value: BinaryValue::Int(i as i64),
        });
    }

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

// ============================================================================
// Version Validation Tests
// Requirements: 4.1, 4.7
// ============================================================================

#[test]
fn test_version_0x04_accepted() {
    let bytes = vec![0x04, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(result.is_ok(), "Version 0x04 should be accepted");
}

#[test]
fn test_version_0x00_rejected() {
    let bytes = vec![0x00, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x00, .. })
        ),
        "Version 0x00 should be rejected"
    );
}

#[test]
fn test_version_0x01_rejected() {
    let bytes = vec![0x01, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x01, .. })
        ),
        "Version 0x01 should be rejected"
    );
}

#[test]
fn test_version_0x02_rejected() {
    let bytes = vec![0x02, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x02, .. })
        ),
        "Version 0x02 should be rejected"
    );
}

#[test]
fn test_version_0x03_rejected() {
    let bytes = vec![0x03, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x03, .. })
        ),
        "Version 0x03 should be rejected"
    );
}

#[test]
fn test_version_0x05_rejected() {
    let bytes = vec![0x05, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x05, .. })
        ),
        "Version 0x05 should be rejected"
    );
}

#[test]
fn test_version_0x_ff_rejected() {
    let bytes = vec![0xFF, 0x00, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0xFF, .. })
        ),
        "Version 0xFF should be rejected"
    );
}

#[test]
fn test_all_non_0x04_versions_rejected() {
    for version in 0x00..=0xFF {
        if version == 0x04 {
            continue; // Skip valid version
        }

        let bytes = vec![version, 0x00, 0x00];
        let result = BinaryFrame::decode(&bytes);

        assert!(
            matches!(result, Err(BinaryError::UnsupportedVersion { .. })),
            "Version 0x{:02X} should be rejected",
            version
        );
    }
}

// ============================================================================
// Flags Byte Preservation Tests
// Requirements: 3.2
// ============================================================================

#[test]
fn test_flags_byte_preserved_empty_frame() {
    let frame = BinaryFrame::new(vec![]);
    let bytes = frame.encode();

    assert_eq!(bytes[1], 0x00, "FLAGS byte should be 0x00");

    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let bytes2 = decoded.encode();

    assert_eq!(bytes2[1], 0x00, "FLAGS byte should be preserved as 0x00");
}

#[test]
fn test_flags_byte_preserved_with_entries() {
    let entries = vec![BinaryEntry {
        fid: 1,
        tag: TypeTag::Int,
        value: BinaryValue::Int(42),
    }];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();

    assert_eq!(bytes[1], 0x00, "FLAGS byte should be 0x00");

    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let bytes2 = decoded.encode();

    assert_eq!(bytes2[1], 0x00, "FLAGS byte should be preserved as 0x00");
}

#[test]
fn test_flags_byte_in_encoded_output() {
    let entries = vec![
        BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        },
        BinaryEntry {
            fid: 12,
            tag: TypeTag::Int,
            value: BinaryValue::Int(14532),
        },
    ];

    let frame = BinaryFrame::new(entries);
    let bytes = frame.encode();

    // Verify FLAGS is at position 1 and is 0x00
    assert_eq!(bytes[1], 0x00, "FLAGS byte at position 1 should be 0x00");
}

// ============================================================================
// Integration with LnmpRecord Tests
// Requirements: 3.1, 3.2, 3.3, 3.4
// ============================================================================

#[test]
fn test_from_empty_record() {
    let record = LnmpRecord::new();
    let frame = BinaryFrame::from_record(&record).unwrap();

    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let decoded_record = decoded.to_record();

    assert_eq!(decoded_record.fields().len(), 0);
}

#[test]
fn test_from_record_single_field() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let frame = BinaryFrame::from_record(&record).unwrap();
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();

    assert_eq!(decoded, frame);
}

#[test]
fn test_from_record_multiple_fields() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let frame = BinaryFrame::from_record(&record).unwrap();
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let decoded_record = decoded.to_record();

    assert_eq!(decoded_record.fields().len(), 3);
}

#[test]
fn test_record_roundtrip_preserves_data() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(-42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14159),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("hello".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
    });

    let frame = BinaryFrame::from_record(&record).unwrap();
    let bytes = frame.encode();
    let decoded = BinaryFrame::decode(&bytes).unwrap();
    let decoded_record = decoded.to_record();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Int(-42)
    );
    assert_eq!(
        decoded_record.get_field(2).unwrap().value,
        LnmpValue::Float(3.14159)
    );
    assert_eq!(
        decoded_record.get_field(3).unwrap().value,
        LnmpValue::Bool(true)
    );
    assert_eq!(
        decoded_record.get_field(4).unwrap().value,
        LnmpValue::String("hello".to_string())
    );
    assert_eq!(
        decoded_record.get_field(5).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
    );
}

// ============================================================================
// Error Handling Tests
// Requirements: 4.7
// ============================================================================

#[test]
fn test_decode_insufficient_data_for_version() {
    let bytes = vec![];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(
            result,
            Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: 0
            })
        ),
        "Should error on insufficient data for version"
    );
}

#[test]
fn test_decode_insufficient_data_for_flags() {
    let bytes = vec![0x04];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Should error on insufficient data for flags"
    );
}

#[test]
fn test_decode_insufficient_data_for_entry_count() {
    let bytes = vec![0x04, 0x00];
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(result, Err(BinaryError::InvalidVarInt { .. })),
        "Should error on insufficient data for entry count"
    );
}

#[test]
fn test_decode_insufficient_data_for_entries() {
    let bytes = vec![0x04, 0x00, 0x01]; // Says 1 entry but no entry data
    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(result, Err(BinaryError::UnexpectedEof { .. })),
        "Should error on insufficient data for entries"
    );
}

#[test]
fn test_decode_negative_entry_count() {
    let mut bytes = vec![0x04, 0x00];
    // Encode -1 as VarInt
    bytes.extend_from_slice(&[0x7F]); // -1 in LEB128

    let result = BinaryFrame::decode(&bytes);

    assert!(
        matches!(result, Err(BinaryError::InvalidValue { .. })),
        "Should error on negative entry count"
    );
}

#[test]
fn test_decode_unsorted_fids_rejected() {
    // Frame claims 2 entries: FID 2 then FID 1 (out of order)
    let entry1 = BinaryEntry::new(2, BinaryValue::Int(1));
    let entry2 = BinaryEntry::new(1, BinaryValue::Int(2));

    let mut bytes = vec![0x04, 0x00, 0x02]; // version, flags, entry count=2
    bytes.extend_from_slice(&entry1.encode());
    bytes.extend_from_slice(&entry2.encode());

    let result = BinaryFrame::decode(&bytes);
    match result {
        Err(BinaryError::CanonicalViolation { reason }) => {
            assert!(
                reason.contains("sorted"),
                "Expected sorted-by-FID message, got: {}",
                reason
            );
        }
        other => panic!("Expected CanonicalViolation, got {:?}", other),
    }
}
