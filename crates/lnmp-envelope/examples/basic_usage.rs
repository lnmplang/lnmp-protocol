//! Basic usage example for LNMP Envelope

use lnmp_envelope::{EnvelopeBuilder, LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP Envelope Basic Usage ===\n");

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
        fid: 20,
        value: LnmpValue::String("Alice".to_string()),
    });

    println!("Created record with {} fields", record.fields().len());

    // Wrap with envelope using builder
    let envelope = EnvelopeBuilder::new(record)
        .timestamp(1732373147000)
        .source("auth-service")
        .trace_id("abc-123-xyz")
        .sequence(42)
        .label("tenant", "acme")
        .label("env", "production")
        .build();

    println!("\nEnvelope metadata:");
    if let Some(ts) = envelope.metadata.timestamp {
        println!("  Timestamp: {}", ts);
    }
    if let Some(ref source) = envelope.metadata.source {
        println!("  Source: {}", source);
    }
    if let Some(ref trace_id) = envelope.metadata.trace_id {
        println!("  Trace ID: {}", trace_id);
    }
    if let Some(seq) = envelope.metadata.sequence {
        println!("  Sequence: {}", seq);
    }
    if !envelope.metadata.labels.is_empty() {
        println!("  Labels:");
        for (key, value) in &envelope.metadata.labels {
            println!("    {}: {}", key, value);
        }
    }

    println!("\nRecord fields:");
    for field in envelope.record.fields() {
        println!("  F{} = {:?}", field.fid, field.value);
    }

    // Binary encoding
    use lnmp_envelope::binary_codec::TlvEncoder;

    let binary = TlvEncoder::encode(&envelope.metadata).unwrap();
    println!("\nBinary TLV encoding: {} bytes", binary.len());
    println!("Hex: {}", hex_string(&binary));

    // Validate
    envelope.validate().unwrap();
    println!("\n✅ Envelope validation passed!");
}

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
