use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use lnmp_sanitize::{sanitize_lnmp_text, SanitizationConfig, SanitizationLevel};

fn sample_payloads() -> Vec<String> {
    vec![
        "F1=1;F2=\"ok\";F3=[admin,dev]".to_string(),
        "F7=1 ; F12=14532 ; F23=[\"admin\" , \"dev\"]".to_string(),
        "F42=hello world;F9=\"unterminated".to_string(),
        r#"F5="already \"escaped\""#.to_string(),
        "F99=true;F100=no;F13=00042".to_string(),
    ]
}

fn bench_sanitize(c: &mut Criterion) {
    let payloads = sample_payloads();
    let configs = vec![
        SanitizationConfig {
            level: SanitizationLevel::Minimal,
            ..Default::default()
        },
        SanitizationConfig {
            level: SanitizationLevel::Normal,
            ..Default::default()
        },
        SanitizationConfig {
            level: SanitizationLevel::Aggressive,
            normalize_numbers: true,
            ..Default::default()
        },
    ];

    let mut group = c.benchmark_group("sanitize_lnmp_text");
    group.throughput(Throughput::Elements(payloads.len() as u64));

    for config in configs {
        group.bench_function(format!("level_{:?}", config.level), |b| {
            b.iter_batched(
                || payloads.clone(),
                |inputs| {
                    for input in inputs {
                        let _ = sanitize_lnmp_text(black_box(&input), &config);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_sanitize);
criterion_main!(benches);
