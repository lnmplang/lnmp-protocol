//! Example demonstrating explain mode encoding in LNMP v0.3
//!
//! Explain mode adds human-readable comments to LNMP output for debugging
//! and inspection. This example shows how to use the ExplainEncoder with
//! a semantic dictionary to annotate field values.

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_llb::{ExplainEncoder, SemanticDictionary};

fn main() {
    println!("=== LNMP v0.3 Explain Mode Example ===\n");

    // Example 1: Basic explain mode
    println!("1. Basic Explain Mode");
    println!("---------------------");
    basic_explain_mode();
    println!();

    // Example 2: Explain mode without type hints
    println!("2. Explain Mode Without Type Hints");
    println!("-----------------------------------");
    explain_mode_no_type_hints();
    println!();

    // Example 3: Custom comment alignment
    println!("3. Custom Comment Alignment");
    println!("---------------------------");
    custom_comment_alignment();
    println!();

    // Example 4: Nested structures with explain mode
    println!("4. Nested Structures with Explain Mode");
    println!("---------------------------------------");
    nested_with_explain_mode();
    println!();

    // Example 5: Fields without dictionary entries
    println!("5. Fields Without Dictionary Entries");
    println!("-------------------------------------");
    fields_without_names();
}

fn basic_explain_mode() {
    // Create a semantic dictionary mapping field IDs to names
    let dict = SemanticDictionary::from_pairs(vec![
        (12, "user_id"),
        (7, "is_active"),
        (23, "roles"),
        (15, "score"),
        (20, "username"),
    ]);

    // Create a record with various field types
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });
    record.add_field(LnmpField {
        fid: 15,
        value: LnmpValue::Float(98.5),
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("alice".to_string()),
    });

    // Encode with explain mode
    let encoder = ExplainEncoder::new(dict);
    let output = encoder.encode_with_explanation(&record);

    println!("Encoded with explanations:");
    println!("{}", output);
    println!();
    println!("Note: Comments are aligned for readability");
    println!("      Fields are automatically sorted by FID");
}

fn explain_mode_no_type_hints() {
    let dict = SemanticDictionary::from_pairs(vec![
        (1, "id"),
        (2, "name"),
        (3, "active"),
    ]);

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

    // Create encoder without type hints
    let encoder = ExplainEncoder::new(dict).with_type_hints(false);
    let output = encoder.encode_with_explanation(&record);

    println!("Without type hints:");
    println!("{}", output);
    println!();
    println!("Note: Type hints omitted for more compact output");
}

fn custom_comment_alignment() {
    let dict = SemanticDictionary::from_pairs(vec![
        (1, "id"),
        (100, "very_long_field_name"),
    ]);

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Int(12345678),
    });

    // Default alignment (column 20)
    let encoder_default = ExplainEncoder::new(dict.clone());
    let output_default = encoder_default.encode_with_explanation(&record);
    println!("Default alignment (column 20):");
    println!("{}", output_default);
    println!();

    // Custom alignment (column 30)
    let encoder_custom = ExplainEncoder::new(dict).with_comment_column(30);
    let output_custom = encoder_custom.encode_with_explanation(&record);
    println!("Custom alignment (column 30):");
    println!("{}", output_custom);
}

fn nested_with_explain_mode() {
    let dict = SemanticDictionary::from_pairs(vec![
        (50, "user_profile"),
        (12, "user_id"),
        (7, "is_active"),
    ]);

    // Create nested record
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    inner.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });

    let encoder = ExplainEncoder::new(dict);
    let output = encoder.encode_with_explanation(&record);

    println!("Nested structure with explain mode:");
    println!("{}", output);
    println!();
    println!("Note: Nested fields are encoded inline");
    println!("      Only the outer field has a comment");
}

fn fields_without_names() {
    // Dictionary with only some field names
    let dict = SemanticDictionary::from_pairs(vec![
        (12, "user_id"),
        (23, "roles"),
    ]);

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string()]),
    });
    record.add_field(LnmpField {
        fid: 99,
        value: LnmpValue::Int(42),
    });

    let encoder = ExplainEncoder::new(dict);
    let output = encoder.encode_with_explanation(&record);

    println!("Mixed fields (some with names, some without):");
    println!("{}", output);
    println!();
    println!("Note: Fields without dictionary entries (F7, F99)");
    println!("      are encoded without comments");
}
