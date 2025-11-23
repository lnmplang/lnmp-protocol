//! Example: Basic context scoring usage
//!
//! Demonstrates how to use the context profiling system to score
//! LNMP records based on freshness, importance, risk, and confidence.

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_sfe::{ContextScorer, ContextScorerConfig};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn create_sample_record(user_id: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(user_id),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("Alice".to_string()),
    });
    record
}

fn main() {
    println!("=== LNMP Context Scoring Example ===\n");

    let now = current_timestamp_ms();

    // Configure scorer with trusted and suspicious sources
    let config = ContextScorerConfig::new()
        .with_freshness_decay(24.0) // 24-hour decay
        .add_trusted_source("auth-service".to_string())
        .add_suspicious_source("malicious".to_string());

    let scorer = ContextScorer::with_config(config);

    // Example 1: Fresh, trusted source
    println!("Example 1: Fresh data from trusted source");
    let record1 = create_sample_record(14532);
    let envelope1 = EnvelopeBuilder::new(record1)
        .timestamp(now - 3_600_000) // 1 hour ago
        .source("auth-service")
        .trace_id("trace-001")
        .build();

    let profile1 = scorer.score_envelope(&envelope1, now);
    println!("  Freshness: {:.4}", profile1.freshness_score);
    println!("  Importance: {}", profile1.importance);
    println!("  Risk: {:?}", profile1.risk_level);
    println!("  Confidence: {:.2}", profile1.confidence);
    println!("  Composite Score: {:.4}\n", profile1.composite_score());

    // Example 2: Old data from unknown source
    println!("Example 2: Old data from unknown source");
    let record2 = create_sample_record(14533);
    let envelope2 = EnvelopeBuilder::new(record2)
        .timestamp(now - 604_800_000) // 1 week ago
        .source("unknown-service")
        .build();

    let profile2 = scorer.score_envelope(&envelope2, now);
    println!("  Freshness: {:.4}", profile2.freshness_score);
    println!("  Importance: {}", profile2.importance);
    println!("  Risk: {:?}", profile2.risk_level);
    println!("  Confidence: {:.2}", profile2.confidence);
    println!("  Composite Score: {:.4}\n", profile2.composite_score());

    // Example 3: Fresh data from suspicious source
    println!("Example 3: Fresh data from suspicious source");
    let record3 = create_sample_record(14534);
    let envelope3 = EnvelopeBuilder::new(record3)
        .timestamp(now)
        .source("malicious-bot")
        .build();

    let profile3 = scorer.score_envelope(&envelope3, now);
    println!("  Freshness: {:.4}", profile3.freshness_score);
    println!("  Importance: {}", profile3.importance);
    println!("  Risk: {:?}", profile3.risk_level);
    println!("  Confidence: {:.2}", profile3.confidence);
    println!("  Composite Score: {:.4}\n", profile3.composite_score());

    // Example 4: Data without timestamp
    println!("Example 4: Data without timestamp");
    let record4 = create_sample_record(14535);
    let envelope4 = EnvelopeBuilder::new(record4).source("data-service").build();

    let profile4 = scorer.score_envelope(&envelope4, now);
    println!(
        "  Freshness: {:.4} (default - no timestamp)",
        profile4.freshness_score
    );
    println!("  Importance: {}", profile4.importance);
    println!("  Risk: {:?}", profile4.risk_level);
    println!("  Confidence: {:.2}", profile4.confidence);
    println!("  Composite Score: {:.4}\n", profile4.composite_score());

    // Example 5: Different freshness decay rates
    println!("Example 5: Comparing decay rates");
    let record5 = create_sample_record(14536);
    let envelope5 = EnvelopeBuilder::new(record5)
        .timestamp(now - 86_400_000) // 1 day ago
        .source("test-service")
        .build();

    let config_fast = ContextScorerConfig::new().with_freshness_decay(12.0); // Fast decay
    let scorer_fast = ContextScorer::with_config(config_fast);

    let config_slow = ContextScorerConfig::new().with_freshness_decay(48.0); // Slow decay
    let scorer_slow = ContextScorer::with_config(config_slow);

    let profile_default = scorer.score_envelope(&envelope5, now);
    let profile_fast = scorer_fast.score_envelope(&envelope5, now);
    let profile_slow = scorer_slow.score_envelope(&envelope5, now);

    println!("  1 day old data:");
    println!(
        "    Default (24h decay): {:.4}",
        profile_default.freshness_score
    );
    println!(
        "    Fast (12h decay):    {:.4}",
        profile_fast.freshness_score
    );
    println!(
        "    Slow (48h decay):    {:.4}\n",
        profile_slow.freshness_score
    );

    println!("✅ Context scoring demonstration complete!");
    println!("\n💡 Use Cases:");
    println!("   - Prioritize recent data in RAG systems");
    println!("   - Filter out suspicious sources");
    println!("   - Boost confidence for trusted services");
    println!("   - Adjust decay rates based on data type (news vs reference)");
}
