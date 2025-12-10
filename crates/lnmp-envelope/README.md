# lnmp-envelope

[![Crates.io](https://img.shields.io/crates/v/lnmp-envelope.svg)](https://crates.io/crates/lnmp-envelope)
[![Documentation](https://docs.rs/lnmp-envelope/badge.svg)](https://docs.rs/lnmp-envelope)

Operational metadata envelope for LNMP records, aligned with CloudEvents, Kafka Headers, and W3C Trace Context standards.

## Overview

LNMP Envelope adds operational context (timestamp, source, trace ID, sequence) to LNMP records without affecting deterministic properties or semantic checksums.

### Key Features

- ✅ **Preserves Core Determinism**: Envelope metadata does NOT affect `SemanticChecksum`
- ✅ **Zero Overhead**: Unused envelope features have no performance cost  
- ✅ **Standards Aligned**: Compatible with CloudEvents, Kafka Headers, OpenTelemetry
- ✅ **Transport Agnostic**: Defined independently, bindings for HTTP/Kafka/gRPC
- ✅ **Future Proof**: Extensible via labels, unknown fields skipped gracefully

### Industry Standards Alignment

| Standard | LNMP Envelope Mapping |
|----------|----------------------|
| **CloudEvents** | `time` → `timestamp`, `source` → `source`, `id` → `trace_id` + `sequence` |
| **Kafka Headers** | Record-level metadata separate from payload |
| **W3C Trace Context** | Compatible `trace_id` format for distributed tracing |
| **OpenTelemetry** | Seamless span context propagation |

## Quick Start

```rust
use lnmp_envelope::{EnvelopeBuilder, LnmpRecord, LnmpField, LnmpValue};

// Create a record
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });

// Wrap with envelope
let envelope = EnvelopeBuilder::new(record)
    .timestamp(1732373147000)
    .source("auth-service")
    .trace_id("abc-123-xyz")
    .sequence(42)
    .build();

assert!(envelope.has_metadata());
```

## Encoding Formats

### Binary (TLV)

Envelope metadata encoded as Type-Length-Value entries:

```text
Type: 0x10 (Timestamp) | Length: 8 | Value: u64 BE
Type: 0x11 (Source)    | Length: N | Value: UTF-8 string
Type: 0x12 (TraceID)   | Length: M | Value: UTF-8 string
Type: 0x13 (Sequence)  | Length: 8 | Value: u64 BE
```

**Example:**

```rust
use lnmp_envelope::binary_codec::TlvEncoder;

let binary = TlvEncoder::encode(&envelope.metadata).unwrap();
// 51 bytes: 10 00 08 00 00 01 93 59 7c 6d 78 11 00 0c ...
```

### Text (Header Comment)

```text
#ENVELOPE timestamp=1732373147000 source=auth-service trace_id="abc-123-xyz" sequence=42
F12=14532
F7=1
```

## Transport Bindings

### HTTP

```http
POST /api/records HTTP/1.1
X-LNMP-Timestamp: 1732373147000
X-LNMP-Source: auth-service
X-LNMP-Trace-ID: abc-123-xyz
X-LNMP-Sequence: 42
Content-Type: application/lnmp-binary

<binary LNMP record>
```

### Kafka

```rust
ProducerRecord {
    headers: [
        ("lnmp.timestamp", "1732373147000"),
        ("lnmp.source", "auth-service"),
        ("lnmp.trace_id", "abc-123-xyz"),
        ("lnmp.sequence", "42"),
    ],
    value: <binary LNMP record>,
}
```

### gRPC

```protobuf
metadata {
  "lnmp-timestamp": "1732373147000"
  "lnmp-source": "auth-service"
  "lnmp-trace-id": "abc-123-xyz"
}
```

## Use Cases

### Distributed Tracing

```rust
use opentelemetry::trace::TraceContextExt;

let span = tracer.start("process_record");
let trace_id = span.span_context().trace_id().to_string();

let envelope = EnvelopeBuilder::new(record)
    .trace_id(trace_id)
    .build();
```

### LLM Freshness Scoring

```rust
fn score_freshness(envelope: &LnmpEnvelope, now: u64) -> f64 {
    if let Some(ts) = envelope.metadata.timestamp {
        let age_ms = now.saturating_sub(ts);
        let age_hours = age_ms as f64 / 3_600_000.0;
        (-age_hours / 24.0).exp() // Exponential decay
    } else {
        0.5 // Unknown age
    }
}
```

### Multi-Tenant Routing

```rust
let envelope = EnvelopeBuilder::new(record)
    .source("tenant:acme")
    .label("tenant", "acme")
    .label("region", "us-east-1")
    .build();

// Route based on source
match envelope.metadata.source.as_deref() {
    Some(s) if s.starts_with("tenant:acme") => route_to_acme_cluster(),
    _ => route_to_default(),
}
```

## API Reference

### EnvelopeMetadata

```rust
pub struct EnvelopeMetadata {
    pub timestamp: Option<u64>,    // Unix epoch ms (UTC)
    pub source: Option<String>,    // Service/device identifier
    pub trace_id: Option<String>,  // Distributed tracing ID
    pub sequence: Option<u64>,     // Monotonic sequence number
    pub labels: HashMap<String, String>, // Future extensibility
}
```

### LnmpEnvelope

```rust
pub struct LnmpEnvelope {
    pub record: LnmpRecord,       // LNMP record (mandatory)
    pub metadata: EnvelopeMetadata, // Operational metadata
}
```

### EnvelopeBuilder

Fluent API for constructing envelopes:

```rust
EnvelopeBuilder::new(record)
    .timestamp(1732373147000)
    .source("my-service")
    .trace_id("abc-123")
    .sequence(42)
    .label("key", "value")
    .build()
```

## Determinism Guarantee

**Critical Invariant:**

```rust
SemanticChecksum(Record) = f(Record.fields)
```

Envelope metadata is **NOT** included in checksum computation.

**Verification:**

```rust
let record = /* ... */;
let cs1 = SemanticChecksum::compute_record(&record);

let envelope = EnvelopeBuilder::new(record.clone())
    .timestamp(123456789)
    .build();

let cs2 = SemanticChecksum::compute_record(&envelope.record);

assert_eq!(cs1, cs2); // ✅ MUST pass
```

## Features

- `serde`: Enable serde serialization support (optional)

## Performance

- **Zero overhead when unused**: Envelope-unaware code runs at full speed
- **TLV encoding**: <100ns for typical metadata (4 fields)
- **Binary size**: ~50 bytes for full metadata (timestamp + source + trace_id + sequence)

## Examples

See [`examples/`](examples/) directory:

- [`envelope_basic_usage.rs`](examples/envelope_basic_usage.rs) - Creating and encoding envelopes (binary TLV)
- [`text_format.rs`](examples/text_format.rs) - Text header format demonstration
- [`http_binding.rs`](examples/http_binding.rs) - HTTP header mapping (X-LNMP-* pattern)
- [`kafka_binding.rs`](examples/kafka_binding.rs) - Kafka record headers integration
- [`llm_freshness.rs`](examples/llm_freshness.rs) - LLM freshness scoring with exponential decay

**Run an example:**
```bash
cargo run --package lnmp-envelope --example envelope_basic_usage
cargo run --package lnmp-envelope --example llm_freshness
```

## Documentation

### Core Documentation

- **[PERFORMANCE.md](PERFORMANCE.md)** - Comprehensive performance guide
  - Benchmark results and profiling
  - Optimization strategies
  - Comparison with alternatives (JSON, Protobuf, MessagePack)
  - Zero-overhead guarantees

- **[Formal Specification](../../spec/lnmp-envelope-v1.0.md)** - Technical specification v1.0
  - Binary TLV format details
  - Text encoding rules
  - Transport binding specifications
  - Conformance requirements

- **[API Documentation](https://docs.rs/lnmp-envelope)** - Rust API docs (docs.rs)

### Guides

- **Quick Start** - See [Quick Start](#quick-start) section above
- **Use Cases** - See [Use Cases](#use-cases) section  
- **Transport Bindings** - See [Transport Bindings](#transport-bindings) section
- **Performance** - See [PERFORMANCE.md](PERFORMANCE.md)

### Benchmarks

Run performance benchmarks:
```bash
cargo bench --package lnmp-envelope
```

**Latest Results** (Apple M1):
- Binary TLV encode: ~125ns
- Binary TLV decode: ~72ns  
- Text header encode: ~482ns
- Text header decode: ~415ns

See [PERFORMANCE.md](PERFORMANCE.md) for detailed analysis.

## Specification

Full technical specification: [lnmp-envelope-v1.0.md](../../spec/lnmp-envelope-v1.0.md)

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
