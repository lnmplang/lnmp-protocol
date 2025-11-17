//! Example demonstrating nested record and nested array encoding/parsing in LNMP v0.3
//!
//! This example shows:
//! - Encoding nested records with hierarchical data
//! - Encoding nested arrays of records
//! - Structural canonicalization (automatic field sorting)
//! - Optional checksum integration
//! - Round-trip parsing and encoding

use lnmp_codec::{Encoder, Parser};
use lnmp_codec::config::EncoderConfig;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.3 Nested Structures Example ===\n");

    // Example 1: Simple nested record
    println!("1. Simple Nested Record:");
    simple_nested_record();
    println!();

    // Example 2: Nested array of records
    println!("2. Nested Array of Records:");
    nested_array_example();
    println!();

    // Example 3: Complex nested structure
    println!("3. Complex Nested Structure:");
    complex_nested_structure();
    println!();

    // Example 4: Nested structures with checksums
    println!("4. Nested Structures with Checksums:");
    nested_with_checksums();
    println!();

    // Example 5: Structural canonicalization
    println!("5. Structural Canonicalization:");
    canonicalization_example();
}

fn simple_nested_record() {
    // Create a nested record: user profile with embedded address
    let mut address = LnmpRecord::new();
    address.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("123 Main St".to_string()),
    });
    address.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("Springfield".to_string()),
    });

    let mut user = LnmpRecord::new();
    user.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("Alice".to_string()),
    });
    user.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::NestedRecord(Box::new(address)),
    });

    let encoder = Encoder::new();
    let encoded = encoder.encode(&user);
    println!("Encoded: {}", encoded);

    // Parse it back
    let mut parser = Parser::new(&encoded).unwrap();
    let parsed = parser.parse_record().unwrap();
    println!("Parsed successfully: {} fields", parsed.fields().len());
}

fn nested_array_example() {
    // Create a nested array: list of users
    let mut user1 = LnmpRecord::new();
    user1.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Alice".to_string()),
    });
    user1.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("admin".to_string()),
    });

    let mut user2 = LnmpRecord::new();
    user2.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Bob".to_string()),
    });
    user2.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("user".to_string()),
    });

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::NestedArray(vec![user1, user2]),
    });

    let encoder = Encoder::new();
    let encoded = encoder.encode(&record);
    println!("Encoded: {}", encoded);

    // Parse it back
    let mut parser = Parser::new(&encoded).unwrap();
    let parsed = parser.parse_record().unwrap();
    
    if let Some(field) = parsed.get_field(100) {
        if let LnmpValue::NestedArray(users) = &field.value {
            println!("Parsed {} users successfully", users.len());
        }
    }
}

fn complex_nested_structure() {
    // Create a complex structure with multiple nesting levels
    let mut level3 = LnmpRecord::new();
    level3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("deep value".to_string()),
    });

    let mut level2 = LnmpRecord::new();
    level2.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("middle value".to_string()),
    });
    level2.add_field(LnmpField {
        fid: 11,
        value: LnmpValue::NestedRecord(Box::new(level3)),
    });

    let mut level1 = LnmpRecord::new();
    level1.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("top value".to_string()),
    });
    level1.add_field(LnmpField {
        fid: 200,
        value: LnmpValue::NestedRecord(Box::new(level2)),
    });

    let encoder = Encoder::new();
    let encoded = encoder.encode(&level1);
    println!("Encoded: {}", encoded);
    println!("Depth: {} levels", get_max_depth(&level1));
}

fn nested_with_checksums() {
    // Create nested structure with checksums enabled
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

    let config = EncoderConfig::new()
        .with_type_hints(true)
        .with_canonical(true)
        .with_checksums(true);
    let encoder = Encoder::with_config(config);
    let encoded = encoder.encode(&record);
    println!("Encoded with checksum: {}", encoded);
    println!("Note: Checksum protects the entire nested structure");
}

fn canonicalization_example() {
    // Create a record with unsorted fields
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::Int(2),
    });
    inner.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(1),
    });

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("test".to_string()),
    });

    let encoder = Encoder::new();
    let encoded = encoder.encode(&record);
    println!("Original field order: F100, F50 (outer), F20, F10 (inner)");
    println!("Canonical encoding: {}", encoded);
    println!("Note: Fields are automatically sorted at all nesting levels");
}

fn get_max_depth(record: &LnmpRecord) -> usize {
    record
        .fields()
        .iter()
        .map(|field| field.value.depth())
        .max()
        .unwrap_or(0)
}
