//! Basic example showing how to create and use LNMP records.
//!
//! Run with: `cargo run --example basic_record -p lnmp-core`

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("Creating a basic LNMP record...\n");

    // Create a new record
    let mut record = LnmpRecord::new();

    // Add fields
    // F12: User ID (Int)
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    // F7: Is Active (Bool)
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    // F23: Username (String)
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::String("alice".to_string()),
    });

    // Access fields
    if let Some(field) = record.get_field(12) {
        println!("User ID (F12): {:?}", field.value);
    }

    if let Some(field) = record.get_field(23) {
        println!("Username (F23): {:?}", field.value);
    }

    // Iterate over fields
    println!("\nAll fields:");
    for field in record.fields() {
        println!("F{} = {:?}", field.fid, field.value);
    }

    println!("\nRecord has {} fields", record.fields().len());
}
