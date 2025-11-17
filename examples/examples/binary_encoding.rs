//! Basic binary encoding example for LNMP v0.4
//!
//! This example demonstrates:
//! - Creating LNMP records
//! - Encoding records to binary format
//! - Decoding binary format back to records
//! - Converting between text and binary formats

use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

fn main() {
    println!("=== LNMP v0.4 Binary Encoding Example ===\n");

    // Example 1: Encode a simple record
    println!("1. Basic Record Encoding");
    println!("{}", "-".repeat(40));
    
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
    
    println!("Original record: {} fields", record.fields().len());
    println!("Binary size: {} bytes", binary.len());
    println!("Binary (hex): {}", hex_dump(&binary));
    println!();

    // Example 2: Decode binary back to record
    println!("2. Binary Decoding");
    println!("{}", "-".repeat(40));
    
    let decoder = BinaryDecoder::new();
    let decoded_record = decoder.decode(&binary).unwrap();
    
    println!("Decoded record: {} fields", decoded_record.fields().len());
    for field in decoded_record.fields() {
        println!("  F{} = {:?}", field.fid, field.value);
    }
    println!();

    // Example 3: Text to binary conversion
    println!("3. Text to Binary Conversion");
    println!("{}", "-".repeat(40));
    
    let text = "F7=1;F12=14532;F23=[\"admin\",\"dev\"]";
    println!("Input text: {}", text);
    
    let binary_from_text = encoder.encode_text(text).unwrap();
    println!("Binary size: {} bytes", binary_from_text.len());
    println!("Binary (hex): {}", hex_dump(&binary_from_text));
    println!();

    // Example 4: Binary to text conversion
    println!("4. Binary to Text Conversion");
    println!("{}", "-".repeat(40));
    
    let decoded_text = decoder.decode_to_text(&binary_from_text).unwrap();
    println!("Decoded text: {}", decoded_text);
    println!();

    // Example 5: All value types
    println!("5. All Value Types");
    println!("{}", "-".repeat(40));
    
    let mut all_types_record = LnmpRecord::new();
    all_types_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(-42),
    });
    all_types_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14159),
    });
    all_types_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(false),
    });
    all_types_record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("hello\nworld".to_string()),
    });
    all_types_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
    });

    let all_types_binary = encoder.encode(&all_types_record).unwrap();
    println!("Record with all types:");
    println!("  Binary size: {} bytes", all_types_binary.len());
    
    let all_types_text = decoder.decode_to_text(&all_types_binary).unwrap();
    println!("  Text format:\n{}", all_types_text.replace('\n', "\n    "));
    println!();

    // Example 6: Space efficiency comparison
    println!("6. Space Efficiency");
    println!("{}", "-".repeat(40));
    
    let text_size = decoded_text.len();
    let binary_size = binary_from_text.len();
    let savings = ((text_size as f64 - binary_size as f64) / text_size as f64) * 100.0;
    
    println!("Text format: {} bytes", text_size);
    println!("Binary format: {} bytes", binary_size);
    println!("Space savings: {:.1}%", savings);
    println!();

    // Example 7: Canonical form guarantee
    println!("7. Canonical Form (Field Sorting)");
    println!("{}", "-".repeat(40));
    
    let unsorted_text = "F23=[\"admin\"];F7=1;F12=14532";
    println!("Unsorted input: {}", unsorted_text);
    
    let binary_unsorted = encoder.encode_text(unsorted_text).unwrap();
    let canonical_text = decoder.decode_to_text(&binary_unsorted).unwrap();
    
    println!("Canonical output: {}", canonical_text);
    println!("(Fields automatically sorted by FID)");
}

/// Helper function to format binary data as hex dump
fn hex_dump(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
