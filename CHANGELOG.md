# Changelog

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

- `examples/basic_usage.rs` - Binary TLV encoding demonstration
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
