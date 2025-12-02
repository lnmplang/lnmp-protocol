//! Example showing how to encode LNMP records with type hints.
//!
//! Run with: `cargo run --example encode_with_hints -p lnmp-codec`

use lnmp_codec::{Encoder, EncoderConfig};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("Encoding record with type hints...\n");

    // Create a record
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
        value: LnmpValue::String("alice".to_string()),
    });

    // 1. Default encoding (no hints)
    let encoder = Encoder::new();
    println!("Default encoding:");
    println!("{}", encoder.encode(&record));

    // 2. Encoding with type hints
    let config = EncoderConfig {
        include_type_hints: true,
        ..Default::default()
    };

    let hint_encoder = Encoder::with_config(config);
    println!("\nWith type hints:");
    println!("{}", hint_encoder.encode(&record));

    // 3. Encoding with checksums
    let config_checksum = EncoderConfig {
        enable_checksums: true,
        ..Default::default()
    };

    let checksum_encoder = Encoder::with_config(config_checksum);
    println!("\nWith checksums:");
    println!("{}", checksum_encoder.encode(&record));
}
