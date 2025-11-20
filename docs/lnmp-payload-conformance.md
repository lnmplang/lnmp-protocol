# LNMP Payload Conformance (planning)

This note captures the next layer of interoperability checks beyond header/metadata validation. These cases are Phase 2 follow-ons for `.lnmp` and focus on mode-specific payload behavior.

## Stream Mode (`0x03`)
- **Chunk boundaries**: Verify a stream payload parses into whole chunks; truncated final chunks must error.
- **Checksum presence**: When `checksum_type != 0`, ensure checksum bytes exist per chunk and fail when missing/mismatched.
- **Flags**: If the stream metadata `flags` indicate ACK-required, parsers should surface the expectation; absence of ACK material should produce a validation error in stream-aware tooling (not header parsing).
- **Payload shape embedded**:
  - Chunk = `[len(4 bytes BE)][payload bytes][checksum(4 bytes SC32)]`.
  - Fixtures use two payloads: `hello` and `world!`, each with SC32 checksum, concatenated.
- **Fixtures (payload filled, exercised in `delta_encoding_tests.rs`)**:
  - `valid-stream-4k-sc32-chunks.lnmp` (two complete chunks with SC32).
  - `invalid-stream-truncated-chunk.lnmp` (final checksum truncated).
  - `invalid-stream-missing-checksum.lnmp` (length+payload without checksum).

## Delta Mode (`0x04`)
- **Base snapshot enforcement**: Applying a delta to an unexpected base snapshot must error.
- **Algorithm gating**: Unknown `algorithm` or `compression` codes must error before application.
- **Minimal op-list**: Provide a smallest valid delta payload that applies cleanly to a known base.
- **Payload shape embedded (op-list example)**:
  - Generated via delta encoder (`cargo run -p lnmp-codec --example gen_delta_fixture`) to produce a minimal op-list delta over base snapshot 1.
- **Fixtures (payload filled, exercised in `delta_encoding_tests.rs`)**:
  - `valid-delta-base1-oplist.lnmp`.
  - `invalid-delta-base-mismatch.lnmp`.
  - `invalid-delta-unknown-algorithm.lnmp`.
  - `invalid-delta-unknown-compression.lnmp`.

## Test Placement
- Header-only checks remain in `crates/lnmp-codec/tests/container_conformance.rs`.
- Payload conformance should live in mode-specific suites (e.g., `streaming_layer_tests.rs`, `delta_encoding_tests.rs`) to avoid coupling to header-only parsing.

## Goal

Once these payload fixtures/tests are green across implementations, Phase 2 can be marked complete and Phase 3 (schema finalization) can begin.
