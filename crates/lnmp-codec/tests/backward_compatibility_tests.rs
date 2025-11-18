//! Backward compatibility tests for LNMP v0.5
//!
//! These tests verify that:
//! - v0.5 decoder can parse v0.4 binary format
//! - v0.5 decoder can parse v0.3 text format
//! - v0.4 decoder rejects v0.5 nested types with clear error
//! - v0.5 encoder produces v0.4-compatible output when nested features disabled
//! - Semantic equivalence is maintained across version boundaries

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder, BinaryError, DecoderConfig, EncoderConfig};
use lnmp_codec::{Encoder, Parser};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

/// Test that v0.5 decoder can parse v0.4 binary format (Requirement 13.1)
#[test]
fn test_v05_decoder_parses_v04_binary() {
    // Create a v0.4 binary record (no nested structures)
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    // Encode with v0.4 encoder (default config has nested disabled)
    let v04_encoder = BinaryEncoder::new();
    let v04_binary = v04_encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder (should work seamlessly)
    let v05_decoder = BinaryDecoder::new();
    let decoded = v05_decoder.decode(&v04_binary).unwrap();

    // Verify fields match
    assert_eq!(decoded.fields().len(), 3);
    assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(decoded.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(
        decoded.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

/// Test that v0.5 decoder can parse v0.3 text format (Requirement 13.2)
#[test]
fn test_v05_decoder_parses_v03_text() {
    // v0.3 text format (semicolon-separated, unsorted)
    let v03_text = r#"F23=["admin","dev"];F7=1;F12=14532"#;

    // Parse with v0.3 parser
    let mut parser = Parser::new(v03_text).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode to binary with v0.5 encoder
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    // Verify fields match (should be sorted)
    assert_eq!(decoded.fields().len(), 3);
    assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(decoded.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(
        decoded.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

/// Test that v0.5 encoder produces v0.4-compatible output when nested features disabled (Requirement 13.4)
#[test]
fn test_v05_encoder_v04_compatible_output() {
    // Create a simple record (no nested structures)
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    // Encode with v0.5 encoder in v0.4 compatibility mode
    let v05_config = EncoderConfig::new().with_v0_4_compatibility();
    let v05_encoder = BinaryEncoder::with_config(v05_config);
    let v05_binary = v05_encoder.encode(&record).unwrap();

    // Encode with v0.4 encoder (default)
    let v04_encoder = BinaryEncoder::new();
    let v04_binary = v04_encoder.encode(&record).unwrap();

    // Binary outputs should be identical
    assert_eq!(v05_binary, v04_binary);

    // Both should decode correctly
    let decoder = BinaryDecoder::new();
    let decoded_v05 = decoder.decode(&v05_binary).unwrap();
    let decoded_v04 = decoder.decode(&v04_binary).unwrap();

    assert_eq!(decoded_v05.fields().len(), 2);
    assert_eq!(decoded_v04.fields().len(), 2);
    assert_eq!(
        decoded_v05.get_field(7).unwrap().value,
        decoded_v04.get_field(7).unwrap().value
    );
    assert_eq!(
        decoded_v05.get_field(12).unwrap().value,
        decoded_v04.get_field(12).unwrap().value
    );
}

/// Test semantic equivalence across version boundaries (Requirement 13.5)
#[test]
fn test_semantic_equivalence_across_versions() {
    // Original v0.3 text
    let v03_text = "F7=1\nF12=14532\nF23=[admin,dev]";

    // Parse v0.3 text
    let mut parser = Parser::new(v03_text).unwrap();
    let v03_record = parser.parse_record().unwrap();

    // Encode to v0.4 binary
    let encoder = BinaryEncoder::new();
    let v04_binary = encoder.encode(&v03_record).unwrap();

    // Decode v0.4 binary
    let decoder = BinaryDecoder::new();
    let v04_record = decoder.decode(&v04_binary).unwrap();

    // Encode back to v0.3 text
    let text_encoder = Encoder::new();
    let output_text = text_encoder.encode(&v04_record);

    // Should be semantically equivalent (canonical form)
    assert_eq!(output_text, v03_text);

    // Field values should match exactly
    assert_eq!(v03_record.fields().len(), v04_record.fields().len());
    for field in v03_record.fields() {
        let decoded_field = v04_record.get_field(field.fid).unwrap();
        assert_eq!(field.value, decoded_field.value);
    }
}

/// Test that v0.4 decoder rejects v0.5 nested record types with clear error (Requirement 13.3)
#[test]
fn test_v04_decoder_rejects_v05_nested_record() {
    // Create a binary frame with nested record type tag (0x06)
    // This simulates what a v0.5 encoder would produce
    let mut bytes = vec![
        0x04, // VERSION
        0x00, // FLAGS
        0x01, // ENTRY_COUNT = 1
        0x0A, 0x00, // FID = 10 (little-endian)
        0x06, // TAG = NestedRecord (0x06) - v0.5 type
    ];
    // Add minimal nested record data (would be more complex in real v0.5)
    bytes.push(0x00); // Empty nested record

    // v0.4 decoder should reject this
    let v04_decoder = BinaryDecoder::new();
    let result = v04_decoder.decode(&bytes);

    // Should get an error about nested structures not being supported
    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            type_tag, reason, ..
        }) => {
            assert_eq!(type_tag, 0x06);
            assert!(reason.contains("not yet implemented") || reason.contains("not supported"));
        }
        _ => panic!("Expected InvalidValue error for nested record type"),
    }
}

/// Test that v0.4 decoder rejects v0.5 nested array types with clear error (Requirement 13.3)
#[test]
fn test_v04_decoder_rejects_v05_nested_array() {
    // Create a binary frame with nested array type tag (0x07)
    let mut bytes = vec![
        0x04, // VERSION
        0x00, // FLAGS
        0x01, // ENTRY_COUNT = 1
        0x0A, 0x00, // FID = 10 (little-endian)
        0x07, // TAG = NestedArray (0x07) - v0.5 type
    ];
    // Add minimal nested array data
    bytes.push(0x00); // Empty nested array

    // v0.4 decoder should reject this
    let v04_decoder = BinaryDecoder::new();
    let result = v04_decoder.decode(&bytes);

    // Should get an error about nested structures not being supported
    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            type_tag, reason, ..
        }) => {
            assert_eq!(type_tag, 0x07);
            assert!(reason.contains("not yet implemented") || reason.contains("not supported"));
        }
        _ => panic!("Expected InvalidValue error for nested array type"),
    }
}

/// Test that v0.5 encoder rejects nested structures in v0.4 compatibility mode
#[test]
fn test_v05_encoder_rejects_nested_in_v04_mode() {
    // Create a record with nested structure
    let mut inner_record = LnmpRecord::new();
    inner_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut outer_record = LnmpRecord::new();
    outer_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::NestedRecord(Box::new(inner_record)),
    });

    // Try to encode with v0.4 compatibility mode (nested disabled)
    let v04_config = EncoderConfig::new().with_v0_4_compatibility();
    let encoder = BinaryEncoder::with_config(v04_config);
    let result = encoder.encode(&outer_record);

    // Should fail with clear error message
    assert!(result.is_err());
    match result {
        Err(BinaryError::InvalidValue {
            field_id,
            type_tag,
            reason,
        }) => {
            assert_eq!(field_id, 10);
            assert_eq!(type_tag, 0x06);
            assert!(reason.contains("not supported in v0.4"));
        }
        _ => panic!("Expected InvalidValue error for nested record"),
    }
}

/// Test version detection functionality
#[test]
fn test_version_detection() {
    // Create v0.4 binary
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Detect version
    let decoder = BinaryDecoder::new();
    let version = decoder.detect_version(&binary).unwrap();

    assert_eq!(version, 0x04);
}

/// Test supports_nested detection for v0.4 binary
#[test]
fn test_supports_nested_v04_binary() {
    // Create v0.4 binary (no nested structures)
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

    // Check if it supports nested structures
    let decoder = BinaryDecoder::new();
    let has_nested = decoder.supports_nested(&binary);

    // v0.4 binary should not have nested structures
    assert!(!has_nested);
}

/// Test roundtrip: v0.3 text -> v0.4 binary -> v0.3 text
#[test]
fn test_roundtrip_v03_v04_v03() {
    let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";

    // v0.3 text -> v0.4 binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(original_text).unwrap();

    // v0.4 binary -> v0.3 text
    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    // Should match original (canonical form)
    assert_eq!(decoded_text, original_text);
}

/// Test multiple roundtrips maintain stability
#[test]
fn test_multiple_roundtrips_stable() {
    let original_text = "F7=1\nF12=14532";

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    let mut current_text = original_text.to_string();

    // Perform 5 roundtrips
    for _ in 0..5 {
        let binary = encoder.encode_text(&current_text).unwrap();
        current_text = decoder.decode_to_text(&binary).unwrap();
    }

    // Should still match original
    assert_eq!(current_text, original_text);
}

/// Test that all v0.4 types work correctly in v0.5
#[test]
fn test_all_v04_types_in_v05() {
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
        value: LnmpValue::Bool(false),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("hello world".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
    });

    // Encode with v0.5 encoder in v0.4 mode
    let v05_config = EncoderConfig::new().with_v0_4_compatibility();
    let encoder = BinaryEncoder::with_config(v05_config);
    let binary = encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    // All fields should match
    assert_eq!(decoded.fields().len(), 5);
    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(-42));
    assert_eq!(
        decoded.get_field(2).unwrap().value,
        LnmpValue::Float(3.14159)
    );
    assert_eq!(decoded.get_field(3).unwrap().value, LnmpValue::Bool(false));
    assert_eq!(
        decoded.get_field(4).unwrap().value,
        LnmpValue::String("hello world".to_string())
    );
    assert_eq!(
        decoded.get_field(5).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

/// Test that v0.5 decoder config defaults are backward compatible
#[test]
fn test_v05_decoder_config_defaults_backward_compatible() {
    let config = DecoderConfig::default();

    // v0.5 features should be disabled by default for backward compatibility
    assert!(!config.allow_streaming);
    assert!(!config.validate_nesting);
    assert!(!config.allow_delta);

    // v0.4 features should work as before
    assert!(!config.validate_ordering);
    assert!(!config.strict_parsing);
}

/// Test that v0.5 encoder config defaults are backward compatible
#[test]
fn test_v05_encoder_config_defaults_backward_compatible() {
    let config = EncoderConfig::default();

    // v0.5 features should be disabled by default for backward compatibility
    assert!(!config.enable_nested_binary);
    assert!(!config.streaming_mode);
    assert!(!config.delta_mode);

    // v0.4 features should work as before
    assert!(config.sort_fields);
    assert!(!config.validate_canonical);
}

/// Test empty record backward compatibility
#[test]
fn test_empty_record_backward_compatibility() {
    let record = LnmpRecord::new();

    // Encode with v0.5 encoder in v0.4 mode
    let v05_config = EncoderConfig::new().with_v0_4_compatibility();
    let v05_encoder = BinaryEncoder::with_config(v05_config);
    let v05_binary = v05_encoder.encode(&record).unwrap();

    // Encode with v0.4 encoder
    let v04_encoder = BinaryEncoder::new();
    let v04_binary = v04_encoder.encode(&record).unwrap();

    // Should be identical
    assert_eq!(v05_binary, v04_binary);

    // Both should decode to empty record
    let decoder = BinaryDecoder::new();
    let decoded_v05 = decoder.decode(&v05_binary).unwrap();
    let decoded_v04 = decoder.decode(&v04_binary).unwrap();

    assert_eq!(decoded_v05.fields().len(), 0);
    assert_eq!(decoded_v04.fields().len(), 0);
}

/// Test field ordering is preserved across versions
#[test]
fn test_field_ordering_preserved_across_versions() {
    // Create record with unsorted fields
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(2),
    });

    // Encode with v0.5 encoder in v0.4 mode
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    // Fields should be sorted by FID
    let fields = decoded.fields();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].fid, 7);
    assert_eq!(fields[1].fid, 12);
    assert_eq!(fields[2].fid, 23);
}

/// Test special characters in strings across versions
#[test]
fn test_special_characters_across_versions() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("hello\nworld".to_string()),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("path\\to\\file".to_string()),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::String("say \"hello\"".to_string()),
    });

    // Encode with v0.5 encoder in v0.4 mode
    let v05_config = EncoderConfig::new().with_v0_4_compatibility();
    let encoder = BinaryEncoder::with_config(v05_config);
    let binary = encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    // Special characters should be preserved
    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::String("hello\nworld".to_string())
    );
    assert_eq!(
        decoded.get_field(2).unwrap().value,
        LnmpValue::String("path\\to\\file".to_string())
    );
    assert_eq!(
        decoded.get_field(3).unwrap().value,
        LnmpValue::String("say \"hello\"".to_string())
    );
}

/// Test Unicode support across versions
#[test]
fn test_unicode_across_versions() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Hello 🎯 World 你好".to_string()),
    });

    // Encode with v0.5 encoder in v0.4 mode
    let v05_config = EncoderConfig::new().with_v0_4_compatibility();
    let encoder = BinaryEncoder::with_config(v05_config);
    let binary = encoder.encode(&record).unwrap();

    // Decode with v0.5 decoder
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    // Unicode should be preserved
    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::String("Hello 🎯 World 你好".to_string())
    );
}
