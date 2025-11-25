# LNMP Protocol Implementation

LNMP (LLM Native Minimal Protocol) is a minimal, tokenizer-friendly, semantic-ID-based data format designed for data exchange with large language models (LLMs).

[![Crates.io](https://img.shields.io/crates/v/lnmp.svg)](https://crates.io/crates/lnmp)
[![Documentation](https://docs.rs/lnmp/badge.svg)](https://docs.rs/lnmp)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org)
[![Downloads](https://img.shields.io/crates/d/lnmp.svg)](https://crates.io/crates/lnmp)

**Current Version: v0.5.12

## Features

- 🚀 **Minimal syntax** - 7-12× token reduction compared to JSON
- 🔢 **Semantic IDs** - Uses numeric field IDs for efficient learning
- 📖 **Human-readable** - Text-based format that's easy to read and debug
- 🦀 **Pure Rust** - Memory-safe implementation with minimal dependencies
- 🏗️ **Nested structures** - Support for hierarchical data modeling (v0.3)
- 🔒 **Semantic checksums** - SC32 checksums for drift prevention (v0.3)
- 🎯 **LLM-optimized** - Prompt visibility optimization and ShortForm encoding (v0.3)
- ⚡ **Binary protocol** - Efficient binary encoding with 30-50% size reduction (v0.4)
- 🔄 **Bidirectional conversion** - Seamless text ↔ binary transformation (v0.4)
- 📦 **VarInt encoding** - Space-efficient integer representation (v0.4)
- 🌳 **Binary nested structures** - Recursive encoding for hierarchical data (v0.5)
- 📡 **Streaming support** - Chunked transmission for large payloads (v0.5)
- 🤝 **Schema negotiation** - Capability exchange and version negotiation (v0.5)
- 🔄 **Delta encoding** - Bandwidth-efficient incremental updates (v0.5)
- 🧠 **LLB2 optimization** - Enhanced LLM context optimization (v0.5)
- 🧬 **Embedding support** - Native vector embeddings with delta encoding (v0.5)
- 🛡️ **Input sanitization** - Security-focused input validation and sanitization (v0.5)
- 📍 **Spatial awareness** - Physical coordinates and transformations with hybrid protocol (v0.5.2)
- ✅ **Well-tested** - Comprehensive test suite with multi-language compliance tests
- 📉 **Quantization** - Adaptive vector quantization (FP16, Int8, Int4, Binary) with batch processing (v0.5.4)
- 🔧 **Generic Arrays** - IntArray, FloatArray, BoolArray for efficient numeric data (v0.5.5)
- 🎯 **Strict Profiles** - Configurable validation levels (Loose/Standard/Strict) (v0.5.5)
- 🏗️ **RecordBuilder** - Fluent API for canonical record construction (v0.5.5)
- 🧠 **Context Profiling** - Automatic scoring for LLM prioritization (freshness, importance, risk, confidence) (v0.5.7)
- 🚀 **Transport bindings** - Standard mappings for HTTP, Kafka, gRPC, NATS with W3C Trace Context and OpenTelemetry integration (v0.5.7)

## Quick Start

### Installation

Add LNMP to your `Cargo.toml`:

```toml
[dependencies]
lnmp = "0.5.12"
```

Or use cargo:

```bash
cargo add lnmp
```

### Basic Usage

```rust
use lnmp::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse LNMP text
    let text = "F12=14532;F7=1;F23=[admin,dev]";
    let mut parser = Parser::new(text)?;
    let record = parser.parse_record()?;
    
    // Access fields
    if let Some(field) = record.get_field(12) {
        println!("User ID: {:?}", field.value);
    }
    
    // Create records with RecordBuilder
    let new_record = RecordBuilder::new()
        .add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) })
        .add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) })
        .build();
    
    // Encode to text
    let encoder = Encoder::new();
    let output = encoder.encode(&new_record);
    println!("{}", output); // F7=1\nF12=14532
    
    Ok(())
}
```

For more examples, see the [examples directory](examples/).

## Project Structure

This is a Rust workspace containing multiple crates:

- **lnmp** (v0.5.7): **Meta crate** - All-in-one package that re-exports all LNMP modules (recommended for most users)
- **lnmp-core** (v0.5.7): Core type definitions for LNMP data structures (including nested structures and checksums)
- **lnmp-codec** (v0.5.7): Parser and encoder implementations for LNMP text format (with normalization and equivalence mapping)
- **lnmp-embedding** (v0.5.7): Vector embedding support with efficient delta encoding
- **lnmp-envelope** (v0.5.7): Operational metadata envelope (timestamp, source, trace_id, sequence) - CloudEvents/Kafka/OTel aligned
- **lnmp-spatial** (v0.5.7): Spatial awareness types and hybrid protocol for robotics and real-time control
- **lnmp-quant** (v0.5.7): Adaptive quantization and compression for embedding vectors
- **lnmp-llb** (v0.5.7): LNMP-LLM Bridge Layer - prompt optimization, explain mode, and ShortForm encoding
- **lnmp-sfe** (v0.5.7): Semantic Fidelity Engine - semantic dictionary, equivalence mapping, and **context profiling for LLM decision support**
- **lnmp-net** (v0.5.7): Network behavior layer - semantic message types (Event/Alert/etc), QoS, and intelligent routing (ECO profile)
- **lnmp-sanitize** (v0.5.7): Security-focused input validation and sanitization
- **lnmp-transport** (v0.5.7): Transport bindings for HTTP, Kafka, gRPC, NATS - W3C Trace Context integration and observability support


## Quick Start

### Recommended: Use the Meta Crate (All-in-One)

The easiest way to get started is with the `lnmp` meta crate, which includes all LNMP modules:

```toml
[dependencies]
lnmp = "0.5.12"
```

Then use the convenient prelude:

```rust
use lnmp::prelude::*;

// Parse LNMP text
let lnmp_text = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
let mut parser = Parser::new(lnmp_text).unwrap();
let record = parser.parse_record().unwrap();

// Encode LNMP text
let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

let encoder = Encoder::new();
println!("{}", encoder.encode(&record)); // F12=14532
```

**All modules are accessible:**

```rust
use lnmp::{core, codec, embedding, spatial, quant, llb, sfe, sanitize};
```

### Alternative: Individual Crates

For fine-grained control, you can use individual crates:

```toml
[dependencies]
lnmp-core = "0.5.7"
lnmp-codec = "0.5.7"
```

> Developing against a local checkout? Replace the version strings with `path = "../lnmp-protocol/crates/…"`. See `REPO-STRUCTURE.md` for details.

### Parsing LNMP Text Format.

```rust
use lnmp_codec::Parser;

let lnmp_text = r#"F12=14532;F7=1;F23=["admin","dev"]"#;

let mut parser = Parser::new(lnmp_text).unwrap();
let record = parser.parse_record().unwrap();

println!("User ID: {:?}", record.get_field(12).unwrap().value);
```

### Encoding LNMP Text Format

```rust
use lnmp_codec::Encoder;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

let encoder = Encoder::new();
println!("{}", encoder.encode(&record)); // F12=14532
```

### Binary Format (v0.4+)

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

// Encode text to binary
let text = "F7=1\nF12=14532\nF23=[admin,dev]";
let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(text).unwrap();

// Decode binary to text
let decoder = BinaryDecoder::new();
let decoded_text = decoder.decode_to_text(&binary).unwrap();

// Round-trip conversion maintains data integrity
assert_eq!(text, decoded_text);
```

### Binary Nested Structures (v0.5)

```rust
use lnmp_codec::binary::{BinaryNestedEncoder, BinaryNestedDecoder, NestedEncoderConfig};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// Create nested record
let mut inner = LnmpRecord::new();
inner.add_field(LnmpField { fid: 1, value: LnmpValue::Int(42) });

let mut outer = LnmpRecord::new();
outer.add_field(LnmpField {
    fid: 10,
    value: LnmpValue::NestedRecord(Box::new(inner)),
});

// Encode with nested support
let config = NestedEncoderConfig::new().with_max_depth(32);
let encoder = BinaryNestedEncoder::with_config(config);
let binary = encoder.encode_nested_record(&outer).unwrap();
```

### Streaming Large Payloads (v0.5)

```rust
use lnmp_codec::binary::{StreamingEncoder, StreamingConfig};

let config = StreamingConfig::new()
    .with_chunk_size(4096)
    .with_checksums(true);

let mut encoder = StreamingEncoder::with_config(config);

// Stream in chunks
let begin = encoder.begin_stream().unwrap();
let chunk = encoder.write_chunk(&data).unwrap();
let end = encoder.end_stream().unwrap();
```

### Delta Encoding (v0.5)

```rust
use lnmp_codec::binary::{DeltaEncoder, DeltaConfig};

let config = DeltaConfig::new().with_delta_enabled(true);
let encoder = DeltaEncoder::with_config(config);

// Compute delta between records
let delta_ops = encoder.compute_delta(&old_record, &new_record).unwrap();
let delta_binary = encoder.encode_delta(&delta_ops).unwrap();

// 50%+ bandwidth savings for typical updates
```

## LNMP Format

LNMP uses field assignments in the format `F<field_id>=<value>`:

```
# Inline format (semicolon-separated)
F12=14532;F7=1;F23=["admin","dev"]

# Multiline format (canonical)
F7=1
F12=14532
F20="Halil"
F23=["admin","dev"]

# With type hints
F12:i=14532
F7:b=1
F23:sa=["admin","dev"]

# With checksums (v0.3)
F12:i=14532#36AAE667
F7:b=1#A3F2B1C4

# Nested structures (v0.3)
F50={F12=1;F7=1}
F60=[{F1=alice},{F1=bob}]
```

### Supported Types

**Primitive Types:**
- **Integers**: `F1=42`, `F2=-123`
- **Floats**: `F3=3.14`, `F4=-2.5`
- **Booleans**: `F5=1` (true), `F6=0` (false)
- **Strings**: `F7="hello world"`, `F8=simple_string`
- **String Arrays**: `F9=["a","b","c"]`
- **Numeric Arrays**: `F10=[1,2,3]` (IntArray), `F11=[1.1,2.2]` (FloatArray), `F12=[1,0,1]` (BoolArray)

**Nested Types (v0.3):**
- **Nested Records**: `F50={F12=1;F7=1}` - Records within records
- **Nested Arrays**: `F60=[{F1=alice},{F1=bob}]` - Arrays of records

### Escape Sequences

Quoted strings support escape sequences:
- `\"` - Double quote
- `\\` - Backslash
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab

## v0.3 Features

### Semantic Fidelity Engine (SFE)

Prevent LLM input drift with semantic checksums:

```rust
use lnmp_codec::{Encoder, EncoderConfig};

let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
// Output: F12:i=14532#36AAE667
```

**Features:**
- SC32 checksums (32-bit CRC32-based)
- Value normalization (booleans, floats, strings)
- Equivalence mapping (synonym recognition)

### Structural Extensibility Layer (SEL)

Model hierarchical data with nested structures:

```rust
// Nested record
let input = "F50={F12=1;F7=1}";

// Nested array
let input = "F60=[{F1=alice},{F1=bob}]";

// Deep nesting
let input = "F100={F1=user;F2={F10=nested;F11=data}}";
```

**Features:**
- Arbitrary nesting depth
- Structural canonicalization
- Deterministic field ordering at all levels

### LNMP-LLM Bridge Layer (LLB)

Optimize for LLM tokenization:

```rust
// Explain mode - human-readable comments
F12:i=14532  # user_id
F7:b=1       # is_active

// ShortForm - extreme token reduction
12=14532 7=1 23=[admin,dev]
```

**Features:**
- Prompt visibility optimization
- Explain mode encoding
- ShortForm encoding (7-12× reduction vs JSON)

### Zero-Ambiguity Grammar (ZAG)

Formal grammar specification for multi-language implementations:

- PEG grammar specification
- EBNF specification
- Error classification system
- Multi-language compliance test suite (Rust, Python, TypeScript, C++)

## v0.5 Features

### Binary Nested Structures (BNS)

Encode hierarchical data in binary format with depth and size validation:

```rust
use lnmp_codec::binary::{BinaryNestedEncoder, NestedEncoderConfig};

let config = NestedEncoderConfig::new()
    .with_max_depth(32)
    .with_max_record_size(Some(1_000_000));

let encoder = BinaryNestedEncoder::with_config(config);
let binary = encoder.encode_nested_record(&nested_record).unwrap();
```

**Features:**
- TypeTag 0x06 for nested records
- TypeTag 0x07 for nested arrays
- Configurable depth limits (default: 32)
- Optional size limits for security
- Automatic canonical ordering

### Streaming Frame Layer (SFL)

Stream large payloads in chunks with integrity checking:

```rust
use lnmp_codec::binary::{StreamingEncoder, StreamingConfig};

let config = StreamingConfig::new()
    .with_chunk_size(4096)
    .with_checksums(true);

let mut encoder = StreamingEncoder::with_config(config);

// BEGIN frame
let begin = encoder.begin_stream().unwrap();

// CHUNK frames
let chunk = encoder.write_chunk(&data).unwrap();

// END frame
let end = encoder.end_stream().unwrap();
```

**Features:**
- Frame types: BEGIN (0xA0), CHUNK (0xA1), END (0xA2), ERROR (0xA3)
- XOR checksum validation
- Backpressure flow control
- Configurable chunk size (default: 4KB)
- Error recovery

### Schema Negotiation Layer (SNL)

Negotiate capabilities before data exchange:

```rust
use lnmp_codec::binary::{SchemaNegotiator, Capabilities, FeatureFlags};

let features = FeatureFlags {
    supports_nested: true,
    supports_streaming: true,
    supports_delta: true,
    supports_llb: true,
    requires_checksums: true,
    requires_canonical: true,
};

let caps = Capabilities {
    version: 5,
    features,
    supported_types: vec![/* ... */],
};

let mut negotiator = SchemaNegotiator::new(caps);
let msg = negotiator.initiate().unwrap();
```

**Features:**
- Capability exchange
- Feature flag negotiation
- FID conflict detection
- Type mismatch detection
- Protocol version negotiation
- Graceful degradation

### Delta Encoding & Partial Update Layer (DPL)

Send only changed fields for bandwidth efficiency:

```rust
use lnmp_codec::binary::{DeltaEncoder, DeltaConfig};

let config = DeltaConfig::new().with_delta_enabled(true);
let encoder = DeltaEncoder::with_config(config);

let delta_ops = encoder.compute_delta(&old_record, &new_record).unwrap();
let delta_binary = encoder.encode_delta(&delta_ops).unwrap();
```

**Features:**
- Delta operations: SET_FIELD, DELETE_FIELD, UPDATE_FIELD, MERGE_RECORD
- 50%+ bandwidth savings for typical updates
- Nested record merging
- Incremental update chains

### LLM Optimization Layer v2 (LLB2)

Enhanced context optimization with binary integration:

```rust
use lnmp_llb::{LlbConverter, LlbConfig};

let config = LlbConfig::new()
    .with_flattening(true)
    .with_semantic_hints(true)
    .with_collision_safe_ids(true);

let converter = LlbConverter::new(config);

// Binary to ShortForm
let shortform = converter.binary_to_shortform(&binary).unwrap();

// Flatten nested structures
let flattened = converter.flatten_nested(&nested_record).unwrap();
```

**Features:**
- Binary ↔ ShortForm conversion
- Binary ↔ FullText conversion
- Nested structure flattening
- Semantic hint embedding
- Collision-safe ID generation

## Building

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run examples
cargo run --example parse_example
cargo run --example encode_example
cargo run --example round_trip

# Run v0.3 examples
cargo run --example nested_structures
cargo run --example semantic_checksums
cargo run --example explain_mode
cargo run --example shortform

# Run v0.4 binary format examples
cargo run --example binary_encoding
cargo run --example binary_roundtrip

# Run v0.5 advanced protocol examples
cargo run --example v05_nested_binary
cargo run --example v05_streaming
cargo run --example v05_schema_negotiation
cargo run --example v05_delta_encoding
cargo run --example v05_llb2_binary

# Run compliance tests
cargo test --package lnmp-codec -- spec_compatibility

# Generate documentation
cargo doc --open
```

## Examples

See the `examples/` directory for complete examples:

**Basic Examples:**
- `parse_example.rs` - Parsing LNMP text
- `encode_example.rs` - Creating and encoding records
- `round_trip.rs` - Parse → Encode → Parse round-trip

**v0.2 Examples:**
- `type_hints.rs` - Type hint usage
- `strict_vs_loose.rs` - Parsing mode comparison
- `deterministic_serialization.rs` - Canonical format

**v0.3 Examples:**
- `nested_structures.rs` - Nested records and arrays
- `semantic_checksums.rs` - SC32 checksum usage
- `explain_mode.rs` - Explain mode encoding
- `shortform.rs` - ShortForm encoding/parsing
- `structural_canonicalization.rs` - Structural canonicalization

**v0.4 Examples (Binary Format):**
- `binary_encoding.rs` - Basic binary encoding and decoding
- `binary_roundtrip.rs` - Round-trip conversion and data integrity

**v0.5 Examples (Advanced Protocol):**
- `v05_nested_binary.rs` - Binary nested structures with depth/size validation
- `v05_streaming.rs` - Streaming Frame Layer with backpressure control
- `v05_schema_negotiation.rs` - Capability negotiation and conflict detection
- `v05_delta_encoding.rs` - Delta operations with bandwidth savings
- `v05_llb2_binary.rs` - LLB2 integration with binary format

## Requirements

- Rust 1.70.0 or later
- Cargo (comes with Rust)

## Documentation

- **v0.5 Specification**: See `specs/lnmp-v0.5-advanced-protocol/`
  - `requirements.md` - Formal requirements using EARS patterns
  - `design.md` - Architecture and component design
  - `tasks.md` - Implementation task list
- **v0.5 Migration Guide**: See `MIGRATION_V05.md` for upgrading from v0.4
- **v0.5 API Reference**: See `API_V05.md` for complete API documentation
- **v0.3 Specification**: See `specs/lnmp-v0.3-semantic-fidelity/`
- **Grammar**: See `spec/grammar.md` for PEG/EBNF specification
- **Error Classes**: See `spec/error-classes.md` for error classification
- **API Docs**: Run `cargo doc --open`

## Testing

The project includes comprehensive tests:

- **Unit tests**: Core functionality tests in all crates
- **Integration tests**: Round-trip and cross-crate tests
- **Spec compatibility**: Tests verifying spec compliance
- **Compliance tests**: Multi-language test suite (Rust, Python, TypeScript, C++)

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package lnmp-codec
cargo test --package lnmp-core
cargo test --package lnmp-sfe
cargo test --package lnmp-llb

# Run compliance tests
cd tests/compliance/rust && cargo test
cd tests/compliance/python && pytest
cd tests/compliance/typescript && npm test
cd tests/compliance/cpp && make test
```

## Performance

- Zero-copy where possible
- No regex dependencies
- Hand-written lexer for optimal performance
- Minimal allocations during parsing
- SC32 checksum computation: <1μs per field
- Nested parsing: <10μs for 3-level nesting
- Token reduction: 7-12× vs JSON (ShortForm mode)
- Binary nested encoding: <2μs per field (v0.5)
- Streaming overhead: <5% vs non-streaming (v0.5)
- Delta encoding savings: >50% for typical updates (v0.5)

## Roadmap

### v0.3 - Semantic Fidelity & Structural Extensibility ✅
- Semantic checksums (SC32)
- Nested structures (records and arrays)
- Value normalization and equivalence mapping
- Formal PEG/EBNF grammar
- Multi-language compliance test suite
- LLM-optimized encoding (explain mode, ShortForm)

### v0.4 - Binary Protocol & Transport Layer ✅
- Binary protocol format with version 0x04
- VarInt encoding for space-efficient integers
- Bidirectional text ↔ binary conversion
- 30-50% size reduction compared to text format
- Canonical binary form with deterministic field ordering
- Type-safe binary encoding with explicit type tags
- Round-trip conversion guarantees

### v0.5 - Advanced Protocol & M2M Transport ✅ (Current)
- **Binary Nested Structures (BNS)**: Recursive encoding with TypeTag 0x06/0x07
- **Streaming Frame Layer (SFL)**: Chunked transmission with BEGIN/CHUNK/END frames
- **Schema Negotiation Layer (SNL)**: Capability exchange and version negotiation
- **Delta Encoding & Partial Update Layer (DPL)**: Bandwidth-efficient incremental updates
- **LLM Optimization Layer v2 (LLB2)**: Enhanced context optimization with binary integration
- Depth validation (default 32 levels)
- Size limits for security
- XOR checksum validation
- Backpressure flow control
- FID conflict detection
- Type mismatch detection
- 50%+ bandwidth savings with delta encoding

### v0.6 (0.5.2) - LNMP-Spatial Protocol ✅ (Current)
- **Spatial Type System**: Position, Rotation, Velocity, Acceleration, Quaternion, BoundingBox
- **Binary Spatial Codec**: 2-3ns encode/decode latency
- **Delta Encoding**: Position/Rotation deltas with 99% bandwidth reduction
- **Hybrid Protocol**: Automatic ABS/DELTA mixing (1% ABS, 99% DELTA)
- **Predictive Delta**: Dead reckoning for packet loss resilience
- **Frame Integrity**: CRC32 checksums and nanosecond timestamps
- **High Frequency**: Verified at 1kHz control loops
- **Streaming Telemetry**: Continuous robot state transmission
- **Safety Features**: Configurable prediction limits, drift correction
- **Use Cases**: Robot control, autonomous vehicles, AR/VR, simulation

#### LNMP-Spatial Quick Start

```rust
use lnmp_spatial::protocol::{SpatialStreamer, SpatialStreamerConfig};

// Configure hybrid protocol
let config = SpatialStreamerConfig {
    abs_interval: 100,        // ABS frame every 100 frames (drift correction)
    enable_prediction: true,   // Enable predictive delta (packet loss handling)
    max_prediction_frames: 3,  // Max 3 consecutive predictions
};

let mut streamer = SpatialStreamer::with_config(config);

// Sender: Generate frames
let frame = streamer.next_frame(&robot_state, timestamp_ns)?;
// Automatically uses DELTA (99%) or ABS (1%) based on sequence

// Receiver: Process frames
let state = streamer.process_frame(&frame)?;
// Automatically handles packet loss with prediction fallback
```

**Performance:**
- Encode/Decode: **2-3 ns** per operation
- Bandwidth: **99% reduction** with DELTA
- Frequency: **1kHz** control loop verified
- Packet Loss: **Predictive fallback** maintains smooth operation

**Examples:**
```bash
cargo run --example spatial_robot        # Robot + Embedding integration
cargo run --example spatial_stream       # Continuous telemetry
cargo run --example spatial_jitter_sim   # 1kHz control loop
cargo run --example spatial_reflex_sim   # Prediction vs non-prediction
```


### v0.7 (0.5.4) - Quantization & Compression ✅ (Current)
- **Adaptive Quantization**: Auto-select scheme based on accuracy/compression targets
- **FP16 Passthrough**: Near-lossless 2x compression (~99.9% accuracy)
- **QInt8**: 4x compression with high fidelity (~99% similarity)
- **QInt4**: 8x compression for balanced efficiency (~95% similarity)
- **Binary**: 32x compression for maximum density (~85% similarity)
- **Batch Processing**: Efficient API for processing multiple vectors
- **Zero Overhead**: Adaptive selection compiles to direct calls

#### LNMP-Quant Quick Start

```rust
use lnmp_quant::adaptive::{quantize_adaptive, AccuracyTarget};
use lnmp_quant::batch::quantize_batch;

// Adaptive: Select best scheme for target
let q = quantize_adaptive(&emb, AccuracyTarget::High)?; // Uses QInt8

// Batch: Process multiple vectors efficiently
let results = quantize_batch(&embeddings, QuantScheme::QInt8);
println!("Processed {} vectors in {:?}", results.stats.total, results.stats.total_time);
```

**Performance:**
- **Adaptive Overhead**: Zero (negligible)
- **Batch Overhead**: <150ns per vector
- **Throughput**: >2M vectors/sec (FP16)

### v0.5.4 - Determinism & Generic Arrays ✅ (Current)
- **Generic Arrays**: `IntArray`, `FloatArray`, `BoolArray` for efficient numeric data
- **Strict Determinism**: `LnmpProfile` (Loose, Standard, Strict) for validation
- **RecordBuilder**: Fluent API for canonical record construction
- **Enhanced Validation**: Runtime field ordering checks
- **Enhanced Validation**: Runtime field ordering checks
- **Performance**: Sub-microsecond operations for all core types

### v0.5.7 - LNMP-Net & Intelligent Routing ✅ (Current)
- **Semantic Message Types**: Event, State, Command, Query, Alert
- **QoS Primitives**: Priority (0-255) and TTL for network behavior
- **ECO Profile**: Energy/Token Optimization routing (90% reduction in LLM calls)
- **Intelligent Routing**: Route based on importance score + freshness
- **Transport Integration**: Header mappings for HTTP, Kafka, NATS, gRPC

## Migration Guide

### From v0.4 to v0.5

v0.5 is fully backward compatible with v0.4 and v0.3. All existing code continues to work without changes.

**New Features (Optional):**
- Binary nested structures: `BinaryNestedEncoder` and `BinaryNestedDecoder`
- Streaming support: `StreamingEncoder` and `StreamingDecoder`
- Schema negotiation: `SchemaNegotiator` with `Capabilities`
- Delta encoding: `DeltaEncoder` and `DeltaDecoder`
- Enhanced LLB2: `LlbConverter` with binary integration

**No Breaking Changes:**
- All v0.4 binary format remains valid
- All v0.3 text format remains valid
- Existing encoders/decoders work unchanged
- New features are opt-in via configuration

**Recommended Updates:**
1. Use binary nested structures for hierarchical data
2. Enable streaming for large payloads (>4KB)
3. Add schema negotiation for new integrations
4. Use delta encoding for frequent updates
5. Apply LLB2 flattening for LLM consumption

**See `MIGRATION_V05.md` for detailed migration guide.**

### From v0.2 to v0.3

v0.3 is fully backward compatible with v0.2. All v0.2 code continues to work without changes.

**New Features (Optional):**
- Enable checksums: `EncoderConfig { enable_checksums: true, .. }`
- Use nested structures: `LnmpValue::NestedRecord` and `LnmpValue::NestedArray`
- Apply normalization: `EncoderConfig { normalization_config: Some(..), .. }`
- Add equivalence mapping: `EncoderConfig { equivalence_mapper: Some(..), .. }`

See individual crate READMEs for detailed migration guides.

## Contributing

Contributions are welcome! Please see the following:

- **Issues**: Report bugs or request features on GitHub
- **Pull Requests**: Submit PRs with tests and documentation
- **Compliance Tests**: Add test cases to `tests/compliance/test-cases.yaml`
- **Language Implementations**: Follow the compliance test suite for new languages

## License

MIT OR Apache-2.0
