# LNMP SDK API Reference

This document provides a detailed reference for the `@lnmplang/lnmp` TypeScript SDK.

## Table of Contents

1. [Core & Codec](#core--codec)
   - [Parser](#parser)
   - [Encoder](#encoder)
   - [RecordBuilder](#recordbuilder)
2. [Envelope](#envelope)
3. [Network](#network)
4. [Embedding](#embedding)
5. [Spatial](#spatial)
6. [Sanitization](#sanitization)
7. [LLB (Explain Mode)](#llb-explain-mode)
8. [SFE (Context Scoring)](#sfe-context-scoring)

---

## Core & Codec

### Parser

Parses LNMP text format into `LnmpRecord` objects.

```typescript
import { Parser } from '@lnmplang/lnmp';

const parser = new Parser('F12:i=14532;F7:b=1');
const record = parser.parseRecord();
```

### Encoder

Encodes `LnmpRecord` objects into canonical LNMP text format.

```typescript
import { Encoder } from '@lnmplang/lnmp';

const encoder = new Encoder();
const text = encoder.encode(record);
```

### RecordBuilder

Fluent API for constructing `LnmpRecord`s.

```typescript
import { RecordBuilder } from '@lnmplang/lnmp';

const record = new RecordBuilder()
    .addField({ fid: 12, value: { type: 'Int', value: 14532 } })
    .addField({ fid: 7, value: { type: 'Bool', value: true } })
    .build();
```

---

## Envelope

Wraps records with operational metadata (timestamp, source, trace ID).

```typescript
import { Envelope } from '@lnmplang/lnmp';

// Wrap a record
const envelope = Envelope.wrap(record, {
    source: 'my-service',
    trace_id: 'abc-123',
    labels: { env: 'prod' }
});

// Unwrap
const innerRecord = Envelope.unwrap(envelope);

// HTTP Headers Interop
const headers = Envelope.toHeaders(envelope);
const restoredMetadata = Envelope.fromHeaders(headers);
```

---

## Network

AI-driven routing and importance scoring.

```typescript
import { Network } from '@lnmplang/lnmp';

const msg = {
    envelope: envelope,
    kind: 'Event',
    priority: 128,
    ttl_ms: 5000
};

// Calculate Importance (0.0 - 1.0)
const score = Network.importance(msg);

// Decide Routing
// Returns: "SendToLLM" | "ProcessLocally" | "Drop"
const decision = Network.decide(msg);
```

---

## Embedding

Vector operations, delta compression, and quantization.

```typescript
import { Embedding, QuantScheme } from '@lnmplang/lnmp';

// Delta Compression
const delta = Embedding.computeDelta(vec1, vec2);
const reconstructed = Embedding.applyDelta(vec1, delta);

// Quantization
// Schemes: QInt8, QInt4, Binary, FP16
const quantized = Embedding.quantize(vec1, QuantScheme.QInt8);
const restored = Embedding.dequantize(quantized);
```

---

## Spatial

Efficient encoding/decoding of spatial frames for hybrid reality.

```typescript
import { Spatial, FrameMode } from '@lnmplang/lnmp';

const frame = {
    header: {
        mode: FrameMode.Absolute,
        sequence_id: 1,
        timestamp: BigInt(Date.now()) * 1000000n, // Nanoseconds (BigInt)
        checksum: 0
    },
    payload: { /* SpatialState */ }
};

const bytes = Spatial.encodeFrame(frame);
const decoded = Spatial.decodeFrame(bytes);
```

---

## Sanitization

Cleans untrusted input to ensure safe parsing.

```typescript
import { Sanitizer } from '@lnmplang/lnmp';

const cleanText = Sanitizer.sanitize('  "Unsafe Input"  ');
```

---

## LLB (Explain Mode)

Generates human-readable explanations for LNMP records using a semantic dictionary.

```typescript
import { ExplainEncoder } from '@lnmplang/lnmp';

const dictionary = {
    12: "user_id",
    7: "is_active"
};

const explanation = ExplainEncoder.encode(record, dictionary);
// Output:
// F7:b=1              # is_active
// F12:i=14532         # user_id
```

---

## SFE (Context Scoring)

Scores contexts (envelopes) to help LLMs prioritize data for RAG or prompt construction.

```typescript
import { ContextScorer } from '@lnmplang/lnmp';

const profile = ContextScorer.score(envelope);

console.log(profile.freshness_score); // 0.0 (stale) to 1.0 (fresh)
console.log(profile.risk_level);      // "Low", "Medium", "High", "Critical"
console.log(profile.importance);      // 0-255
console.log(profile.confidence);      // 0.0 - 1.0
```
