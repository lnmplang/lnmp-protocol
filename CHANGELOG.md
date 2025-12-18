# Changelog

## [0.5.7] - 2025-11-23

### Added

- **NEW: Context Profiling in `lnmp-sfe`** - LLM decision support system
  - `ContextProfile` struct with freshness, importance, risk, and confidence metrics
  - `ContextScorer` for automatic scoring of LNMP records
  - `ContextPrioritizer` with filtering, ranking, and top-K selection for RAG systems
  - `RiskLevel` enum (Low/Medium/High/Critical)
  - Configurable scoring weights for different use cases
  - Exponential decay freshness scoring (configurable decay rate)
  - Source-based trust and risk assessment
  - Field importance levels (0-255) in semantic dictionary
  - Statistics computation for context collections

- **NEW: `lnmp-transport` crate** - Transport protocol bindings with observability support
  - HTTP, Kafka, gRPC, and NATS transport bindings
  - W3C Trace Context / OpenTelemetry integration (`traceparent` generation and parsing)
  - Standard header naming conventions (X-LNMP-*, lnmp.*, lnmp-*)
  - Fail-safe metadata parsing (graceful degradation on missing/invalid headers)
  - HTTP body encoding/decoding (binary and text formats)
  - Kafka value encoding with full round-trip support
  - Production-ready with comprehensive documentation and examples
  - Type alias `KafkaHeaders` for reduced type complexity
  - `envelope_to_nats_message()` and `kafka_record_to_envelope()` helpers

### Changed

- **Version Synchronization**: All crates synchronized to **v0.5.7** for consistency
  - Updated `lnmp-core`, `lnmp-codec`, `lnmp-llb`, `lnmp-sanitize`, `lnmp-embedding`, `lnmp-quant`, `lnmp-spatial`, `lnmp-envelope` to v0.5.7
  - Updated workspace dependencies to use v0.5.7 baseline

### Enhanced

- **`SemanticDictionary`**: Extended with optional `importance` field (0-255)
  - Backward compatible YAML schema extension
  - New API: `get_importance()`, `add_importance()`, `importance_count()`
  
### Examples

- `context_scoring.rs` - Basic context scoring with freshness decay demonstration
- `rag_prioritization.rs` - RAG system use cases (top-K, filtering, ranking)

### Use Cases

- **RAG Systems**: Prioritize fresh, important, high-confidence contexts for LLM prompts
- **News/Events**: Weight freshness heavily (80%) for time-sensitive queries
- **Factual Data**: Weight confidence (70%) for reliable information retrieval
- **Multi-tenant**: Filter by source risk level for security
- **Token Budget Control**: Select top-K contexts to fit prompt limits

## [0.5.6] - 2024-11-23

### Added

- **NEW: `lnmp-envelope` crate** - Operational metadata envelope for LNMP records
  - `EnvelopeMetadata` struct with timestamp, source, trace_id, sequence fields
  - `LnmpEnvelope` wrapper for records with operational context
  - `EnvelopeBuilder` fluent API for constructing envelopes
  - Binary TLV codec for container metadata extension
  - Text codec with `#ENVELOPE` header comment format
  - Standards-aligned with CloudEvents, Kafka Headers, W3C Trace Context
  - Transport binding examples for HTTP and Kafka
  - Comprehensive documentation and examples

### Technical Details

- **Binary Format**: TLV (Type-Length-Value) encoding with canonical ordering
  - Type codes: 0x10 (Timestamp), 0x11 (Source), 0x12 (TraceID), 0x13 (Sequence)
  - Forward-compatible: unknown types gracefully skipped
  - Deterministic encoding ensures same metadata → same binary output

- **Text Format**: Header comment line before LNMP record
  - Syntax: `#ENVELOPE timestamp=... source=... trace_id=... sequence=...`
  - Backward compatible: parsers can ignore envelope if not present
  - Space-separated key=value pairs with optional quoting

- **Zero Overhead**: Envelope metadata does NOT affect `SemanticChecksum`
  - Core determinism preserved
  - Same record produces same checksum regardless of envelope

### Examples

- `examples/envelope_basic_usage.rs` - Binary TLV encoding demonstration
- `examples/text_format.rs` - Text header format demonstration
- `examples/http_binding.rs` - HTTP header mapping (X-LNMP-* pattern)
- `examples/kafka_binding.rs` - Kafka record headers integration

### Migration

No breaking changes. lnmp-envelope is a new optional module.

To use:
```toml
[dependencies]
lnmp-envelope = "0.5.6"
```

## [Unreleased]

## [0.5.15] - 2025-12-19

### Added

- **NEW: `EmbeddingView` in `lnmp-embedding`** - Quasi-zero-copy embedding access
  - `EmbeddingView::from_bytes()` - Parse embedding from raw bytes
  - `EmbeddingView::as_f32_vec()` - Safe copy to Vec<f32> (recommended)
  - `EmbeddingView::as_f32_slice()` - Experimental zero-copy (alignment-dependent)
  - `cosine_similarity()`, `dot_product()`, `euclidean_distance()` - SIMD-optimized
  - AVX2 optimization for x86_64 with scalar fallback
  - Works seamlessly with `decode_view()` for high-performance embedding pipelines

- **NEW: Content-Aware Routing in `lnmp-net`** - Zero-copy field inspection
  - `ContentRouter` with rule-based routing decisions
  - `RoutingRule` enum: HeaderContains, FieldEquals, HasField, EmbeddingDim
  - `RoutingPriority` levels: Critical, High, Normal, Low, Background
  - Zero-copy field access from `LnmpRecordView` for routing decisions

- **Enhanced Examples**
  - `zerocopy_similarity.rs` - Complete embedding similarity pipeline demo
  - `content_routing.rs` - Content-based routing with benchmarks
  - `zero_copy_routing.rs` - Routing performance comparison

### Changed

- **`bytemuck` dependency** added to `lnmp-embedding` (optional, via `zerocopy` feature)
- All similarity methods now use safe `as_f32_vec()` internally for reliability

### Technical Notes

- **Memory Alignment**: True zero-copy f32 slice access requires aligned memory pointers.
  Since arbitrary buffer slices may not be aligned, `as_f32_vec()` (safe copy) is recommended.
  The experimental `as_f32_slice()` may panic on unaligned buffers.

- **Performance**: For 256-dim embeddings, `as_f32_vec()` adds ~1.4μs overhead for allocation.
  This is negligible for most use cases and guarantees reliability.

### Examples

```rust
use lnmp_codec::binary::BinaryDecoder;
use lnmp_embedding::EmbeddingView;

let decoder = BinaryDecoder::new();
let view = decoder.decode_view(&bytes)?;

if let Some(field) = view.get_field(512) {
    if let LnmpValueView::Embedding(emb_bytes) = &field.value {
        let emb_view = EmbeddingView::from_bytes(emb_bytes)?;
        let similarity = emb_view.cosine_similarity(&other_view)?;
    }
}
```

## [0.5.14] - 2025-12-17

### Added

- **Schema Negotiation v2**: Enhanced SchemaNegotiator with registry awareness
  - `with_registry_version()` - Set local registry version
  - `request_registry()` - Create registry request message
  - `handle_registry_response()` - Process registry response
  - `peer_supports_fid()` - Check peer FID support
  - `agreed_fids()` - Get negotiated FID set

- **Dynamic FID Discovery Protocol**: Runtime FID registry exchange
  - `RequestRegistry` message with range filtering
  - `RegistryResponse` message with FID definitions
  - `RegistryDelta` message for incremental sync
  - `FidDefinition` and `FidDefStatus` wire types

- **Registry Sync**: Multi-peer registry synchronization
  - `RegistrySync` struct for version tracking
  - `is_ahead_of()` and `is_behind()` version comparison
  - `delta_fids_for()` compute sync delta
  - Semantic version comparison (major.minor.patch)

- **FID Runtime Validation**: Parser and Encoder validation
  - `ValidationMode::Warn` with log integration
  - `encode_validated()` method for pre-encoding check
  - `log` feature for warning output

### Changed

- All documentation examples updated to use registry-defined FIDs
- spec/grammar.md examples corrected for FID consistency

## [0.5.13] - 2025-12-01


## [0.5.12] - 2025-11-25


## [0.5.11] - 2025-11-24


## [0.5.10] - 2025-11-24


## [0.5.9] - 2025-11-23


## [0.5.8] - 2025-11-23


- Nothing yet.

## v0.5.0 - 2025-11-19

### Added

- Introduced delta encoding (DPL) end-to-end support including `DeltaEncoder`, `DeltaDecoder`, and `BinaryEncoder::encode_delta_from` with `DeltaConfig::enable_delta` gating (defaults to `false`).
- Added `BinaryEncoder::with_delta_mode(bool)` convenience constructor to align with `EncoderConfig::with_delta_mode`.
- Added `TypeHint::parse` with a `FromStr` implementation and a deprecated `TypeHint::from_str` wrapper for downstream compatibility.

### Changed

- Updated tests and examples to explicitly enable delta features when needed and added gating regression tests.

### Fixed

- Addressed `clippy` style and simplification warnings across the workspace.
