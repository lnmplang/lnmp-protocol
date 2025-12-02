//! Example: Context filtering and ranking
//!
//! Demonstrates advanced filtering and ranking capabilities of the
//! ContextPrioritizer for refining LLM contexts.

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_sfe::{ContextPrioritizer, ContextScorer, RiskLevel, ScoringWeights};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn create_record(id: i64, content: &str) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(id),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String(content.to_string()),
    });
    record
}

fn main() {
    println!("=== Context Filtering & Ranking Example ===\n");

    let now = current_timestamp_ms();
    // Create a diverse set of contexts
    let inputs = vec![
        // 1. High quality: Fresh, trusted, important
        (1, "Critical system alert", now, "monitor-sys", 255),
        // 2. Medium quality: Recent, normal source
        (2, "User activity log", now - 3_600_000, "auth-service", 128),
        // 3. Low quality: Old, unknown source
        (
            3,
            "Archived debug info",
            now - 86_400_000 * 7,
            "legacy-app",
            50,
        ),
        // 4. Suspicious: Fresh but bad source
        (4, "External bot traffic", now, "malicious-ip", 100),
        // 5. High confidence: Reference data
        (5, "System configuration", now - 1000, "config-service", 200),
    ];

    let mut contexts = Vec::new();

    // Configure scorer with some rules
    let config = lnmp_sfe::ContextScorerConfig::new()
        .add_trusted_source("monitor-sys".to_string())
        .add_trusted_source("config-service".to_string())
        .add_suspicious_source("malicious-ip".to_string());

    let scorer = ContextScorer::with_config(config);

    println!("Generating contexts...");
    for (id, content, ts, src, importance) in inputs {
        let record = create_record(id, content);
        // We can manually set importance on the profile if we want,
        // but here we'll simulate it by having the scorer use default or dictionary.
        // For this example, we'll just let the scorer do its thing and then override
        // importance to match our scenario for demonstration purposes.

        let envelope = EnvelopeBuilder::new(record.clone())
            .timestamp(ts)
            .source(src)
            .build();

        let mut profile = scorer.score_envelope(&envelope, now);
        profile.importance = importance; // Manual override for demo

        println!(
            "  ID {}: {} (Score: {:.4})",
            id,
            content,
            profile.composite_score()
        );
        contexts.push((envelope, profile));
    }

    // 1. Filter by Threshold
    println!("\n🔍 Filter: Minimum Score >= 0.6");
    let high_score = ContextPrioritizer::filter_by_threshold(contexts.clone(), 0.6);
    for (env, prof) in &high_score {
        println!(
            "  ✅ {} (Score: {:.4})",
            env.metadata.source.as_ref().unwrap(),
            prof.composite_score()
        );
    }

    // 2. Filter by Risk
    println!("\n🛡️ Filter: Low Risk Only");
    let safe_only = ContextPrioritizer::filter_by_risk(contexts.clone(), RiskLevel::Low);
    for (env, prof) in &safe_only {
        println!(
            "  ✅ {} (Risk: {:?})",
            env.metadata.source.as_ref().unwrap(),
            prof.risk_level
        );
    }

    // 3. Filter by Freshness
    println!("\n⏰ Filter: Freshness >= 0.8 (Recent)");
    let fresh_only = ContextPrioritizer::filter_by_freshness(contexts.clone(), 0.8);
    for (env, prof) in &fresh_only {
        println!(
            "  ✅ {} (Freshness: {:.4})",
            env.metadata.source.as_ref().unwrap(),
            prof.freshness_score
        );
    }

    // 4. Complex Selection: Top 2 weighted for Importance
    println!("\n⚖️ Selection: Top 2 by Importance (Weight 0.8)");
    let imp_weights = ScoringWeights::new(0.1, 0.8, 0.1);
    let top_important = ContextPrioritizer::select_top_k(contexts.clone(), 2, imp_weights);

    for (env, prof) in &top_important {
        println!(
            "  ✅ {} (Importance: {})",
            env.metadata.source.as_ref().unwrap(),
            prof.importance
        );
    }

    // 5. Statistics
    println!("\n📊 Context Statistics");
    let stats = ContextPrioritizer::compute_stats(&contexts);
    println!("  Total: {}", stats.count);
    println!(
        "  Avg Score: {:.4}",
        contexts
            .iter()
            .map(|(_, p)| p.composite_score())
            .sum::<f64>()
            / stats.count as f64
    );
    println!(
        "  Risk Profile: {} Low, {} Medium, {} High, {} Critical",
        stats.risk_low, stats.risk_medium, stats.risk_high, stats.risk_critical
    );
}
