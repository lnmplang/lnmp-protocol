use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_embedding::{EmbeddingType, Vector};

fn bench_zero_copy_suite(c: &mut Criterion) {
    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    let mut group = c.benchmark_group("decoding_performance");

    // Test Case 1: Small Record (Integers/Bools only) - Zero Copy advantage should be minimal
    {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Float(123.456),
        });

        let bytes = encoder.encode(&record).unwrap();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function("small_rec_std", |b| {
            b.iter(|| decoder.decode(&bytes).unwrap())
        });
        group.bench_function("small_rec_view", |b| {
            b.iter(|| decoder.decode_view(&bytes).unwrap())
        });
    }

    // Test Case 2: Mixed Record (Typical usage: some strings, some ints)
    {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("user_session_id_12345".to_string()),
        });
        record.add_field(LnmpField {
            fid: 11,
            value: LnmpValue::String("high".to_string()),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(998877),
        });

        let bytes = encoder.encode(&record).unwrap();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function("mixed_rec_std", |b| {
            b.iter(|| decoder.decode(&bytes).unwrap())
        });
        group.bench_function("mixed_rec_view", |b| {
            b.iter(|| decoder.decode_view(&bytes).unwrap())
        });
    }

    // Test Case 3: Large Payload (Large String + Embedding) - Zero Copy advantage should be massive
    {
        let mut record = LnmpRecord::new();
        let large_string = "a".repeat(10_000); // 10KB Alloc
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::String(large_string),
        });

        // 1KB Embedding (256 floats)
        let vector_data = vec![1.0; 256];
        let vector = Vector::new(
            EmbeddingType::F32,
            256,
            bytemuck::cast_slice(&vector_data).to_vec(),
        );
        record.add_field(LnmpField {
            fid: 101,
            value: LnmpValue::Embedding(vector),
        });

        let bytes = encoder.encode(&record).unwrap();
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function("large_rec_std", |b| {
            b.iter(|| decoder.decode(&bytes).unwrap())
        });
        group.bench_function("large_rec_view", |b| {
            b.iter(|| decoder.decode_view(&bytes).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_zero_copy_suite);
criterion_main!(benches);
