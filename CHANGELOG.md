# Changelog

## Unreleased

- Added delta encoding (v0.5 DPL) support: `DeltaEncoder`, `DeltaDecoder`, and integration into `BinaryEncoder` via `encode_delta_from` and gating via `DeltaConfig::enable_delta` (default `false`).
- Added `BinaryEncoder::with_delta_mode(bool)` convenience API; `EncoderConfig::with_delta_mode(bool)` already available.
- Added `TypeHint::parse` and `FromStr` trait implementation. Deprecated `TypeHint::from_str` (kept wrapper for backward compatibility).
- Updated tests and examples to enable delta when needed and added gating tests.
- Addressed clippy style and simplification warnings across the workspace.
