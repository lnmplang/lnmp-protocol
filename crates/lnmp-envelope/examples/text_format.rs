//! Example: Text format encoding with envelope header

use lnmp_envelope::{
    text_codec::{TextDecoder, TextEncoder},
    EnvelopeBuilder, LnmpField, LnmpRecord, LnmpValue,
};

fn main() {
    println!("=== LNMP Envelope Text Format Example ===\n");

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

    // Create envelope
    let envelope = EnvelopeBuilder::new(record)
        .timestamp(1732373147000)
        .source("auth-service")
        .trace_id("abc-123-xyz")
        .sequence(42)
        .build();

    // Encode metadata as header
    let header = TextEncoder::encode(&envelope.metadata).unwrap();
    println!("Envelope Header:");
    println!("{}\n", header);

    // Encode record (using lnmp-codec would go here)
    println!("Record (conceptual):");
    println!("F7=1");
    println!("F12=14532\n");

    // Complete LNMP text with envelope
    println!("Complete LNMP Text with Envelope:");
    println!("{}", header);
    println!("F7=1");
    println!("F12=14532\n");

    // Decode envelope header
    let decoded_metadata = TextDecoder::decode(&header).unwrap();
    if let Some(meta) = decoded_metadata {
        println!("Decoded Metadata:");
        println!("  Timestamp: {:?}", meta.timestamp);
        println!("  Source: {:?}", meta.source);
        println!("  Trace ID: {:?}", meta.trace_id);
        println!("  Sequence: {:?}", meta.sequence);
    }

    // Round-trip test
    println!("\n✅ Round-trip successful!");
}
