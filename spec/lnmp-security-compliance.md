# LNMP Security & Compliance Specification

**Status:** Working Draft (modularized from v0.4 RFC §8 + §9 + sanitizer docs)  
**Scope:** Security considerations, checksum behavior, sanitizer requirements, structural limits, and compliance tooling referencing actual implementations.  
**Audience:** Security reviewers, implementation authors, and compliance test maintainers.

---

## 1. Status of This Document

- Builds on `spec/lnmp-core-spec.md` (checksums, profiles) and `spec/lnmp-canonicalization.md`.  
- Normative requirements reference the Rust crates providing ground truth: `lnmp-core`, `lnmp-sanitize`, `lnmp-codec`, `tests/compliance`.

---

## 2. Semantic Checksums

- **REQ-SC-01:** Implementations supporting semantic checksums MUST compute CRC32/ISO-HDLC over canonical serialization of `F<fid>[:hint]=<value>` and encode as `#XXXXXXXX`.  
  - Evidence: `crates/lnmp-core/src/checksum.rs:100-320`, tests `checksum.rs:483-508`.

- **REQ-SC-02:** Decoders MUST NOT treat checksum mismatches as fatal by default but MUST provide configuration to reject mismatches (strict compliance mode).  
  - Evidence: `crates/lnmp-codec/src/parser.rs` (optional validation), `tests/binary_roundtrip.rs:1081-1100`.

- **REQ-SC-03:** Checksums are **not** stored in binary v0.4 frames; decoders MAY recompute and inject them into text output when configured.  
  - Evidence: `spec/lnmp-v0.4-rfc` note persists; implementation hooks in `binary/decoder.rs:195-240`.

---

## 3. Structural Limits and Resource Safety

- **REQ-LIM-01:** Parsers MUST enforce configurable structural limits (depth, field count, array length) defined in `crates/lnmp-core/src/limits.rs`. Default limits MUST protect against stack exhaustion and DoS.  
  - Evidence: `lnmp-core` limit structs; tests `crates/lnmp-core/tests` + integration tests `lnmp-codec`.

- **REQ-LIM-02:** Binary decoders MUST validate payload lengths before allocation to avoid buffer overflows.  
  - Evidence: `binary/decoder.rs` uses `checked_add`; tests `tests/binary_error_handling.rs`.

- **REQ-LIM-03:** Float/array parsing MUST guard against unbounded memory use by capping lengths per configuration.  
  - Evidence: `lnmp-core::limits`, `lnmp-codec/src/parser.rs` referencing them.

---

## 4. Sanitization and Normalization

- **REQ-SAN-01:** `lnmp-sanitize` MUST provide best-effort normalization for loose inputs (normalize booleans, trim whitespace, fix quoting) before strict parsing.  
  - Evidence: `crates/lnmp-sanitize/src/lib.rs`, property tests `tests/sanitize_property.rs:33-285`.

- **REQ-SAN-02:** Sanitizer MUST leave canonical text unchanged (idempotence).  
  - Evidence: Property test `sanitize_preserves_canonical` in `tests/property_roundtrip.rs:65-85`.

- **REQ-SAN-03:** Equivalence mappings (semantic dictionaries) MUST map source values to canonical replacements per-field without leaking across FIDs.  
  - Evidence: `crates/lnmp-codec/src/equivalence.rs`, tests `equivalence.rs:437+`.

---

## 5. Error Handling and Compliance

- **REQ-ERR-01:** Implementations MUST map validation failures to the error classes defined in `spec/error-classes.md` (e.g., `Strict Mode Violations`, `ChecksumMismatch`).  
  - Evidence: `crates/lnmp-codec/src/error.rs`, `tests/binary_error_handling.rs`, `tests/integration_tests.rs`.

- **REQ-ERR-02:** Compliance runner (`tests/compliance/rust/runner.rs`) MUST record expected errors and canonical outputs per test case to act as source of truth for SDKs.  
  - Evidence: `runner.rs:91-368`.

- **REQ-ERR-03:** When a compliance test fails, the runner MUST append the associated requirement IDs from the test definition to the failure message so downstream SDKs can trace issues to specific clauses.  
  - Evidence: `tests/compliance/rust/runner.rs` (requirement-aware `run_test` + `annotate_result` logic).

---

## 6. Version Negotiation & Capability Exchange (Security Context)

- **REQ-NEG-01:** Schema negotiation examples (`crates/lnmp-core/examples/v05_schema_negotiation.rs`) demonstrate capability checks (requires canonical, requires checksums). Implementations MUST enforce agreed capabilities when transmitting/receiving data.  
  - Evidence: Example + tests `crates/lnmp-codec/tests/schema_negotiation_tests.rs:170-325`.

---

## 7. Next Steps

1. Link sanitizer requirement IDs to future multi-language sanitizers.  
2. Integrate checksum enforcement toggles into compliance runner output for better CI visibility.
