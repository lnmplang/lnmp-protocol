use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lnmp_embedding::Vector;
use lnmp_quant::adaptive::{quantize_adaptive, AccuracyTarget};
use lnmp_quant::batch::quantize_batch;
use lnmp_quant::{dequantize_embedding, quantize_embedding, QuantScheme};

/// Benchmark quantization across different embedding dimensions for all schemes
fn bench_quantize_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantize_by_dimension");

    for dim in [128, 256, 512, 768, 1024, 1536, 2048] {
        let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
        let vec = Vector::from_f32(values);

        // QInt8 (4x compression)
        group.bench_with_input(BenchmarkId::new("QInt8", dim), &vec, |b, v| {
            b.iter(|| quantize_embedding(black_box(v), QuantScheme::QInt8))
        });

        // QInt4 (8x compression)
        group.bench_with_input(BenchmarkId::new("QInt4", dim), &vec, |b, v| {
            b.iter(|| quantize_embedding(black_box(v), QuantScheme::QInt4))
        });

        // Binary (32x compression)
        group.bench_with_input(BenchmarkId::new("Binary", dim), &vec, |b, v| {
            b.iter(|| quantize_embedding(black_box(v), QuantScheme::Binary))
        });

        // FP16 (2x compression, near-lossless)
        group.bench_with_input(BenchmarkId::new("FP16", dim), &vec, |b, v| {
            b.iter(|| quantize_embedding(black_box(v), QuantScheme::FP16Passthrough))
        });
    }

    group.finish();
}

/// Benchmark dequantization across different embedding dimensions for all schemes
fn bench_dequantize_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("dequantize_by_dimension");

    for dim in [128, 256, 512, 768, 1024, 1536, 2048] {
        let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
        let vec = Vector::from_f32(values);

        // QInt8
        let q8 = quantize_embedding(&vec, QuantScheme::QInt8).unwrap();
        group.bench_with_input(BenchmarkId::new("QInt8", dim), &q8, |b, qv| {
            b.iter(|| dequantize_embedding(black_box(qv)))
        });

        // QInt4
        let q4 = quantize_embedding(&vec, QuantScheme::QInt4).unwrap();
        group.bench_with_input(BenchmarkId::new("QInt4", dim), &q4, |b, qv| {
            b.iter(|| dequantize_embedding(black_box(qv)))
        });

        // Binary
        let qbin = quantize_embedding(&vec, QuantScheme::Binary).unwrap();
        group.bench_with_input(BenchmarkId::new("Binary", dim), &qbin, |b, qv| {
            b.iter(|| dequantize_embedding(black_box(qv)))
        });

        // FP16
        let qfp16 = quantize_embedding(&vec, QuantScheme::FP16Passthrough).unwrap();
        group.bench_with_input(BenchmarkId::new("FP16", dim), &qfp16, |b, qv| {
            b.iter(|| dequantize_embedding(black_box(qv)))
        });
    }

    group.finish();
}

/// Benchmark full roundtrip (quantize + dequantize) for all schemes
fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for dim in [128, 256, 512, 768, 1024, 1536] {
        let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
        let vec = Vector::from_f32(values);

        group.bench_with_input(BenchmarkId::new("QInt8", dim), &vec, |b, v| {
            b.iter(|| {
                let quantized = quantize_embedding(black_box(v), QuantScheme::QInt8).unwrap();
                dequantize_embedding(&quantized).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("QInt4", dim), &vec, |b, v| {
            b.iter(|| {
                let quantized = quantize_embedding(black_box(v), QuantScheme::QInt4).unwrap();
                dequantize_embedding(&quantized).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("Binary", dim), &vec, |b, v| {
            b.iter(|| {
                let quantized = quantize_embedding(black_box(v), QuantScheme::Binary).unwrap();
                dequantize_embedding(&quantized).unwrap()
            })
        });

        group.bench_with_input(BenchmarkId::new("FP16", dim), &vec, |b, v| {
            b.iter(|| {
                let quantized =
                    quantize_embedding(black_box(v), QuantScheme::FP16Passthrough).unwrap();
                dequantize_embedding(&quantized).unwrap()
            })
        });
    }

    group.finish();
}

/// Benchmark with realistic embedding data (random normalized values)
fn bench_realistic_embeddings(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_embeddings");

    // Simulate typical embedding distributions
    let dim = 512;
    let values: Vec<f32> = (0..dim)
        .map(|i| {
            // Simulate normalized embeddings with some variation
            let x = (i as f32) / (dim as f32);
            (x * 2.0 - 1.0) * 0.5 // Values roughly in [-0.5, 0.5]
        })
        .collect();
    let vec = Vector::from_f32(values);

    // QInt8
    group.bench_function("QInt8/quantize_512", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::QInt8))
    });

    let q8 = quantize_embedding(&vec, QuantScheme::QInt8).unwrap();
    group.bench_function("QInt8/dequantize_512", |b| {
        b.iter(|| dequantize_embedding(black_box(&q8)))
    });

    group.bench_function("QInt8/roundtrip_512", |b| {
        b.iter(|| {
            let q = quantize_embedding(black_box(&vec), QuantScheme::QInt8).unwrap();
            dequantize_embedding(&q).unwrap()
        })
    });

    // QInt4
    group.bench_function("QInt4/quantize_512", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::QInt4))
    });

    let q4 = quantize_embedding(&vec, QuantScheme::QInt4).unwrap();
    group.bench_function("QInt4/dequantize_512", |b| {
        b.iter(|| dequantize_embedding(black_box(&q4)))
    });

    group.bench_function("QInt4/roundtrip_512", |b| {
        b.iter(|| {
            let q = quantize_embedding(black_box(&vec), QuantScheme::QInt4).unwrap();
            dequantize_embedding(&q).unwrap()
        })
    });

    // Binary
    group.bench_function("Binary/quantize_512", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::Binary))
    });

    let qbin = quantize_embedding(&vec, QuantScheme::Binary).unwrap();
    group.bench_function("Binary/dequantize_512", |b| {
        b.iter(|| dequantize_embedding(black_box(&qbin)))
    });

    group.bench_function("Binary/roundtrip_512", |b| {
        b.iter(|| {
            let q = quantize_embedding(black_box(&vec), QuantScheme::Binary).unwrap();
            dequantize_embedding(&q).unwrap()
        })
    });

    // FP16
    group.bench_function("FP16/quantize_512", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::FP16Passthrough))
    });

    let qfp16 = quantize_embedding(&vec, QuantScheme::FP16Passthrough).unwrap();
    group.bench_function("FP16/dequantize_512", |b| {
        b.iter(|| dequantize_embedding(black_box(&qfp16)))
    });

    group.bench_function("FP16/roundtrip_512", |b| {
        b.iter(|| {
            let q = quantize_embedding(black_box(&vec), QuantScheme::FP16Passthrough).unwrap();
            dequantize_embedding(&q).unwrap()
        })
    });

    group.finish();
}

/// Benchmark compression ratios
fn bench_compression_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_metrics");

    let dim = 512;
    let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
    let vec = Vector::from_f32(values);

    // Benchmark data size calculation
    group.bench_function("QInt8/data_size", |b| {
        let q = quantize_embedding(&vec, QuantScheme::QInt8).unwrap();
        b.iter(|| black_box(&q).data_size())
    });

    group.bench_function("QInt4/data_size", |b| {
        let q = quantize_embedding(&vec, QuantScheme::QInt4).unwrap();
        b.iter(|| black_box(&q).data_size())
    });

    group.bench_function("Binary/data_size", |b| {
        let q = quantize_embedding(&vec, QuantScheme::Binary).unwrap();
        b.iter(|| black_box(&q).data_size())
    });

    group.bench_function("FP16/data_size", |b| {
        let q = quantize_embedding(&vec, QuantScheme::FP16Passthrough).unwrap();
        b.iter(|| black_box(&q).data_size())
    });

    group.finish();
}

/// Benchmark memory overhead
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    let dim = 512;
    let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
    let vec = Vector::from_f32(values);

    // Test vector allocation overhead
    group.bench_function("vec_allocation_512", |b| {
        b.iter(|| {
            let v: Vec<u8> = Vec::with_capacity(black_box(dim));
            black_box(v)
        })
    });

    // Test nibble allocation (QInt4)
    group.bench_function("vec_allocation_256_nibbles", |b| {
        b.iter(|| {
            let v: Vec<u8> = Vec::with_capacity(black_box(dim / 2));
            black_box(v)
        })
    });

    // Test bit allocation (Binary)
    group.bench_function("vec_allocation_64_bits", |b| {
        b.iter(|| {
            let v: Vec<u8> = Vec::with_capacity(black_box(dim / 8));
            black_box(v)
        })
    });

    // Test quantize without cloning input
    group.bench_function("QInt8/quantize_no_clone", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::QInt8))
    });

    group.bench_function("QInt4/quantize_no_clone", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::QInt4))
    });

    group.bench_function("Binary/quantize_no_clone", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::Binary))
    });

    group.bench_function("FP16/quantize_no_clone", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::FP16Passthrough))
    });

    group.finish();
}

/// Benchmark adaptive quantization overhead
fn bench_adaptive(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_overhead");
    let dim = 512;
    let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
    let vec = Vector::from_f32(values);

    // Baseline: Direct QInt8
    group.bench_function("Direct/QInt8", |b| {
        b.iter(|| quantize_embedding(black_box(&vec), QuantScheme::QInt8))
    });

    // Adaptive: High (maps to QInt8)
    group.bench_function("Adaptive/High", |b| {
        b.iter(|| quantize_adaptive(black_box(&vec), AccuracyTarget::High))
    });

    // Adaptive: Maximum (maps to FP16)
    group.bench_function("Adaptive/Maximum", |b| {
        b.iter(|| quantize_adaptive(black_box(&vec), AccuracyTarget::Maximum))
    });

    group.finish();
}

/// Benchmark batch processing performance
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    let dim = 512;
    let values: Vec<f32> = (0..dim).map(|i| (i as f32) / (dim as f32)).collect();
    let vec = Vector::from_f32(values);

    // Create batches of different sizes
    for size in [10, 100, 1000] {
        let batch: Vec<Vector> = (0..size).map(|_| vec.clone()).collect();

        // Sequential loop (Baseline)
        group.bench_with_input(BenchmarkId::new("Sequential", size), &batch, |b, batch| {
            b.iter(|| {
                for v in batch {
                    black_box(quantize_embedding(v, QuantScheme::QInt8).unwrap());
                }
            })
        });

        // Batch API
        group.bench_with_input(BenchmarkId::new("BatchAPI", size), &batch, |b, batch| {
            b.iter(|| {
                black_box(quantize_batch(batch, QuantScheme::QInt8));
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_quantize_dimensions,
    bench_dequantize_dimensions,
    bench_roundtrip,
    bench_realistic_embeddings,
    bench_compression_ratios,
    bench_memory_operations,
    bench_adaptive,
    bench_batch_processing
);
criterion_main!(benches);
