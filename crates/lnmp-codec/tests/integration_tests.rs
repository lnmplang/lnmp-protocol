#![allow(clippy::approx_constant)]

use lnmp_codec::{Encoder, EncoderConfig, Parser};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

#[test]
fn test_round_trip_multiline() {
    let input = "F12=14532\nF7=1\nF20=Halil";

    // Parse
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Parse again
    let mut parser2 = Parser::new(&output).unwrap();
    let record2 = parser2.parse_record().unwrap();

    // Verify
    assert_eq!(record.fields().len(), record2.fields().len());
    assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(
        record.get_field(20).unwrap().value,
        LnmpValue::String("Halil".to_string())
    );
}

#[test]
fn test_round_trip_inline() {
    let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;

    // Parse
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode
    let encoder = Encoder::with_config(EncoderConfig {
        canonical: false,
        ..EncoderConfig::default()
    });
    let output = encoder.encode(&record);

    // Parse again
    let mut parser2 = Parser::new(&output).unwrap();
    let record2 = parser2.parse_record().unwrap();

    // Verify
    assert_eq!(record.fields().len(), 3);
    assert_eq!(record2.fields().len(), 3);
    assert_eq!(
        record.get_field(12).unwrap().value,
        record2.get_field(12).unwrap().value
    );
    assert_eq!(
        record.get_field(7).unwrap().value,
        record2.get_field(7).unwrap().value
    );
    assert_eq!(
        record.get_field(23).unwrap().value,
        record2.get_field(23).unwrap().value
    );
}

#[test]
fn test_round_trip_all_types() {
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
        value: LnmpValue::Float(3.14),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(-2.5),
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
        value: LnmpValue::String("simple".to_string()),
    });
    record.add_field(LnmpField {
        fid: 8,
        value: LnmpValue::String("hello world".to_string()),
    });
    record.add_field(LnmpField {
        fid: 9,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
    });

    // Encode
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Parse
    let mut parser = Parser::new(&output).unwrap();
    let record2 = parser.parse_record().unwrap();

    // Verify all fields
    assert_eq!(record.fields().len(), record2.fields().len());
    for i in 1..=9 {
        assert_eq!(
            record.get_field(i).unwrap().value,
            record2.get_field(i).unwrap().value
        );
    }
}

#[test]
fn test_round_trip_with_escapes() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("hello \"world\"".to_string()),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("line1\nline2".to_string()),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::String("back\\slash".to_string()),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("tab\there".to_string()),
    });

    // Encode
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Parse
    let mut parser = Parser::new(&output).unwrap();
    let record2 = parser.parse_record().unwrap();

    // Verify
    assert_eq!(record.fields().len(), record2.fields().len());
    for i in 1..=4 {
        assert_eq!(
            record.get_field(i).unwrap().value,
            record2.get_field(i).unwrap().value
        );
    }
}

#[test]
fn test_multiline_to_inline_conversion() {
    let input = "F1=42\nF2=3.14\nF3=test";

    // Parse multiline
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode as inline
    let encoder = Encoder::with_config(EncoderConfig {
        canonical: false,
        ..EncoderConfig::default()
    });
    let output = encoder.encode(&record);

    assert_eq!(output, "F1=42;F2=3.14;F3=test");
}

#[test]
fn test_inline_to_multiline_conversion() {
    let input = "F1=42;F2=3.14;F3=test";

    // Parse inline
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode as multiline
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    assert_eq!(output, "F1=42\nF2=3.14\nF3=test");
}

// Round-trip deterministic guarantee tests

#[test]
fn test_parse_encode_parse_equals_parse() {
    // Test: parse(encode(parse(input))) equals parse(input)
    let input = r#"F23=["admin","dev"]
F7=1
F12=14532"#;

    // First parse
    let mut parser1 = Parser::new(input).unwrap();
    let record1 = parser1.parse_record().unwrap();

    // Encode
    let encoder = Encoder::new();
    let encoded = encoder.encode(&record1);

    // Second parse (of encoded)
    let mut parser2 = Parser::new(&encoded).unwrap();
    let record2 = parser2.parse_record().unwrap();

    // Encode again
    let encoded2 = encoder.encode(&record2);

    // Third parse
    let mut parser3 = Parser::new(&encoded2).unwrap();
    let record3 = parser3.parse_record().unwrap();

    // Verify: record2 and record3 should be identical
    assert_eq!(record2.fields().len(), record3.fields().len());
    for i in 0..record2.fields().len() {
        assert_eq!(record2.fields()[i].fid, record3.fields()[i].fid);
        assert_eq!(record2.fields()[i].value, record3.fields()[i].value);
    }

    // Verify: encoded and encoded2 should be identical (deterministic)
    assert_eq!(encoded, encoded2);
}

#[test]
fn test_unsorted_input_becomes_sorted_after_encode() {
    // Unsorted input
    let input = "F100=3\nF5=1\nF50=2";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Verify input order is preserved in record
    assert_eq!(record.fields()[0].fid, 100);
    assert_eq!(record.fields()[1].fid, 5);
    assert_eq!(record.fields()[2].fid, 50);

    // Encode (should sort)
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Verify output is sorted
    assert_eq!(output, "F5=1\nF50=2\nF100=3");

    // Parse sorted output
    let mut parser2 = Parser::new(&output).unwrap();
    let record2 = parser2.parse_record().unwrap();

    // Verify sorted order
    assert_eq!(record2.fields()[0].fid, 5);
    assert_eq!(record2.fields()[1].fid, 50);
    assert_eq!(record2.fields()[2].fid, 100);
}

#[test]
fn test_loose_format_becomes_canonical_after_encode() {
    // Loose format: semicolons, whitespace, unsorted
    let input = "F3  =  test  ;  F1  =  42  ;  F2  =  3.14";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode to canonical format
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Canonical format: newlines, no extra whitespace, sorted
    assert_eq!(output, "F1=42\nF2=3.14\nF3=test");
}

#[test]
fn test_multiple_encode_cycles_produce_identical_output() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Int(2),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(1),
    });

    let encoder = Encoder::new();

    // Encode multiple times
    let output1 = encoder.encode(&record);
    let output2 = encoder.encode(&record);
    let output3 = encoder.encode(&record);

    // All outputs should be identical
    assert_eq!(output1, output2);
    assert_eq!(output2, output3);

    // Verify sorted
    assert_eq!(output1, "F7=2\nF12=1\nF23=3");
}

#[test]
fn test_round_trip_with_comments_removed() {
    // Input with comments
    let input = "# This is a comment\nF1=42\n# Another comment\nF2=100";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode (comments should be removed)
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Verify no comments in output
    assert!(!output.contains('#'));
    assert_eq!(output, "F1=42\nF2=100");
}

#[test]
fn test_deterministic_with_duplicates() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("first".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("second".to_string()),
    });

    let encoder = Encoder::new();

    // Encode multiple times
    let output1 = encoder.encode(&record);
    let output2 = encoder.encode(&record);

    // Should be identical and deterministic
    assert_eq!(output1, output2);

    // Verify stable sort (duplicates maintain insertion order)
    assert_eq!(output1, "F5=1\nF10=first\nF10=second");
}

// Strict mode compatibility tests

#[test]
fn test_canonical_output_passes_strict_mode() {
    use lnmp_codec::ParsingMode;

    // Create a record
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

    // Encode to canonical format
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Parse with strict mode - should succeed
    let mut parser = Parser::with_mode(&output, ParsingMode::Strict).unwrap();
    let result = parser.parse_record();

    assert!(result.is_ok());
    let record2 = result.unwrap();
    assert_eq!(record2.fields().len(), 3);
}

#[test]
fn test_encoder_output_always_strict_compliant() {
    use lnmp_codec::ParsingMode;

    // Create various records with different orderings
    let test_cases = vec![
        vec![(100, 3), (5, 1), (50, 2)],
        vec![(1, 1), (2, 2), (3, 3)],
        vec![(50, 5), (10, 1), (30, 3), (20, 2)],
    ];

    let encoder = Encoder::new();

    for case in test_cases {
        let mut record = LnmpRecord::new();
        for (fid, val) in case {
            record.add_field(LnmpField {
                fid,
                value: LnmpValue::Int(val),
            });
        }

        // Encode
        let output = encoder.encode(&record);

        // Parse with strict mode - should always succeed
        let result = Parser::with_mode(&output, ParsingMode::Strict);
        assert!(result.is_ok());

        let mut parser = result.unwrap();
        let result = parser.parse_record();
        assert!(result.is_ok());
    }
}

#[test]
fn test_loose_input_encode_strict_parse_succeeds() {
    use lnmp_codec::ParsingMode;

    // Loose input: unsorted, semicolons, whitespace
    let input = "F23=[admin,dev];F7=1;F12=14532";

    // Parse with loose mode
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Encode to canonical
    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Parse with strict mode - should succeed
    let mut strict_parser = Parser::with_mode(&output, ParsingMode::Strict).unwrap();
    let result = strict_parser.parse_record();

    assert!(result.is_ok());
    let record2 = result.unwrap();

    // Verify data integrity
    assert_eq!(record2.fields().len(), 3);
    assert_eq!(record2.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(record2.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(
        record2.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

#[test]
fn test_encoder_never_produces_semicolons() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(2),
    });

    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Canonical format uses newlines, not semicolons
    assert!(!output.contains(';'));
    assert!(output.contains('\n'));
}

#[test]
fn test_encoder_never_produces_comments() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("test".to_string()),
    });

    let encoder = Encoder::new();
    let output = encoder.encode(&record);

    // Encoder never outputs comments
    assert!(!output.contains('#'));
}

#[test]
fn test_round_trip_preserves_data_integrity() {
    use lnmp_codec::ParsingMode;

    // Complex input with all types
    let input = r#"F1=42
F2=-123
F3=3.14
F4=-2.5
F5=1
F6=0
F7=simple
F8="hello world"
F9=[a,b,c]"#;

    // Parse with loose mode
    let mut parser = Parser::new(input).unwrap();
    let record1 = parser.parse_record().unwrap();

    // Encode
    let encoder = Encoder::new();
    let output = encoder.encode(&record1);

    // Parse with strict mode
    let mut strict_parser = Parser::with_mode(&output, ParsingMode::Strict).unwrap();
    let record2 = strict_parser.parse_record().unwrap();

    // Verify all values are preserved
    assert_eq!(record1.fields().len(), record2.fields().len());
    for i in 1..=9 {
        assert_eq!(
            record1.get_field(i).unwrap().value,
            record2.get_field(i).unwrap().value
        );
    }
}
