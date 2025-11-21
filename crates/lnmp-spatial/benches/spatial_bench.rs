use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lnmp_spatial::delta::Delta;
use lnmp_spatial::protocol::{SpatialStreamer, SpatialStreamerConfig};
use lnmp_spatial::*;

fn get_timestamp_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn benchmark_encoding(c: &mut Criterion) {
    let pos = Position3D {
        x: 123.45,
        y: 678.90,
        z: -543.21,
    };
    let val = SpatialValue::S2(pos);
    let mut buf = Vec::with_capacity(1024);

    c.bench_function("encode_position3d", |b| {
        b.iter(|| {
            buf.clear();
            encode_spatial(black_box(&val), &mut buf).unwrap();
        })
    });

    let rotation = Rotation {
        pitch: 0.1,
        yaw: 0.2,
        roll: 0.3,
    };
    let rot_val = SpatialValue::S3(rotation);

    c.bench_function("encode_rotation", |b| {
        b.iter(|| {
            buf.clear();
            encode_spatial(black_box(&rot_val), &mut buf).unwrap();
        })
    });

    let state = SpatialState {
        position: Some(pos),
        rotation: Some(rotation),
        velocity: Some(Velocity {
            vx: 1.0,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };
    let state_val = SpatialValue::S10(state);

    c.bench_function("encode_spatial_state", |b| {
        b.iter(|| {
            buf.clear();
            encode_spatial(black_box(&state_val), &mut buf).unwrap();
        })
    });
}

fn benchmark_decoding(c: &mut Criterion) {
    let pos = Position3D {
        x: 123.45,
        y: 678.90,
        z: -543.21,
    };
    let val = SpatialValue::S2(pos);
    let mut buf = Vec::new();
    encode_spatial(&val, &mut buf).unwrap();

    c.bench_function("decode_position3d", |b| {
        b.iter(|| {
            let mut slice = buf.as_slice();
            let _ = decode_spatial(black_box(&mut slice)).unwrap();
        })
    });
}

fn benchmark_delta(c: &mut Criterion) {
    let start = Position3D {
        x: 10.0,
        y: 20.0,
        z: 30.0,
    };
    let end = Position3D {
        x: 11.0,
        y: 19.0,
        z: 32.0,
    };

    c.bench_function("compute_delta", |b| {
        b.iter(|| Position3D::compute_delta(black_box(&start), black_box(&end)))
    });

    let delta = Position3D::compute_delta(&start, &end);

    c.bench_function("apply_delta", |b| {
        b.iter(|| Position3D::apply_delta(black_box(&start), black_box(&delta)))
    });

    // Full state delta
    let state_start = SpatialState {
        position: Some(start),
        rotation: Some(Rotation {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }),
        velocity: Some(Velocity {
            vx: 1.0,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };

    let state_end = SpatialState {
        position: Some(end),
        rotation: Some(Rotation {
            pitch: 0.1,
            yaw: 0.1,
            roll: 0.0,
        }),
        velocity: Some(Velocity {
            vx: 1.1,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };

    c.bench_function("compute_state_delta", |b| {
        b.iter(|| SpatialState::compute_delta(black_box(&state_start), black_box(&state_end)))
    });
}

fn benchmark_math(c: &mut Criterion) {
    let pos1 = Position3D {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let pos2 = Position3D {
        x: 100.0,
        y: 100.0,
        z: 100.0,
    };

    c.bench_function("spatial_distance", |b| {
        b.iter(|| spatial_distance(black_box(&pos1), black_box(&pos2)))
    });

    let transform = Transform {
        position: Position3D {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        },
        rotation: Rotation {
            pitch: 0.1,
            yaw: 0.2,
            roll: 0.3,
        },
        scale: Position3D {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };

    c.bench_function("spatial_transform", |b| {
        b.iter(|| spatial_transform(black_box(&pos1), black_box(&transform)))
    });
}

fn benchmark_hybrid_protocol(c: &mut Criterion) {
    let config = SpatialStreamerConfig {
        abs_interval: 100,
        enable_prediction: false,
        max_prediction_frames: 0,
    };

    let mut streamer = SpatialStreamer::with_config(config);
    let state = SpatialState {
        position: Some(Position3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        rotation: Some(Rotation {
            pitch: 0.1,
            yaw: 0.2,
            roll: 0.3,
        }),
        velocity: Some(Velocity {
            vx: 0.1,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };

    c.bench_function("hybrid_next_frame", |b| {
        b.iter(|| {
            streamer
                .next_frame(black_box(&state), get_timestamp_ns())
                .unwrap()
        })
    });
}

fn benchmark_predictive_delta(c: &mut Criterion) {
    let config = SpatialStreamerConfig {
        abs_interval: 100,
        enable_prediction: true,
        max_prediction_frames: 3,
    };

    let mut streamer = SpatialStreamer::with_config(config);
    let state = SpatialState {
        position: Some(Position3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        rotation: Some(Rotation {
            pitch: 0.1,
            yaw: 0.2,
            roll: 0.3,
        }),
        velocity: Some(Velocity {
            vx: 10.0,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };

    c.bench_function("predictive_next_frame", |b| {
        b.iter(|| {
            streamer
                .next_frame(black_box(&state), get_timestamp_ns())
                .unwrap()
        })
    });
}

criterion_group!(
    benches,
    benchmark_encoding,
    benchmark_decoding,
    benchmark_delta,
    benchmark_math,
    benchmark_hybrid_protocol,
    benchmark_predictive_delta
);
criterion_main!(benches);
