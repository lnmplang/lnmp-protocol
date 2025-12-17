# lnmp-quant

> Quantization and compression for LNMP embedding vectors with minimal accuracy loss

> **FID Registry:** All examples use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

[![Crates.io](https://img.shields.io/crates/v/lnmp-quant)](https://crates.io/crates/lnmp-quant)
[![Documentation](https://docs.rs/lnmp-quant/badge.svg)](https://docs.rs/lnmp-quant)

## Overview

`lnmp-quant` provides efficient quantization schemes to compress embedding vectors while maintaining high semantic accuracy. It offers a spectrum of compression options from 4x to 32x:

- **Multiple schemes**: QInt8 (4x), QInt4 (8x), Binary (32x)
- **Fast quantization/dequantization** (sub-microsecond performance for 512-dim)
- **LNMP protocol integration** for efficient agent-to-agent communication
- **Flexible accuracy trade-offs** (99% to 85% similarity preservation)

## Key Benefits

| Scheme | Compression | Accuracy | 512-dim Quantize | 512-dim Dequantize |
|--------|-------------|----------|------------------|--------------------|
| **FP16** | 2x | ~99.9% | ~300 ns | ~150 ns |
| **QInt8** | 4x | ~99% | 1.17 µs | 457 ns |
| **QInt4** | 8x | ~95-97% | ~600 ns | ~230 ns |
| **Binary** | 32x | ~85-90% | ~200 ns | ~100 ns |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lnmp-quant = "0.5.2"
lnmp-embedding = "0.5.2"
```

### Basic Usage

```rust
use lnmp_quant::{quantize_embedding, dequantize_embedding, QuantScheme};
use lnmp_embedding::Vector;

// Create an embedding
let embedding = Vector::from_f32(vec![0.12, -0.45, 0.33, /* ... */]);

// Quantize to QInt8
let quantized = quantize_embedding(&embedding, QuantScheme::QInt8)?;

println!("Original size: {} bytes", embedding.dim * 4);
println!("Quantized size: {} bytes", quantized.data_size());
println!("Compression ratio: {:.1}x", quantized.compression_ratio());

// Dequantize back to F32
let restored = dequantize_embedding(&quantized)?;

// Verify accuracy
use lnmp_embedding::SimilarityMetric;
let similarity = embedding.similarity(&restored, SimilarityMetric::Cosine)?;
assert!(similarity > 0.99);
```

### LNMP Integration

```rust
use lnmp_core::{LnmpValue, LnmpField, LnmpRecord, TypeHint};
use lnmp_quant::quantize_embedding;

// Quantize an embedding
let quantized = quantize_embedding(&embedding, QuantScheme::QInt8)?;

// Add to LNMP record (F512=embedding from registry)
let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 512,  // F512=embedding
    value: LnmpValue::QuantizedEmbedding(quantized),
});

// Type hint support
let hint = TypeHint::QuantizedEmbedding; // :qv
assert_eq!(hint.as_str(), "qv");
```

## Adaptive Quantization

Automatically select the best scheme based on your requirements:

```rust
use lnmp_quant::adaptive::{quantize_adaptive, AccuracyTarget};

// Maximum accuracy (FP16)
let q = quantize_adaptive(&emb, AccuracyTarget::Maximum)?;

// High accuracy (QInt8)
let q = quantize_adaptive(&emb, AccuracyTarget::High)?;

// Balanced (QInt4)
let q = quantize_adaptive(&emb, AccuracyTarget::Balanced)?;

// Compact (Binary)
let q = quantize_adaptive(&emb, AccuracyTarget::Compact)?;
```

## Batch Processing

Efficiently process multiple embeddings with statistics tracking:

```rust
use lnmp_quant::batch::quantize_batch;

let embeddings = vec![emb1, emb2, emb3];
let result = quantize_batch(&embeddings, QuantScheme::QInt8);

println!("Processed: {}/{}", result.stats.succeeded, result.stats.total);
println!("Time: {:?}", result.stats.total_time);

for q in result.results {
    if let Ok(quantized) = q {
        // Use quantized vector
    }
}
```

For detailed benchmarks, see [PERFORMANCE.md](PERFORMANCE.md).

## Quantization Schemes

### FP16Passthrough: Near-Lossless (2x)

- **Compression**: 2x (half-precision float)
- **Accuracy**: ~99.9% (near-lossless)
- **Use Case**: High accuracy with moderate space savings
- **Status**: ✅ Production Ready
- **Note**: Hardware-accelerated on modern GPUs/CPUs

### QInt8: High Accuracy (4x)

- **Range**: -128 to 127 (8-bit signed)
- **Compression**: 4x (F32 → Int8)
- **Accuracy**: ~99% cosine similarity
- **Use Case**: General purpose, high accuracy needed
- **Status**: ✅ Production Ready

### QInt4: Balanced (8x)

- **Range**: 0 to 15 (4-bit unsigned, nibble-packed)
- **Compression**: 8x (2 values per byte)
- **Accuracy**: ~95-97% cosine similarity
- **Use Case**: Large-scale storage, balanced compression
- **Status**: ✅ Production Ready

### Binary: Maximum Compression (32x)

- **Range**: {0, 1} (1-bit sign-based)
- **Compression**: 32x (8 values per byte)
- **Accuracy**: ~85-90% similarity preservation
- **Use Case**: Similarity search, ANN indexing, maximum compression
- **Status**: ✅ Production Ready
- **Note**: Dequantizes to normalized +1/-1 values

### FP16Passthrough: Near-Lossless (2x)

- **Compression**: 2x (half-precision float)
- **Accuracy**: ~99.9% (near-lossless)
- **Status**: 🔜 Roadmap

## How It Works

### Quantization Algorithm (QInt8)

1. **Min/Max Calculation**: Find value range `[min_val, max_val]`
2. **Scale Computation**: `scale = (max_val - min_val) / 255`
3. **Normalization**: `normalized = (value - min_val) / scale`
4. **Quantization**: `quantized = int8(normalized - 128)`
5. **Storage**: Pack into byte vector with metadata

### Dequantization

1. **Unpack**: Read quantized bytes as `i8` values
2. **Reconstruction**: `value = (quantized + 128) * scale + min_val`
3. **Return**: F32 vector with approximate values

## Use Cases

### 🤖 Robot Control

```rust
// Brake sensitivity embedding quantized for microsecond transfer
let brake_embedding = Vector::from_f32(sensor_data);
let quantized = quantize_embedding(&brake_embedding, QuantScheme::QInt8)?;
// Send over low-latency channel
send_to_controller(&quantized);
```

### 🧠 Multi-Agent Systems

```rust
// 30 agents sharing embedding pool with minimal bandwidth
for agent in agents {
    let q_emb = quantize_embedding(&agent.embedding(), QuantScheme::QInt8)?;
    broadcast_to_pool(agent.id, q_emb);
}
```

### 🌐 Edge AI

```rust
// Low bandwidth, high intelligence
let edge_embedding = get_local_embedding();
let quantized = quantize_embedding(&edge_embedding, QuantScheme::QInt8)?;
// 4x smaller payload for network transfer
send_to_cloud(&quantized);
```

## API Reference

### Main Functions

#### `quantize_embedding`

```rust
pub fn quantize_embedding(
    emb: &Vector,
    scheme: QuantScheme
) -> Result<QuantizedVector, QuantError>
```

Quantizes an F32 embedding vector using the specified scheme.

#### `dequantize_embedding`

```rust
pub fn dequantize_embedding(
    q: &QuantizedVector
) -> Result<Vector, QuantError>
```

Dequantizes back to approximate F32 representation.

### Types

#### `QuantizedVector`

```rust
pub struct QuantizedVector {
    pub dim: u32,              // Vector dimension
    pub scheme: QuantScheme,   // Quantization scheme used
    pub scale: f32,            // Scaling factor
    pub zero_point: i8,        // Zero-point offset
    pub min_val: f32,          // Minimum value (for reconstruction)
    pub data: Vec<u8>,         // Packed quantized data
}
```

#### `QuantScheme`

```rust
pub enum QuantScheme {
    QInt8,              // 8-bit signed quantization
    QInt4,              // 4-bit packed (future)
    Binary,             // 1-bit sign-based (future)
    FP16Passthrough,    // Half-precision float (future)
}
```

#### `QuantError`

```rust
pub enum QuantError {
    InvalidDimension(String),
    InvalidScheme(String),
    DataCorrupted(String),
    EncodingFailed(String),
    DecodingFailed(String),
}
```

## Performance

Benchmarks on standard hardware (512-dimensional embeddings):

### QInt8 (Optimized)
```
quantize_512dim      time: [1.17 µs]
dequantize_512dim    time: [457 ns]
roundtrip_512dim     time: [1.63 µs]
accuracy             cosine: >0.99
```

### QInt4 (Nibble-Packed)
```
quantize_512dim      time: [~600 ns]
dequantize_512dim    time: [~230 ns]
compression          ratio: 8.0x
accuracy             cosine: >0.95
```

### Binary (Bit-Packed)
```
quantize_512dim      time: [~200 ns]
dequantize_512dim    time: [~100 ns]
compression          ratio: 32.0x
accuracy             similarity: >0.85
```

Run benchmarks:

```bash
cargo bench -p lnmp-quant
```

## Examples

See [`examples/`](examples/) directory:

- [`quant_basic.rs`](examples/quant_basic.rs) - Basic quantization/dequantization
- [`lnmp_integration.rs`](examples/lnmp_integration.rs) - Integration with LNMP records
- [`quant_debug.rs`](examples/quant_debug.rs) - Debugging quantization behavior

Run an example:

```bash
cargo run -p lnmp-quant --example lnmp_integration
```

## Testing

```bash
# Run all tests
cargo test -p lnmp-quant

# Run roundtrip tests only
cargo test -p lnmp-quant --test quant_roundtrip

# Run accuracy tests
cargo test -p lnmp-quant --test accuracy_tests
```

## Roadmap

### Completed ✅
- [x] QInt8 quantization (4x compression)
- [x] QInt4 packed quantization (8x compression)
- [x] Binary (1-bit) quantization (32x compression)
- [x] LNMP TypeHint integration (`:qv`)
- [x] Comprehensive test suite (32 tests)
- [x] Benchmark suite with Criterion
- [x] Sub-microsecond quantization performance
- [x] Codec integration (text & binary)

### Future Enhancements
- [ ] FP16 passthrough (2x, near-lossless)
- [ ] SIMD optimization (AVX2/NEON)
- [ ] GPU-accelerated quantization
- [ ] Adaptive quantization (auto-select scheme)
- [ ] Batch quantization APIs

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Related Crates

- [`lnmp-core`](../lnmp-core) - Core LNMP type definitions
- [`lnmp-embedding`](../lnmp-embedding) - Vector embedding support with delta encoding
- [`lnmp-codec`](../lnmp-codec) - Binary codec for LNMP protocol
