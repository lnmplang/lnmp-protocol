# LNMP Container Overview (v1)

- **Single extension:** `.lnmp` for all modes.
- **Header:** 12 bytes `LNMP` magic, version (0x01), mode byte, flags, metadata length (BE u32).
- **Modes:** Text (0x01), Binary (0x02), Stream (0x03), Delta (0x04), Quantum-Safe reserved (0x05).
- **Flags:** Only checksum bit used in v1; other bits reserved and must be zero.
- **Metadata:** Optional; fixed layouts for Stream (6 bytes: chunk_size u32, checksum_type u8, flags u8) and Delta (10 bytes: base_snapshot u64, algorithm u8, compression u8). Delta alg/comp must be 0x00/0x01.
- **Payload:** Follows metadata; interpreted per mode (UTF-8 text, binary encoding, stream chunks with checksums, delta ops).
- **Validation:** Conformance fixtures in `fixtures/`; tests in `crates/lnmp-codec/tests/{container_conformance.rs,streaming_layer_tests.rs,delta_encoding_tests.rs}`; CI workflow `.github/workflows/conformance.yml`.
- **Freeze policy:** Header/mode bytes and stream/delta metadata for v1 are frozen; changes require version bump + RFC. Fixtures/tests must stay green.
- **Future extension guard:** Header flag bit 15 is reserved to signal a future Metadata Extension Block (TLV chain after fixed metadata). It stays `0` in v1; TLV registry is frozen (inactive) in `spec/lnmp-metadata-extension-rfc.md`.
- **Tools:** `scripts/run-conformance.sh` runs conformance + full suite; `scripts/package-fixtures.sh` produces `artifacts/lnmp-fixtures-v1.tar.gz` for third parties.
