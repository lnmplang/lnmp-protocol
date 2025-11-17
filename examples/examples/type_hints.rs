//! Example demonstrating type hint usage in LNMP v0.2

use lnmp_codec::{Encoder, EncoderConfig, Parser};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.2 Type Hints Example ===\n");

    // Create a record with various types
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("hello".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    // Encode WITHOUT type hints (default)
    println!("Without type hints:");
    let encoder = Encoder::new();
    let output = encoder.encode(&record);
    println!("{}\n", output);

    // Encode WITH type hints
    println!("With type hints:");
    let config = EncoderConfig::new()
        .with_type_hints(true)
        .with_canonical(true);
    let encoder_with_hints = Encoder::with_config(config);
    let output_with_hints = encoder_with_hints.encode(&record);
    println!("{}\n", output_with_hints);

    // Parse input with type hints
    println!("Parsing with type hints:");
    let input = "F10:i=100\nF20:s=test\nF30:sa=[a,b,c]";
    println!("Input: {}", input);

    let mut parser = Parser::new(input).unwrap();
    let parsed_record = parser.parse_record().unwrap();

    println!("Parsed {} fields:", parsed_record.fields().len());
    for field in parsed_record.fields() {
        println!("  F{} = {:?}", field.fid, field.value);
    }

    // Type hint validation
    println!("\n=== Type Hint Validation ===");
    let valid_input = "F1:i=42";
    println!("Valid: {}", valid_input);
    match Parser::new(valid_input) {
        Ok(mut p) => match p.parse_record() {
            Ok(_) => println!("✓ Parsed successfully"),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }

    let invalid_input = "F1:i=3.14"; // Type hint says int, but value is float
    println!("\nInvalid: {}", invalid_input);
    match Parser::new(invalid_input) {
        Ok(mut p) => match p.parse_record() {
            Ok(_) => println!("✓ Parsed successfully"),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }
}
