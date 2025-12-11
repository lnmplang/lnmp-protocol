# LNMP Core Specification (v0.5 lineage)

**Status:** Working Draft (linked to repo `README.md` “Current Version: v0.5.13”)  
**Scope:** Normative definitions for records, field identifiers, canonical ordering, conformance profiles, and requirement-to-test mappings that underpin the broader LNMP document set.  
**Audience:** Implementers of `lnmp-core`, `lnmp-codec`, and dependent crates.  
**Change Control:** LNMP Protocol Working Group (see `CODEOWNERS`).

---

## 1. Status of This Document

- This draft supersedes the v0.4 monolithic RFC core sections and is referenced by the modular RFC structure described in `spec/lnmp-v0.4-rfc.md`.
- Normative requirements follow RFC 2119 / RFC 8174 terminology (MUST, SHOULD, MAY) and are backed by specific repository artifacts (code/tests).
- Implementation evidence is drawn exclusively from files under `crates/` and `tests/` in this repository; no speculative behavior is described.

---

## 2. Conformance and Profiles

### 2.1 Conformance Levels

| Profile | Description | Enforcement Source |
|---------|-------------|--------------------|
| `Loose` | Accepts maximum input variance; no canonical ordering enforcement. | `crates/lnmp-core/src/profile.rs` (`StrictDeterministicConfig::loose`). |
| `Standard` | Default profile; canonical output, flexible input; booleans normalized. | `profile.rs` (`standard`). |
| `Strict` | Full deterministic mode; requires canonical ordering, type hints, binary v0.5+. | `profile.rs` (`strict`). |

Each profile corresponds to `StrictDeterministicConfig` presets verified in unit tests (`profile.rs` tests at ~L260-L310). Conformant implementations MUST expose an equivalent mechanism or enforce the strictest profile by default for agent/LLM workflows.

### 2.2 Normative Language

- **REQ-CONF-01:** Parsers operating under `Strict` profile MUST reject input where fields are unsorted by ascending FID.  
  - *Evidence:* `StrictDeterministicConfig::strict()` sets `reject_unsorted_fields = true` (`profile.rs` around L120-L150); enforcement exercised in `crates/lnmp-codec/tests/integration_tests.rs:362-400`.
- **REQ-CONF-02:** Encoders operating in any profile MUST emit canonical ordering (sorted FIDs) to ensure deterministic downstream handling.  
  - *Evidence:* `crates/lnmp-codec/src/encoder.rs:79-122` sorts via `canonicalize_record`, validated by `tests/canonical_encoding_tests.rs:7-26`.

---

## 3. Terminology

| Term | Definition | Source |
|------|------------|--------|
| **Field Identifier (FID)** | Unsigned 16-bit ID associated with semantic meaning. Stored as `FieldId` in `lnmp-core`. | `crates/lnmp-core/src/lib.rs` exports `type FieldId = u16`. |
| **LnmpField** | Struct containing `fid` and `LnmpValue`. | `crates/lnmp-core/src/record.rs:18-34`. |
| **LnmpRecord** | Collection of fields supporting insertion order plus canonical helpers. | `record.rs:36-210`. |
| **Canonical Order** | Fields sorted ascending by FID; stable sort for duplicates. | `record.rs:82-111` (`sorted_fields`). |
| **Semantic Checksum (SC32)** | CRC32/ISO-HDLC computed over canonical serialization of FID/value. | `crates/lnmp-core/src/checksum.rs:100-240`. |

Terminology is canonicalized in `spec/lnmp-v0.4-rfc.md §1.3`, but this document ties each definition to living code for verification.

---

## 4. Core Requirements

### 4.1 Record Structure

- **REQ-REC-01:** An `LnmpRecord` MUST allow arbitrary insertion order but MUST provide a deterministic view via `sorted_fields()`.  
  - Evidence: Implementation `record.rs:72-111`; unit tests verifying deterministic ordering appear in `record.rs` doc tests and `tests` module.

- **REQ-REC-02:** `LnmpRecord::from_fields` MUST sort incoming fields before storage to enforce canonical ordering even when builders provide unsorted data.  
  - Evidence: `record.rs:112-151` sorts before constructing; doc tests demonstrate effect.

- **REQ-REC-03:** Canonical equality (`canonical_eq`) MUST compare sorted fields so that order-agnostic semantics hold.  
  - Evidence: `record.rs:167-189`; tests `record.rs:838-907`.

- **REQ-REC-04:** Canonical hashing MUST hash sorted fields and nested structures recursively to guarantee order independence.  
  - Evidence: `record.rs:190-285`; tests `record.rs:960-1054`.

### 4.2 Builder Guarantees

- **REQ-BLD-01:** `RecordBuilder::build` MUST sort accumulated fields prior to returning an `LnmpRecord`.  
  - Evidence: `crates/lnmp-core/src/builder.rs:78-112`; tests `builder.rs:120-190`.

- **REQ-BLD-02:** Builder APIs MUST allow chained additions (`add_field`, `add_fields`) without mutating previously built records.  
  - Evidence: `builder.rs` design uses value semantics returning `Self`; validated in `test_builder_chaining`.

### 4.3 Field Ordering Validation

- **REQ-ORD-01:** `LnmpRecord::validate_field_ordering` MUST detect the first descending FID pair and return a structured error containing the violation index and involved FIDs.  
  - Evidence: `record.rs:214-260`; tests `record.rs:471-505`.

- **REQ-ORD-02:** `LnmpRecord::is_canonical_order` MUST be equivalent to `validate_field_ordering().is_ok()`.  
  - Evidence: Implementation `record.rs:297-361`.

- **REQ-ORD-03:** `count_ordering_violations` MUST count all out-of-order pairs for diagnostic tooling.  
  - Evidence: `record.rs:363-403`.

### 4.4 Canonical Serialization Hooks

- **REQ-CAN-01:** Any component computing checksums MUST serialize values using canonical ordering and canonicalized nested content prior to hashing.  
  - Evidence: `crates/lnmp-core/src/checksum.rs:228-320` serializes primitives/nested items; unit tests `checksum.rs:483-508`.

- **REQ-CAN-02:** `lnmp-codec` encoders MUST invoke `canonicalize_record` (which leverages `lnmp-core` helpers) before emitting text or binary.  
  - Evidence: `crates/lnmp-codec/src/encoder.rs:308-353`, with extensive tests `encoder.rs:928-2291`. This document references them to keep canonical behavior traceable to implementation.

### 4.5 Profiles and Strict Mode

- **REQ-PRO-01:** Profiles MUST be exposed as stable string IDs (`"loose"`, `"standard"`, `"strict"`) to support negotiation.  
  - Evidence: `profile.rs:33-60`.

- **REQ-PRO-02:** Strict profile MUST set `min_binary_version = 0x05` to guarantee nested-binary support, matching `crates/lnmp-codec/src/binary/nested_encoder.rs`.  
  - Evidence: `profile.rs:108-135` sets the field; `tests/v05_integration_tests.rs` cover nested encoding requiring 0x05.

- **REQ-PRO-03:** Standard profile MUST still normalize booleans even when other canonical checks are relaxed.  
  - Evidence: `profile.rs:147-176` sets `canonical_boolean = true`; cross-checked by `lnmp-codec` normalization tests (e.g., `tests/sanitize_property.rs:258-285`).

---

## 5. Implementation Evidence Table

| Requirement | Code Reference | Test / Verification |
|-------------|----------------|---------------------|
| REQ-REC-01 | `crates/lnmp-core/src/record.rs` (`sorted_fields`) | Doc tests in file + `crates/lnmp-core/tests` (inline). |
| REQ-REC-02 | `record.rs::from_fields` | Doc tests show sorted outputs. |
| REQ-REC-03/04 | `record.rs::canonical_eq`, `canonical_hash` | Unit tests `record.rs:838-1054`. |
| REQ-BLD-01/02 | `crates/lnmp-core/src/builder.rs` | `test_builder_sorted_automatically`, `test_builder_chaining`. |
| REQ-ORD-01/02/03 | `record.rs::validate_field_ordering` et al. | Tests `record.rs:471-505`. |
| REQ-CAN-01 | `crates/lnmp-core/src/checksum.rs` | `test_serialize_bool_canonical` et al. |
| REQ-CAN-02 | `crates/lnmp-codec/src/encoder.rs` (`canonicalize_record`) | `tests/canonical_encoding_tests.rs`, `tests/binary_roundtrip.rs`. |
| REQ-CONF-01..03 | `crates/lnmp-core/src/profile.rs` | `crates/lnmp-codec/tests/integration_tests.rs`, `tests/schema_negotiation_tests.rs`. |
| REQ-PRO-02 | `profile.rs::strict` | `crates/lnmp-codec/tests/v05_integration_tests.rs:553-760`. |

(Line numbers referenced are approximate anchors derived from current repo revision; they should be updated alongside code changes.)

---

## 6. Next Steps

1. Link this document from the refreshed umbrella RFC once other modular specs exist.  
2. Expand the Implementation Evidence table to include multi-language SDK conformance once compliance runners (e.g., `tests/compliance/rust/runner.rs`) emit REQ IDs.  
3. Mirror this structure for text/binary/canonicalization/security documents to keep requirements tightly coupled to tested behavior.
