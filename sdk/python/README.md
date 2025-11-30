# LNMP Python SDK

Python bindings for the **LNMP (LLM Native Minimal Protocol)** - an efficient, deterministic protocol designed for LLM context optimization.

## 🚀 Quick Start

### Installation

```bash
pip install lnmp
```

### Basic Usage

```python
import lnmp

# Parse LNMP text
record = lnmp.core.parse("F12=14532;F7=1")

# Encode to text
text = record.encode()

# Binary encoding
binary = record.encode_binary()

# Wrap with envelope
envelope = lnmp.envelope.wrap(record, source="my-service")

# Context scoring
score = lnmp.net.context_score(envelope)
print(f"Composite score: {score.composite}")

# Routing decision
if lnmp.net.should_send_to_llm(envelope, threshold=0.7):
    # Send to LLM
    pass
```

## 📦 Modules

### Core (`lnmp.core`)

Parse and encode LNMP records.

```python
# Parse from text
record = lnmp.core.parse("F12=14532;F7=1")

# Encode to text
text = record.encode()

# Binary encoding/decoding
binary = record.encode_binary()
decoded = lnmp.core.decode_binary(binary)
```

### Envelope (`lnmp.envelope`)

Wrap records with operational metadata.

```python
envelope = lnmp.envelope.wrap(
    record,
    source="auth-service",
    timestamp_ms=1234567890,
    trace_id="trace-123"
)
```

### Network (`lnmp.net`)

Context scoring and routing decisions.

```python
# Score envelope
score = lnmp.net.context_score(envelope)
print(score.composite)  # 0.0-1.0

# Routing decision
decision = lnmp.net.routing_decide(envelope)

# Helper
if lnmp.net.should_send_to_llm(envelope, threshold=0.7):
    send_to_llm(envelope)
```

### LLM Workflows (`lnmp.llm`)

High-level helpers for common workflows.

```python
result = lnmp.llm.normalize_and_route(
    "F12=14532",
    source="api-gateway",
    threshold=0.7
)

if result["send_to_llm"]:
    process_with_llm(result["envelope"])
```

### Embedding (`lnmp.embedding`)

Vector operations and delta compression.

```python
base = [0.1, 0.2, 0.3]
updated = [0.1, 0.25, 0.3]

delta_info = lnmp.embedding.delta(base, updated)
print(delta_info["change_count"])
```

### Spatial (`lnmp.spatial`)

Spatial data encoding and streaming.

```python
# Encode 3D position
data = lnmp.spatial.encode_position3d(1.5, 2.5, 3.5)

# Decode
x, y, z = lnmp.spatial.decode_position3d(data)
```

### Utils (`lnmp.utils`)

Utility functions for quantization, sanitization, etc.

```python
# Quantize vectors
quantized = lnmp.utils.quantize([0.1, 0.2, 0.3], "QInt8")

# Sanitize input
clean = lnmp.utils.sanitize("F12= 14532 ; F7=1")

# Debug explain
explained = lnmp.utils.debug_explain("F12=14532")
```

## 🎯 Complete Example

```python
import lnmp

# High-level workflow
result = lnmp.llm.normalize_and_route(
    "F12=14532;F7=1",
    source="my-service",
    trace_id="req-001",
    threshold=0.7
)

print(f"Score: {result['score'].composite:.3f}")
print(f"Decision: {result['decision']}")

if result['send_to_llm']:
    # Route to LLM
    send_to_llm(result['envelope'])
else:
    # Process locally
    process_locally(result['record'])
```

## 🏗️ Architecture

The Python SDK uses a two-layer architecture:

1. **`lnmp-python`** (Rust layer): Native extension built with PyO3, exposing core LNMP functionality
2. **`lnmp`** (Python layer): Pythonic API wrapper providing a developer-friendly interface

This ensures:
- ✅ **Performance**: Core operations run at native Rust speed
- ✅ **Determinism**: Identical behavior across all LNMP SDKs
- ✅ **Developer UX**: Clean, Pythonic API

## 📚 Examples

See the `examples/` directory for more:

- `basic_usage.py` - Simple parse/encode operations
- `complete_workflow.py` - Full LLM routing workflow
- (More examples in the repository)

## 🔧 Development

### Build from Source

```bash
# Clone repository
git clone https://github.com/lnmplang/lnmp-protocol
cd lnmp-protocol/sdk/python

# Install maturin
pip install maturin

# Build and install
maturin develop

# Run tests
pytest tests/
```

## 📖 Documentation

- [LNMP Protocol Spec](https://lnmp.io/spec)
- [API Reference](https://lnmp.io/docs/python)
- [GitHub Repository](https://github.com/lnmplang/lnmp-protocol)

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](../../LICENSE) for details.

## 🆚 Version

Current version: **0.5.7**

Synchronized with LNMP Rust crate version for consistency.
