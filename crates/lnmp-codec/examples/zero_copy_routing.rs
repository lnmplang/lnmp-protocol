//! High-performance routing using zero-copy decoding.
//!
//! This example demonstrates how to route messages based on field values
//! without allocating memory for the entire record.

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue, LnmpValueView};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: Create sample messages
    let critical_msg = create_message("critical", 255);
    let normal_msg = create_message("normal", 100);

    let encoder = BinaryEncoder::new();
    let critical_bytes = encoder.encode(&critical_msg)?;
    let normal_bytes = encoder.encode(&normal_msg)?;

    // Routing with zero-copy
    let decoder = BinaryDecoder::new();

    println!("=== Zero-Copy Routing Demo ===\n");

    // Route critical message
    let view = decoder.decode_view(&critical_bytes)?;
    let decision = route_message_zerocopy(&view)?;
    println!("Critical message → {}", decision);

    // Route normal message
    let view = decoder.decode_view(&normal_bytes)?;
    let decision = route_message_zerocopy(&view)?;
    println!("Normal message → {}", decision);

    // Performance comparison
    benchmark_routing(&critical_bytes, &decoder)?;

    Ok(())
}

fn route_message_zerocopy(
    view: &lnmp_core::LnmpRecordView,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    // Check status field (F50) without allocation
    if let Some(field) = view.get_field(50) {
        match &field.value {
            LnmpValueView::String(status) => {
                if *status == "critical" {
                    return Ok("ROUTE_TO_LLM");
                }
            }
            _ => {}
        }
    }

    // Check priority field (F32)
    if let Some(field) = view.get_field(32) {
        match &field.value {
            LnmpValueView::Int(priority) if *priority > 200 => {
                return Ok("ROUTE_TO_LLM");
            }
            _ => {}
        }
    }

    Ok("ROUTE_LOCALLY")
}

fn create_message(status: &str, priority: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50, // F50: status
        value: LnmpValue::String(status.to_string()),
    });
    record.add_field(LnmpField {
        fid: 32, // F32: priority
        value: LnmpValue::Int(priority),
    });
    record
}

fn benchmark_routing(
    bytes: &[u8],
    decoder: &BinaryDecoder,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    let iterations = 100_000;

    // Benchmark zero-copy
    let start = Instant::now();
    for _ in 0..iterations {
        let view = decoder.decode_view(bytes)?;
        let _ = route_message_zerocopy(&view)?;
    }
    let zerocopy_time = start.elapsed();

    // Benchmark standard decode
    let start = Instant::now();
    for _ in 0..iterations {
        let record = decoder.decode(bytes)?;
        let _ = route_message_standard(&record)?;
    }
    let standard_time = start.elapsed();

    println!("\n=== Performance ({} iterations) ===", iterations);
    println!(
        "Standard decode: {:?} ({:.2} μs/iter)",
        standard_time,
        standard_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Zero-copy view:  {:?} ({:.2} μs/iter)",
        zerocopy_time,
        zerocopy_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Speedup: {:.2}x faster",
        standard_time.as_secs_f64() / zerocopy_time.as_secs_f64()
    );

    Ok(())
}

fn route_message_standard(record: &LnmpRecord) -> Result<&'static str, Box<dyn std::error::Error>> {
    if let Some(field) = record.get_field(50) {
        match &field.value {
            LnmpValue::String(status) if status == "critical" => {
                return Ok("ROUTE_TO_LLM");
            }
            _ => {}
        }
    }

    if let Some(field) = record.get_field(32) {
        match &field.value {
            LnmpValue::Int(priority) if *priority > 200 => {
                return Ok("ROUTE_TO_LLM");
            }
            _ => {}
        }
    }

    Ok("ROUTE_LOCALLY")
}
