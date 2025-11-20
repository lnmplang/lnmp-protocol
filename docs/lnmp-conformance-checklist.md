# LNMP Conformance Checklist (v1)

This document defines fixture expectations for `.lnmp` container v1. It is intended for implementers to self-validate parsing and header handling. Payload contents are irrelevant unless noted; focus on header + metadata correctness.

## Valid Fixtures

- **Text + checksum flag, no metadata**  
  Header: `4C 4E 4D 50 01 01 00 01 00 00 00 00`  
  Notes: minimum valid text container; payload can be empty.

- **Binary + 16-byte metadata**  
  Header: `4C 4E 4D 50 01 02 00 00 00 00 00 10`  
  Notes: metadata blob is opaque; length must match.

- **Stream default (chunk=4096, SC32)**  
  Header: `4C 4E 4D 50 01 03 00 00 00 00 00 06`  
  Metadata: `00 00 10 00 02 00` (`chunk_size=4096`, `checksum_type=0x02`, `flags=0x00`)  
  Notes: payload may be empty for header parsing tests.

- **Delta op-list (base snapshot = 1)**  
  Header: `4C 4E 4D 50 01 04 00 00 00 00 00 0A`  
  Metadata: `00 00 00 00 00 00 00 01 00 00` (`base_snapshot=1`, `algorithm=0x00`, `compression=0x00`)

## Invalid Fixtures (must be rejected)

- **Bad magic**: `46 41 4B 45 ...`
- **Unsupported version**: version byte != `0x01`.
- **Unknown mode**: mode byte `0xFF`.
- **Reserved flag set**: any flag beyond bit 0 (`checksum`) set in v1.
- **Metadata overflow**: `metadata_length` larger than available buffer.
- **Stream/Delta without metadata**: `metadata_length = 0` when `mode` is `0x03` or `0x04`.
- **Reserved bits in metadata**: nonzero reserved bits/bytes in stream/delta metadata.

## Parser Expectations

- Always validate magic, version, mode, and `metadata_length` against buffer size.
- Reject reserved-flag violations.
- Stream metadata length must be exactly 6 bytes in v1; delta metadata length must be exactly 10 bytes.
- Unknown `checksum_type` in stream MAY be tolerated; unknown `algorithm`/`compression` in delta MUST error.
- Unknown trailing metadata bytes are only allowed if indicated by `metadata_length` and not in reserved space.

## Purpose

These fixtures form the minimum interoperable subset for Phase 2. Implementers should publish pass/fail results against this matrix to demonstrate `.lnmp` compatibility. Payload-specific tests (chunk boundaries, delta op validity) are defined separately in codec-level suites.

## How to Run (reference implementation)
- Fixtures live under `fixtures/`.
- Conformance assertions live in `crates/lnmp-codec/tests/container_conformance.rs`.
- Run: `cargo test -p lnmp-codec --test container_conformance`.
- CI-friendly: `scripts/run-conformance.sh` runs container + streaming + delta fixture tests plus the full suite.

If you have CI, add this target to your test matrix so `.lnmp` header accept/reject behavior stays locked across releases.

## Next: Payload-Level Conformance (planning)
- Stream: add fixtures for chunk boundary correctness, checksum presence when `checksum_type != 0`, and erroring on truncated chunks.
- Delta: add fixtures for base snapshot mismatch, unknown algorithm/compression codes, and minimal op-list payload that applies cleanly.
- These payload checks build on the header matrix but live in codec-level suites, not in the header-only fixtures above (see `docs/lnmp-payload-conformance.md` for the plan).

## Fixture File Suggestions

When materializing fixtures in `target/fixtures/` (or equivalent), use concise naming:
- `fixtures/valid-text-checksum.lnmp`
- `fixtures/valid-binary-meta16.lnmp`
- `fixtures/valid-stream-4k-sc32.lnmp`
- `fixtures/valid-delta-base1.lnmp`
- `fixtures/valid-stream-4k-sc32-chunks.lnmp` (payload placeholder for chunk tests)
- `fixtures/invalid-bad-magic.lnmp`
- `fixtures/invalid-version-ff.lnmp`
- `fixtures/invalid-mode-ff.lnmp`
- `fixtures/invalid-reserved-flag.lnmp`
- `fixtures/invalid-stream-no-meta.lnmp`
- `fixtures/invalid-delta-no-meta.lnmp`
- `fixtures/invalid-meta-overflow.lnmp`
- `fixtures/invalid-stream-truncated-chunk.lnmp`
- `fixtures/invalid-stream-missing-checksum.lnmp`
- `fixtures/invalid-delta-base-mismatch.lnmp`
- `fixtures/invalid-delta-unknown-algorithm.lnmp`
- `fixtures/invalid-delta-unknown-compression.lnmp`

## Expected Outcomes (reference)

| Fixture                             | Expected result                        |
|-------------------------------------|----------------------------------------|
| valid-text-checksum.lnmp            | Parse OK                               |
| valid-binary-meta16.lnmp            | Parse OK                               |
| valid-stream-4k-sc32.lnmp           | Parse OK                               |
| valid-delta-base1.lnmp              | Parse OK                               |
| valid-stream-4k-sc32-chunks.lnmp    | Parse OK (payload validation occurs in stream tests) |
| invalid-bad-magic.lnmp              | Error: `InvalidMagic`                  |
| invalid-version-ff.lnmp             | Error: `UnsupportedVersion(0xFF)`      |
| invalid-mode-ff.lnmp                | Error: `UnknownMode(0xFF)`             |
| invalid-reserved-flag.lnmp          | Error: `ReservedFlags`                 |
| invalid-stream-no-meta.lnmp         | Error: `InvalidMetadataLength` (stream)|
| invalid-delta-no-meta.lnmp          | Error: `InvalidMetadataLength` (delta) |
| invalid-meta-overflow.lnmp          | Error: `TruncatedMetadata`             |
| invalid-stream-truncated-chunk.lnmp | Parse OK header; payload validation occurs in stream tests |
| invalid-stream-missing-checksum.lnmp| Parse OK header; payload validation occurs in stream tests |
| invalid-delta-base-mismatch.lnmp    | Parse OK header; payload validation occurs in delta tests  |
| invalid-delta-unknown-algorithm.lnmp| Error: `InvalidMetadataValue` (algorithm) |
| invalid-delta-unknown-compression.lnmp| Error: `InvalidMetadataValue` (compression) |
