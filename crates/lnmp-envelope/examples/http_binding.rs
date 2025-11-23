//! Example: HTTP transport binding with envelope headers

use lnmp_envelope::{EnvelopeBuilder, LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP Envelope HTTP Binding Example ===\n");

    // Create envelope with metadata
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
        .build();

    // Simulate HTTP headers
    println!("HTTP Request Headers:");
    println!("POST /api/records HTTP/1.1");
    println!("Content-Type: application/lnmp-binary");

    if let Some(ts) = envelope.metadata.timestamp {
        println!("X-LNMP-Timestamp: {}", ts);
    }

    if let Some(ref source) = envelope.metadata.source {
        println!("X-LNMP-Source: {}", source);
    }

    if let Some(ref trace_id) = envelope.metadata.trace_id {
        println!("X-LNMP-Trace-ID: {}", trace_id);
    }

    if let Some(seq) = envelope.metadata.sequence {
        println!("X-LNMP-Sequence: {}", seq);
    }

    println!("\nBody: <binary LNMP record>");

    println!("\n--- Receiving Side ---\n");

    // Simulate parsing headers back to envelope
    let received_timestamp = 1732373147000u64;
    let received_source = "auth-service";
    let received_trace_id = "abc-123-xyz";
    let received_sequence = 42u64;

    println!("Parsed Metadata from Headers:");
    println!("  Timestamp: {}", received_timestamp);
    println!("  Source: {}", received_source);
    println!("  Trace ID: {}", received_trace_id);
    println!("  Sequence: {}", received_sequence);

    println!("\n✅ HTTP binding demonstration complete!");

    println!("\n💡 In production, use:");
    println!("   - axum/actix-web for Rust HTTP servers");
    println!("   - reqwest/hyper for Rust HTTP clients");
    println!("   - Custom middleware to extract/inject envelope headers");
}
