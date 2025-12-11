# LNMP Canonicalization Specification

**Status:** Working Draft (extracted from v0.4 RFC §§4 + 6)  
**Scope:** Deterministic rules for text and binary canonical forms, requirement IDs tied to encoder/decoder implementations, and references to the compliance suite.  
**Audience:** Implementers of encoders/decoders, conformance tooling authors, and protocol reviewers.

---

## 1. Status of This Document

- Canonicalization underpins semantic checksums, caching, and cross-implementation interoperability.  
- This document consolidates canonical rules from the monolithic RFC and binds them to actual code/tests (no descriptive drift).  
- Requirement IDs (`REQ-CAN-TXT-*`, `REQ-CAN-BIN-*`, `REQ-CAN-RT-*`) MUST be cited by automated tests and error messages when feasible.

---

## 2. Canonicalization Principles

1. **Single Representation:** Each semantic record has exactly one canonical text and binary representation.  
2. **Round-Trip Stability:** `text → binary → text` and `binary → text → binary` sequences MUST stabilize after first iteration.  
3. **Strict Mode Enforcement:** Parsers operating in strict profile MUST reject inputs violating canonical form.

Evidence baseline: `crates/lnmp-codec/src/encoder.rs:308-353`, `crates/lnmp-codec/tests/binary_roundtrip.rs:31-1177`, `tests/compliance/rust/runner.rs:91-368`.

---

## 3. Text Canonical Form Requirements

### 3.1 Ordering and Separators

- **REQ-CAN-TXT-01:** Fields MUST appear in ascending FID order; duplicates MUST retain stable order if allowed but canonical output MUST de-duplicate per encoder policy.  
  - Evidence: `encoder.rs::canonicalize_record`, tests `encoder.rs:928-1176`, compliance suite.

- **REQ-CAN-TXT-02:** Canonical text MUST use newline (`\n`) separators only; semicolons are forbidden in canonical output.  
  - Evidence: `encoder.rs` newline selection; tests `tests/canonical_encoding_tests.rs:7-26`.

### 3.2 Whitespace and Formatting

- **REQ-CAN-TXT-03:** No leading/trailing whitespace around `=` or separators; encoders MUST strip extraneous spaces.  
  - Evidence: canonical encode tests `encoder.rs:779-910`.

- **REQ-CAN-TXT-04:** Integers/floats MUST follow normalized formatting (`-?[1-9][0-9]*` or `0`; floats without trailing zeros unless enforced by significant digits rules).  
  - Evidence: `encoder.rs` normalization tests `encoder.rs:928-1176`, plus float-specific tests around `encoder.rs:1919-2050`.

- **REQ-CAN-TXT-05:** Strings MUST be quoted when containing characters outside `[A-Za-z0-9_.-]` and MUST employ minimal escaping (only `"`, `\`, control characters).  
  - Evidence: `encoder.rs:1389-1490`; tests `tests/binary_roundtrip.rs:964-1100`.

### 3.3 Composite Structures

- **REQ-CAN-TXT-06:** Nested records MUST be canonicalized recursively, including ordering and newline formatting within braces.  
  - Evidence: `encoder.rs:954-1236`; deep nesting tests `encoder.rs:1047-1290`.

- **REQ-CAN-TXT-07:** Array canonicalization removes all whitespace between elements, regardless of element type. String arrays retain quoted/unquoted minimal form, typed numeric arrays (`:ia/:fa/:ba`) emit normalized literals (booleans as `1/0`), and nested record arrays adopt canonical record formatting within `[]`.  
  - Evidence: `encoder.rs:174-2291`; tests `tests/binary_roundtrip.rs:964-1177` plus parser regression tests for typed arrays; fixture `spec/examples/text/array_types.*`.

- **REQ-CAN-TXT-08:** Empty arrays/records MUST be omitted unless semantically required (Requirement 9.3 in historical RFC).  
  - Evidence: `encoder.rs:581-1390`.

### 3.4 Checksums

- **REQ-CAN-TXT-09:** Checksums (`#XXXXXXXX`) MUST be computed over canonical value serialization. Re-encoding a canonical record MUST preserve checksum validity.  
  - Evidence: `crates/lnmp-core/src/checksum.rs:100-320` + tests `checksum.rs:483-508`; round-trip tests `tests/binary_roundtrip.rs:1081-1100`.

### 3.5 Strict Mode

- **REQ-CAN-TXT-10:** Strict mode parsers MUST reject any deviation from requirements 1–9, producing deterministic error codes (see `spec/error-classes.md`).  
  - Evidence: `tests/compliance/rust/runner.rs` (strict mode cases) and `tests/binary_error_handling.rs`.

---

## 4. Binary Canonical Form Requirements

Implementation source: `crates/lnmp-codec/src/binary/encoder.rs`, `crates/lnmp-codec/src/binary/nested_encoder.rs`, `crates/lnmp-codec/tests/v05_integration_tests.rs`, `tests/binary_roundtrip.rs`.

### 4.1 Frame Structure

- **REQ-CAN-BIN-01:** Binary frames MUST start with version byte (`0x04` for v0.4, `0x05` for nested) followed by flags, entry count (VarInt), then entries sorted by FID.  
  - Evidence: `binary/encoder.rs`, tests `tests/binary_roundtrip.rs:31-117`.

- **REQ-CAN-BIN-02:** VarInt encoding MUST use minimal LEB128 representation; encoders MUST reject non-minimal input (e.g., `FF 00` for 127).  
  - Evidence: `binary/encoder.rs::encode_varint`, tests `spec/lnmp-v0.4-rfc` examples but enforced by `tests/binary_error_handling.rs`.

### 4.2 Type Tags and Values

- **REQ-CAN-BIN-03:** Type tags (0x01 integer, 0x02 float, etc.) MUST align with text representation semantics. Future tags reserved (0x06/0x07) MUST remain unused unless negotiated.  
  - Evidence: `binary/mod.rs` definitions; tests `v05_integration_tests.rs:553-760`.

- **REQ-CAN-BIN-04:** Binary encoding of floats MUST use IEEE 754 double precision with canonical NaN bit pattern `00 00 00 00 00 00 F8 7F`.  
  - Evidence: `binary/encoder.rs`, tests `spec/lnmp-v0.4-rfc` example validated by `tests/binary_roundtrip.rs:964-1009`.

- **REQ-CAN-BIN-05:** Nested records/arrays when using version 0x05 MUST canonicalize recursively before encoding (mirrors text requirements).  
  - Evidence: `binary/nested_encoder.rs:126-351` and tests `v05_integration_tests.rs:93-140`.

### 4.3 Order Validation

- **REQ-CAN-BIN-06:** Decoders MUST validate ascending FID order if configured (strict profile). Non-canonical ordering MUST raise `BinaryError::OutOfOrder`.  
  - Evidence: `binary/decoder.rs:230-360`, tests `binary_error_handling.rs:757` and `v05_integration_tests.rs`.

---

## 5. Round-Trip Guarantees

- **REQ-CAN-RT-01:** `text_any → binary → text` MUST equal canonical text of the original record.  
  - Evidence: `tests/binary_roundtrip.rs:31-117`, `tests/binary_roundtrip.rs:929-1177`.

- **REQ-CAN-RT-02:** `binary_canonical → text → binary` MUST reproduce identical bytes.  
  - Evidence: `tests/binary_roundtrip.rs:902-956`; integration tests for nested data.

- **REQ-CAN-RT-03:** Canonicalization is idempotent: `canonical(canonical(record)) = canonical(record)` for both text/binary.  
  - Evidence: `crates/lnmp-codec/src/encoder.rs:385-399`; property tests `tests/property_roundtrip.rs:52-85`.

---

## 6. Compliance and Tooling

- The Rust compliance runner (`tests/compliance/rust/runner.rs`) MUST emit `expected_canonical` cases for every test vector so that future SDKs can verify identical canonical text.  
- Property-based tests (`tests/property_roundtrip.rs`) serve as statistical evidence; implementers SHOULD cite REQ IDs in failure messages to streamline debugging.
- Canonical fixtures are tracked under `spec/examples/` (see README). Each fixture pairs a non-canonical input with its canonical output/binary hex dump and references the relevant REQ IDs. CI tooling MUST round-trip these fixtures to detect drift (see `.github/workflows/spec-fixtures.yml`).

---

## 7. Implementation Evidence Table

| Requirement Group | Evidence |
|-------------------|----------|
| REQ-CAN-TXT-* | `crates/lnmp-codec/src/encoder.rs`, `tests/canonical_encoding_tests.rs`, `tests/binary_roundtrip.rs` |
| REQ-CAN-BIN-* | `crates/lnmp-codec/src/binary/*`, `tests/v05_integration_tests.rs`, `tests/binary_error_handling.rs` |
| REQ-CAN-RT-* | `tests/binary_roundtrip.rs`, `tests/property_roundtrip.rs` |

---

## 8. Next Steps

1. Extend compliance runners to include checksum validation tied to REQ-CAN-TXT-09.  
2. Publish machine-readable canonical test vectors (e.g., JSON manifest referencing `.lnmp` files) in `fixtures/` so non-Rust SDKs can reuse them.
