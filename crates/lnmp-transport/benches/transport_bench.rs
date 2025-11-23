use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use lnmp_transport::{http, kafka};

fn create_bench_envelope() -> LnmpEnvelope {
    let mut labels = std::collections::HashMap::new();
    labels.insert("env".to_string(), "production".to_string());
    labels.insert("region".to_string(), "us-east-1".to_string());

    let meta = EnvelopeMetadata {
        timestamp: Some(1627849200000),
        source: Some("bench-source".to_string()),
        trace_id: Some("bench-trace-id-123456789".to_string()),
        sequence: Some(987654321),
        labels,
    };

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });

    LnmpEnvelope {
        metadata: meta,
        record,
    }
}

fn bench_http_mapping(c: &mut Criterion) {
    let env = create_bench_envelope();
    c.bench_function("http_envelope_to_headers", |b| {
        b.iter(|| http::envelope_to_headers(black_box(&env)).unwrap())
    });
}

fn bench_kafka_mapping(c: &mut Criterion) {
    let env = create_bench_envelope();
    c.bench_function("kafka_envelope_to_headers", |b| {
        b.iter(|| kafka::envelope_to_kafka_headers(black_box(&env)).unwrap())
    });
}

criterion_group!(benches, bench_http_mapping, bench_kafka_mapping);
criterion_main!(benches);
