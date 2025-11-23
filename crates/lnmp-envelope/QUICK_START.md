# LNMP Envelope - Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lnmp-envelope = "0.5.6"
```

Or use cargo:

```bash
cargo add lnmp-envelope
```

## Basic Usage

### 1. Create an Envelope

```rust
use lnmp_envelope::{EnvelopeBuilder, LnmpRecord, LnmpField, LnmpValue};

// Create a record
let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

// Wrap with envelope
let envelope = EnvelopeBuilder::new(record)
    .timestamp(1732373147000)
    .source("my-service")
    .trace_id("abc-123")
    .sequence(42)
    .build();
```

### 2. Binary Encoding

```rust
use lnmp_envelope::binary_codec::TlvEncoder;

let bytes = TlvEncoder::encode(&envelope.metadata)?;
// ~51 bytes for full metadata
```

### 3. Text Encoding

```rust
use lnmp_envelope::text_codec::TextEncoder;

let header = TextEncoder::encode(&envelope.metadata)?;
// Output: #ENVELOPE timestamp=1732373147000 source=my-service ...
```

## Common Use Cases

### Distributed Tracing

```rust
// Extract trace ID from incoming request
let trace_id = extract_trace_id_from_headers();

let envelope = EnvelopeBuilder::new(record)
    .trace_id(trace_id)
    .timestamp(now())
    .build();
```

### LLM Freshness Scoring

```rust
fn score_freshness(envelope: &LnmpEnvelope, now: u64) -> f64 {
    if let Some(ts) = envelope.metadata.timestamp {
        let age_hours = (now - ts) as f64 / 3_600_000.0;
        (-age_hours / 24.0).exp()  // 24-hour decay
    } else {
        0.5
    }
}

// Sort by freshness
contexts.sort_by_key(|env| -score_freshness(env, now()));
```

### Multi-Tenant Routing

```rust
match envelope.metadata.source.as_deref() {
    Some(s) if s.starts_with("tenant:acme") => route_to_acme(),
    Some(s) if s.starts_with("tenant:beta") => route_to_beta(),
    _ => route_to_default(),
}
```

## Transport Bindings

### HTTP

```rust
// Sending
let envelope = EnvelopeBuilder::new(record)
    .timestamp(now())
    .source("api-gateway")
    .build();

let mut headers = HeaderMap::new();
if let Some(ts) = envelope.metadata.timestamp {
    headers.insert("X-LNMP-Timestamp", ts.to_string().parse()?);
}
if let Some(ref source) = envelope.metadata.source {
    headers.insert("X-LNMP-Source", source.parse()?);
}

// Receiving
let timestamp = headers.get("X-LNMP-Timestamp")
    .and_then(|h| h.to_str().ok())
    .and_then(|s| s.parse().ok());

let metadata = EnvelopeMetadata {
    timestamp,
    source: headers.get("X-LNMP-Source")
        .and_then(|h| h.to_str().ok())
        .map(String::from),
    ..Default::default()
};
```

### Kafka

```rust
use rdkafka::message::OwnedHeaders;

// Sending
let mut headers = OwnedHeaders::new();
headers = headers.add("lnmp.timestamp", &envelope.metadata.timestamp.unwrap().to_string());
headers = headers.add("lnmp.source", envelope.metadata.source.as_ref().unwrap());

let record = FutureRecord::to("topic")
    .payload(&payload)
    .headers(headers);

// Receiving
let metadata = EnvelopeMetadata {
    timestamp: msg.headers()
        .and_then(|h| h.get("lnmp.timestamp"))
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|s| s.parse().ok()),
    ..Default::default()
};
```

## Next Steps

- **[README.md](README.md)** - Full feature documentation
- **[PERFORMANCE.md](PERFORMANCE.md)** - Benchmarks and optimization
- **[Examples](examples/)** - 5 working examples
- **[Specification](../../spec/lnmp-envelope-v1.0.md)** - Technical details

## FAQ

**Q: Does envelope affect record checksums?**  
A: No. Envelope metadata is separate and does NOT affect `SemanticChecksum`.

**Q: What's the performance overhead?**  
A: ~125ns for binary encoding, ~72ns for decoding. See [PERFORMANCE.md](PERFORMANCE.md).

**Q: Can I use only some fields?**  
A: Yes! All fields are optional. Use only what you need.

**Q: Is it compatible with CloudEvents?**  
A: Yes. Timestamp→time, source→source, trace_id→id. See [spec](../../spec/lnmp-envelope-v1.0.md).

**Q: How do I migrate existing code?**  
A: Envelope is optional and non-breaking. Wrap records as needed:
```rust
// Before
send_record(record);

// After (add envelope)
let envelope = EnvelopeBuilder::new(record).timestamp(now()).build();
send_envelope(envelope);
```
