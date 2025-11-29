//! Basic LNMP encoding example
//!  
//! This example demonstrates how to:
//! - Create LNMP values
//! - Build LNMP records  
//! - Use the encoder

use lnmp::core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Basic Encoding Example ===\n");

    // 1. Simple values
    println!("--- Basic Values ---");
    let int_val = LnmpValue::Int(42);
    let float_val = LnmpValue::Float(2.5);
    let bool_val = LnmpValue::Bool(true);
    let str_val = LnmpValue::String("Hello, LNMP!".to_string());

    println!("Int: {:?}", int_val);
    println!("Float: {:?}", float_val);
    println!("Bool: {:?}", bool_val);
    println!("String: {:?}\n", str_val);

    // 2. Create an LNMP record
    println!("--- LNMP Record ---");
    let mut record = LnmpRecord::new();

    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("LNMP Protocol".to_string()),
    });

    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("0.5.12".to_string()),
    });

    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    println!("Created record with {} fields", record.fields().len());
    for field in record.fields() {
        println!("  Field {}: {:?}", field.fid, field.value);
    }

    // 3. Encode the record
    println!("\n--- Encoding ---");
    let encoder = Encoder::new();
    let encoded = encoder.encode(&record);
    println!("Encoded LNMP format:\n{}", encoded);

    println!("\n✅ All encoding operations completed successfully!");

    Ok(())
}
