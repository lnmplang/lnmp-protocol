# LNMP Binary Format Specification

**Status:** Working Draft (modularized from v0.4 RFC §3 + §5)  
**Scope:** Binary frame layout, VarInt encoding, type tags, nested structures, and decoder validation tied directly to `crates/lnmp-codec/src/binary/*`.  
**Audience:** Implementers of LNMP binary encoders/decoders, transport designers, and interoperability testers.

---

## 1. Status of This Document

- Reflects the currently shipping binary implementation (`README.md` lists v0.5.13 with nested support).  
- Requirements reference concrete code/tests to avoid divergence.  
- Versioning model aligns with migration doc (pending); version byte semantics defined here.

---

## 2. Binary Frame Overview

```
+---------+--------+-----------------+--------------------+
| Version | Flags  | Entry Count VAR | Entries (sorted)   |
+---------+--------+-----------------+--------------------+
```

- Version (`0x04` = base binary, `0x05` = nested).  
- Flags reserved for compression/envelope features (future spec).  
- Entry count is VarInt (LEB128 minimal encoding).  
- Entries follow canonical FID order.

Implementation: `crates/lnmp-codec/src/binary/encoder.rs`, `binary/nested_encoder.rs`.

---

## 3. Normative Requirements

### 3.1 Versioning

- **REQ-BIN-VER-01:** Encoders MUST set version byte to `0x04` when no nested structures are encoded and to `0x05` when nested records/arrays may appear.  
  - Evidence: `binary/encoder.rs` configuration; tests `tests/binary_roundtrip.rs:31-117`, `tests/v05_integration_tests.rs:93-140`.

- **REQ-BIN-VER-02:** Decoders MUST reject frames with version lower than the configured minimum from `StrictDeterministicConfig` (default `0x04`, strict `0x05`).  
  - Evidence: `binary/decoder.rs:150-210`; profile integration `lnmp-core/src/profile.rs`.

### 3.2 VarInt Encoding

- **REQ-BIN-VAR-01:** All integer fields in the frame header and values using VarInt MUST use minimal LEB128 representation (no redundant continuation bytes).  
  - Evidence: `binary/encoder.rs::encode_varint`, `tests/binary_error_handling.rs`.

- **REQ-BIN-VAR-02:** Decoders MUST reject non-minimal encodings with `BinaryError::NonCanonicalVarInt`.  
  - Evidence: `binary/decoder.rs`, tests `tests/binary_error_handling.rs:650-720`.

### 3.3 Entry Structure

```
entry := FID_VARINT | type_tag | payload
```

- **REQ-BIN-ENT-01:** Field IDs MUST be encoded as VarInt sorted ascending. Duplicate IDs only allowed when protocol layer explicitly permits (future extension).  
  - Evidence: `binary/encoder.rs` sorts via canonical record; tests `binary_roundtrip.rs`.

- **REQ-BIN-ENT-02:** Type tags are single bytes with current assignments: `0x01` int, `0x02` float, `0x03` bool, `0x04` string, `0x05` string array, `0x06` nested record (v0.5), `0x07` nested array (v0.5). Reserved tags MUST NOT be used.  
  - Evidence: `binary/mod.rs` definitions; `tests/v05_integration_tests.rs`.

### 3.4 Primitive Payloads

- **REQ-BIN-PRI-01:** Integers use signed VarInt (same canonicalization as text).  
  - Evidence: `binary/encoder.rs::encode_int`; tests `binary_roundtrip.rs:964-1009`.

- **REQ-BIN-PRI-02:** Floats use 64-bit IEEE 754 big-endian representation with canonical NaN pattern `00 00 00 00 00 00 F8 7F`.  
  - Evidence: `binary/encoder.rs`, tests `binary_roundtrip.rs`.

- **REQ-BIN-PRI-03:** Strings encoded as `len VarInt` + UTF-8 bytes. Encoders MUST validate UTF-8; decoders MUST reject invalid sequences.  
  - Evidence: `binary/encoder.rs::encode_string`, `binary/decoder.rs`; tests `binary_error_handling.rs`.

### 3.5 Composite Payloads

- **REQ-BIN-CMP-01:** String arrays encode element count (VarInt) followed by length-prefixed strings; canonical order same as text (original insertion order preserved).  
  - Evidence: `binary/encoder.rs`, tests `binary_roundtrip.rs:964-1100`.

- **REQ-BIN-CMP-02:** Nested records/arrays (v0.5) MUST reuse the same canonicalization pipeline as text: encode canonical child records recursively before serialization.  
  - Evidence: `binary/nested_encoder.rs:126-351`, `tests/v05_integration_tests.rs:93-140`.

### 3.6 Ordering Validation

- **REQ-BIN-ORD-01:** Encoders MUST output entries sorted by FID; decoders MUST enforce ordering when `validate_ordering` flag is enabled (default in strict profile).  
  - Evidence: `binary/encoder.rs` sorting; `binary/decoder.rs:230-320`; tests `binary_error_handling.rs:757`.

- **REQ-BIN-ORD-02:** Binary compliance tests MUST include non-canonical order fixtures to ensure rejection paths remain covered.  
  - Evidence: `tests/binary_roundtrip.rs` (non-canonical case) and `tests/compliance/rust/runner.rs`.

### 3.7 Error Classes

- Errors follow `spec/error-classes.md` (Binary section). Decoders MUST map detection of non-minimal VarInt, ordering issues, truncated payloads, and invalid encodings to the documented codes.  
  - Evidence: `crates/lnmp-codec/src/binary/error.rs`, `tests/binary_error_handling.rs`.

---

## 4. Implementation Evidence Table

| Requirement Group | Evidence |
|-------------------|----------|
| REQ-BIN-VER-* | `binary/encoder.rs`, `binary/decoder.rs`, `lnmp-core/src/profile.rs` |
| REQ-BIN-VAR-* | `binary/encoder.rs::encode_varint`, `binary/decoder.rs`, tests `binary_error_handling.rs` |
| REQ-BIN-ENT-* | `binary/mod.rs`, `binary_roundtrip.rs` |
| REQ-BIN-PRI-* | `binary/encoder.rs`, `binary/decoder.rs`, `tests/binary_roundtrip.rs` |
| REQ-BIN-CMP-* | `binary/nested_encoder.rs`, `tests/v05_integration_tests.rs` |
| REQ-BIN-ORD-* | `binary/encoder.rs`, `binary/decoder.rs`, tests `binary_error_handling.rs` |

---

## 5. Next Steps

1. Document future type tags (e.g., embeddings, quantized arrays) once their binary encoding stabilizes.  
2. Publish canonical binary fixtures (hex dumps + expected text) under `spec/examples/` for cross-language verification.
