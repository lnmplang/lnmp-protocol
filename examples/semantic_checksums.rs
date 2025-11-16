//! Example demonstrating semantic checksums (SC32) in LNMP v0.3
//!
//! This example shows how to:
//! - Compute checksums for field values
//! - Validate checksums
//! - Format checksums for encoding
//! - Use checksums to prevent LLM input drift

use lnmp_core::checksum::SemanticChecksum;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue, TypeHint};

fn main() {
    println!("=== LNMP v0.3 Semantic Checksums (SC32) ===\n");

    // Example 1: Basic checksum computation
    println!("1. Basic Checksum Computation");
    println!("------------------------------");
    let value = LnmpValue::Int(14532);
    let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
    let formatted = SemanticChecksum::format(checksum);
    println!("Field: F12:i=14532");
    println!("Checksum: {}", formatted);
    println!("Encoded: F12:i=14532#{}\n", formatted);

    // Example 2: Checksum validation
    println!("2. Checksum Validation");
    println!("----------------------");
    let is_valid = SemanticChecksum::validate(12, Some(TypeHint::Int), &value, checksum);
    println!("Valid checksum: {}", is_valid);
    
    let wrong_checksum = checksum + 1;
    let is_invalid = SemanticChecksum::validate(12, Some(TypeHint::Int), &value, wrong_checksum);
    println!("Invalid checksum: {}\n", is_invalid);

    // Example 3: Different field types
    println!("3. Checksums for Different Types");
    println!("---------------------------------");
    
    let bool_value = LnmpValue::Bool(true);
    let bool_checksum = SemanticChecksum::compute(7, Some(TypeHint::Bool), &bool_value);
    println!("F7:b=1#{}", SemanticChecksum::format(bool_checksum));
    
    let float_value = LnmpValue::Float(3.14);
    let float_checksum = SemanticChecksum::compute(15, Some(TypeHint::Float), &float_value);
    println!("F15:f=3.14#{}", SemanticChecksum::format(float_checksum));
    
    let string_value = LnmpValue::String("admin".to_string());
    let string_checksum = SemanticChecksum::compute(23, Some(TypeHint::String), &string_value);
    println!("F23:s=admin#{}", SemanticChecksum::format(string_checksum));
    
    let array_value = LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]);
    let array_checksum = SemanticChecksum::compute(30, Some(TypeHint::StringArray), &array_value);
    println!("F30:sa=[admin,dev]#{}\n", SemanticChecksum::format(array_checksum));

    // Example 4: Semantic equivalence - normalized values
    println!("4. Semantic Equivalence");
    println!("-----------------------");
    
    // -0.0 and 0.0 should produce the same checksum
    let neg_zero = LnmpValue::Float(-0.0);
    let pos_zero = LnmpValue::Float(0.0);
    let checksum_neg = SemanticChecksum::compute(10, Some(TypeHint::Float), &neg_zero);
    let checksum_pos = SemanticChecksum::compute(10, Some(TypeHint::Float), &pos_zero);
    println!("F10:f=-0.0 checksum: {}", SemanticChecksum::format(checksum_neg));
    println!("F10:f=0.0 checksum:  {}", SemanticChecksum::format(checksum_pos));
    println!("Checksums match: {}\n", checksum_neg == checksum_pos);

    // Example 5: Complete record with checksums
    println!("5. Complete Record with Checksums");
    println!("----------------------------------");
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
    
    println!("Record with checksums:");
    for field in record.sorted_fields() {
        let type_hint = match &field.value {
            LnmpValue::Int(_) => TypeHint::Int,
            LnmpValue::Bool(_) => TypeHint::Bool,
            LnmpValue::StringArray(_) => TypeHint::StringArray,
            _ => TypeHint::String,
        };
        
        let checksum = SemanticChecksum::compute(field.fid, Some(type_hint), &field.value);
        let formatted = SemanticChecksum::format(checksum);
        
        let value_str = match &field.value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            LnmpValue::StringArray(arr) => format!("[{}]", arr.join(",")),
            _ => String::new(),
        };
        
        println!("F{}:{}={}#{}", field.fid, type_hint.as_str(), value_str, formatted);
    }
    
    println!("\n=== Checksums prevent LLM input drift ===");
    println!("Any modification to FID, type, or value will invalidate the checksum!");
}
