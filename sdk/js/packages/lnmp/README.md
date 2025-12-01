# LNMP TypeScript SDK

TypeScript implementation of the LNMP (LLM Native Minimal Protocol) for Node.js.

## Installation

```bash
npm install @lnmplang/lnmp
```

## Usage

### 1. Core & Codec (Parsing/Encoding)

```typescript
import { Parser, Encoder, RecordBuilder } from '@lnmplang/lnmp';

// Parse LNMP text
const text = 'F12=14532;F7=1;F23=[admin,dev]';
const parser = new Parser(text);
const record = parser.parseRecord();

// Create records with RecordBuilder
const newRecord = new RecordBuilder()
  .addField({ fid: 12, value: { type: 'Int', value: 14532 } })
  .addField({ fid: 7, value: { type: 'Bool', value: true } })
  .build();

// Encode to canonical format
const encoder = new Encoder();
const output = encoder.encode(newRecord);
console.log(output); // F7=1\nF12=14532
```



### 2. Envelope (Metadata)

Wrap records with operational metadata (timestamp, source, trace ID).

```typescript
import { Envelope } from '@lnmplang/lnmp';

const envelope = Envelope.wrap(record, {
    source: 'sensor-node-01',
    trace_id: 'abc-123',
    labels: { env: 'prod' }
});

// Convert to/from HTTP headers
const headers = Envelope.toHeaders(envelope);
const restoredMetadata = Envelope.fromHeaders(headers);
```

### 3. Network (Routing & QoS)

Make intelligent routing decisions and calculate message importance.

```typescript
import { Network } from '@lnmplang/lnmp';

const msg = {
    envelope: envelope,
    kind: 'Event',
    priority: 200,
    ttl_ms: 5000
};

// Calculate importance (0.0 - 1.0)
const score = Network.importance(msg);

// Decide routing (SendToLLM, ProcessLocally, Drop)
const decision = Network.decide(msg);
```

### 4. Embedding (Vectors & Quantization)

Handle vector embeddings, delta updates, and quantization.

```typescript
import { Embedding, QuantScheme } from '@lnmplang/lnmp';

const vec1 = [0.1, 0.2, 0.3];
const vec2 = [0.2, 0.3, 0.4];

// Compute delta (bandwidth optimization)
const delta = Embedding.computeDelta(vec1, vec2);
const reconstructed = Embedding.applyDelta(vec1, delta);

// Quantization (compression)
const quantized = Embedding.quantize(vec1, QuantScheme.QInt8); // 4x compression
const restored = Embedding.dequantize(quantized);
```

### 5. Spatial (Streaming)

Encode and decode spatial frames for hybrid reality streams.

```typescript
import { Spatial } from '@lnmplang/lnmp';

const frame = {
    header: {
        mode: 'Absolute',
        sequence_id: 1,
        timestamp: BigInt(Date.now()) * 1000000n, // ns
        checksum: 0
    },
    payload: {
        S10: { // SpatialState
            position: { x: 10, y: 20, z: 30 },
            velocity: { vx: 1, vy: 0, vz: 0 },
            orientation: { pitch: 0, yaw: 0, roll: 0 }
        }
    }
};

const bytes = Spatial.encodeFrame(frame);
const decoded = Spatial.decodeFrame(bytes);
```

### 6. Sanitization

Clean untrusted input before parsing.

```typescript
import { Sanitizer } from '@lnmplang/lnmp';

const dirty = '  "Hello"   ';
const clean = Sanitizer.sanitize(dirty); // "Hello"
```

### 7. LLB (Explain Mode)

Generate human-readable explanations for LNMP records.

```typescript
import { ExplainEncoder } from '@lnmplang/lnmp';

const dict = { 12: "user_id", 7: "is_active" };
const explanation = ExplainEncoder.encode(record, dict);
// Output:
// F7:b=1              # is_active
// F12:i=14532         # user_id
```

### 8. SFE (Context Scoring)

Score contexts for RAG and LLM prioritization.

```typescript
import { ContextScorer } from '@lnmplang/lnmp';

const profile = ContextScorer.score(envelope);
console.log(profile.freshness_score); // 0.0 - 1.0
console.log(profile.risk_level);      // Low, Medium, High, Critical
```

## Features

- ✅ **Core**: Full TypeScript support with strict typing
- ✅ **Codec**: Parse/Encode LNMP text format (canonical)
- ✅ **Envelope**: Metadata wrapping & HTTP header interop
- ✅ **Network**: AI-driven routing & importance scoring
- ✅ **Embedding**: Vector delta compression & quantization (QInt8, QInt4, Binary)
- ✅ **Spatial**: Efficient spatial frame streaming
- ✅ **LLB**: Explain mode for debugging & LLM input
- ✅ **SFE**: Context scoring & risk assessment
- ✅ **WASM**: High-performance Rust bindings via WebAssembly

## Performance

Benchmarks run on M-series Mac (Single Thread):

| Module | Operation | Speed (ops/sec) |
|--------|-----------|-----------------|
| **Core** | Parse Record | ~284,000 |
| **Core** | Encode Record | ~253,000 |
| **Spatial** | Decode Frame | ~920,000 |
| **Spatial** | Encode Frame | ~491,000 |
| **SFE** | Context Score | ~341,000 |
| **Embedding** | Quantize (1536d) | ~23,000 |
| **Embedding** | Delta (1536d) | ~5,000 |

*Note: Spatial operations are extremely fast due to zero-copy WASM memory access where possible.*

## Documentation

- [**API Reference**](./API.md) - Detailed API guide for all modules.
- [Core Package](../core/README.md)
- [Codec Package](../codec/README.md)
- [LNMP Protocol Specification](https://github.com/lnmplang/lnmp-protocol)


## Version

Current version: **0.1.0** (Phase 1 MVP)

Matches Rust implementation patterns from LNMP Protocol v0.5.12.

## License

MIT
