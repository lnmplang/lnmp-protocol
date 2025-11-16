//! Performance benchmarks for LNMP v0.5 features
//!
//! This benchmark suite measures:
//! - Nested structure encoding/decoding speed
//! - Streaming throughput and latency
//! - Delta encoding efficiency
//! - Comparison with v0.4 flat encoding
//!
//! Run with: cargo bench --bench v05_performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use lnmp_codec::binary::{
    BinaryEncoder, BinaryDecoder, EncoderConfig, DecoderConfig,
    StreamingEncoder, StreamingDecoder, StreamingConfig,
    DeltaEncoder, DeltaDecoder, DeltaConfig,
};
use lnmp_core::{LnmpRecord, LnmpValue};

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a flat record with N fields (v0.4 style)
fn create_flat_record(num_fields: usize) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    for i in 0..num_fields {
        record.add_field(lnmp_core::LnmpField {
            fid: (i as u16 + 1),
            value: LnmpValue::Int(i as i64),
        });
    }
    record
}

/// Creates a nested record with specified depth and breadth
fn create_nested_record(depth: usize, breadth: usize) -> LnmpRecord {
    fn build_nested(current_depth: usize, max_depth: usize, breadth: usize) -> LnmpRecord {
        let mut record = LnmpRecord::new();
        
        for i in 0..breadth {
            let fid = i as u16 + 1;
            if current_depth < max_depth {
                // Add nested record
                record.add_field(lnmp_core::LnmpField {
                    fid,
                    value: LnmpValue::NestedRecord(Box::new(build_nested(current_depth + 1, max_depth, breadth))),
                });
            } else {
                // Leaf node - add primitive value
                record.add_field(lnmp_core::LnmpField {
                    fid,
                    value: LnmpValue::Int(i as i64),
                });
            }
        }
        
        record
    }
    
    build_nested(0, depth, breadth)
}

/// Creates a record with nested arrays
fn create_nested_array_record(array_size: usize, fields_per_record: usize) -> LnmpRecord {
    let mut records = Vec::new();
    for i in 0..array_size {
        let mut record = LnmpRecord::new();
        for j in 0..fields_per_record {
            record.add_field(lnmp_core::LnmpField {
                fid: j as u16 + 1,
                value: LnmpValue::Int((i * fields_per_record + j) as i64),
            });
        }
        records.push(record);
    }
    
    let mut root = LnmpRecord::new();
    root.add_field(lnmp_core::LnmpField {
        fid: 1,
        value: LnmpValue::NestedArray(records),
    });
    root
}

/// Creates a large payload for streaming tests
fn create_large_payload(size_kb: usize) -> Vec<u8> {
    // Create a record that will encode to approximately size_kb kilobytes
    let fields_needed = (size_kb * 1024) / 10; // Rough estimate: ~10 bytes per field
    let record = create_flat_record(fields_needed);
    
    let config = EncoderConfig::default().with_nested_binary(true);
    let encoder = BinaryEncoder::with_config(config);
    encoder.encode(&record).unwrap()
}

// ============================================================================
// Benchmark: Nested Structure Encoding Speed
// ============================================================================

fn bench_nested_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_encoding");
    
    // Test different nesting depths
    for depth in [1, 2, 3, 5] {
        let record = create_nested_record(depth, 3);
        let config = EncoderConfig::default().with_nested_binary(true);
        let encoder = BinaryEncoder::with_config(config);
        
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            &record,
            |b, record| {
                b.iter(|| {
                    black_box(encoder.encode(black_box(record)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_nested_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_decoding");
    
    // Test different nesting depths
    for depth in [1, 2, 3, 5] {
        let record = create_nested_record(depth, 3);
        let config = EncoderConfig::default().with_nested_binary(true);
        let encoder = BinaryEncoder::with_config(config);
        let encoded = encoder.encode(&record).unwrap();
        
        let decoder_config = DecoderConfig::default().with_validate_nesting(true);
        let decoder = BinaryDecoder::with_config(decoder_config);
        
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            &encoded,
            |b, encoded| {
                b.iter(|| {
                    black_box(decoder.decode(black_box(encoded)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Benchmark: Streaming Throughput and Latency
// ============================================================================

fn bench_streaming_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_throughput");
    
    // Test different payload sizes
    for size_kb in [4, 16, 64, 256] {
        let payload = create_large_payload(size_kb);
        group.throughput(Throughput::Bytes(payload.len() as u64));
        
        let config = StreamingConfig::new().with_chunk_size(4096);
        
        group.bench_with_input(
            BenchmarkId::new("encode_kb", size_kb),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let mut encoder = StreamingEncoder::with_config(config.clone());
                    let mut frames = Vec::new();
                    
                    frames.extend(encoder.begin_stream().unwrap());
                    
                    for chunk in payload.chunks(config.chunk_size) {
                        frames.extend(encoder.write_chunk(black_box(chunk)).unwrap());
                    }
                    
                    frames.extend(encoder.end_stream().unwrap());
                    black_box(frames)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_streaming_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_decode");
    
    // Test different payload sizes
    for size_kb in [4, 16, 64, 256] {
        let payload = create_large_payload(size_kb);
        let config = StreamingConfig::new().with_chunk_size(4096);
        
        // Pre-encode the payload into frames
        let mut encoder = StreamingEncoder::with_config(config.clone());
        let mut frames = Vec::new();
        frames.extend(encoder.begin_stream().unwrap());
        for chunk in payload.chunks(config.chunk_size) {
            frames.extend(encoder.write_chunk(chunk).unwrap());
        }
        frames.extend(encoder.end_stream().unwrap());
        
        group.throughput(Throughput::Bytes(payload.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("decode_kb", size_kb),
            &frames,
            |b, frames| {
                b.iter(|| {
                    let mut decoder = StreamingDecoder::with_config(config.clone());
                    
                    // Feed all frames
                    let mut offset = 0;
                    while offset < frames.len() {
                        // Find frame boundary (simplified - assumes frames are concatenated)
                        let frame_len = std::cmp::min(1024, frames.len() - offset);
                        let _ = decoder.feed_frame(black_box(&frames[offset..offset + frame_len]));
                        offset += frame_len;
                    }
                    
                    let payload = decoder.get_complete_payload().map(|p| p.to_vec());
                    black_box(payload)
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Benchmark: Delta Encoding Efficiency
// ============================================================================

fn bench_delta_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_encoding");
    
    // Create base record
    let base = create_flat_record(100);
    
    // Test different change percentages
    for change_pct in [10, 25, 50, 75] {
        let mut modified = base.clone();
        let num_changes = (100 * change_pct) / 100;
        
        // Modify some fields
        for i in 0..num_changes {
            modified.add_field(lnmp_core::LnmpField {
                fid: i as u16 + 1,
                value: LnmpValue::Int(9999),
            });
        }
        
        let config = DeltaConfig::new().with_enable_delta(true);
        let encoder = DeltaEncoder::with_config(config);
        
        group.bench_with_input(
            BenchmarkId::new("changes_pct", change_pct),
            &(&base, &modified),
            |b, (base, modified)| {
                b.iter(|| {
                    let delta = encoder.compute_delta(black_box(base), black_box(modified)).unwrap();
                    black_box(encoder.encode_delta(black_box(&delta)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_delta_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_application");
    
    // Create base record
    let base = create_flat_record(100);
    
    // Test different change percentages
    for change_pct in [10, 25, 50, 75] {
        let mut modified = base.clone();
        let num_changes = (100 * change_pct) / 100;
        
        // Modify some fields
        for i in 0..num_changes {
            modified.add_field(lnmp_core::LnmpField {
                fid: i as u16 + 1,
                value: LnmpValue::Int(9999),
            });
        }
        
        let config = DeltaConfig::new().with_enable_delta(true);
        let encoder = DeltaEncoder::with_config(config.clone());
        
        let delta = encoder.compute_delta(&base, &modified).unwrap();
        let encoded_delta = encoder.encode_delta(&delta).unwrap();
        
        let base_owned = base.clone();
        let encoded_delta_owned = encoded_delta.clone();
        
        group.bench_function(
            BenchmarkId::new("changes_pct", change_pct),
            |b| {
                b.iter(|| {
                    let config = DeltaConfig::new().with_enable_delta(true);
                    let decoder = DeltaDecoder::with_config(config);
                    let delta = decoder.decode_delta(black_box(&encoded_delta_owned)).unwrap();
                    let mut result = base_owned.clone();
                    decoder.apply_delta(&mut result, &delta).unwrap();
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Benchmark: v0.4 vs v0.5 Comparison
// ============================================================================

fn bench_v04_vs_v05_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("v04_vs_v05_encoding");
    
    // Test with flat records (both should handle these)
    for num_fields in [10, 50, 100, 500] {
        let record = create_flat_record(num_fields);
        
        // v0.4 style (nested disabled)
        let v04_config = EncoderConfig::default().with_nested_binary(false);
        let v04_encoder = BinaryEncoder::with_config(v04_config);
        
        group.bench_with_input(
            BenchmarkId::new("v0.4", num_fields),
            &record,
            |b, record| {
                b.iter(|| {
                    black_box(v04_encoder.encode(black_box(record)).unwrap())
                });
            },
        );
        
        // v0.5 style (nested enabled)
        let v05_config = EncoderConfig::default().with_nested_binary(true);
        let v05_encoder = BinaryEncoder::with_config(v05_config);
        
        group.bench_with_input(
            BenchmarkId::new("v0.5", num_fields),
            &record,
            |b, record| {
                b.iter(|| {
                    black_box(v05_encoder.encode(black_box(record)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_v04_vs_v05_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("v04_vs_v05_decoding");
    
    // Test with flat records
    for num_fields in [10, 50, 100, 500] {
        let record = create_flat_record(num_fields);
        
        // Encode with v0.4
        let v04_config = EncoderConfig::default().with_nested_binary(false);
        let v04_encoder = BinaryEncoder::with_config(v04_config);
        let v04_encoded = v04_encoder.encode(&record).unwrap();
        
        let v04_decoder_config = DecoderConfig::default().with_validate_nesting(false);
        let v04_decoder = BinaryDecoder::with_config(v04_decoder_config);
        
        group.bench_with_input(
            BenchmarkId::new("v0.4", num_fields),
            &v04_encoded,
            |b, encoded| {
                b.iter(|| {
                    black_box(v04_decoder.decode(black_box(encoded)).unwrap())
                });
            },
        );
        
        // Encode with v0.5
        let v05_config = EncoderConfig::default().with_nested_binary(true);
        let v05_encoder = BinaryEncoder::with_config(v05_config);
        let v05_encoded = v05_encoder.encode(&record).unwrap();
        
        let v05_decoder_config = DecoderConfig::default().with_validate_nesting(true);
        let v05_decoder = BinaryDecoder::with_config(v05_decoder_config);
        
        group.bench_with_input(
            BenchmarkId::new("v0.5", num_fields),
            &v05_encoded,
            |b, encoded| {
                b.iter(|| {
                    black_box(v05_decoder.decode(black_box(encoded)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Benchmark: Nested Array Performance
// ============================================================================

fn bench_nested_array_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_array_encoding");
    
    // Test different array sizes
    for array_size in [10, 50, 100, 500] {
        let record = create_nested_array_record(array_size, 5);
        let config = EncoderConfig::default().with_nested_binary(true);
        let encoder = BinaryEncoder::with_config(config);
        
        group.bench_with_input(
            BenchmarkId::new("array_size", array_size),
            &record,
            |b, record| {
                b.iter(|| {
                    black_box(encoder.encode(black_box(record)).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_nested_encoding,
    bench_nested_decoding,
    bench_streaming_throughput,
    bench_streaming_decode,
    bench_delta_encoding,
    bench_delta_application,
    bench_v04_vs_v05_encoding,
    bench_v04_vs_v05_decoding,
    bench_nested_array_encoding,
);

criterion_main!(benches);
