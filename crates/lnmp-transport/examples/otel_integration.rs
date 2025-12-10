#[cfg(feature = "http")]
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
#[cfg(feature = "http")]
use lnmp_envelope::EnvelopeBuilder;
#[cfg(feature = "http")]
use lnmp_transport::http;

#[cfg(not(feature = "http"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "HTTP feature not enabled. Run with: cargo run --example otel_integration --features http"
    );
    Ok(())
}

#[cfg(feature = "http")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP OpenTelemetry Integration Example ===\n");

    // Simulate an OpenTelemetry trace context from an incoming request
    let incoming_traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

    println!("1. Incoming W3C Trace Context:");
    println!("   traceparent: {}\n", incoming_traceparent);

    // Extract trace_id from traceparent
    let trace_id = http::traceparent_to_trace_id(incoming_traceparent)?;

    println!("2. Extracted Trace ID:");
    println!("   trace_id: {}\n", trace_id);

    // Create an LNMP envelope with the extracted trace_id
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Processed with distributed tracing".into()),
    });

    let envelope = EnvelopeBuilder::new(record)
        .timestamp(1732373147000)
        .source("otel-service")
        .trace_id(&trace_id) // Use extracted trace_id
        .sequence(1)
        .label("span.kind", "server")
        .label("service.name", "lnmp-processor")
        .build();

    println!("3. Created LNMP Envelope:");
    println!(
        "   Trace ID: {}",
        envelope.metadata.trace_id.as_ref().unwrap()
    );
    println!("   Source: {}", envelope.metadata.source.as_ref().unwrap());
    println!();

    // Convert to HTTP headers (this will generate a new traceparent)
    #[cfg(feature = "http")]
    {
        let headers = http::envelope_to_headers(&envelope)?;

        println!("4. Outgoing HTTP Headers:");
        println!(
            "   X-LNMP-Trace-Id: {:?}",
            headers.get("x-lnmp-trace-id").unwrap()
        );
        println!("   traceparent: {:?}", headers.get("traceparent").unwrap());
        println!();

        // Demonstrate trace propagation to downstream service
        println!("5. Downstream Service Propagation:");

        let downstream_traceparent = headers
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .unwrap();

        println!("   Propagated traceparent: {}", downstream_traceparent);

        // Verify trace_id is preserved
        let downstream_trace_id = http::traceparent_to_trace_id(downstream_traceparent)?;
        println!("   Verified trace_id: {}", downstream_trace_id);

        assert_eq!(downstream_trace_id, trace_id);
        println!("\n✓ Trace context successfully propagated!");

        // Example: Custom trace_id format
        println!("\n6. Custom Trace ID Examples:");

        let custom_trace_ids = vec![
            "abc-123-xyz",
            "my-custom-trace-2024",
            "550e8400-e29b-41d4-a716-446655440000",
        ];

        for custom_id in custom_trace_ids {
            let traceparent = http::trace_id_to_traceparent(custom_id, None, 0x01);
            println!("   {} → {}", custom_id, traceparent);
        }

        println!("\n=== Integration Summary ===");
        println!("✓ OTel traceparent parsing");
        println!("✓ LNMP envelope trace_id integration");
        println!("✓ W3C Trace Context generation");
        println!("✓ Distributed trace propagation");
    }

    #[cfg(not(feature = "http"))]
    {
        println!("HTTP feature not enabled. Run with: cargo run --example otel_integration --features http");
    }

    Ok(())
}
