//! Content-Aware Routing Example
//!
//! Demonstrates zero-copy content-based routing decisions using field inspection.

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_net::{ContentAwarePolicy, ContentRule, FieldCondition, MessageKind, RoutingDecision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Content-Aware Routing Demo ===\n");

    // Create policy with content rules
    let policy = ContentAwarePolicy::new()
        .with_threshold(0.7)
        .with_rule(ContentRule::critical_status_to_llm(50)) // F50: status
        .with_rule(ContentRule::drop_spam(24)) // F24: tags
        .with_rule(ContentRule::new(
            52, // F52: error_code
            FieldCondition::IntGreaterThan(5000),
            RoutingDecision::SendToLLM,
        ));

    println!("📋 Routing Policy:");
    println!("   - Rule 1: F50=critical → Send to LLM");
    println!("   - Rule 2: F24 contains 'spam' → Drop");
    println!("   - Rule 3: F52 > 5000 → Send to LLM");
    println!("   - Fallback: Priority threshold (0.7)\n");

    // Test scenarios
    let scenarios = vec![
        (
            "Critical Status",
            create_message_with_status("critical", 100),
        ),
        ("Normal Status", create_message_with_status("normal", 100)),
        (
            "Spam Message",
            create_message_with_tags(vec!["spam", "junk"]),
        ),
        ("High Error Code", create_message_with_error_code(5001, 100)),
        ("Low Error Code", create_message_with_error_code(404, 100)),
        ("High Priority Fallback", create_simple_message(250)),
        ("Low Priority Fallback", create_simple_message(50)),
    ];

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    for (name, record) in scenarios {
        let bytes = encoder.encode(&record)?;
        let view = decoder.decode_view(&bytes)?;

        let decision = policy.decide_content_aware(
            MessageKind::Event,
            100, // Default priority
            None,
            &view,
            1000,
        )?;

        println!("📨 Scenario: {}", name);
        println!("   Decision: {:?}", decision);
        println!();
    }

    // Performance benchmark
    benchmark_content_routing(&policy)?;

    Ok(())
}

fn create_message_with_status(status: &str, priority: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String(status.to_string()),
    });
    record.add_field(LnmpField {
        fid: 32,
        value: LnmpValue::Int(priority),
    });
    record
}

fn create_message_with_tags(tags: Vec<&str>) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    let tags_str: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
    record.add_field(LnmpField {
        fid: 24,
        value: LnmpValue::StringArray(tags_str),
    });
    record
}

fn create_message_with_error_code(code: i64, priority: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 52,
        value: LnmpValue::Int(code),
    });
    record.add_field(LnmpField {
        fid: 32,
        value: LnmpValue::Int(priority),
    });
    record
}

fn create_simple_message(priority: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 32,
        value: LnmpValue::Int(priority),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(42),
    });
    record
}

fn benchmark_content_routing(
    policy: &ContentAwarePolicy,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    let record = create_message_with_status("critical", 100);
    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();
    let bytes = encoder.encode(&record)?;

    let iterations = 100_000;

    println!("=== Performance Benchmark ({} iterations) ===", iterations);

    // Zero-copy content-aware routing
    let start = Instant::now();
    for _ in 0..iterations {
        let view = decoder.decode_view(&bytes)?;
        let _ = policy.decide_content_aware(MessageKind::Event, 100, None, &view, 1000)?;
    }
    let zerocopy_time = start.elapsed();

    // Standard decode approach (simulated)
    let start = Instant::now();
    for _ in 0..iterations {
        let record = decoder.decode(&bytes)?;
        let _ = decide_with_owned_record(&record)?;
    }
    let standard_time = start.elapsed();

    println!(
        "Standard decode + routing: {:?} ({:.2} μs/iter)",
        standard_time,
        standard_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Zero-copy view + routing:  {:?} ({:.2} μs/iter)",
        zerocopy_time,
        zerocopy_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Speedup: {:.2}x faster",
        standard_time.as_secs_f64() / zerocopy_time.as_secs_f64()
    );

    Ok(())
}

fn decide_with_owned_record(
    record: &LnmpRecord,
) -> Result<RoutingDecision, Box<dyn std::error::Error>> {
    // Simulate content-based routing with owned record
    if let Some(field) = record.get_field(50) {
        if let LnmpValue::String(status) = &field.value {
            if status == "critical" {
                return Ok(RoutingDecision::SendToLLM);
            }
        }
    }
    Ok(RoutingDecision::ProcessLocally)
}
