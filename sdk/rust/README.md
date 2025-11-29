# LNMP SDK for Rust

The official Rust SDK for the **LLM Native Minimal Protocol (LNMP)**.

This SDK provides a unified entry point to the entire LNMP ecosystem, leveraging the `lnmp` meta crate to give you access to all protocol features, from basic encoding to advanced spatial streaming and neural embeddings.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
lnmp = "0.5.12"
```

> [!NOTE]
> Most users should depend directly on the `lnmp` crate from [crates.io](https://crates.io/crates/lnmp). This SDK repository serves as a documentation hub and example collection.

## Quick Start

```rust
use lnmp::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple text token
    let token = Token::text("Hello LNMP");
    
    // Encode it
    let encoded = token.to_lnmp()?;
    println!("Encoded: {:?}", encoded);
    
    Ok(())
}
```

## Modules Overview

The SDK exposes several powerful modules via the `lnmp` crate:

### 1. Core (`lnmp::core`)
The foundation of the protocol. Handles basic types, serialization traits, and the unified `Token` enum.

**Key Features:**
- Token-based data representation
- LNMP encoding/decoding
- Type-safe serialization

### 2. Embedding (`lnmp::embedding`)
Optimized vector operations for AI applications.

**Key Features:**
- Multiple quantization types (F32, F16, I8, Binary)
- **Delta Mode**: Efficiently transmit only changes
- Dimension validation

```rust
use lnmp::embedding::Vector;

let vec = Vector::from_f32(vec![0.1, 0.2, 0.3]);
```

### 3. Spatial (`lnmp::spatial`)
Protocol extensions for 3D environments, robotics, and AR/VR.

**Key Features:**
- Pose tracking (position + orientation)
- SpatialStreamer for efficient updates
- Predictive delta compression

### 4. Net (`lnmp::net`)
Networking primitives for LNMP.

**Key Features:**
- Abstract transport layer
- QUIC support
- Message routing

### 5. Envelope (`lnmp::envelope`)
Metadata wrapping for routing, priority, and distributed tracing.

**Key Features:**
- Priority levels
- Sender/recipient routing
- Custom metadata fields

## Examples

The SDK includes comprehensive examples demonstrating various LNMP features. Each example is a standalone program you can run directly.

### Running Examples

```bash
# Navigate to SDK directory
cd sdk/rust

# Run any example with:
cargo run --example <example_name>
```

### Available Examples

#### 1. Basic Encoding (`basic_encoding.rs`)
Learn the fundamentals of LNMP encoding and decoding.

```bash
cargo run --example basic_encoding
```

**Covers:**
- Creating text, number, float, and boolean tokens
- Encoding to LNMP format
- Building compound structures (records)

#### 2. Embedding Operations (`embedding_ops.rs`)
Work with vector embeddings and delta compression.

```bash
cargo run --example embedding_ops
```

**Covers:**
- Creating F32 vectors
- Computing delta between vectors
- Applying delta patches
- Different quantization types (F16, I8)

#### 3. Spatial Protocol (`spatial_demo.rs`)
3D spatial tracking and streaming for robotics/AR/VR.

```bash
cargo run --example spatial_demo
```

**Covers:**
- Creating spatial poses (position + orientation)
- Tracking multiple objects
- Streaming spatial updates
- LNMP encoding for spatial data

#### 4. Envelope Wrapping (`envelope_demo.rs`)
Add metadata, routing, and priority to messages.

```bash
cargo run --example envelope_demo
```

**Covers:**
- Wrapping messages with envelopes
- Setting priority levels
- Adding routing information (sender/recipient)
- Custom metadata fields

#### 5. Complete Workflow (`complete_workflow.rs`)
Real-world scenario: IoT sensor data pipeline.

```bash
cargo run --example complete_workflow
```

**Covers:**
- Collecting sensor readings
- Generating embeddings for semantic search
- Enriching data with metadata
- Routing through envelopes
- Complete encode-transmit pipeline

#### 6. All Modules Overview (`all_modules.rs`)
Comprehensive demonstration of ALL 11 LNMP modules.

```bash
cargo run --example all_modules
```

**Covers:**
- Complete module inventory
- Core, Codec, Embedding, Envelope
- LLB, Net, Quant, Sanitize
- SFE, Spatial, Transport
- Real examples from each module

### Example Output

All examples include detailed console output showing:
- Step-by-step operations
- Data transformations
- Encoded LNMP format
- Success confirmations

## Advanced Usage

### Custom Protocol Extensions

You can extend LNMP with custom types:

```rust
use lnmp::prelude::*;

// Define custom types using Token::record
let custom_message = Token::record(vec![
    ("type".to_string(), Token::text("custom_sensor")),
    ("data".to_string(), Token::int(42)),
]);
```

### Performance Tips

- Use delta mode for frequently updated embeddings
- Enable quantization (F16/I8) for large vectors
- Batch spatial updates when possible
- Set appropriate envelope priorities

## Resources

- **Main Documentation**: See project [README.md](../../README.md)
- **API Reference**: Run `cargo doc --open` in the workspace root
- **Specification**: Check [spec/](../../spec/) directory
- **More Examples**: Browse [examples/](../../examples/) in the main repository

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file for guidelines.

## License

See [LICENSE](../../LICENSE) for details.
