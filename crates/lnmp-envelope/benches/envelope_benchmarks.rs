//! Envelope performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lnmp_envelope::{
    binary_codec::{TlvDecoder, TlvEncoder},
    text_codec::{TextDecoder, TextEncoder},
    EnvelopeBuilder, EnvelopeMetadata, LnmpField, LnmpRecord, LnmpValue,
};

fn create_sample_metadata() -> EnvelopeMetadata {
    let mut metadata = EnvelopeMetadata::new();
    metadata.timestamp = Some(1732373147000);
    metadata.source = Some("auth-service".to_string());
    metadata.trace_id = Some("abc-123-xyz-trace-id".to_string());
    metadata.sequence = Some(42);
    metadata
}

fn create_sample_record() -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
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

fn bench_binary_encoding(c: &mut Criterion) {
    let metadata = create_sample_metadata();

    c.bench_function("binary_tlv_encode", |b| {
        b.iter(|| {
            let _encoded = TlvEncoder::encode(black_box(&metadata));
        });
    });
}

fn bench_binary_decoding(c: &mut Criterion) {
    let metadata = create_sample_metadata();
    let encoded = TlvEncoder::encode(&metadata).unwrap();

    c.bench_function("binary_tlv_decode", |b| {
        b.iter(|| {
            let _decoded = TlvDecoder::decode(black_box(&encoded));
        });
    });
}

fn bench_binary_roundtrip(c: &mut Criterion) {
    let metadata = create_sample_metadata();

    c.bench_function("binary_tlv_roundtrip", |b| {
        b.iter(|| {
            let encoded = TlvEncoder::encode(black_box(&metadata)).unwrap();
            let _decoded = TlvDecoder::decode(&encoded).unwrap();
        });
    });
}

fn bench_text_encoding(c: &mut Criterion) {
    let metadata = create_sample_metadata();

    c.bench_function("text_header_encode", |b| {
        b.iter(|| {
            let _encoded = TextEncoder::encode(black_box(&metadata));
        });
    });
}

fn bench_text_decoding(c: &mut Criterion) {
    let metadata = create_sample_metadata();
    let encoded = TextEncoder::encode(&metadata).unwrap();

    c.bench_function("text_header_decode", |b| {
        b.iter(|| {
            let _decoded = TextDecoder::decode(black_box(&encoded));
        });
    });
}

fn bench_text_roundtrip(c: &mut Criterion) {
    let metadata = create_sample_metadata();

    c.bench_function("text_header_roundtrip", |b| {
        b.iter(|| {
            let encoded = TextEncoder::encode(black_box(&metadata)).unwrap();
            let _decoded = TextDecoder::decode(&encoded).unwrap();
        });
    });
}

fn bench_envelope_builder(c: &mut Criterion) {
    let record = create_sample_record();

    c.bench_function("envelope_builder", |b| {
        b.iter(|| {
            let _envelope = EnvelopeBuilder::new(black_box(record.clone()))
                .timestamp(1732373147000)
                .source("auth-service")
                .trace_id("abc-123-xyz")
                .sequence(42)
                .build();
        });
    });
}

fn bench_metadata_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_field_count");

    for field_count in [1, 2, 3, 4].iter() {
        let mut metadata = EnvelopeMetadata::new();

        if *field_count >= 1 {
            metadata.timestamp = Some(1732373147000);
        }
        if *field_count >= 2 {
            metadata.source = Some("auth-service".to_string());
        }
        if *field_count >= 3 {
            metadata.trace_id = Some("abc-123-xyz".to_string());
        }
        if *field_count >= 4 {
            metadata.sequence = Some(42);
        }

        group.bench_with_input(
            BenchmarkId::new("binary_encode", field_count),
            &metadata,
            |b, meta| {
                b.iter(|| {
                    let _encoded = TlvEncoder::encode(black_box(meta));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("text_encode", field_count),
            &metadata,
            |b, meta| {
                b.iter(|| {
                    let _encoded = TextEncoder::encode(black_box(meta));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_binary_encoding,
    bench_binary_decoding,
    bench_binary_roundtrip,
    bench_text_encoding,
    bench_text_decoding,
    bench_text_roundtrip,
    bench_envelope_builder,
    bench_metadata_size
);

criterion_main!(benches);
