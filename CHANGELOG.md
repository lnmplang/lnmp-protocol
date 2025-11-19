# Changelog

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
