# LNMP - LLM Native Minimal Protocol

[![Crates.io](https://img.shields.io/crates/v/lnmp.svg)](https://crates.io/crates/lnmp)
[![Documentation](https://docs.rs/lnmp/badge.svg)](https://docs.rs/lnmp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)

**LNMP** is a meta crate that provides unified access to all modules of the LLM Native Minimal Protocol ecosystem. Instead of managing multiple dependencies, simply add `lnmp` to your project and get everything you need.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
lnmp = "0.5.4"
```

Then use any LNMP module:

```rust
use lnmp::prelude::*;
use lnmp::core::{Message, MessageType};
use lnmp::codec::{encode, decode};
use lnmp::embedding::VectorDelta;
use lnmp::spatial::SpatialStreamer;
```

## What's Included

This meta crate re-exports all LNMP modules:

| Module | Description |
|--------|-------------|
| **`core`** | Core type definitions and protocol structures |
| **`codec`** | Encoding and decoding functionality |
| **`embedding`** | Embedding vector operations and delta compression |
| **`envelope`** | Secure message enveloping and integrity |
| **`llb`** | Large Language Block operations |
| **`quant`** | Quantization utilities for efficient data representation |
| **`sanitize`** | Data sanitization and validation |
| **`sfe`** | Secure Function Evaluation primitives |
| **`spatial`** | Spatial data streaming and hybrid protocols |
| **`transport`** | Transport protocol bindings (HTTP, Kafka, gRPC, NATS) |
| **`net`** | Network behavior layer (MessageKind, QoS, ECO routing) |

## Why Use the Meta Crate?

### Before (Multiple Dependencies)
```toml
[dependencies]
lnmp-core = "0.5.4"
lnmp-codec = "0.5.4"
lnmp-embedding = "0.5.4"
lnmp-spatial = "0.5.4"
# ... which ones do I need?
```

### After (Single Dependency)
```toml
[dependencies]
lnmp = "0.5.4"  # Everything included!
```

## Usage Examples

### Basic Message Encoding

```rust
use lnmp::prelude::*;

let message = Message::new(MessageType::Text, b"Hello, LNMP!");
let encoded = encode(&message)?;
let decoded = decode(&encoded)?;
```

### Embedding Delta Compression

```rust
use lnmp::embedding::{VectorDelta, DeltaMode};

let old_vector = vec![1.0, 2.0, 3.0];
let new_vector = vec![1.1, 2.0, 3.2];

let delta = VectorDelta::compute(&old_vector, &new_vector, DeltaMode::Sparse)?;
```

### Spatial Streaming

```rust
use lnmp::spatial::{SpatialStreamer, HybridProtocol};

let streamer = SpatialStreamer::new(HybridProtocol::default());
// Stream spatial data efficiently...
```

### Network Routing

```rust
use lnmp::net::{NetMessage, MessageKind, RoutingPolicy};

// Create a network message
let msg = NetMessage::new(envelope, MessageKind::Alert);

// Make intelligent routing decision
let policy = RoutingPolicy::default();
let decision = policy.decide(&msg, now_ms)?;
```

## Individual Modules

If you prefer fine-grained control and only need specific functionality, you can still depend on individual crates:

- [`lnmp-core`](https://crates.io/crates/lnmp-core) - Core protocol definitions
- [`lnmp-codec`](https://crates.io/crates/lnmp-codec) - Encoding/decoding
- [`lnmp-embedding`](https://crates.io/crates/lnmp-embedding) - Embedding operations
- [`lnmp-llb`](https://crates.io/crates/lnmp-llb) - Large Language Blocks
- [`lnmp-quant`](https://crates.io/crates/lnmp-quant) - Quantization
- [`lnmp-sanitize`](https://crates.io/crates/lnmp-sanitize) - Sanitization
- [`lnmp-sfe`](https://crates.io/crates/lnmp-sfe) - Secure Function Evaluation
- [`lnmp-spatial`](https://crates.io/crates/lnmp-spatial) - Spatial streaming
- [`lnmp-transport`](https://crates.io/crates/lnmp-transport) - Transport bindings
- [`lnmp-net`](https://crates.io/crates/lnmp-net) - Network behavior layer

## Documentation

- [API Documentation](https://docs.rs/lnmp)
- [Protocol Specification](https://github.com/lnmplang/lnmp-protocol)
- [Migration Guide](https://github.com/lnmplang/lnmp-protocol/blob/main/MIGRATION.md)

## License

MIT License - see [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
