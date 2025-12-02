//! Example: Context Profiling and Scoring
//!
//! Demonstrates how to use the Semantic Fidelity Engine for context management.

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

fn main() {
    println!("=== LNMP Context Profiling Examples ===\n");

    // Example 1: Fresh data from trusted source
    println!("Example 1: High-Quality Context");
    let now = current_timestamp_ms();

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    let envelope = EnvelopeBuilder::new(record.clone())
        .timestamp(now - 3_600_000) // 1 hour old
        .source("auth-service")
        .build();

    let config = ContextScorerConfig::new().add_trusted_source("auth-service".to_string());
    let scorer = ContextScorer::with_config(config);

    let profile = scorer.score_envelope(&envelope, now);

    println!("  Freshness: {:.4} (1 hour old)", profile.freshness_score);
    println!("  Importance: {}", profile.importance);
    println!("  Risk: {:?}", profile.risk_level);
    println!("  Confidence: {:.2}", profile.confidence);
    println!("  Composite Score: {:.4}\n", profile.composite_score());

    // Example 2: Stale data
    println!("Example 2: Stale Context");
    let old_envelope = EnvelopeBuilder::new(record.clone())
        .timestamp(now - 604_800_000) // 1 week old
        .source("data-service")
        .build();

    let old_profile = scorer.score_envelope(&old_envelope, now);
    println!(
        "  Freshness: {:.4} (1 week old)",
        old_profile.freshness_score
    );
    println!("  ⚠️  Should consider refreshing or deprioritizing\n");

    // Example 3: Different decay rates
    println!("Example 3: Decay Rate Comparison");
    let test_timestamp = now - 86_400_000; // 1 day old
    let test_envelope = EnvelopeBuilder::new(record)
        .timestamp(test_timestamp)
        .source("test-service")
        .build();

    let fast_config = ContextScorerConfig::new().with_freshness_decay(12.0);
    let fast_scorer = ContextScorer::with_config(fast_config);

    let slow_config = ContextScorerConfig::new().with_freshness_decay(72.0);
    let slow_scorer = ContextScorer::with_config(slow_config);

    println!("  1 day old data:");
    println!(
        " Fast decay (12h): {:.4}",
        fast_scorer
            .score_envelope(&test_envelope, now)
            .freshness_score
    );
    println!(
        "    Default (24h):    {:.4}",
        scorer.score_envelope(&test_envelope, now).freshness_score
    );
    println!(
        "    Slow decay (72h): {:.4}\n",
        slow_scorer
            .score_envelope(&test_envelope, now)
            .freshness_score
    );

    println!("✅ Context profiling examples completed!");
    println!("\n💡 Use Cases:");
    println!("   - RAG system prioritization");
    println!("   - Cache invalidation decisions");
    println!("   - Data freshness monitoring");
    println!("   - Source trust evaluation");
}
