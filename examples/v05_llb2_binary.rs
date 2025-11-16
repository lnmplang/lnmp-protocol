//! Example demonstrating LNMP v0.5 LLB2 with Binary Format Integration
//!
//! This example shows:
//! - Binary ↔ ShortForm conversion for token efficiency
//! - Binary ↔ FullText conversion for readability
//! - Flattening nested binary structures for LLM consumption
//! - Semantic hint embedding in binary context
//! - Collision-safe ID generation for binary fields
//! - Round-trip conversions maintaining semantic equivalence

use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder, BinaryNestedEncoder, NestedEncoderConfig};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_llb::{LlbConfig, LlbConverter};
use std::collections::HashMap;

fn main() {
    println!("=== LNMP v0.5 LLB2 Binary Integration Example ===\n");

    // Example 1: Binary to ShortForm for LLM input
    println!("1. Binary → ShortForm (Token Optimization):");
    binary_to_shortform();
    println!();

    // Example 2: Binary to FullText for debugging
    println!("2. Binary → FullText (Human Readable):");
    binary_to_fulltext();
    println!();

    // Example 3: Flatten nested binary for LLM
    println!("3. Flatten Nested Binary Structures:");
    flatten_nested_binary();
    println!();

    // Example 4: Semantic hints with binary
    println!("4. Semantic Hints in Binary Context:");
    semantic_hints_binary();
    println!();

    // Example 5: Collision-safe IDs for binary fields
    println!("5. Collision-Safe IDs:");
    collision_safe_ids_example();
    println!();

    // Example 6: Complete workflow
    println!("6. Complete LLM Optimization Workflow:");
    complete_workflow();
    println!();
}

fn binary_to_shortform() {
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

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    println!("   Binary size: {} bytes", binary.len());

    // Convert to ShortForm for LLM input
    let converter = LlbConverter::default();
    let shortform = converter.binary_to_shortform(&binary).unwrap();
    println!("   ShortForm: {}", shortform);
    println!("   (Optimized for minimal token usage)");

    // Estimate token savings
    let standard_text = "F7=1;F12=14532;F23=[admin,dev]";
    println!("   Standard LNMP: {} chars", standard_text.len());
    println!("   ShortForm: {} chars", shortform.len());
    println!("   Savings: ~{:.1}%", 
             (1.0 - shortform.len() as f64 / standard_text.len() as f64) * 100.0);
}

fn binary_to_fulltext() {
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

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    println!("   Binary: {} bytes", binary.len());

    // Convert to FullText for human inspection
    let converter = LlbConverter::default();
    let fulltext = converter.binary_to_fulltext(&binary).unwrap();
    println!("   FullText (canonical LNMP):");
    for line in fulltext.lines() {
        println!("     {}", line);
    }

    // Round-trip: FullText → Binary
    let binary2 = converter.fulltext_to_binary(&fulltext).unwrap();
    println!("   ✓ Round-trip successful: {}", binary == binary2);
}

fn flatten_nested_binary() {
    // Create a nested record
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

    println!("   Original: F10=Alice, F20={{F1=123 Main St, F2=Springfield}}");

    // Encode to binary with nested support (use explicit nested encoder config)
    let config = NestedEncoderConfig::new().with_max_depth(32);
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&user).unwrap();
    println!("   Binary size: {} bytes", binary.len());

    // Flatten for LLM consumption
    let config = LlbConfig::new().with_flattening(true);
    let converter = LlbConverter::new(config);
    let flattened = converter.flatten_nested(&user).unwrap();

    println!("   Flattened representation:");
    for field in flattened.fields() {
        match &field.value {
            LnmpValue::String(s) => println!("     F{}={}", field.fid, s),
            LnmpValue::Int(i) => println!("     F{}={}", field.fid, i),
            _ => println!("     F{}={:?}", field.fid, field.value),
        }
    }
    println!("   (Nested structure converted to flat fields with dot notation)");

    // Unflatten back to original structure
    let unflattened = converter.unflatten(&flattened).unwrap();
    println!("   ✓ Unflattened back to original structure");
    println!("   Original fields: {}, Unflattened fields: {}", 
             user.fields().len(), unflattened.fields().len());
}

fn semantic_hints_binary() {
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
        value: LnmpValue::StringArray(vec!["admin".to_string()]),
    });

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    println!("   Binary size: {} bytes", binary.len());

    // Add semantic hints for LLM understanding
    let mut hints = HashMap::new();
    hints.insert(7, "is_admin".to_string());
    hints.insert(12, "user_id".to_string());
    hints.insert(23, "roles".to_string());

    let config = LlbConfig::new().with_semantic_hints(true);
    let converter = LlbConverter::new(config);
    let with_hints = converter.add_semantic_hints(&record, &hints);

    println!("   With semantic hints:");
    for line in with_hints.lines() {
        println!("     {}", line);
    }
    println!("   (Helps LLMs understand field semantics)");

    // Convert to ShortForm with hints
    let shortform_with_hints = converter.binary_to_shortform(&binary).unwrap();
    println!("   ShortForm: {}", shortform_with_hints);
}

fn collision_safe_ids_example() {
    // Define field names that might collide
    let field_names = vec![
        "user_id".to_string(),
        "user_name".to_string(),
        "user_email".to_string(),
        "order_id".to_string(),
        "order_date".to_string(),
    ];

    println!("   Field names: {:?}", field_names);

    // Generate collision-safe short IDs
    let config = LlbConfig::new().with_collision_safe_ids(true);
    let converter = LlbConverter::new(config);
    let ids = converter.generate_short_ids(&field_names).unwrap();

    println!("   Generated collision-safe IDs:");
    for (name, id) in &ids {
        println!("     {} → {}", name, id);
    }

    // Create a record using these IDs
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("alice".to_string()),
    });

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();

    // Convert to ShortForm with collision-safe IDs
    let shortform = converter.binary_to_shortform(&binary).unwrap();
    println!("   ShortForm with safe IDs: {}", shortform);
    println!("   ✓ No ID collisions, optimal for LLM tokenization");
}

fn complete_workflow() {
    println!("   Scenario: Sending nested data to LLM with optimal token usage");
    println!();

    // Step 1: Create complex nested data
    let mut metadata = LnmpRecord::new();
    metadata.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("2024-01-15".to_string()),
    });
    metadata.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(42),
    });

    let mut user = LnmpRecord::new();
    user.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("Alice".to_string()),
    });
    user.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    user.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::NestedRecord(Box::new(metadata)),
    });

    println!("   Step 1: Created nested record");
    println!("     User: F10=Alice, F12=14532");
    println!("     Metadata: F20={{F1=2024-01-15, F2=42}}");

    // Step 2: Encode to binary
    let config = NestedEncoderConfig::new().with_max_depth(32);
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&user).unwrap();
    println!("   Step 2: Encoded to binary ({} bytes)", binary.len());

    // Step 3: Flatten for LLM
    let llb_config = LlbConfig::new()
        .with_flattening(true)
        .with_semantic_hints(true);
    let converter = LlbConverter::new(llb_config);
    let flattened = converter.flatten_nested(&user).unwrap();
    println!("   Step 3: Flattened nested structure");

    // Step 4: Add semantic hints
    let mut hints = HashMap::new();
    hints.insert(10, "name".to_string());
    hints.insert(12, "user_id".to_string());
    let with_hints = converter.add_semantic_hints(&flattened, &hints);
    println!("   Step 4: Added semantic hints");

    // Step 5: Convert to ShortForm for minimal tokens
    let encoder = BinaryEncoder::new();
    let flat_binary = encoder.encode(&flattened).unwrap();
    let shortform = converter.binary_to_shortform(&flat_binary).unwrap();
    println!("   Step 5: Converted to ShortForm");
    println!();

    println!("   Final LLM input:");
    println!("     {}", shortform);
    println!();

    // Compare sizes
    let original_text = "F10=Alice;F12=14532;F20={F1=2024-01-15;F2=42}";
    println!("   Size comparison:");
    println!("     Original text: {} chars", original_text.len());
    println!("     Binary: {} bytes", binary.len());
    println!("     ShortForm: {} chars", shortform.len());
    println!("     Token savings: ~{:.1}%", 
             (1.0 - shortform.len() as f64 / original_text.len() as f64) * 100.0);
    println!();

    println!("   ✓ Workflow complete: Optimal LLM representation achieved");
}
