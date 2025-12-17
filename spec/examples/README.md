# LNMP Canonical Example Fixtures

This directory stores canonical text (`*.lnmp`) and binary/container fixtures referenced by:
- `spec/lnmp-canonicalization.md` (examples section)
- Compliance runner (`tests/compliance/rust/runner.rs`)
- CI round-trip workflow (`.github/workflows/spec-fixtures.yml`)

> **✅ FID Registry Compliance:** All FIDs in these examples follow the official [`registry/fids.yaml`](../../registry/fids.yaml). Each file includes comments referencing the semantic meaning of FIDs used.

## Structure

```
spec/examples/
  text/
    simple_record.lnmp (+ .canonical)
    nested_record.lnmp (+ .canonical)
    string_array.lnmp (+ .canonical)
    escaped_strings.lnmp (+ .canonical)
    array_types.lnmp (+ .canonical)
    nested_array.lnmp (+ .canonical)
    commented_record.lnmp (+ .canonical)
    checksummed_record.lnmp (+ .canonical)
    envelope_payload.lnmp (+ .canonical)
  binary/
    simple_record.hex
  container/
    text_mode.hex (+ .yaml)
    binary_mode.hex (+ .yaml)
    stream_mode.hex (+ .yaml)
    delta_mode.hex (+ .yaml)
  README.md (this file)
```

Each pair captures:
1. Source text mirroring the spec snippet (may include comments/whitespace).
2. Canonical text output (post-parser/encoder).
3. Optional binary hex dump (when canonical record uses only binary-supported types).
4. REQ IDs referenced in the snippet (`spec/lnmp-*.md`).

Container manifests also support an optional `payload` block:
- `text_fixture`: path to a canonical `.lnmp` file whose contents MUST match the payload (text mode) or the decoded binary payload (binary mode).
- `binary_hex_fixture`: path to a `.hex` file whose bytes MUST match the binary payload exactly.

## Planned CI Step

Add a script under `scripts/verify_examples.sh` (or similar) that:
- Parses each `.lnmp` file using `lnmp-codec` parser (standard profile)
- Encodes it back to canonical text and binary
- Verifies `text → binary → text` equality against stored canonical form

The CI job will fail if any canonical fixture drifts from implementation behavior, ensuring that documentation and code stay synchronized.

## Verification

Run locally (or rely on the `spec-fixtures` GitHub workflow):

```bash
cargo run -p lnmp-compliance-tests --bin lnmp-verify-examples
```

Current coverage (✅ = exercised by `lnmp-verify-examples`):

| Fixture | Origin (spec section) | Notes |
|---------|----------------------|-------|
| ✅ `simple_record` | Text format intro & canonicalization §3.1 | Bool/int/string-array ordering (REQ-CAN-TXT-01/02). |
| ✅ `nested_record` | Grammar §4.1 & canonicalization §3.3 | Recursive canonicalization of `{…}` (REQ-TXT-12, REQ-CAN-TXT-06). |
| ✅ `string_array` | Text format §3.4 | Quoted/unquoted entries and array canonicalization (REQ-TXT-10/11). |
| ✅ `array_types` | Grammar §4 (Primitive Types) | `:ia/:fa/:ba` typed arrays normalized via new parser support (REQ-TXT-11, REQ-CAN-TXT-07). |
| ✅ `escaped_strings` | Grammar §4.3 (Escape sequences) | Ensures encoder preserves valid escapes (REQ-TXT-10). |
| ✅ `nested_array` | Grammar §4.2 | Record arrays remain canonical but skip binary round-trip (contains nested records). |
| ✅ `commented_record` | Text format §3.2 (comments) | Parser strips comments; canonical output stable (REQ-TXT-04). |
| ✅ `checksummed_record` | Canonicalization §3.4 & security spec §3 | Demonstrates checksum ordering/validation (REQ-CAN-TXT-09). Binary round-trip intentionally skipped because annotations are stripped in binary decoding. |
| ✅ `envelope_payload` | Envelope v1.0 §4 | Header comment + payload canonicalization. |
| ✅ `binary/simple_record.hex` | Binary spec §3 | Canonical binary dump kept in sync via verifier. |
| ✅ `container/text_mode.hex` | Container spec §Modes/Flags | Header + checksum flag + canonical text payload validated against `text/simple_record`. |
| ✅ `container/binary_mode.hex` | Container spec §Header Layout | Header + metadata + binary payload checked against both canonical text and `binary/simple_record.hex`. |
| ✅ `container/stream_mode.hex` | Container spec §Stream Metadata | Chunk size / checksum type / flag bits parsed and validated. |
| ✅ `container/delta_mode.hex` | Container spec §Delta Metadata | Base snapshot + algorithm/compression validated. |
| ⚠️ `container/invalid_*.hex` | Container spec §Header Error Mapping | Negative fixtures exercise magic/version/mode/metadata error paths; `lnmp-verify-examples` expects the documented failure reason. |

> **Note on error snippets:** `spec/error-classes.md` documents invalid LNMP fragments meant to trigger parser errors. Since these inputs cannot be canonicalized, they continue to live in the compliance YAML suites (`tests/compliance/test-cases*.yaml`) rather than `spec/examples/`. Each YAML case cites the relevant REQ IDs/error classes for traceability.
