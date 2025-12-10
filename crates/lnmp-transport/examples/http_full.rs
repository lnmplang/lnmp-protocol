#[cfg(feature = "http")]
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
#[cfg(feature = "http")]
use lnmp_envelope::EnvelopeBuilder;
#[cfg(feature = "http")]
use lnmp_transport::http;

#[cfg(not(feature = "http"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HTTP feature not enabled. Run with: cargo run --example http_full --features http");
    Ok(())
}

#[cfg(feature = "http")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP HTTP Full Example ===\n");

    // 1. Create a record with multiple fields
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("user@example.com".into()),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(std::f64::consts::PI),
    });

    // 2. Build envelope with metadata
    let envelope = EnvelopeBuilder::new(record)
        .timestamp(1732373147000)
        .source("http-example-service")
        .trace_id("550e8400-e29b-41d4-a716-446655440000")
        .sequence(1)
        .label("environment", "production")
        .label("region", "us-east-1")
        .build();

    println!("Original Envelope:");
    println!("  Timestamp: {:?}", envelope.metadata.timestamp);
    println!("  Source: {:?}", envelope.metadata.source);
    println!("  Trace ID: {:?}", envelope.metadata.trace_id);
    println!("  Fields: {}\n", envelope.record.fields().len());

    // 3. Convert to HTTP headers
    #[cfg(feature = "http")]
    {
        let headers = http::envelope_to_headers(&envelope)?;

        println!("HTTP Headers:");
        for (name, value) in &headers {
            println!("  {}: {:?}", name, value);
        }
        println!();

        // 4. Encode body (binary format)
        let (body, content_type) = http::record_to_http_body(&envelope.record)?;

        println!("HTTP Body:");
        println!("  Content-Type: {}", content_type);
        println!("  Body size: {} bytes", body.len());
        println!(
            "  Body (hex): {}...",
            hex::encode(&body[..16.min(body.len())])
        );
        println!();

        // 5. Simulate receiving the HTTP request
        println!("=== Simulating HTTP Request Reception ===\n");

        // 6. Parse headers back to metadata
        let received_metadata = http::headers_to_envelope_metadata(&headers)?;

        println!("Parsed Metadata:");
        println!("  Timestamp: {:?}", received_metadata.timestamp);
        println!("  Source: {:?}", received_metadata.source);
        println!("  Trace ID: {:?}", received_metadata.trace_id);
        println!("  Sequence: {:?}", received_metadata.sequence);
        println!("  Labels: {:?}", received_metadata.labels);
        println!();

        // 7. Decode body back to record
        let received_record = http::http_body_to_record(&body, content_type)?;

        println!("Parsed Record:");
        println!("  Fields: {}", received_record.fields().len());
        for field in received_record.fields() {
            println!("    F{}: {:?}", field.fid, field.value);
        }
        println!();

        // 8. Verify round-trip
        println!("=== Round-Trip Verification ===");
        assert_eq!(received_metadata.timestamp, envelope.metadata.timestamp);
        assert_eq!(received_metadata.source, envelope.metadata.source);
        assert_eq!(received_metadata.trace_id, envelope.metadata.trace_id);
        assert_eq!(
            received_record.fields().len(),
            envelope.record.fields().len()
        );
        println!("✓ Round-trip successful!");
    }

    #[cfg(not(feature = "http"))]
    {
        println!(
            "HTTP feature not enabled. Run with: cargo run --example http_full --features http"
        );
    }

    Ok(())
}
