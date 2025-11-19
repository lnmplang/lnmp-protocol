#![allow(clippy::approx_constant)]

//! Value encoding unit tests for LNMP binary format (v0.4)
//!
//! These tests verify the encoding and decoding of individual value types:
//! - Integer encoding/decoding (VarInt format)
//! - Float encoding/decoding (IEEE 754)
//! - Boolean encoding/decoding (0x00/0x01)
//! - String encoding/decoding (length-prefixed UTF-8)
//! - String array encoding/decoding (count + length-prefixed strings)
//!
//! Requirements: 2.4, 2.5, 2.6, 3.6, 3.7, 3.8

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// ============================================================================
// Integer Encoding/Decoding Tests
// Requirements: 2.4, 2.5, 3.6
// ============================================================================

#[test]
fn test_integer_encoding_small_positive() {
    let values = vec![0, 1, 42, 100, 127];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(val),
            "Failed for small positive integer: {}",
            val
        );
    }
}

#[test]
fn test_integer_encoding_medium_positive() {
    let values = vec![128, 255, 256, 1000, 16383, 16384];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(val),
            "Failed for medium positive integer: {}",
            val
        );
    }
}

#[test]
fn test_integer_encoding_large_positive() {
    let values = vec![65535, 100000, 1000000, i32::MAX as i64, i64::MAX];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(val),
            "Failed for large positive integer: {}",
            val
        );
    }
}

#[test]
fn test_integer_encoding_negative() {
    let values = vec![
        -1,
        -42,
        -100,
        -127,
        -128,
        -255,
        -1000,
        -65535,
        i32::MIN as i64,
        i64::MIN,
    ];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(val),
            "Failed for negative integer: {}",
            val
        );
    }
}

#[test]
fn test_integer_encoding_zero() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(0),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(0));
}

// ============================================================================
// Float Encoding/Decoding Tests
// Requirements: 2.4, 2.5, 3.7
// ============================================================================

#[test]
fn test_float_encoding_simple() {
    let values = vec![0.0, 1.0, -1.0, 3.14, -3.14, 2.718, 1.414];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Float(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Float(val),
            "Failed for float: {}",
            val
        );
    }
}

#[test]
fn test_float_encoding_nan() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Float(f64::NAN),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    match decoded.get_field(1).unwrap().value {
        LnmpValue::Float(f) => assert!(f.is_nan(), "NaN not preserved"),
        _ => panic!("Expected Float value"),
    }
}

#[test]
fn test_float_encoding_infinity() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Float(f64::INFINITY),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::Float(f64::INFINITY)
    );
}

#[test]
fn test_float_encoding_neg_infinity() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Float(f64::NEG_INFINITY),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::Float(f64::NEG_INFINITY)
    );
}

#[test]
fn test_float_encoding_edge_cases() {
    let values = vec![f64::MIN, f64::MAX, f64::MIN_POSITIVE, f64::EPSILON];

    for val in values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Float(val),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Float(val),
            "Failed for float edge case: {}",
            val
        );
    }
}

// ============================================================================
// Boolean Encoding/Decoding Tests
// Requirements: 2.4, 2.5, 3.8
// ============================================================================

#[test]
fn test_boolean_encoding_true() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Verify the boolean is encoded as 0x01
    // Binary format: VERSION(1) + FLAGS(1) + COUNT(1) + FID(2) + TAG(1) + VALUE(1)
    // The VALUE byte should be at position 6
    assert_eq!(binary[6], 0x01, "Boolean true should be encoded as 0x01");

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Bool(true));
}

#[test]
fn test_boolean_encoding_false() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Bool(false),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Verify the boolean is encoded as 0x00
    // Binary format: VERSION(1) + FLAGS(1) + COUNT(1) + FID(2) + TAG(1) + VALUE(1)
    // The VALUE byte should be at position 6
    assert_eq!(binary[6], 0x00, "Boolean false should be encoded as 0x00");

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Bool(false));
}

// ============================================================================
// String Encoding/Decoding Tests
// Requirements: 2.4, 2.6, 3.6
// ============================================================================

#[test]
fn test_string_encoding_empty() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::String("".to_string())
    );
}

#[test]
fn test_string_encoding_ascii() {
    let test_strings = vec![
        "a",
        "hello",
        "Hello World",
        "test123",
        "UPPERCASE",
        "lowercase",
        "MixedCase123",
    ];

    for s in test_strings {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(s.to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for ASCII string: {}",
            s
        );
    }
}

#[test]
fn test_string_encoding_utf8() {
    let test_strings = vec![
        "emoji: 🎯",
        "日本語",
        "Ελληνικά",
        "Русский",
        "العربية",
        "中文",
        "한국어",
        "mixed: hello 世界 🌍",
    ];

    for s in test_strings {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(s.to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for UTF-8 string: {}",
            s
        );
    }
}

// ============================================================================
// String Array Encoding/Decoding Tests
// Requirements: 2.4, 2.6, 3.6
// ============================================================================

#[test]
fn test_string_array_encoding_empty() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::StringArray(vec![]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::StringArray(vec![])
    );
}

#[test]
fn test_string_array_encoding_single() {
    let test_arrays = vec![vec!["a"], vec!["hello"], vec!["single item with spaces"]];

    for arr in test_arrays {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
            "Failed for single-item array: {:?}",
            arr
        );
    }
}

#[test]
fn test_string_array_encoding_multiple() {
    let test_arrays = vec![
        vec!["a", "b"],
        vec!["admin", "dev"],
        vec!["one", "two", "three"],
        vec!["hello", "world", "test", "data", "array"],
    ];

    for arr in test_arrays {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
            "Failed for multi-item array: {:?}",
            arr
        );
    }
}
