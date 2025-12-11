# LNMP Text Format Specification

**Status:** Working Draft (modularization of v0.4 RFC §2)  
**Scope:** Normative text syntax, parsing rules, comments/whitespace handling, and strict-mode expectations anchored to current implementations in `crates/lnmp-codec`.  
**Audience:** Parser/encoder implementers, test authors, and tooling integrators.

---

## 1. Status of This Document

- This document replaces §2 of `lnmp-v0.4-rfc.md` and is referenced by the umbrella RFC as part of the ongoing modularization effort.  
- Requirements use RFC 2119 / RFC 8174 keywords and reference concrete code/tests instead of prose-only descriptions.  
- Whenever parser or encoder behavior changes, the associated requirement IDs MUST be updated together with citations to keep the spec “kanıtlı”.

---

## 2. Encoding Overview

| Concept | Requirement ID | Evidence |
|---------|----------------|----------|
| Field prefix `F` + numeric FID | REQ-TXT-01 | `crates/lnmp-codec/src/parser.rs` (fid parsing); property tests in `tests/property_roundtrip.rs`. |
| Assignment operator `=` | REQ-TXT-02 | Parser state machine around tokenization in `parser.rs`. |
| Semicolon/newline separators | REQ-TXT-03 | Parser whitespace handling in `parser.rs::consume_separator`; tests `tests/canonical_encoding_tests.rs`. |
| Comments `#…` vs checksum `#XXXXXXXX` | REQ-TXT-04 | Lexer branch `parser.rs` + tests `tests/binary_roundtrip.rs:775`. |

### Syntax Summary

```
record   := field (separator field)*
field    := "F" fid [type_hint] "=" value [checksum]
separator:= ";" | newline
```

Formal grammars remain in `spec/grammar.md`; this spec ties grammar to implementation evidence.

---

## 3. Normative Requirements

### 3.1 Field Identifiers and Operators

- **REQ-TXT-01:** Field identifiers MUST be unsigned 16-bit integers without leading zeros (except zero itself). Parsers MUST reject `F012` under strict profile and SHOULD normalize under loose mode.  
  - Evidence: `crates/lnmp-codec/src/parser.rs` numeric parsing; strict enforcement validated via `crates/lnmp-codec/tests/integration_tests.rs:267-400`.

- **REQ-TXT-02:** Exactly one ASCII `=` MUST separate `F<fid>` (with optional `:hint`) from the value; whitespace around `=` MUST be ignored during parsing but omitted when encoding canonically.  
  - Evidence: Parser `parse_assignment` logic; encoder canonical output tests `tests/canonical_encoding_tests.rs`.

### 3.2 Separators and Whitespace

- **REQ-TXT-03:** Parsers MUST treat `;` and newline as equivalent separators, allowing mixed usage. Canonical encoders MUST emit newline-separated fields by default.  
  - Evidence: Parser loop `parser.rs::consume_separator`; encoder config forced canonical newline output `crates/lnmp-codec/src/encoder.rs:60-122`; validated by `tests/binary_roundtrip.rs:31-117`.

- **REQ-TXT-04:** Inline comments start with `#` unless immediately followed by exactly eight hexadecimal digits (checksum). Parsers MUST treat lines ending with comments as if comments were removed.  
  - Evidence: `parser.rs` comment branch; compliance tests `tests/binary_roundtrip.rs:775` (checksum vs comment), `tests/integration_tests.rs:210-260`.

### 3.3 Type Hints

- **REQ-TXT-05:** Type hints use `:<code>` syntax with canonical codes (`i`, `f`, `b`, `s`, `sa`, `r`, `ra`). Parsers MUST validate hints against available value syntax.  
  - Evidence: `parser.rs::parse_type_hint`; tests covering mismatched hints in `tests/integration_tests.rs:439-520`.

- **REQ-TXT-06:** Strict profile MUST require type hints for every field (delegated from `spec/lnmp-core-spec.md` REQ-CONF-01); parser errors propagate from `StrictDeterministicConfig`.  
  - Evidence: `crates/lnmp-codec/src/parser.rs` consults profile flags; `tests/integration_tests.rs:362-400`.

### 3.4 Value Types

- **REQ-TXT-07 (Integers):** Signed decimal integers MUST fit within 64-bit range (checked during parsing). Canonical output MUST avoid leading zeros.  
  - Evidence: `parser.rs::parse_integer`; tests in `crates/lnmp-codec/src/encoder.rs:928-1177`.

- **REQ-TXT-08 (Floats):** Floats MUST use decimal notation with at least one digit before/after `.` when authored in text. Canonical formatting rules are covered by the canonicalization spec; parser accepts `-?[0-9]+\.[0-9]+`.  
  - Evidence: `parser.rs::parse_float`; tests `encoder.rs:779-910`.

- **REQ-TXT-09 (Booleans):** Accepted literals include `0`, `1`, plus loose representations (`true`, `false`, `yes`, `no`) when not in strict mode. Sanitizer/normalizer ensures canonical `0/1`.  
  - Evidence: `crates/lnmp-codec/src/normalizer.rs:62-115`, property tests `tests/sanitize_property.rs:258-285`.

- **REQ-TXT-10 (Strings):** Unquoted token syntax `[A-Za-z0-9_.-]+` and quoted `"` form with escaping. Parser MUST enforce UTF-8 validity after unescaping.  
  - Evidence: `parser.rs::parse_string`; tests `tests/binary_roundtrip.rs:964-1100`.

- **REQ-TXT-11 (Arrays):** Bracket syntax applies to both plain string arrays (no type hint) and typed arrays when `:ia`, `:fa`, or `:ba` is supplied. Parsers MUST validate each element according to the hinted type (integers, IEEE754 floats, canonical booleans) and differentiate nested-record arrays (`:ra`) by inspecting the first token.  
  - Evidence: `parser.rs::parse_string_array`, `parse_int_array`, `parse_float_array`, `parse_bool_array`, and `parse_nested_array`; regression tests `crates/lnmp-codec/src/parser.rs:2390-2434`; compliance case `tests/compliance/test-cases.yaml` (“Typed numeric arrays”).

### 3.5 Nested Records

- **REQ-TXT-12:** Nested records use `{…}` with interior fields following the same syntax. Parser MUST support recursive descent up to the structural limit configured in `lnmp-core::limits`.  
  - Evidence: `parser.rs::parse_nested_record`, `crates/lnmp-core/src/limits.rs`; tests `crates/lnmp-codec/src/encoder.rs:954-1236`.

### 3.6 Errors and Strict Mode

- **REQ-TXT-13:** When `StrictDeterministicConfig` sets `reject_unsorted_fields`, parser MUST fail fast with `ParseError::FieldOrdering` referencing the offending FID order.  
  - Evidence: Parser uses `LnmpRecord::validate_field_ordering` (core spec) when `strict` profile engaged; tested in `tests/integration_tests.rs:362-400`.

- **REQ-TXT-14:** Strict mode MUST reject duplicate FIDs unless explicitly allowed by profile/higher-level behavior. Loose mode MAY keep the first occurrence.  
  - Evidence: `parser.rs` duplicate handling; compliance suite `tests/compliance/rust/runner.rs` expects canonical dedup behavior.

---

## 4. Encoder Requirements

- **REQ-ENC-01:** `Encoder::new()` MUST canonicalize input records prior to emission, guaranteeing newline separators and sorted FIDs.  
  - Evidence: `crates/lnmp-codec/src/encoder.rs:79-122`, tests `tests/canonical_encoding_tests.rs`.

- **REQ-ENC-02:** When `EncoderConfig::with_canonical(false)` is used (legacy), semicolons MAY be emitted but canonical ordering still applies. Implementations MUST mark this mode as deprecated (see doc comments).  
  - Evidence: `encoder.rs:60-80` (deprecated constructor); tests `tests/integration_tests.rs:174-230`.

- **REQ-ENC-03:** Encoders MUST omit empty arrays/records according to canonicalization rules (cross-reference `spec/lnmp-canonicalization.md` once available).  
  - Evidence: `encoder.rs:581-2123` (empty omission tests).

---

## 5. Implementation Evidence Table

| Requirement | Evidence Reference |
|-------------|-------------------|
| REQ-TXT-01..06 | `crates/lnmp-codec/src/parser.rs`, `tests/integration_tests.rs` |
| REQ-TXT-07..12 | Parser functions + encoder canonicalization tests `crates/lnmp-codec/src/encoder.rs` |
| REQ-TXT-13..14 | Parser strict-mode enforcement + compliance runner `tests/compliance/rust/runner.rs` |
| REQ-ENC-01..03 | `crates/lnmp-codec/src/encoder.rs`, `tests/canonical_encoding_tests.rs`, `tests/binary_roundtrip.rs` |

Line numbers vary as the code evolves; requirement IDs must be updated alongside code/test modifications.

---

## 6. Next Steps

1. Link canonicalization-specific details (e.g., newline-only separators, whitespace removal) to `spec/lnmp-canonicalization.md`.  
2. Expand evidence table with multi-language SDK test references when compliance runners export artifacts beyond Rust.
