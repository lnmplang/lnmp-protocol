use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lnmp_codec::binary::BinaryEncoder;
use lnmp_core::{LnmpField, LnmpValue, RecordBuilder};
use lnmp_llb::{ExplainEncoder, LlbConfig, LlbConverter, PromptOptimizer, SemanticDictionary};

fn bench_llb_converter(c: &mut Criterion) {
    let config = LlbConfig::default();
    let converter = LlbConverter::new(config);

    // 1. ShortForm -> Binary (Simple)
    let shortform_simple = "1=42;2=test_value;3=1";
    c.bench_function("llb_sf_to_bin_simple", |b| {
        b.iter(|| converter.shortform_to_binary(black_box(shortform_simple)))
    });

    // 2. ShortForm -> Binary (Arrays)
    let shortform_arrays = "10=[1,2,3,4,5];11=[1.1,2.2,3.3];12=[1,0,1]";
    c.bench_function("llb_sf_to_bin_arrays", |b| {
        b.iter(|| converter.shortform_to_binary(black_box(shortform_arrays)))
    });

    // 3. Binary -> ShortForm (Simple)
    let record_simple = RecordBuilder::new()
        .add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        })
        .add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("test_value".to_string()),
        })
        .add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(true),
        })
        .build();
    let binary_simple = BinaryEncoder::new().encode(&record_simple).unwrap();

    c.bench_function("llb_bin_to_sf_simple", |b| {
        b.iter(|| converter.binary_to_shortform(black_box(&binary_simple)))
    });

    // 4. Binary -> ShortForm (Arrays)
    let record_arrays = RecordBuilder::new()
        .add_field(LnmpField {
            fid: 10,
            value: LnmpValue::IntArray(vec![1, 2, 3, 4, 5]),
        })
        .add_field(LnmpField {
            fid: 11,
            value: LnmpValue::FloatArray(vec![1.1, 2.2, 3.3]),
        })
        .add_field(LnmpField {
            fid: 12,
            value: LnmpValue::BoolArray(vec![true, false, true]),
        })
        .build();
    let binary_arrays = BinaryEncoder::new().encode(&record_arrays).unwrap();

    c.bench_function("llb_bin_to_sf_arrays", |b| {
        b.iter(|| converter.binary_to_shortform(black_box(&binary_arrays)))
    });
}

fn bench_explain_encoder(c: &mut Criterion) {
    let dict = SemanticDictionary::from_pairs(vec![
        (1, "user_id"),
        (2, "username"),
        (3, "is_active"),
        (10, "scores"),
    ]);
    let encoder = ExplainEncoder::new(dict);

    let record = RecordBuilder::new()
        .add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        })
        .add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("alice".to_string()),
        })
        .add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(true),
        })
        .add_field(LnmpField {
            fid: 10,
            value: LnmpValue::IntArray(vec![100, 95, 88]),
        })
        .build();

    c.bench_function("explain_encode", |b| {
        b.iter(|| encoder.encode_with_explanation(black_box(&record)))
    });
}

fn bench_prompt_optimizer(c: &mut Criterion) {
    let optimizer = PromptOptimizer::default();

    let record = RecordBuilder::new()
        .add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        })
        .add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("alice".to_string()),
        })
        .add_field(LnmpField {
            fid: 10,
            value: LnmpValue::IntArray(vec![1, 2, 3, 4, 5]),
        })
        .build();

    c.bench_function("prompt_optimize", |b| {
        b.iter(|| {
            for field in record.fields() {
                black_box(optimizer.optimize_field(field));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_llb_converter,
    bench_explain_encoder,
    bench_prompt_optimizer
);
criterion_main!(benches);
