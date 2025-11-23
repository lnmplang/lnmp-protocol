//! Example: Kafka transport binding with record headers

use lnmp_envelope::{EnvelopeBuilder, LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP Envelope Kafka Binding Example ===\n");

    // Create envelope
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    let envelope = EnvelopeBuilder::new(record)
        .timestamp(1732373147000)
        .source("auth-service")
        .trace_id("abc-123-xyz")
        .sequence(42)
        .label("tenant", "acme")
        .label("env", "production")
        .build();

    // Simulate Kafka ProducerRecord
    println!("Kafka ProducerRecord {{");
    println!("  topic: \"lnmp-events\",");
    println!("  key: None,");
    println!("  headers: [");

    if let Some(ts) = envelope.metadata.timestamp {
        println!("    (\"lnmp.timestamp\", \"{}\"),", ts);
    }

    if let Some(ref source) = envelope.metadata.source {
        println!("    (\"lnmp.source\", \"{}\"),", source);
    }

    if let Some(ref trace_id) = envelope.metadata.trace_id {
        println!("    (\"lnmp.trace_id\", \"{}\"),", trace_id);
    }

    if let Some(seq) = envelope.metadata.sequence {
        println!("    (\"lnmp.sequence\", \"{}\"),", seq);
    }

    for (key, value) in &envelope.metadata.labels {
        println!("    (\"lnmp.label.{}\", \"{}\"),", key, value);
    }

    println!("  ],");
    println!("  value: <binary LNMP record>,");
    println!("}}\n");

    println!("--- Consuming Side ---\n");
    println!("Received Kafka Record:");
    println!("  Topic: lnmp-events");
    println!("  Partition: 0");
    println!("  Offset: 12345");
    println!("\nExtracted Metadata from Headers:");
    println!("  lnmp.timestamp → {}", 1732373147000u64);
    println!("  lnmp.source → auth-service");
    println!("  lnmp.trace_id → abc-123-xyz");
    println!("  lnmp.sequence → 42");
    println!("  lnmp.label.tenant → acme");
    println!("  lnmp.label.env → production");

    println!("\n✅ Kafka binding demonstration complete!");

    println!("\n💡 In production, use:");
    println!("   - rdkafka crate for Kafka integration");
    println!("   - kafka-rust for pure Rust implementation");
    println!("   - Custom serializer/deserializer for automatic envelope handling");
}
