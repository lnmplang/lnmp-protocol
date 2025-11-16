//! LLB2 (LLM Optimization Layer v2) demonstration
//!
//! This example demonstrates the key features of LLB2:
//! - Binary ↔ ShortForm conversion
//! - Binary ↔ FullText conversion
//! - Nested structure flattening
//! - Semantic hint embedding
//! - Collision-safe ID generation

use lnmp_codec::binary::BinaryEncoder;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_llb::{LlbConfig, LlbConverter};
use std::collections::HashMap;

fn main() {
    println!("=== LLB2 (LLM Optimization Layer v2) Demo ===\n");

    // Create a sample record
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

    // 1. Binary ↔ ShortForm conversion
    println!("1. Binary ↔ ShortForm Conversion");
    println!("   Original record: F7=1, F12=14532, F23=[admin,dev]");
    
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    println!("   Binary size: {} bytes", binary.len());

    let converter = LlbConverter::default();
    let shortform = converter.binary_to_shortform(&binary).unwrap();
    println!("   ShortForm: {}", shortform);
    println!("   (Note: 'F' prefix removed for token efficiency)\n");

    // 2. Binary ↔ FullText conversion
    println!("2. Binary ↔ FullText Conversion");
    let fulltext = converter.binary_to_fulltext(&binary).unwrap();
    println!("   FullText (canonical LNMP):");
    for line in fulltext.lines() {
        println!("     {}", line);
    }
    println!();

    // 3. Semantic hints
    println!("3. Semantic Hint Embedding");
    let mut hints = HashMap::new();
    hints.insert(7, "is_admin".to_string());
    hints.insert(12, "user_id".to_string());
    hints.insert(23, "roles".to_string());

    let config = LlbConfig::new().with_semantic_hints(true);
    let converter_with_hints = LlbConverter::new(config);
    let with_hints = converter_with_hints.add_semantic_hints(&record, &hints);
    println!("   With semantic hints:");
    for line in with_hints.lines() {
        println!("     {}", line);
    }
    println!("   (Helps LLMs understand field meanings)\n");

    // 4. Nested structure flattening
    println!("4. Nested Structure Flattening");
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    inner.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    let mut nested_record = LnmpRecord::new();
    nested_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });

    println!("   Original nested: F10:r={{F1=42;F2=test}}");
    
    let config = LlbConfig::new().with_flattening(true);
    let converter_flatten = LlbConverter::new(config);
    let flattened = converter_flatten.flatten_nested(&nested_record).unwrap();
    println!("   Flattened fields: {} fields", flattened.fields().len());
    for field in flattened.fields() {
        println!("     F{}={:?}", field.fid, field.value);
    }
    println!("   (Nested structures converted to flat representation)\n");

    // 5. Collision-safe ID generation
    println!("5. Collision-Safe ID Generation");
    let field_names = vec![
        "user_id".to_string(),
        "username".to_string(),
        "email".to_string(),
        "age".to_string(),
    ];

    let config = LlbConfig::new().with_collision_safe_ids(true);
    let converter_ids = LlbConverter::new(config);
    let ids = converter_ids.generate_short_ids(&field_names).unwrap();
    
    println!("   Field name → Short ID mapping:");
    for (name, id) in &ids {
        println!("     {} → {}", name, id);
    }
    println!("   (Generates unique, short IDs for token efficiency)\n");

    println!("=== Demo Complete ===");
}
