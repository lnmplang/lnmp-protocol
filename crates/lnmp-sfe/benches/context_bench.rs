//! Benchmarks for context profiling performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_sfe::{ContextPrioritizer, ContextScorer, ContextScorerConfig, ScoringWeights};
use std::sync::Arc;

fn create_sample_envelope(timestamp: u64) -> lnmp_envelope::LnmpEnvelope {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    EnvelopeBuilder::new(record)
        .timestamp(timestamp)
        .source("test-service")
        .trace_id("trace-123")
        .build()
}

fn bench_score_envelope(c: &mut Criterion) {
    let scorer = ContextScorer::new();
    let now = 1732373147000u64;
    let envelope = create_sample_envelope(now - 3_600_000);

    c.bench_function("score_envelope", |b| {
        b.iter(|| scorer.score_envelope(black_box(&envelope), black_box(now)))
    });
}

fn bench_composite_score(c: &mut Criterion) {
    let scorer = ContextScorer::new();
    let now = 1732373147000u64;
    let envelope = create_sample_envelope(now - 3_600_000);
    let profile = scorer.score_envelope(&envelope, now);

    c.bench_function("composite_score", |b| {
        b.iter(|| black_box(&profile).composite_score())
    });
}

fn bench_filter_by_threshold(c: &mut Criterion) {
    let scorer = ContextScorer::new();
    let now = 1732373147000u64;
    let mut contexts = Vec::new();

    for i in 0..100 {
        let envelope = create_sample_envelope(now - (i * 3_600_000));
        let profile = scorer.score_envelope(&envelope, now);
        contexts.push((envelope, profile));
    }

    c.bench_function("filter_by_threshold_100", |b| {
        b.iter(|| {
            ContextPrioritizer::filter_by_threshold(black_box(contexts.clone()), black_box(0.6))
        })
    });
}

fn bench_select_top_k(c: &mut Criterion) {
    let scorer = ContextScorer::new();
    let now = 1732373147000u64;
    let mut contexts = Vec::new();

    for i in 0..100 {
        let envelope = create_sample_envelope(now - (i * 3_600_000));
        let profile = scorer.score_envelope(&envelope, now);
        contexts.push((envelope, profile));
    }

    c.bench_function("select_top_k_5_from_100", |b| {
        b.iter(|| {
            ContextPrioritizer::select_top_k(
                black_box(contexts.clone()),
                black_box(5),
                black_box(ScoringWeights::default()),
            )
        })
    });
}

fn bench_with_dictionary(c: &mut Criterion) {
    use lnmp_sfe::SemanticDictionary;

    let mut dict = SemanticDictionary::new();
    dict.add_field_name(12, "user_id".to_string());
    dict.add_importance(12, 200);
    dict.add_field_name(7, "is_active".to_string());
    dict.add_importance(7, 150);

    let scorer = ContextScorer::new().with_dictionary(Arc::new(dict));

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    c.bench_function("score_record_with_dictionary", |b| {
        b.iter(|| scorer.score_record(black_box(&record)))
    });
}

fn bench_compute_stats(c: &mut Criterion) {
    let scorer = ContextScorer::new();
    let now = 1732373147000u64;
    let mut contexts = Vec::new();

    for i in 0..100 {
        let envelope = create_sample_envelope(now - (i * 3_600_000));
        let profile = scorer.score_envelope(&envelope, now);
        contexts.push((envelope, profile));
    }

    c.bench_function("compute_stats_100", |b| {
        b.iter(|| ContextPrioritizer::compute_stats(black_box(&contexts)))
    });
}

criterion_group!(
    benches,
    bench_score_envelope,
    bench_composite_score,
    bench_filter_by_threshold,
    bench_select_top_k,
    bench_with_dictionary,
    bench_compute_stats
);
criterion_main!(benches);
