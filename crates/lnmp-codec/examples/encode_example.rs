#![allow(clippy::approx_constant)]

use lnmp_codec::Encoder;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    // Create a new record
    let mut record = LnmpRecord::new();

    // Add various fields
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("Halil Ibrahim".to_string()),
    });

    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Float(3.14159),
    });

    println!("Created record with {} fields\n", record.fields().len());

    // Encode as multiline format
    let encoder = Encoder::new();
    let multiline = encoder.encode(&record);
    println!("Multiline format:");
    println!("{}\n", multiline);

    // Encode as inline format
    let encoder_inline = Encoder::with_semicolons(true);
    let inline = encoder_inline.encode(&record);
    println!("Inline format:");
    println!("{}", inline);
}
