PR Title: Binary Encoder: Delta Support, Gating, Tests, Lint Fixes, and API Updates

Summary
-------
- Added full delta encoding (DPL) support and integration into `BinaryEncoder`.
- Gate delta behavior using `DeltaConfig::enable_delta` (default `false`). `BinaryEncoder::encode_delta_from` will merge `EncoderConfig.delta_mode` with `DeltaConfig` and enforce gating.
- Added tests to validate delta compute/encode/decode/apply and gating error behavior.
- Implemented `TypeHint::parse` (previously `from_str`) and added `FromStr` trait impl. Added a deprecated `TypeHint::from_str` wrapper to preserve compatibility.
- Addressed clippy suggestions across `lnmp-codec`, `lnmp-core`, `lnmp-sfe` — replaced `len() < 1` checks, used `or_default()` and `unwrap_or_default()`, collapsed nested ifs, derived `Default` where applicable, and added `matches!` optimizations.
- Added `BinaryEncoder::with_delta_mode(bool)` convenience method.

Breaking Changes & Migration Notes
-------------------------------
- Type hint parsing: `TypeHint::from_str("i")` is now deprecated in favor of `TypeHint::parse("i")` or `str::parse::<TypeHint>("i")`. We provide a deprecated wrapper for backward compatibility.
- Delta encoding: Delta mode is opt-in and disabled by default. To enable delta encoding, either:
  - Use `EncoderConfig::with_delta_mode(true)`; or
  - Pass `DeltaConfig::new().with_enable_delta(true)` to `BinaryEncoder::encode_delta_from(base, updated, Some(config))`.

Tests / Validation
------------------
- Updated `delta_encoding_tests.rs` to enable `DeltaConfig` where needed.
- Added gating tests that verify `compute_delta`, `decode_delta`, and `apply_delta` error out when delta is disabled.
- Ran `cargo test --all` and fixed doc & doctest issues.

Next Steps
----------
1. Sweep remaining clippy warnings in `lnmp-llb` and compliance tests and address them in follow-up commits.
2. Optionally, add `EncoderConfig::with_delta_mode` convenience builder and `BinaryEncoder::encode_delta_from_with_mode` method as further helpers.
3. Add a changelog entry and consider tagging this for a minor version release that introduces v0.5 forward-compatible features.

Files of Note
-------------
- `crates/lnmp-codec/src/binary/delta.rs` - delta logic and config.
- `crates/lnmp-codec/src/binary/encoder.rs` - `encode_delta_from`, `with_delta_mode`, gating enforcement.
- `crates/lnmp-core/src/types.rs` - `TypeHint::parse`, `FromStr` trait, and deprecated `from_str` wrapper.
- `crates/lnmp-codec/tests/delta_encoding_tests.rs` - updated tests.

Checklist
---------
- [x] Tests updated/added for delta gating and encoder integration.
- [x] Docs updated: `API.md`, `API_V05.md`, `MIGRATION_V05.md`.
- [x] `TypeHint::from_str` migration (wrapper added and docs updated).
- [ ] Sweep and fix remaining clippy lints across `lnmp-llb` and compliance tests.
- [ ] Add a `CHANGELOG.md` entry and finalize PR title/labels.

