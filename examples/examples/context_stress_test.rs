//! Example: Context Profiling Stress Test
//!
//! Simulates a high-volume scenario to measure throughput and latency
//! of the context profiling system.

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_sfe::{ContextPrioritizer, ContextScorer, ScoringWeights};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BATCH_SIZE: usize = 10_000;
const NUM_BATCHES: usize = 10;
const TOTAL_RECORDS: usize = BATCH_SIZE * NUM_BATCHES;

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn create_record(id: i64) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(id),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("Stress test content payload".to_string()),
    });
    record
}

fn main() {
    println!("=== Context Profiling Stress Test ===\n");
    println!("Configuration:");
    println!("  Total Records: {}", TOTAL_RECORDS);
    println!("  Batch Size:    {}", BATCH_SIZE);
    println!("  Batches:       {}", NUM_BATCHES);
    println!("----------------------------------------");

    let scorer = ContextScorer::new();
    let now = current_timestamp_ms();
    let mut total_scoring_time = Duration::new(0, 0);
    let total_ranking_time;

    let mut all_contexts = Vec::with_capacity(TOTAL_RECORDS);

    println!("\nPhase 1: Scoring (Envelope + Record)");
    let start_total = Instant::now();

    for b in 0..NUM_BATCHES {
        let batch_start = Instant::now();
        for i in 0..BATCH_SIZE {
            let id = (b * BATCH_SIZE + i) as i64;
            let record = create_record(id);
            let envelope = EnvelopeBuilder::new(record)
                .timestamp(now - (id as u64 % 86400000)) // Random freshness within 24h
                .source("stress-test-source")
                .build();

            let profile = scorer.score_envelope(&envelope, now);
            all_contexts.push((envelope, profile));
        }
        let batch_duration = batch_start.elapsed();
        total_scoring_time += batch_duration;

        if (b + 1) % 2 == 0 {
            print!(".");
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }
    println!("\nScoring Complete.");

    println!("\nPhase 2: Prioritization (Top-K Selection)");
    let rank_start = Instant::now();

    // Select top 100 from the massive list
    let weights = ScoringWeights::default();
    let top_k = ContextPrioritizer::select_top_k(all_contexts.clone(), 100, weights);

    total_ranking_time = rank_start.elapsed();
    let total_duration = start_total.elapsed();

    println!("\n----------------------------------------");
    println!("Results:");
    println!("----------------------------------------");

    // Scoring Metrics
    let scoring_micros = total_scoring_time.as_micros() as f64;
    let scoring_per_op_ns = (scoring_micros * 1000.0) / TOTAL_RECORDS as f64;
    let scoring_ops_sec = TOTAL_RECORDS as f64 / total_scoring_time.as_secs_f64();

    println!("Scoring Performance:");
    println!("  Total Time:    {:.2?}", total_scoring_time);
    println!("  Throughput:    {:.2} ops/sec", scoring_ops_sec);
    println!("  Latency:       {:.2} ns/op", scoring_per_op_ns);

    // Ranking Metrics
    println!(
        "\nRanking Performance (Select Top-100 from {}):",
        TOTAL_RECORDS
    );
    println!("  Total Time:    {:.2?}", total_ranking_time);

    // Overall
    println!("\nTotal Execution Time: {:.2?}", total_duration);
    println!("----------------------------------------");

    println!("\nTop 3 Selected Contexts:");
    for (i, (_env, prof)) in top_k.iter().take(3).enumerate() {
        println!(
            "  {}. Score: {:.4} (Freshness: {:.4})",
            i + 1,
            prof.composite_score(),
            prof.freshness_score
        );
    }
}
