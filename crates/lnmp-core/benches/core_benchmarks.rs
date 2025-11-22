use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lnmp_core::{
    checksum::SemanticChecksum, LnmpField, LnmpRecord, LnmpValue, RecordBuilder, TypeHint,
};

/// Benchmark record creation with RecordBuilder vs manual construction
fn bench_record_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("record_creation");

    // Small record (5 fields)
    group.bench_function("builder_small", |b| {
        b.iter(|| {
            black_box(
                RecordBuilder::new()
                    .add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::Int(42),
                    })
                    .add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("test".to_string()),
                    })
                    .add_field(LnmpField {
                        fid: 3,
                        value: LnmpValue::Bool(true),
                    })
                    .add_field(LnmpField {
                        fid: 4,
                        value: LnmpValue::Float(3.15),
                    })
                    .add_field(LnmpField {
                        fid: 5,
                        value: LnmpValue::Int(100),
                    })
                    .build(),
            )
        })
    });

    group.bench_function("manual_small", |b| {
        b.iter(|| {
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 1,
                value: LnmpValue::Int(42),
            });
            record.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("test".to_string()),
            });
            record.add_field(LnmpField {
                fid: 3,
                value: LnmpValue::Bool(true),
            });
            record.add_field(LnmpField {
                fid: 4,
                value: LnmpValue::Float(3.15),
            });
            record.add_field(LnmpField {
                fid: 5,
                value: LnmpValue::Int(100),
            });
            black_box(record)
        })
    });

    // Large record (50 fields)
    group.bench_function("builder_large", |b| {
        b.iter(|| {
            let mut builder = RecordBuilder::new();
            for i in 0..50 {
                builder = builder.add_field(LnmpField {
                    fid: i,
                    value: LnmpValue::Int(i as i64),
                });
            }
            black_box(builder.build())
        })
    });

    group.bench_function("from_fields_large", |b| {
        b.iter(|| {
            let fields: Vec<_> = (0..50)
                .map(|i| LnmpField {
                    fid: i,
                    value: LnmpValue::Int(i as i64),
                })
                .collect();
            black_box(LnmpRecord::from_fields(fields))
        })
    });

    group.finish();
}

/// Benchmark field sorting performance
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_sorting");

    for size in [10, 50, 100, 500].iter() {
        // Create unsorted record
        let mut record = LnmpRecord::new();
        for i in (0..*size).rev() {
            record.add_field(LnmpField {
                fid: i,
                value: LnmpValue::Int(i as i64),
            });
        }

        group.bench_with_input(BenchmarkId::new("sorted_fields", size), size, |b, _| {
            b.iter(|| black_box(record.sorted_fields()))
        });
    }

    group.finish();
}

/// Benchmark canonical equality and hashing
fn bench_canonical_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_operations");

    // Create two records with same data, different order
    let mut rec1 = LnmpRecord::new();
    for i in 0..20 {
        rec1.add_field(LnmpField {
            fid: i,
            value: LnmpValue::Int(i as i64),
        });
    }

    let mut rec2 = LnmpRecord::new();
    for i in (0..20).rev() {
        rec2.add_field(LnmpField {
            fid: i,
            value: LnmpValue::Int(i as i64),
        });
    }

    group.bench_function("canonical_eq", |b| {
        b.iter(|| black_box(rec1.canonical_eq(&rec2)))
    });

    group.bench_function("canonical_hash", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            rec1.canonical_hash(&mut hasher);
            black_box(hasher.finish())
        })
    });

    group.bench_function("structural_eq", |b| b.iter(|| black_box(rec1 == rec2)));

    group.finish();
}

/// Benchmark checksum computation
fn bench_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");

    group.bench_function("int", |b| {
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::Int),
                &LnmpValue::Int(42),
            ))
        })
    });

    group.bench_function("string", |b| {
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::String),
                &LnmpValue::String("test string".to_string()),
            ))
        })
    });

    group.bench_function("string_array", |b| {
        let arr = LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::StringArray),
                &arr,
            ))
        })
    });

    group.bench_function("nested_record", |b| {
        let mut inner = LnmpRecord::new();
        for i in 0..10 {
            inner.add_field(LnmpField {
                fid: i,
                value: LnmpValue::Int(i as i64),
            });
        }
        let value = LnmpValue::NestedRecord(Box::new(inner));
        b.iter(|| black_box(SemanticChecksum::compute(1, Some(TypeHint::Record), &value)))
    });

    group.finish();
}

/// Benchmark array operations
fn bench_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_operations");

    // String array
    group.bench_function("string_array_checksum", |b| {
        let arr = LnmpValue::StringArray((0..100).map(|i| format!("item{}", i)).collect());
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::StringArray),
                &arr,
            ))
        })
    });

    // Int array
    group.bench_function("int_array_checksum", |b| {
        let arr = LnmpValue::IntArray((0..100).collect());
        b.iter(|| black_box(SemanticChecksum::compute(1, Some(TypeHint::IntArray), &arr)))
    });

    // Float array
    group.bench_function("float_array_checksum", |b| {
        let arr = LnmpValue::FloatArray((0..100).map(|i| i as f64 * 1.5).collect());
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::FloatArray),
                &arr,
            ))
        })
    });

    // Bool array
    group.bench_function("bool_array_checksum", |b| {
        let arr = LnmpValue::BoolArray((0..100).map(|i| i % 2 == 0).collect());
        b.iter(|| {
            black_box(SemanticChecksum::compute(
                1,
                Some(TypeHint::BoolArray),
                &arr,
            ))
        })
    });

    group.finish();
}

/// Benchmark validation operations
fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    // Sorted record
    let sorted = LnmpRecord::from_fields(
        (0..50)
            .map(|i| LnmpField {
                fid: i,
                value: LnmpValue::Int(i as i64),
            })
            .collect(),
    );

    group.bench_function("validate_sorted", |b| {
        b.iter(|| black_box(sorted.validate_field_ordering()))
    });

    group.bench_function("is_canonical_order", |b| {
        b.iter(|| black_box(sorted.is_canonical_order()))
    });

    // Unsorted record
    let mut unsorted = LnmpRecord::new();
    for i in (0..50).rev() {
        unsorted.add_field(LnmpField {
            fid: i,
            value: LnmpValue::Int(i as i64),
        });
    }

    group.bench_function("count_violations", |b| {
        b.iter(|| black_box(unsorted.count_ordering_violations()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_record_creation,
    bench_sorting,
    bench_canonical_ops,
    bench_checksum,
    bench_arrays,
    bench_validation
);
criterion_main!(benches);
