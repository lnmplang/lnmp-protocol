//! Example: LLM freshness scoring using envelope timestamps

use lnmp_envelope::{EnvelopeBuilder, LnmpEnvelope, LnmpField, LnmpRecord, LnmpValue};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("=== LNMP Envelope LLM Freshness Scoring Example ===\n");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Create records with different timestamps
    let records = vec![
        create_envelope("Recent data", now - 3_600_000), // 1 hour ago
        create_envelope("Medium data", now - 86_400_000), // 1 day ago
        create_envelope("Old data", now - 604_800_000),  // 1 week ago
        create_envelope("Ancient data", now - 2_592_000_000), // 30 days ago
    ];

    println!("Scoring {} records for freshness:\n", records.len());

    let mut scored: Vec<_> = records
        .iter()
        .map(|env| {
            let score = score_freshness(env, now);
            (env, score)
        })
        .collect();

    // Sort by freshness (highest first)
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (env, score) in &scored {
        if let Some(ref source) = env.metadata.source {
            if let Some(ts) = env.metadata.timestamp {
                let age_hours = (now - ts) as f64 / 3_600_000.0;
                println!(
                    "  {} - Age: {:.1}h - Score: {:.4}",
                    source, age_hours, score
                );
            }
        }
    }

    println!("\n--- LLM Context Ranking ---\n");
    println!("When constructing LLM context, use this order:");
    for (i, (env, score)) in scored.iter().enumerate() {
        if let Some(ref source) = env.metadata.source {
            println!("  {}. {} (score: {:.4})", i + 1, source, score);
        }
    }

    println!("\n--- Threshold Filtering ---\n");
    let threshold = 0.5;
    let filtered: Vec<_> = scored
        .iter()
        .filter(|(_, score)| *score >= threshold)
        .collect();

    println!("Records passing threshold ({}):", threshold);
    for (env, score) in filtered {
        if let Some(ref source) = env.metadata.source {
            println!("  ✅ {} (score: {:.4})", source, score);
        }
    }

    println!("\n✅ LLM freshness scoring complete!");

    println!("\n💡 Use Cases:");
    println!("   - Prioritize recent data in RAG systems");
    println!("   - Filter out stale information");
    println!("   - Weight context by age in embeddings");
    println!("   - Time-decay for news/events data");
}

/// Score freshness using exponential decay
///
/// Formula: e^(-age_hours / decay_constant)
/// - age_hours: hours since record timestamp
/// - decay_constant: 24.0 (half-life of ~17 hours)
///
/// Score ranges:
/// - 1.0: Current (0 hours old)
/// - 0.61: 1 day old
/// - 0.37: 2 days old
/// - 0.22: 3 days old
/// - 0.01: 1 week old
fn score_freshness(envelope: &LnmpEnvelope, now: u64) -> f64 {
    if let Some(ts) = envelope.metadata.timestamp {
        let age_ms = now.saturating_sub(ts);
        let age_hours = age_ms as f64 / 3_600_000.0;

        // Exponential decay: e^(-t/τ) where τ=24 hours
        (-age_hours / 24.0).exp()
    } else {
        // No timestamp = medium priority
        0.5
    }
}

/// Score with custom decay rate
#[allow(dead_code)]
fn score_freshness_custom(envelope: &LnmpEnvelope, now: u64, decay_hours: f64) -> f64 {
    if let Some(ts) = envelope.metadata.timestamp {
        let age_ms = now.saturating_sub(ts);
        let age_hours = age_ms as f64 / 3_600_000.0;
        (-age_hours / decay_hours).exp()
    } else {
        0.5
    }
}

fn create_envelope(source: &str, timestamp: u64) -> LnmpEnvelope {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String(format!("Content from {}", source)),
    });

    EnvelopeBuilder::new(record)
        .timestamp(timestamp)
        .source(source)
        .build()
}
