//! Example: RAG system context prioritization
//!
//! Demonstrates how to use context profiling to select the best
//! contexts for LLM prompts in a RAG (Retrieval-Augmented Generation) system.

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_sfe::{ContextPrioritizer, ContextScorer, ScoringWeights};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn create_document(id: i64, title: &str, content: &str) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(id),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String(title.to_string()),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::String(content.to_string()),
    });
    record
}

fn main() {
    println!("=== RAG Context Prioritization Example ===\n");

    let now = current_timestamp_ms();
    let scorer = ContextScorer::new();

    // Create a collection of documents with different freshness and sources
    let documents = [
        // Recent, official documentation
        (
            "Official API docs (fresh)",
            now - 3_600_000, // 1 hour ago
            "api-docs",
        ),
        // Week-old blog post
        (
            "Community blog post",
            now - 604_800_000, // 1 week ago
            "community-blog",
        ),
        // Fresh tutorial
        (
            "New tutorial",
            now - 7_200_000, // 2 hours ago
            "tutorials",
        ),
        // Month-old reference
        (
            "Old reference manual",
            now - 2_592_000_000, // 30 days ago
            "reference",
        ),
        // Very fresh news
        (
            "Latest release notes",
            now - 1_800_000, // 30 minutes ago
            "releases",
        ),
        // Fresh but from unknown source
        (
            "Stack Overflow answer",
            now - 5_400_000, // 1.5 hours ago
            "unknown-source",
        ),
    ];

    // Create envelopes and score them
    let mut contexts = Vec::new();
    println!("📄 Available Documents:");
    println!(
        "{:<30} {:>10} {:>15} {:>10}",
        "Title", "Age", "Source", "Score"
    );
    println!("{}", "-".repeat(70));

    for (i, (title, timestamp, source)) in documents.iter().enumerate() {
        let record = create_document((i + 1) as i64, title, "Sample content...");
        let envelope = EnvelopeBuilder::new(record)
            .timestamp(*timestamp)
            .source(*source)
            .build();

        let profile = scorer.score_envelope(&envelope, now);
        let age_hours = (now - timestamp) / 3_600_000;

        println!(
            "{:<30} {:>7}h ago {:>15} {:>10.4}",
            title,
            age_hours,
            source,
            profile.composite_score()
        );

        contexts.push((envelope, profile));
    }

    // Statistics for all contexts
    println!("\n📊 Overall Statistics:");
    let stats = ContextPrioritizer::compute_stats(&contexts);
    println!("  Total documents: {}", stats.count);
    println!("  Avg freshness: {:.4}", stats.avg_freshness);
    println!("  Avg confidence: {:.4}", stats.avg_confidence);
    println!(
        "  Risk distribution: Low={}, Medium={}, High={}, Critical={}",
        stats.risk_low, stats.risk_medium, stats.risk_high, stats.risk_critical
    );

    // Example 1: Select top 3 for LLM prompt (default weights)
    println!("\n🎯 Example 1: Top 3 Documents (Default Weights)");
    println!("   Freshness: 30%, Importance: 40%, Confidence: 30%");

    let top_3 = ContextPrioritizer::select_top_k(contexts.clone(), 3, ScoringWeights::default());

    for (envelope, profile) in &top_3 {
        if let Some(ref source) = envelope.metadata.source {
            println!("   ✅ {} - Score: {:.4}", source, profile.composite_score());
        }
    }

    // Example 2: Prioritize freshness (RAG for news/events)
    println!("\n📰 Example 2: Top 3 for News/Events (80% Freshness)");

    let weights_fresh = ScoringWeights::new(0.8, 0.1, 0.1);
    let top_3_fresh = ContextPrioritizer::select_top_k(contexts.clone(), 3, weights_fresh);

    for (envelope, profile) in &top_3_fresh {
        if let Some(ref source) = envelope.metadata.source {
            println!(
                "   ✅ {} - Freshness: {:.4}",
                source, profile.freshness_score
            );
        }
    }

    // Example 3: Prioritize confidence (RAG for factual data)
    println!("\n📚 Example 3: Top 3 for Factual Data (70% Confidence)");

    let weights_confidence = ScoringWeights::new(0.1, 0.2, 0.7);
    let top_3_conf = ContextPrioritizer::select_top_k(contexts.clone(), 3, weights_confidence);

    for (envelope, profile) in &top_3_conf {
        if let Some(ref source) = envelope.metadata.source {
            println!("   ✅ {} - Confidence: {:.4}", source, profile.confidence);
        }
    }

    // Example 4: Filter by threshold
    println!("\n🔍 Example 4: Filter by Minimum Score (>= 0.6)");

    let filtered = ContextPrioritizer::filter_by_threshold(contexts.clone(), 0.6);

    println!(
        "   {} out of {} documents pass threshold",
        filtered.len(),
        contexts.len()
    );
    for (envelope, profile) in &filtered {
        if let Some(ref source) = envelope.metadata.source {
            println!("   ✅ {} - Score: {:.4}", source, profile.composite_score());
        }
    }

    // Example 5: Filter by freshness
    println!("\n⏰ Example 5: Filter by Freshness (>= 0.9 = ~3 hours)");

    let very_fresh = ContextPrioritizer::filter_by_freshness(contexts.clone(), 0.9);

    println!("   {} very fresh documents", very_fresh.len());
    for (envelope, profile) in &very_fresh {
        if let Some(ref source) = envelope.metadata.source {
            let age_ms = now - envelope.metadata.timestamp.unwrap();
            let age_hours = age_ms / 3_600_000;
            println!(
                "   ✅ {} - {}h ago (freshness: {:.4})",
                source, age_hours, profile.freshness_score
            );
        }
    }

    // Example 6: Ranked list with scores
    println!("\n🏆 Example 6: Complete Ranking");

    let ranked = ContextPrioritizer::rank_for_llm(contexts.clone(), ScoringWeights::default());

    for (i, (envelope, profile, score)) in ranked.iter().enumerate() {
        if let Some(ref source) = envelope.metadata.source {
            println!(
                "   {}. {} - Score: {:.4} (F:{:.2} I:{} C:{:.2})",
                i + 1,
                source,
                score,
                profile.freshness_score,
                profile.importance,
                profile.confidence
            );
        }
    }

    println!("\n✅ RAG prioritization demonstration complete!");
    println!("\n💡 Key Insights:");
    println!("   - Fresh content scores higher for time-sensitive queries");
    println!("   - Adjust weights based on query type (news vs facts)");
    println!("   - Filtering removes low-quality or stale contexts");
    println!("   - Top-K selection controls prompt token budget");
}
