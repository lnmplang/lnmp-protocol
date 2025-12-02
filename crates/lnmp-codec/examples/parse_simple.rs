//! Basic example showing how to parse LNMP text format.
//!
//! Run with: `cargo run --example parse_simple -p lnmp-codec`

use lnmp_codec::Parser;
use lnmp_core::LnmpValue;

fn main() {
    let input = "F12=14532\nF7=1\nF23=alice";
    println!("Parsing input:\n{}\n", input);

    // Create parser
    let mut parser = Parser::new(input).unwrap();

    // Parse record
    match parser.parse_record() {
        Ok(record) => {
            println!(
                "Successfully parsed record with {} fields:",
                record.fields().len()
            );

            for field in record.fields() {
                println!("F{} = {:?}", field.fid, field.value);
            }

            // Verify values
            if let Some(field) = record.get_field(12) {
                assert_eq!(field.value, LnmpValue::Int(14532));
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
