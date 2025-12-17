# lnmp-net

Network behavior standardization for LNMP agent networks.

[![Crates.io](https://img.shields.io/crates/v/lnmp-net.svg)](https://crates.io/crates/lnmp-net)
[![Documentation](https://docs.rs/lnmp-net/badge.svg)](https://docs.rs/lnmp-net)

> **FID Registry:** All examples use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

## Overview

**LNMP-Net** provides semantic message classification, Quality of Service (QoS) primitives, and intelligent routing decisions for LLM and agent networks. It builds on top of the LNMP ecosystem without replacing existing serialization or transport layers.

### What LNMP-Net Does

- **Message Classification**: Categorize messages as Event/State/Command/Query/Alert
- **QoS Primitives**: Priority (0-255) and TTL (time-to-live) for network behavior
- **Intelligent Routing**: ECO profile for deciding LLM vs. local processing
- **Token Optimization**: Reduce LLM API calls by 90%+ while maintaining quality

### What LNMP-Net Is NOT

- ❌ A new binary format (uses existing LNMP)
- ❌ A transport layer (uses lnmp-transport)
- ❌ A replacement for LLM reasoning (assists in routing decisions)

## Quick Start

```rust
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_net::{MessageKind, NetMessage, RoutingPolicy, RoutingDecision};

//  Create a record
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(42) });

// Wrap with envelope (timestamp required for TTL/freshness)
let envelope = EnvelopeBuilder::new(record)
    .timestamp(1700000000000)
    .source("sensor-01")
    .build();

// Create network message
let msg = NetMessage::new(envelope, MessageKind::Event);

// Make routing decision
let policy = RoutingPolicy::default(); // ECO profile, threshold=0.7
let decision = policy.decide(&msg, 1700000001000).unwrap();

match decision {
    RoutingDecision::SendToLLM => println!("→ LLM"),
    RoutingDecision::ProcessLocally => println!("→ Local"),
    RoutingDecision::Drop => println!("→ Drop"),
}
```

## Message Kinds

LNMP-Net defines 5 semantic message types:

| Kind | Purpose | Default Priority | Default TTL | LLM Routing |
|------|---------|------------------|-------------|-------------|
| **Event** | Sensor data, telemetry | 100 | 5s | Via importance scoring |
| **State** | System state snapshots | 100 | 10s | Via importance scoring |
| **Command** | Imperative actions | 150 | 2s | Local processing |
| **Query** | Information requests | 120 | 5s | Local first, complex → LLM |
| **Alert** | Critical warnings | 255 | 1s | Always → LLM |

Each kind has sensible defaults tuned for typical use cases.

## Routing Logic (ECO Profile)

The `RoutingPolicy` implements Energy/Token Optimization:

1. **Expired messages** → `Drop` (no point processing stale data)
2. **Alerts** with high priority (>200) → `SendToLLM` (critical path)
3. **Events/State**: Compute importance score → threshold check
   - Score = `(priority / 255) * 0.5 + sfe_score * 0.5`
   - If score ≥ threshold → `SendToLLM`
   - Else → `ProcessLocally`
4. **Commands/Queries** → `ProcessLocally` (unless complex)

### Example: Importance Scoring

```rust
use lnmp_net::RoutingPolicy;

let policy = RoutingPolicy::default(); // threshold = 0.7

// High priority event (fresh timestamp)
let high_priority_msg = NetMessage::with_qos(
    envelope,
    MessageKind::Event,
    240, // priority
    10000 // TTL
);

let importance = policy.base_importance(&high_priority_msg, now_ms).unwrap();
// importance ≈ 0.8 → SendToLLM

// Low priority event
let low_priority_msg = NetMessage::with_qos(
    envelope,
    MessageKind::Event,
    30,
    10000
);

let importance = policy.base_importance(&low_priority_msg, now_ms).unwrap();
// importance ≈ 0.3 → ProcessLocally
```

## Features

- **`serde`** (optional): Enable serde serialization support

```toml
[dependencies]
lnmp-net = { version = "0.5.7", features = ["serde", "transport"] }
```

## Transport Integration

LNMP-Net provides standard header mappings for common protocols via the `transport` feature:

| Transport | Kind | Priority | TTL | Class |
|-----------|------|----------|-----|-------|
| **HTTP** | `X-LNMP-Kind` | `X-LNMP-Priority` | `X-LNMP-TTL` | `X-LNMP-Class` |
| **Kafka** | `lnmp.kind` | `lnmp.priority` | `lnmp.ttl` | `lnmp.class` |
| **NATS** | `lnmp-kind` | `lnmp-priority` | `lnmp-ttl` | `lnmp-class` |
| **gRPC** | `lnmp-kind` | `lnmp-priority` | `lnmp-ttl` | `lnmp-class` |

### Example: HTTP Headers

```rust
use lnmp_net::transport::http::{net_to_http_headers, http_headers_to_net_meta};

// Encode
let headers = net_to_http_headers(&msg)?;

// Decode
let (kind, priority, ttl, class) = http_headers_to_net_meta(&headers)?;
```

## Integration with  LNMP Ecosystem

LNMP-Net seamlessly integrates with existing modules:

```
┌─────────────────────────────────────┐
│         LNMP-Net Layer              │  ← MessageKind, Priority, TTL, Class
├─────────────────────────────────────┤
│  lnmp-llb  │  lnmp-sfe  │ lnmp-san │  ← LLM Bridge, SFE Scoring, Sanitize
├─────────────────────────────────────┤
│         lnmp-envelope               │  ← Timestamp, Source, TraceID
├─────────────────────────────────────┤
│         lnmp-core                   │  ← LnmpRecord (data)
├─────────────────────────────────────┤
│         lnmp-transport              │  ← HTTP, Kafka, gRPC, NATS
└─────────────────────────────────────┘
```

## Advanced Usage

### Custom Routing Policy

```rust
use lnmp_net::RoutingPolicy;

// Very selective policy (high threshold)
let strict_policy = RoutingPolicy::new(0.95)
    .with_always_route_alerts(true)
    .with_drop_expired(true);

// Permissive policy (low threshold)
let permissive_policy = RoutingPolicy::new(0.3);
```

### Message with Domain Class

```rust
use lnmp_net::NetMessageBuilder;

let msg = NetMessageBuilder::new(envelope, MessageKind::Alert)
    .priority(255)
    .ttl_ms(1000)
    .class("safety") // Domain classification
    .build();
```

## Architecture

See [spec/lnmp-net-v1.md](../../../spec/lnmp-net-v1.md) for complete specification.

## License

MIT
