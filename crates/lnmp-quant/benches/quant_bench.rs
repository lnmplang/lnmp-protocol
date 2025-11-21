use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lnmp_embedding::Vector;
use lnmp_quant::{dequantize_embedding, quantize_embedding, QuantScheme};

fn bench_quantize_512dim(c: &mut Criterion) {
    let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
    let embedding = Vector::from_f32(values);

    c.bench_function("quantize_512dim", |b| {
        b.iter(|| {
            let _ = quantize_embedding(black_box(&embedding), QuantScheme::QInt8).unwrap();
        });
    });
}

fn bench_dequantize_512dim(c: &mut Criterion) {
    let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
    let embedding = Vector::from_f32(values);
    let quantized = quantize_embedding(&embedding, QuantScheme::QInt8).unwrap();

    c.bench_function("dequantize_512dim", |b| {
        b.iter(|| {
            let _ = dequantize_embedding(black_box(&quantized)).unwrap();
        });
    });
}

fn bench_roundtrip_512dim(c: &mut Criterion) {
    let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
    let embedding = Vector::from_f32(values);

    c.bench_function("roundtrip_512dim", |b| {
        b.iter(|| {
            let quantized = quantize_embedding(black_box(&embedding), QuantScheme::QInt8).unwrap();
            let _ = dequantize_embedding(&quantized).unwrap();
        });
    });
}

fn bench_quantize_128dim(c: &mut Criterion) {
    let values: Vec<f32> = (0..128).map(|i| (i as f32 / 128.0) - 0.5).collect();
    let embedding = Vector::from_f32(values);

    c.bench_function("quantize_128dim", |b| {
        b.iter(|| {
            let _ = quantize_embedding(black_box(&embedding), QuantScheme::QInt8).unwrap();
        });
    });
}

fn bench_quantize_1536dim(c: &mut Criterion) {
    let values: Vec<f32> = (0..1536).map(|i| (i as f32 / 1536.0) - 0.5).collect();
    let embedding = Vector::from_f32(values);

    c.bench_function("quantize_1536dim", |b| {
        b.iter(|| {
            let _ = quantize_embedding(black_box(&embedding), QuantScheme::QInt8).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_quantize_128dim,
    bench_quantize_512dim,
    bench_quantize_1536dim,
    bench_dequantize_512dim,
    bench_roundtrip_512dim
);
criterion_main!(benches);
