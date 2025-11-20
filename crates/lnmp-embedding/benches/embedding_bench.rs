use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lnmp_embedding::{Decoder, Encoder, SimilarityMetric, Vector, VectorDelta};

fn bench_vector_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_creation");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| Vector::from_f32(black_box(data.clone())));
        });
    }

    group.finish();
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let vec = Vector::from_f32(data);

        group.bench_with_input(BenchmarkId::from_parameter(dim), &vec, |b, v| {
            b.iter(|| Encoder::encode(black_box(v)).unwrap());
        });
    }

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let vec = Vector::from_f32(data);
        let encoded = Encoder::encode(&vec).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(dim), &encoded, |b, e| {
            b.iter(|| Decoder::decode(black_box(e)).unwrap());
        });
    }

    group.finish();
}

fn bench_similarity_cosine(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_cosine");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data1: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let data2: Vec<f32> = (0..*dim).map(|i| (*dim - i) as f32 * 0.01).collect();
        let vec1 = Vector::from_f32(data1);
        let vec2 = Vector::from_f32(data2);

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| {
                vec1.similarity(black_box(&vec2), SimilarityMetric::Cosine)
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_similarity_euclidean(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_euclidean");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data1: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let data2: Vec<f32> = (0..*dim).map(|i| (*dim - i) as f32 * 0.01).collect();
        let vec1 = Vector::from_f32(data1);
        let vec2 = Vector::from_f32(data2);

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| {
                vec1.similarity(black_box(&vec2), SimilarityMetric::Euclidean)
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_similarity_dotproduct(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_dotproduct");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data1: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let data2: Vec<f32> = (0..*dim).map(|i| (*dim - i) as f32 * 0.01).collect();
        let vec1 = Vector::from_f32(data1);
        let vec2 = Vector::from_f32(data2);

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| {
                vec1.similarity(black_box(&vec2), SimilarityMetric::DotProduct)
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let data: Vec<f32> = (0..*dim).map(|i| i as f32 * 0.01).collect();
        let vec = Vector::from_f32(data);

        group.bench_with_input(BenchmarkId::from_parameter(dim), &vec, |b, v| {
            b.iter(|| {
                let encoded = Encoder::encode(black_box(v)).unwrap();
                Decoder::decode(&encoded).unwrap()
            });
        });
    }

    group.finish();
}

fn bench_delta_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_compute");

    for dim in [128, 256, 512, 768, 1536].iter() {
        let old = Vector::from_f32(vec![0.1; *dim]);
        let mut new_data = vec![0.1; *dim];
        // Change 1% of values
        new_data.iter_mut()
            .step_by(100)
            .take(dim / 100)
            .for_each(|v| *v += 0.01);
        let new = Vector::from_f32(new_data);

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| VectorDelta::from_vectors(black_box(&old), black_box(&new), 1).unwrap());
        });
    }

    group.finish();
}

fn bench_delta_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_encode");

    for change_count in [1, 10, 50, 100, 500].iter() {
        let old = Vector::from_f32(vec![0.1; 1536]);
        let mut new_data = vec![0.1; 1536];
        new_data.iter_mut()
            .take(*change_count)
            .for_each(|v| *v += 0.01);
        let new = Vector::from_f32(new_data);
        let delta = VectorDelta::from_vectors(&old, &new, 1).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(change_count),
            change_count,
            |b, _| {
                b.iter(|| delta.encode().unwrap());
            },
        );
    }

    group.finish();
}

fn bench_delta_apply(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_apply");

    for change_count in [1, 10, 50, 100, 500].iter() {
        let base = Vector::from_f32(vec![0.1; 1536]);
        let mut new_data = vec![0.1; 1536];
        new_data.iter_mut()
            .take(*change_count)
            .for_each(|v| *v += 0.01);
        let new = Vector::from_f32(new_data);
        let delta = VectorDelta::from_vectors(&base, &new, 1).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(change_count),
            change_count,
            |b, _| {
                b.iter(|| delta.apply(black_box(&base)).unwrap());
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_vector_creation,
    bench_encode,
    bench_decode,
    bench_similarity_cosine,
    bench_similarity_euclidean,
    bench_similarity_dotproduct,
    bench_roundtrip,
    bench_delta_compute,
    bench_delta_encode,
    bench_delta_apply
);
criterion_main!(benches);
