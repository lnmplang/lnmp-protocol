# lnmp-transport

Transport bindings for the LNMP protocol, providing standard mappings between LNMP records with Envelope metadata and various transport protocols (HTTP, Kafka, gRPC).

> **FID Registry:** All examples use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

## Purpose

`lnmp-transport` bridges LNMP's in-memory data model to real-world transport layers by:

- **Standardizing header/metadata names** across all transports
- **Preserving observability** through W3C Trace Context and OpenTelemetry integration
- **Maintaining determinism** - does not modify `LnmpRecord` or `SemanticChecksum`
- **Providing helper functions** for bi-directional mapping

**This crate does NOT**:
- Implement HTTP/Kafka/gRPC clients or servers
- Define a new protocol - it binds existing LNMP to transports
- Handle retry logic, circuit breakers, or other behavioral concerns

## Standard Header Mappings

### HTTP

| Envelope Field | HTTP Header | Example |
|----------------|-------------|---------|
| `timestamp` | `X-LNMP-Timestamp` | `1732373147000` |
| `source` | `X-LNMP-Source` | `auth-service` |
| `trace_id` | `X-LNMP-Trace-Id` | `abc-123-xyz` |
| - | `traceparent` | `00-abc123xyz...-0123456789abcdef-01` |
| `sequence` | `X-LNMP-Sequence` | `42` |
| `labels["key"]` | `X-LNMP-Label-key` | `prod` |

**Body**: LNMP binary or text format  
**Content-Type**: `application/lnmp-binary` or `application/lnmp-text`

### Kafka

| Envelope Field | Kafka Header | Format |
|----------------|--------------|--------|
| `timestamp` | `lnmp.timestamp` | `b"1732373147000"` |
| `source` | `lnmp.source` | `b"auth-service"` |
| `trace_id` | `lnmp.trace_id` | `b"abc-123-xyz"` |
| `sequence` | `lnmp.sequence` | `b"42"` |
| `labels["key"]` | `lnmp.label.key` | `b"prod"` |

**Value**: LNMP binary format

### NATS

**Recommended Subject Pattern**: `lnmp.<domain>.<event>`

Examples:
- `lnmp.llm.request` - LLM inference requests
- `lnmp.robot.command` - Robot control commands
- `lnmp.sensor.telemetry` - Sensor data streams

| Envelope Field | NATS Header | Format |
|----------------|-------------|--------|
| `timestamp` | `lnmp-timestamp` | `"1732373147000"` |
| `source` | `lnmp-source` | `"auth-service"` |
| `trace_id` | `lnmp-trace-id` | `"abc-123-xyz"` |
| `sequence` | `lnmp-sequence` | `"42"` |
| `labels["key"]` | `lnmp-label-key` | `"prod"` |

**Payload**: LNMP binary format

### gRPC

| Envelope Field | gRPC Metadata | Example |
|----------------|---------------|---------|
| `timestamp` | `lnmp-timestamp` | `"1732373147000"` |
| `source` | `lnmp-source` | `"auth-service"` |
| `trace_id` | `lnmp-trace-id` | `"abc-123-xyz"` |
| `sequence` | `lnmp-sequence` | `"42"` |
| `labels["key"]` | `lnmp-label-key` | `"prod"` |

**Payload Strategy**:
1. Embed LNMP binary record inside Protobuf message as `bytes` field
2. Use LNMP metadata only in headers, send application data in Protobuf message

```rust
use lnmp_transport::grpc;

// Convert envelope metadata to gRPC metadata
let metadata = grpc::envelope_to_metadata(&envelope)?;
for (key, value) in &metadata {
    // Attach to gRPC call
    println!("{}: {}", key, value);
}
```

## Quick Start

```rust
use lnmp_envelope::{LnmpEnvelope, EnvelopeMetadata};
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
use lnmp_transport::http;

// Create an envelope
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(42) });

let mut meta = EnvelopeMetadata::default();
meta.timestamp = Some(1732373147000);
meta.source = Some("my-service".to_string());
meta.trace_id = Some("trace-abc-123".to_string());

let env = LnmpEnvelope { metadata: meta, record };

// Convert to HTTP headers
let headers = http::envelope_to_headers(&env)?;

// Convert back
let parsed_meta = http::headers_to_envelope_metadata(&headers)?;
assert_eq!(parsed_meta.source, Some("my-service".to_string()));
```

## W3C Trace Context / OpenTelemetry

`lnmp-transport` automatically generates W3C-compliant `traceparent` headers for distributed tracing:

```rust
use lnmp_transport::http;

let env = /* ... */;
let headers = http::envelope_to_headers(&env)?;

// traceparent header is automatically included if trace_id is present
// Format: 00-{trace_id}-{span_id}-{flags}
```

To extract trace context from incoming requests:

```rust
let trace_id = http::traceparent_to_trace_id(traceparent_header)?;
```

> **Note**: `lnmp-transport` focuses on trace ID propagation and does not manage the full W3C Trace Context semantics (span parent relationships, sampling decisions, etc.). For complete OpenTelemetry SDK behavior, use `lnmp-transport` in conjunction with a dedicated tracing library.

## Features

- `http` (default): HTTP header mappings
- `kafka`: Kafka header mappings
- `grpc`: gRPC metadata mappings
- `nats`: NATS header mappings
- `otel`: OpenTelemetry integration helpers

## Examples

See `examples/` directory:
- `transport_basic_usage.rs` - Simple mapping example
- `http_full.rs` - Complete HTTP request/response with body encoding
- `otel_integration.rs` - OpenTelemetry context propagation

## Testing & CI Guidance

To ensure every optional transport binding stays healthy, run the test matrix locally (and wire the same commands into CI):

| Feature flags | Command |
| --- | --- |
| none | `cargo test -p lnmp-transport --no-default-features` |
| http (default) | `cargo test -p lnmp-transport --features http` |
| kafka only | `cargo test -p lnmp-transport --no-default-features --features kafka` |
| http + kafka | `cargo test -p lnmp-transport --features "http kafka"` |
| all bindings | `cargo test -p lnmp-transport --features "http kafka grpc nats"` |

Benchmarks/examples should also be covered in automation at least once per release:

```bash
cargo bench -p lnmp-transport --features "http kafka"
cargo bench -p lnmp-transport --no-default-features   # verifies graceful skip
cargo run  -p lnmp-transport --example transport_basic_usage --features "http kafka grpc nats"
cargo run  -p lnmp-transport --example http_full --features http
cargo run  -p lnmp-transport --example otel_integration --features http
```

> Tip: Keeping these commands in CI ensures unresolved-import regressions are caught immediately when optional modules change.

## Alignment with Industry Standards

This crate follows the same patterns as:
- **CloudEvents**: Context attributes separate from event payload
- **Kafka Headers**: Record-level metadata best practices
- **W3C Trace Context**: Standard distributed tracing propagation
- **OpenTelemetry**: Seamless telemetry integration

## License

MIT
