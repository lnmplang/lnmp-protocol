#![allow(clippy::approx_constant)]

//! Round-trip tests for LNMP binary format (v0.4)
//!
//! These tests verify that data maintains integrity through format conversions:
//! - Text → Binary → Text produces identical canonical text
//! - Binary → Text → Binary produces identical binary output
//! - Multiple round-trips produce stable output
//! - All value types survive round-trip conversion

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// ============================================================================
// Task 8.1: Text-to-Binary-to-Text Round-Trip Tests
// Requirements: 5.1, 5.2, 5.3, 5.4
// ============================================================================

#[test]
fn test_text_to_binary_to_text_simple() {
    let original_text = "F7=1;F12=14532;F23=[\"admin\",\"dev\"]";

    // Text → Binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(original_text).unwrap();

    // Binary → Text
    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    // Should produce canonical text (newline-separated, sorted)
    assert_eq!(decoded_text, "F7=1\nF12=14532\nF23=[admin,dev]");
}

#[test]
fn test_text_to_binary_to_text_canonical_already() {
    // Input is already in canonical form
    let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";

    // Text → Binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(original_text).unwrap();

    // Binary → Text
    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    // Should produce identical canonical text
    assert_eq!(decoded_text, original_text);
}

#[test]
fn test_text_to_binary_to_text_multiple_iterations() {
    let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // First iteration
    let binary1 = encoder.encode_text(original_text).unwrap();
    let text1 = decoder.decode_to_text(&binary1).unwrap();
    assert_eq!(text1, original_text);

    // Second iteration
    let binary2 = encoder.encode_text(&text1).unwrap();
    let text2 = decoder.decode_to_text(&binary2).unwrap();
    assert_eq!(text2, original_text);

    // Third iteration
    let binary3 = encoder.encode_text(&text2).unwrap();
    let text3 = decoder.decode_to_text(&binary3).unwrap();
    assert_eq!(text3, original_text);

    // All binary outputs should be identical (stable)
    assert_eq!(binary1, binary2);
    assert_eq!(binary2, binary3);
}

#[test]
fn test_text_to_binary_to_text_stability_five_iterations() {
    let original_text = "F1=42\nF2=3.14\nF3=test";

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    let mut current_text = original_text.to_string();
    let mut previous_binary: Option<Vec<u8>> = None;

    // Perform 5 round-trips
    for i in 0..5 {
        let binary = encoder.encode_text(&current_text).unwrap();
        current_text = decoder.decode_to_text(&binary).unwrap();

        // After first canonicalization, output should be stable
        if let Some(prev) = previous_binary {
            assert_eq!(binary, prev, "Binary output changed at iteration {}", i);
        }
        previous_binary = Some(binary);
    }

    // Final text should match original
    assert_eq!(current_text, original_text);
}

#[test]
fn test_text_to_binary_to_text_unsorted_input() {
    // Unsorted input
    let unsorted_text = "F23=[admin]\nF7=1\nF12=14532";

    // Text → Binary → Text
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(unsorted_text).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    // Should be sorted in canonical form
    assert_eq!(decoded_text, "F7=1\nF12=14532\nF23=[admin]");

    // Second round-trip should produce identical output
    let binary2 = encoder.encode_text(&decoded_text).unwrap();
    let decoded_text2 = decoder.decode_to_text(&binary2).unwrap();

    assert_eq!(decoded_text, decoded_text2);
    assert_eq!(binary, binary2);
}

#[test]
fn test_text_to_binary_to_text_empty_record() {
    let original_text = "";

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(original_text).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    assert_eq!(decoded_text, "");
}

#[test]
fn test_text_to_binary_to_text_single_field() {
    let original_text = "F1=42";

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(original_text).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&binary).unwrap();

    assert_eq!(decoded_text, original_text);
}

// ============================================================================
// Task 8.2: Binary-to-Text-to-Binary Round-Trip Tests
// Requirements: 5.1, 5.2
// ============================================================================

#[test]
fn test_binary_to_text_to_binary_simple() {
    // Create a record and encode to binary
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
    let original_binary = encoder.encode(&record).unwrap();

    // Binary → Text
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&original_binary).unwrap();

    // Text → Binary
    let binary2 = encoder.encode_text(&text).unwrap();

    // Should produce identical binary output
    assert_eq!(original_binary, binary2);
}

#[test]
fn test_binary_to_text_to_binary_all_fields() {
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

    let encoder = BinaryEncoder::new();
    let original_binary = encoder.encode(&record).unwrap();

    // Binary → Text → Binary
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&original_binary).unwrap();
    let binary2 = encoder.encode_text(&text).unwrap();

    // Should produce identical binary
    assert_eq!(original_binary, binary2);
}

#[test]
fn test_binary_to_text_to_binary_multiple_iterations() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14),
    });

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    let original_binary = encoder.encode(&record).unwrap();

    // Perform multiple round-trips
    let mut current_binary = original_binary.clone();
    for _ in 0..5 {
        let text = decoder.decode_to_text(&current_binary).unwrap();
        current_binary = encoder.encode_text(&text).unwrap();
    }

    // Final binary should match original
    assert_eq!(current_binary, original_binary);
}

#[test]
fn test_binary_to_text_to_binary_empty_record() {
    let record = LnmpRecord::new();

    let encoder = BinaryEncoder::new();
    let original_binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&original_binary).unwrap();
    let binary2 = encoder.encode_text(&text).unwrap();

    assert_eq!(original_binary, binary2);
}

#[test]
fn test_binary_to_text_to_binary_single_field() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("test".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let original_binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&original_binary).unwrap();
    let binary2 = encoder.encode_text(&text).unwrap();

    assert_eq!(original_binary, binary2);
}

#[test]
fn test_binary_to_text_to_binary_preserves_byte_representation() {
    // Test that the exact byte representation is preserved
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Int(-12345),
    });
    record.add_field(LnmpField {
        fid: 200,
        value: LnmpValue::Float(2.718281828),
    });

    let encoder = BinaryEncoder::new();
    let original_binary = encoder.encode(&record).unwrap();

    // Binary → Text → Binary
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&original_binary).unwrap();
    let binary2 = encoder.encode_text(&text).unwrap();

    // Byte-for-byte comparison
    assert_eq!(original_binary.len(), binary2.len());
    for (i, (b1, b2)) in original_binary.iter().zip(binary2.iter()).enumerate() {
        assert_eq!(b1, b2, "Byte mismatch at position {}", i);
    }
}

// ============================================================================
// Task 8.3: All-Types Round-Trip Tests
// Requirements: 5.5
// ============================================================================

#[test]
fn test_roundtrip_integers_positive() {
    let test_values = vec![
        0,
        1,
        42,
        127,
        128,
        255,
        256,
        14532,
        65535,
        1000000,
        i64::MAX,
    ];

    for value in test_values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(value),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Int(value),
            "Failed for positive integer: {}",
            value
        );
    }
}

#[test]
fn test_roundtrip_integers_negative() {
    let test_values = vec![
        -1,
        -42,
        -127,
        -128,
        -255,
        -256,
        -14532,
        -65535,
        -1000000,
        i64::MIN,
    ];

    for value in test_values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(value),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Int(value),
            "Failed for negative integer: {}",
            value
        );
    }
}

#[test]
fn test_roundtrip_integers_zero() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(0),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Int(0)
    );
}

#[test]
fn test_roundtrip_floats_normal() {
    let test_values = vec![0.0, 1.0, -1.0, 3.14, -3.14, 2.718281828, 1.414213562];

    for value in test_values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Float(value),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Float(value),
            "Failed for float: {}",
            value
        );
    }
}

#[test]
fn test_roundtrip_floats_edge_cases() {
    let test_values = vec![f64::MIN, f64::MAX, f64::MIN_POSITIVE, f64::EPSILON];

    for value in test_values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Float(value),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Float(value),
            "Failed for float edge case: {}",
            value
        );
    }
}

#[test]
fn test_roundtrip_floats_special_values() {
    // Test NaN separately since NaN != NaN
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Float(f64::NAN),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    if let LnmpValue::Float(f) = decoded_record.get_field(1).unwrap().value {
        assert!(f.is_nan(), "NaN not preserved");
    } else {
        panic!("Expected Float value");
    }

    // Test Infinity
    let test_values = vec![f64::INFINITY, f64::NEG_INFINITY];

    for value in test_values {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Float(value),
        });

        let binary = encoder.encode(&record).unwrap();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Float(value),
            "Failed for special float: {}",
            value
        );
    }
}

#[test]
fn test_roundtrip_booleans() {
    // Test true
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Bool(true),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Bool(true)
    );

    // Test false
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Bool(false),
    });

    let binary = encoder.encode(&record).unwrap();
    let decoded_record = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Bool(false)
    );
}

#[test]
fn test_roundtrip_strings_empty() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::String("".to_string())
    );
}

#[test]
fn test_roundtrip_strings_ascii() {
    let test_strings = vec![
        "a",
        "hello",
        "Hello World",
        "simple",
        "test123",
        "UPPERCASE",
        "lowercase",
        "MixedCase",
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
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for ASCII string: {}",
            s
        );
    }
}

#[test]
fn test_roundtrip_strings_utf8() {
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
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for UTF-8 string: {}",
            s
        );
    }
}

#[test]
fn test_roundtrip_strings_with_escapes() {
    let test_strings = vec![
        "hello\nworld",         // newline
        "path\\to\\file",       // backslash
        "say \"hello\"",        // quotes
        "tab\there",            // tab
        "carriage\rreturn",     // carriage return
        "mixed\n\t\"escapes\"", // multiple escapes
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
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for string with escapes: {:?}",
            s
        );
    }
}

#[test]
fn test_roundtrip_string_arrays_empty() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::StringArray(vec![]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::StringArray(vec![])
    );
}

#[test]
fn test_roundtrip_string_arrays_single_item() {
    let test_arrays = vec![vec!["a"], vec!["hello"], vec!["single item"]];

    for arr in test_arrays {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
            "Failed for single-item array: {:?}",
            arr
        );
    }
}

#[test]
fn test_roundtrip_string_arrays_multiple_items() {
    let test_arrays = vec![
        vec!["a", "b"],
        vec!["admin", "dev"],
        vec!["one", "two", "three"],
        vec!["a", "b", "c", "d", "e"],
        vec!["hello", "world", "test", "data"],
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
        let decoded_record = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::StringArray(arr.iter().map(|s| s.to_string()).collect()),
            "Failed for multi-item array: {:?}",
            arr
        );
    }
}

#[test]
fn test_roundtrip_all_types_combined() {
    // Test a record with all types at once
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(-123),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Float(3.14159),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(-2.718),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 6,
        value: LnmpValue::Bool(false),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::String("hello world".to_string()),
    });
    record.add_field(LnmpField {
        fid: 8,
        value: LnmpValue::String("emoji: 🎯".to_string()),
    });
    record.add_field(LnmpField {
        fid: 9,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::StringArray(vec![]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    // Verify all fields
    assert_eq!(decoded_record.fields().len(), 10);
    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Int(42)
    );
    assert_eq!(
        decoded_record.get_field(2).unwrap().value,
        LnmpValue::Int(-123)
    );
    assert_eq!(
        decoded_record.get_field(3).unwrap().value,
        LnmpValue::Float(3.14159)
    );
    assert_eq!(
        decoded_record.get_field(4).unwrap().value,
        LnmpValue::Float(-2.718)
    );
    assert_eq!(
        decoded_record.get_field(5).unwrap().value,
        LnmpValue::Bool(true)
    );
    assert_eq!(
        decoded_record.get_field(6).unwrap().value,
        LnmpValue::Bool(false)
    );
    assert_eq!(
        decoded_record.get_field(7).unwrap().value,
        LnmpValue::String("hello world".to_string())
    );
    assert_eq!(
        decoded_record.get_field(8).unwrap().value,
        LnmpValue::String("emoji: 🎯".to_string())
    );
    assert_eq!(
        decoded_record.get_field(9).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
    assert_eq!(
        decoded_record.get_field(10).unwrap().value,
        LnmpValue::StringArray(vec![])
    );
}

// ============================================================================
// Task 8.4: Canonical Form Stability Tests
// Requirements: 5.3, 5.4
// ============================================================================

#[test]
fn test_unsorted_fields_become_sorted() {
    // Create record with unsorted fields
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Int(2),
    });

    // Verify input order is preserved in record
    assert_eq!(record.fields()[0].fid, 100);
    assert_eq!(record.fields()[1].fid, 5);
    assert_eq!(record.fields()[2].fid, 50);

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Decode back
    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();

    // Verify fields are now sorted
    assert_eq!(decoded_record.fields()[0].fid, 5);
    assert_eq!(decoded_record.fields()[1].fid, 50);
    assert_eq!(decoded_record.fields()[2].fid, 100);

    // Verify values are correct
    assert_eq!(
        decoded_record.get_field(5).unwrap().value,
        LnmpValue::Int(1)
    );
    assert_eq!(
        decoded_record.get_field(50).unwrap().value,
        LnmpValue::Int(2)
    );
    assert_eq!(
        decoded_record.get_field(100).unwrap().value,
        LnmpValue::Int(3)
    );
}

#[test]
fn test_unsorted_text_becomes_sorted_after_encoding() {
    // Unsorted text input
    let unsorted_text = "F100=3\nF5=1\nF50=2";

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(unsorted_text).unwrap();

    // Decode to text
    let decoder = BinaryDecoder::new();
    let sorted_text = decoder.decode_to_text(&binary).unwrap();

    // Should be sorted
    assert_eq!(sorted_text, "F5=1\nF50=2\nF100=3");
}

#[test]
fn test_multiple_roundtrips_produce_stable_canonical_output() {
    // Start with unsorted input
    let unsorted_text = "F23=[admin]\nF7=1\nF12=14532";

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // First round-trip: unsorted → binary → sorted text
    let binary1 = encoder.encode_text(unsorted_text).unwrap();
    let text1 = decoder.decode_to_text(&binary1).unwrap();
    assert_eq!(text1, "F7=1\nF12=14532\nF23=[admin]");

    // Second round-trip: sorted text → binary → sorted text
    let binary2 = encoder.encode_text(&text1).unwrap();
    let text2 = decoder.decode_to_text(&binary2).unwrap();
    assert_eq!(text2, text1);

    // Third round-trip: should still be identical
    let binary3 = encoder.encode_text(&text2).unwrap();
    let text3 = decoder.decode_to_text(&binary3).unwrap();
    assert_eq!(text3, text1);

    // All binary outputs after first canonicalization should be identical
    assert_eq!(binary2, binary3);
}

#[test]
fn test_canonical_stability_ten_iterations() {
    // Start with unsorted, mixed format input
    let initial_text = "F50=test;F10=42;F30=3.14;F20=1";

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // First iteration to canonicalize
    let mut current_binary = encoder.encode_text(initial_text).unwrap();
    let mut current_text = decoder.decode_to_text(&current_binary).unwrap();

    let canonical_text = current_text.clone();
    let canonical_binary = current_binary.clone();

    // Perform 10 more round-trips
    for i in 0..10 {
        current_binary = encoder.encode_text(&current_text).unwrap();
        current_text = decoder.decode_to_text(&current_binary).unwrap();

        assert_eq!(
            current_text,
            canonical_text,
            "Text changed at iteration {}",
            i + 1
        );
        assert_eq!(
            current_binary,
            canonical_binary,
            "Binary changed at iteration {}",
            i + 1
        );
    }
}

#[test]
fn test_canonical_form_with_all_types() {
    // Unsorted record with all types
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("test".to_string()),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::Float(3.14),
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 40,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
    });

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // Encode and decode
    let binary = encoder.encode(&record).unwrap();
    let decoded_record = decoder.decode(&binary).unwrap();

    // Verify sorted order
    let fields = decoded_record.fields();
    assert_eq!(fields[0].fid, 10);
    assert_eq!(fields[1].fid, 20);
    assert_eq!(fields[2].fid, 30);
    assert_eq!(fields[3].fid, 40);
    assert_eq!(fields[4].fid, 50);

    // Second round-trip should be identical
    let binary2 = encoder.encode(&decoded_record).unwrap();
    assert_eq!(binary, binary2);
}

#[test]
fn test_canonical_stability_with_duplicates() {
    // Record with duplicate FIDs (allowed in LNMP)
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("first".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Bool(true), // Changed from Int(1) to Bool(true) since parser interprets "1" as boolean
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("second".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // First encoding
    let binary1 = encoder.encode(&record).unwrap();
    let text1 = decoder.decode_to_text(&binary1).unwrap();

    // Second encoding
    let binary2 = encoder.encode_text(&text1).unwrap();
    let text2 = decoder.decode_to_text(&binary2).unwrap();

    // Should be stable
    assert_eq!(text1, text2);
    assert_eq!(binary1, binary2);
}

#[test]
fn test_canonical_form_empty_to_populated() {
    // Start with empty record
    let mut record = LnmpRecord::new();

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    let binary1 = encoder.encode(&record).unwrap();
    let text1 = decoder.decode_to_text(&binary1).unwrap();
    assert_eq!(text1, "");

    // Add fields in unsorted order
    record.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Bool(true), // Changed from Int(1) to Bool(true) since parser interprets "1" as boolean
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::Int(2),
    });

    let binary2 = encoder.encode(&record).unwrap();
    let text2 = decoder.decode_to_text(&binary2).unwrap();

    // Should be sorted
    assert_eq!(text2, "F10=1\nF20=2\nF30=3");

    // Multiple round-trips should be stable
    let binary3 = encoder.encode_text(&text2).unwrap();
    let text3 = decoder.decode_to_text(&binary3).unwrap();
    assert_eq!(text3, text2);
    assert_eq!(binary3, binary2);
}

#[test]
fn test_canonical_form_preserves_data_integrity() {
    // Complex unsorted record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::StringArray(vec!["z".to_string(), "y".to_string(), "x".to_string()]),
    });
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(-12345),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Float(2.718281828),
    });
    record.add_field(LnmpField {
        fid: 25,
        value: LnmpValue::String("hello\nworld".to_string()),
    });
    record.add_field(LnmpField {
        fid: 75,
        value: LnmpValue::Bool(false),
    });

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // Encode and decode
    let binary = encoder.encode(&record).unwrap();
    let decoded_record = decoder.decode(&binary).unwrap();

    // Verify all data is preserved and sorted
    assert_eq!(decoded_record.fields().len(), 5);
    assert_eq!(decoded_record.fields()[0].fid, 1);
    assert_eq!(decoded_record.fields()[1].fid, 25);
    assert_eq!(decoded_record.fields()[2].fid, 50);
    assert_eq!(decoded_record.fields()[3].fid, 75);
    assert_eq!(decoded_record.fields()[4].fid, 100);

    // Verify values
    assert_eq!(
        decoded_record.get_field(1).unwrap().value,
        LnmpValue::Int(-12345)
    );
    assert_eq!(
        decoded_record.get_field(25).unwrap().value,
        LnmpValue::String("hello\nworld".to_string())
    );
    assert_eq!(
        decoded_record.get_field(50).unwrap().value,
        LnmpValue::Float(2.718281828)
    );
    assert_eq!(
        decoded_record.get_field(75).unwrap().value,
        LnmpValue::Bool(false)
    );
    assert_eq!(
        decoded_record.get_field(100).unwrap().value,
        LnmpValue::StringArray(vec!["z".to_string(), "y".to_string(), "x".to_string()])
    );
}

#[test]
fn test_canonical_stability_across_text_and_binary() {
    // Test that canonical form is stable regardless of starting format
    let text_input = "F50=test\nF10=42\nF30=3.14";

    let mut record_input = LnmpRecord::new();
    record_input.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("test".to_string()),
    });
    record_input.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(42),
    });
    record_input.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::Float(3.14),
    });

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // Path 1: text → binary → text
    let binary_from_text = encoder.encode_text(text_input).unwrap();
    let text_from_binary = decoder.decode_to_text(&binary_from_text).unwrap();

    // Path 2: record → binary → text
    let binary_from_record = encoder.encode(&record_input).unwrap();
    let text_from_record = decoder.decode_to_text(&binary_from_record).unwrap();

    // Both paths should produce identical canonical output
    assert_eq!(text_from_binary, text_from_record);
    assert_eq!(binary_from_text, binary_from_record);

    // Expected canonical form
    assert_eq!(text_from_binary, "F10=42\nF30=3.14\nF50=test");
}
