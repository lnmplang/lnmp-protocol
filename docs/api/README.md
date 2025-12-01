# LNMP API Documentation

Comprehensive API reference for all LNMP implementations.

## 📚 Documentation Structure

### Rust Core API
- **[v0.5 (Latest)](./rust/v0.5.md)** - Advanced features: Binary nested structures, streaming, delta encoding
- **[v0.3 (Legacy)](./rust/v0.3.md)** - Core types, parser, encoder, semantic fidelity

### Language SDKs
- **[TypeScript SDK](../../sdk/js/packages/lnmp/API.md)** - WASM-based TypeScript/JavaScript SDK
- **[Python SDK](../../sdk/python/README.md)** - PyO3-based native Python bindings
- **[Rust SDK](../../sdk/rust/README.md)** - Meta-crate for direct Rust usage
- **[Go SDK](../../sdk/go/README.md)** - (Coming soon)

### Tools & CLIs
- **[CLI Tools](../../tools/cli/README.md)** - Command-line interface
- **[MCP Server](../../tools/mcp/README.md)** - Model Context Protocol server

---

## 🗺️ Quick Navigation

### Core Concepts
- [Types & Values](./rust/v0.5.md#types)
- [Records & Fields](./rust/v0.5.md#records)
- [Checksums](./rust/v0.5.md#checksums)

### Encoding & Decoding
- [Parser](./rust/v0.5.md#parser)
- [Encoder](./rust/v0.5.md#encoder)
- [Binary Format](./rust/v0.5.md#binary-nested-structures)

### Advanced Features (v0.5)
- [Streaming](./rust/v0.5.md#streaming-frame-layer)
- [Delta Encoding](./rust/v0.5.md#delta-encoding-layer)
- [Schema Negotiation](./rust/v0.5.md#schema-negotiation-layer)

### Optimization
- [LLM Bridge (LLB)](./rust/v0.5.md#llb2-optimization-layer)
- [Semantic Fidelity (SFE)](./rust/v0.3.md#semantic-dictionary)

---

## 📖 Version Guide

| Version | Status | Features | Recommended For |
|---------|--------|----------|-----------------|
| v0.5 | ⭐ Latest | Nested structures, streaming, delta | Production use |
| v0.3 | Legacy | Core protocol, basic features | Legacy systems |

---

## 🚀 Getting Started

### Rust

```rust
use lnmp::prelude::*;

// Parse LNMP text
let mut parser = Parser::new("F12=14532\nF7=1")?;
let record = parser.parse_record()?;

// Encode to text
let encoder = Encoder::new();
let output = encoder.encode(&record);
```

### TypeScript

```typescript
import { Core, Encoder } from '@lnmplang/lnmp';

// Parse LNMP text
const record = Core.parse("F12=14532\nF7=1");

// Encode to text
const output = Encoder.encode(record);
```

### Python

```python
import lnmp

# Parse LNMP text
record = lnmp.parse("F12=14532\nF7=1")

# Encode to text
output = lnmp.encode(record)
```

---

## 🔗 External Resources

- **[LNMP Specification](../../spec/)** - Formal protocol specification
- **[Grammar Reference](../../spec/grammar.md)** - PEG grammar
- **[Examples](../../examples/)** - Code examples
- **[Crates.io](https://crates.io/crates/lnmp)** - Rust crate documentation

---

## 📝 Contributing

To update or add API documentation:

1. **Rust API**: Edit files in `docs/api/rust/`
2. **SDK API**: Edit SDK-specific docs in respective SDK directories
3. **Cross-reference**: Use relative links to keep docs connected

## License

MIT License - See [LICENSE](../../LICENSE) for details
