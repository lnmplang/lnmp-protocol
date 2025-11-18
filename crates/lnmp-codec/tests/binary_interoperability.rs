//! Interoperability tests for LNMP binary format (v0.4)
//!
//! These tests verify that the binary format works correctly with the v0.3 text parser
//! and maintains data integrity through complete agent-to-model workflows.
//!
//! Task 10: Implement interoperability tests
//! Requirements: 7.1, 7.2, 7.3, 7.4, 7.5

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_codec::Parser;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// ============================================================================
// Task 10.1: Binary-to-Text-to-Parser Tests
// Requirements: 7.1, 7.2
// ============================================================================

#[test]
fn test_binary_decoded_text_can_be_parsed() {
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
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Binary → Text
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    // Text → Parser
    let mut parser = Parser::new(&text).unwrap();
    let parsed_record = parser.parse_record().unwrap();

    // Verify parsed record matches original data
    assert_eq!(parsed_record.fields().len(), 3);
    assert_eq!(
        parsed_record.get_field(7).unwrap().value,
        LnmpValue::Bool(true)
    );
    assert_eq!(
        parsed_record.get_field(12).unwrap().value,
        LnmpValue::Int(14532)
    );
    assert_eq!(
        parsed_record.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

#[test]
fn test_binary_to_text_to_parser_all_types() {
    // Create record with all supported types
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
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Bool(false),
    });
    record.add_field(LnmpField {
        fid: 6,
        value: LnmpValue::String("hello world".to_string()),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    // Parse the text
    let mut parser = Parser::new(&text).unwrap();
    let parsed_record = parser.parse_record().unwrap();

    // Verify all fields match
    assert_eq!(parsed_record.fields().len(), 7);
    assert_eq!(
        parsed_record.get_field(1).unwrap().value,
        LnmpValue::Int(42)
    );
    assert_eq!(
        parsed_record.get_field(2).unwrap().value,
        LnmpValue::Int(-123)
    );
    assert_eq!(
        parsed_record.get_field(3).unwrap().value,
        LnmpValue::Float(3.14159)
    );
    assert_eq!(
        parsed_record.get_field(4).unwrap().value,
        LnmpValue::Bool(true)
    );
    assert_eq!(
        parsed_record.get_field(5).unwrap().value,
        LnmpValue::Bool(false)
    );
    assert_eq!(
        parsed_record.get_field(6).unwrap().value,
        LnmpValue::String("hello world".to_string())
    );
    assert_eq!(
        parsed_record.get_field(7).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[test]
fn test_binary_to_text_produces_parseable_canonical_format() {
    // Create unsorted record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("last".to_string()),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("first".to_string()),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("middle".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    // Parse the canonical text
    let mut parser = Parser::new(&text).unwrap();
    let parsed_record = parser.parse_record().unwrap();

    // Verify fields are in sorted order
    assert_eq!(parsed_record.fields()[0].fid, 10);
    assert_eq!(parsed_record.fields()[1].fid, 50);
    assert_eq!(parsed_record.fields()[2].fid, 100);

    // Verify values are correct
    assert_eq!(
        parsed_record.get_field(10).unwrap().value,
        LnmpValue::String("first".to_string())
    );
    assert_eq!(
        parsed_record.get_field(50).unwrap().value,
        LnmpValue::String("middle".to_string())
    );
    assert_eq!(
        parsed_record.get_field(100).unwrap().value,
        LnmpValue::String("last".to_string())
    );
}

#[test]
fn test_binary_to_text_empty_record_parseable() {
    let record = LnmpRecord::new();

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    // Parse empty text
    let mut parser = Parser::new(&text).unwrap();
    let parsed_record = parser.parse_record().unwrap();

    assert_eq!(parsed_record.fields().len(), 0);
}

// ============================================================================
// Task 10.2: Agent-to-Model Workflow Tests
// Requirements: 7.2, 7.3
// ============================================================================

#[test]
fn test_agent_to_model_workflow_complete_pipeline() {
    // Agent creates data (LnmpRecord)
    let mut agent_record = LnmpRecord::new();
    agent_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("user_query".to_string()),
    });
    agent_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(42),
    });
    agent_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::StringArray(vec!["context1".to_string(), "context2".to_string()]),
    });

    // Agent encodes to binary for transport
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&agent_record).unwrap();

    // Binary is transmitted...

    // Model side: decode to text
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    // Model parses the text
    let mut parser = Parser::new(&text).unwrap();
    let model_record = parser.parse_record().unwrap();

    // Verify data integrity through entire pipeline
    assert_eq!(model_record.fields().len(), 3);
    assert_eq!(
        model_record.get_field(1).unwrap().value,
        LnmpValue::String("user_query".to_string())
    );
    assert_eq!(model_record.get_field(2).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        model_record.get_field(3).unwrap().value,
        LnmpValue::StringArray(vec!["context1".to_string(), "context2".to_string()])
    );
}

#[test]
fn test_model_to_agent_workflow_reverse_pipeline() {
    // Model generates text response
    let model_text = "F1=response_text\nF2=100\nF3=[tag1,tag2,tag3]";

    // Parse the model's text
    let mut parser = Parser::new(model_text).unwrap();
    let model_record = parser.parse_record().unwrap();

    // Encode to binary for transport back to agent
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&model_record).unwrap();

    // Binary is transmitted...

    // Agent decodes binary
    let decoder = BinaryDecoder::new();
    let agent_record = decoder.decode(&binary).unwrap();

    // Verify data integrity
    assert_eq!(agent_record.fields().len(), 3);
    assert_eq!(
        agent_record.get_field(1).unwrap().value,
        LnmpValue::String("response_text".to_string())
    );
    assert_eq!(
        agent_record.get_field(2).unwrap().value,
        LnmpValue::Int(100)
    );
    assert_eq!(
        agent_record.get_field(3).unwrap().value,
        LnmpValue::StringArray(vec![
            "tag1".to_string(),
            "tag2".to_string(),
            "tag3".to_string()
        ])
    );
}

#[test]
fn test_bidirectional_agent_model_communication() {
    // Agent → Binary → Text → Model
    let mut agent_request = LnmpRecord::new();
    agent_request.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("What is 2+2?".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let request_binary = encoder.encode(&agent_request).unwrap();

    let decoder = BinaryDecoder::new();
    let request_text = decoder.decode_to_text(&request_binary).unwrap();

    let mut parser = Parser::new(&request_text).unwrap();
    let model_received = parser.parse_record().unwrap();

    assert_eq!(
        model_received.get_field(1).unwrap().value,
        LnmpValue::String("What is 2+2?".to_string())
    );

    // Model → Text → Binary → Agent
    let response_text = "F1=\"The answer is 4\"";
    let mut response_parser = Parser::new(response_text).unwrap();
    let model_response = response_parser.parse_record().unwrap();

    let response_binary = encoder.encode(&model_response).unwrap();
    let agent_received = decoder.decode(&response_binary).unwrap();

    assert_eq!(
        agent_received.get_field(1).unwrap().value,
        LnmpValue::String("The answer is 4".to_string())
    );
}

#[test]
fn test_workflow_with_complex_data_structures() {
    // Agent creates complex record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("system_prompt".to_string()),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("user_message".to_string()),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Int(1000),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(0.7),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 6,
        value: LnmpValue::StringArray(vec![
            "tool1".to_string(),
            "tool2".to_string(),
            "tool3".to_string(),
        ]),
    });

    // Complete workflow
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    // Verify all fields
    assert_eq!(parsed.fields().len(), 6);
    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("system_prompt".to_string())
    );
    assert_eq!(
        parsed.get_field(2).unwrap().value,
        LnmpValue::String("user_message".to_string())
    );
    assert_eq!(parsed.get_field(3).unwrap().value, LnmpValue::Int(1000));
    assert_eq!(parsed.get_field(4).unwrap().value, LnmpValue::Float(0.7));
    assert_eq!(parsed.get_field(5).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(
        parsed.get_field(6).unwrap().value,
        LnmpValue::StringArray(vec![
            "tool1".to_string(),
            "tool2".to_string(),
            "tool3".to_string()
        ])
    );
}

#[test]
fn test_workflow_preserves_field_ordering() {
    // Create record with specific order
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::Int(2),
    });

    // Through workflow
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    // Should be in canonical sorted order
    assert_eq!(parsed.fields()[0].fid, 10);
    assert_eq!(parsed.fields()[1].fid, 30);
    assert_eq!(parsed.fields()[2].fid, 50);
}

// ============================================================================
// Task 10.3: Edge-Case String Tests
// Requirements: 7.4, 7.5
// ============================================================================

#[test]
fn test_edge_case_empty_string() {
    // Empty strings are omitted during canonicalization (Requirement 9.3)
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    // Empty string field should be omitted
    assert!(parsed.get_field(1).is_none());
    assert_eq!(parsed.fields().len(), 0);
}

#[test]
fn test_edge_case_string_with_newlines() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("hello\nworld\ntest".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("hello\nworld\ntest".to_string())
    );
}

#[test]
fn test_edge_case_string_with_backslashes() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("path\\to\\file".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("path\\to\\file".to_string())
    );
}

#[test]
fn test_edge_case_string_with_quotes() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("say \"hello\"".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("say \"hello\"".to_string())
    );
}

#[test]
fn test_edge_case_string_with_unicode() {
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

    for (i, s) in test_strings.iter().enumerate() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: (i + 1) as u16,
            value: LnmpValue::String(s.to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let text = decoder.decode_to_text(&binary).unwrap();

        let mut parser = Parser::new(&text).unwrap();
        let parsed = parser.parse_record().unwrap();

        assert_eq!(
            parsed.get_field((i + 1) as u16).unwrap().value,
            LnmpValue::String(s.to_string()),
            "Failed for unicode string: {}",
            s
        );
    }
}

#[test]
fn test_edge_case_string_with_tabs() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("tab\there\tand\tthere".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("tab\there\tand\tthere".to_string())
    );
}

#[test]
fn test_edge_case_all_special_characters_combined() {
    let complex_string = "line1\nline2\ttab\"quote\"\\backslash emoji:🎯 中文";

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String(complex_string.to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String(complex_string.to_string())
    );
}

#[test]
fn test_edge_case_string_array_with_special_characters() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::StringArray(vec![
            "".to_string(),
            "hello\nworld".to_string(),
            "path\\to\\file".to_string(),
            "say \"hello\"".to_string(),
            "emoji: 🎯".to_string(),
            "tab\there".to_string(),
        ]),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::StringArray(vec![
            "".to_string(),
            "hello\nworld".to_string(),
            "path\\to\\file".to_string(),
            "say \"hello\"".to_string(),
            "emoji: 🎯".to_string(),
            "tab\there".to_string(),
        ])
    );
}

#[test]
fn test_edge_case_carriage_return() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("line1\rline2".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("line1\rline2".to_string())
    );
}

#[test]
fn test_edge_case_multiple_escapes_in_sequence() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("\n\n\t\t\\\\\"\"".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary).unwrap();

    let mut parser = Parser::new(&text).unwrap();
    let parsed = parser.parse_record().unwrap();

    assert_eq!(
        parsed.get_field(1).unwrap().value,
        LnmpValue::String("\n\n\t\t\\\\\"\"".to_string())
    );
}
