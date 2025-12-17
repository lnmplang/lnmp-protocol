# LNMP v0.4 Protocol Specification

**Version:** 0.4  
**Date:** November 16, 2025  
**Authors:** LNMP Protocol Working Group  
**Status:** Draft Specification

---

## Abstract

LNMP is a minimal, semantic-ID-based data format designed for efficient communication between AI agents and large language models (LLMs). This specification defines version 0.4 of the protocol, which introduces a binary transport format alongside the existing human-readable text format. LNMP achieves 7-12× token reduction compared to JSON while maintaining readability and providing deterministic canonicalization for semantic integrity. The dual-format architecture enables both human-debuggable text representation and space-efficient binary encoding with bidirectional conversion guarantees.

---

## Status of This Memo

This document specifies the Lightweight Nested Message Protocol version 0.4 for the Internet community. It defines the text format syntax, binary format encoding, type system, canonicalization rules, and interoperability requirements. This specification is intended for protocol implementers creating LNMP libraries across different programming languages and platforms.

This is a draft specification and may be updated based on implementation experience and community feedback. Implementations should be prepared for potential refinements in future versions while maintaining backward compatibility with the text format.

> **Living Spec Note (2025-02 refresh)**  
> Core normative definitions are being split into modular documents. Implementers SHOULD consult:
> - `spec/lnmp-core-spec.md` (records, FIDs, profiles)  
> - `spec/lnmp-text-format.md` (text syntax + parser behavior)  
> - `spec/lnmp-binary-format.md` (frame layout, type tags)  
> - `spec/lnmp-canonicalization.md` (deterministic form + round-trip guarantees)  
> - `spec/lnmp-security-compliance.md` (checksums, sanitizer, limits)  
> - `spec/lnmp-migration-versioning.md` (version deltas and negotiation)  
> These references are authoritative for requirement IDs and test citations until this document is fully refactored.

Distribution of this document is unlimited.

### Implementation Status (informative)

| Feature Set | Implementation | Evidence |
|-------------|----------------|----------|
| Core data structures (records, builder, profiles) | `crates/lnmp-core` (v0.5.13) | Unit tests in `crates/lnmp-core/src/record.rs`, `builder.rs`, `profile.rs`; requirements tracked in `spec/lnmp-core-spec.md`. |
| Text parser/encoder | `crates/lnmp-codec` (`parser.rs`, `encoder.rs`) | Integration/property tests under `crates/lnmp-codec/tests`, compliance runner `tests/compliance/rust/runner.rs`; requirements in `spec/lnmp-text-format.md`. |
| Canonicalization + round-trip | `lnmp-codec` canonical pipeline + property tests | `tests/binary_roundtrip.rs`, `tests/property_roundtrip.rs`; requirements in `spec/lnmp-canonicalization.md`. |
| Binary encoder/decoder (v0.4/v0.5) | `crates/lnmp-codec/src/binary/*` | Tests `tests/binary_roundtrip.rs`, `tests/v05_integration_tests.rs`, `tests/binary_error_handling.rs`; requirements in `spec/lnmp-binary-format.md`. |
| Security / sanitizer / limits | `crates/lnmp-sanitize`, `lnmp-core::checksum`, `lnmp-core::limits` | Property tests `tests/sanitize_property.rs`, checksum tests `crates/lnmp-core/src/checksum.rs`; requirements in `spec/lnmp-security-compliance.md`. |
| Migration / negotiation | `crates/lnmp-core/examples/v05_schema_negotiation.rs`, `tests/schema_negotiation_tests.rs` | Requirement coverage captured in `spec/lnmp-migration-versioning.md`. |

Status aligns with IETF RFC 7942 guidance: implementations listed are current at repository HEAD and may evolve.

---

## Table of Contents

1. [Introduction](#1-introduction)
   - 1.1 [Motivation](#11-motivation)
   - 1.2 [Design Principles](#12-design-principles)
   - 1.3 [Terminology](#13-terminology)

2. [Modular Specification Map](#2-modular-specification-map)
   - 2.1 [Core & Terminology](#21-core--terminology)
   - 2.2 [Text Format](#22-text-format)
   - 2.3 [Binary Format](#23-binary-format)
   - 2.4 [Canonicalization](#24-canonicalization)
   - 2.5 [Security & Compliance](#25-security--compliance)
   - 2.6 [Migration & Versioning](#26-migration--versioning)
   - 2.7 [Grammar & Error Classes](#27-grammar--error-classes)
   - 2.8 [Supplemental Specifications](#28-supplemental-specifications)

3. [Governance & Living Drafts](#3-governance--living-drafts)

4. [References](#4-references)
   - 4.1 [Normative References](#41-normative-references)
   - 4.2 [Informative References](#42-informative-references)

---

## 1. Introduction

### 1.1 Motivation

The proliferation of AI agents and large language models (LLMs) has created a need for data formats optimized for machine-to-machine communication in LLM-centric architectures. Existing data formats, while suitable for traditional applications, present significant challenges when used in LLM workflows.

#### Problems with Existing Formats

**JSON (JavaScript Object Notation)**

JSON has become the de facto standard for data interchange, but it imposes substantial overhead in LLM contexts:

- **Verbose Syntax**: Braces, brackets, quotes, and commas consume 30-50% of the serialized size
- **Token Inefficiency**: Special characters and structural elements tokenize poorly, resulting in 3-5× token overhead compared to semantic content
- **No Semantic IDs**: String-based keys provide no pattern for LLMs to learn field semantics
- **Ambiguous Canonicalization**: Multiple valid representations of the same data complicate caching and checksums

Example JSON representation (87 bytes, ~25 tokens):
```json
{"user_id":14532,"active":true,"roles":["admin","dev"]}
```

**XML (Extensible Markup Language)**

XML's verbosity makes it even less suitable for LLM communication:

- **Extreme Verbosity**: Opening and closing tags double the size of JSON equivalents
- **Complex Parsing**: Attributes, namespaces, and CDATA sections add parsing complexity
- **Poor Tokenization**: Angle brackets and tag names consume excessive tokens
- **Schema Overhead**: DTD and XSD requirements add complexity for simple use cases

**Protocol Buffers**

Protocol Buffers offer efficient binary encoding but lack LLM-friendliness:

- **Binary-Only**: Not human-readable or LLM-processable in native form
- **Schema Compilation**: Requires code generation and compilation steps
- **No Semantic IDs**: Field numbers are arbitrary without semantic meaning
- **Debugging Difficulty**: Binary format requires specialized tools for inspection

**MessagePack**

MessagePack provides compact binary encoding but shares similar limitations:

- **Binary-Only**: Cannot be directly processed by LLMs
- **No Semantic IDs**: Uses positional or string-based keys
- **Limited Type System**: Lacks support for semantic validation
- **No Canonical Form**: Multiple valid encodings of the same data

#### Comparison Table

| Feature | JSON | XML | Protocol Buffers | MessagePack | LNMP |
|---------|------|-----|------------------|-------------|------|
| LLM-Readable | ✓ | ✓ | ✗ | ✗ | ✓ |
| Token Efficient | ✗ | ✗ | N/A | N/A | ✓ |
| Semantic IDs | ✗ | ✗ | ~ | ✗ | ✓ |
| Binary Format | ✗ | ✗ | ✓ | ✓ | ✓ |
| Deterministic | ✗ | ✗ | ✓ | ✗ | ✓ |
| Human-Debuggable | ✓ | ✓ | ✗ | ✗ | ✓ |
| Schema-Free | ✓ | ~ | ✗ | ✓ | ✓ |
| Size Efficiency | 1.0× | 1.5× | 0.3× | 0.4× | 0.15× (text), 0.3× (binary) |

#### LNMP Design Goals

LNMP was designed from first principles to address these limitations:

**1. LLM-Friendly Syntax**

The text format uses minimal punctuation and semantic field IDs (FIDs) that enable LLMs to learn field meanings through pattern recognition. The syntax `F12=14532` is more token-efficient than `"user_id":14532` and provides a numeric pattern for semantic learning.

**2. Deterministic Canonicalization**

Every LNMP record has exactly one canonical representation, achieved through:
- Strict field ordering by FID (ascending)
- Normalized value formatting
- Consistent whitespace rules
- Deterministic type representation

This enables semantic checksums, reliable caching, and drift detection in LLM-generated outputs.

**3. Dual-Format Architecture**

LNMP provides both text and binary formats with bidirectional conversion:
- **Text format**: Human-readable, LLM-processable, debuggable
- **Binary format**: Space-efficient, fast parsing, network-optimized
- **Round-trip guarantee**: Text → Binary → Text produces identical canonical output

**4. Progressive Complexity**

Simple use cases remain simple (`F12=14532`), while complex scenarios are supported through optional features:
- Type hints for validation (`:i`, `:f`, `:b`)
- Checksums for integrity (`#36AAE667`)
- Nested structures for hierarchical data
- String arrays for collections

**5. Extensibility**

The protocol supports future extensions without breaking changes:
- Reserved type codes for new types
- Version negotiation in binary format
- Backward-compatible syntax additions
- Optional features that don't burden simple implementations

#### Use Case Examples

**Agent-to-Model Communication**

AI agents can send structured data to LLMs using minimal tokens:

```
F1=analyze_sentiment;F2="The product exceeded expectations";F3=detailed
```

This 3-field message uses ~15 tokens compared to ~30 tokens for equivalent JSON.

**Structured Prompt Engineering**

LLM prompts can include structured context efficiently:

```
F20=user_query;F22="What is the weather?";F21=location;F5="San Francisco";F50=units;F52=celsius
```

The semantic IDs (F5, F20, F21, etc.) help the LLM understand field relationships.

**Semantic Caching**

Deterministic canonicalization enables reliable caching:

```
F12=14532;F7=1;F23=["admin","dev"]  # Non-canonical input
F7=1                                 # Canonical output
F12=14532
F23=["admin","dev"]
```

The canonical form produces consistent checksums for cache keys, even when input ordering varies.

**Function Calling with Minimal Overhead**

LLMs can generate function calls in LNMP format with 60-70% fewer tokens:

```
F20=create_user;F21=alice;F5=alice@example.com;F23=["admin","dev"]
```

Compared to JSON function calling, this reduces token costs and improves response latency.

**Network Transport with Binary Encoding**

For network transmission, the same data converts to binary format with 30-50% size reduction:

```
Text:   F7=1;F12=14532;F23=["admin","dev"]  (33 bytes)
Binary: 04 00 03 07 00 03 01 0C 00 01 E4 E3 00 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76  (28 bytes)
```

The binary format maintains all semantic information while optimizing for transmission efficiency.

### 1.2 Design Principles

LNMP's design is guided by eight core principles that inform all protocol decisions. These principles ensure the protocol remains focused on its primary goal: efficient, reliable communication in LLM-centric systems.

#### Principle 1: LLM-First Design

**Statement**: The protocol syntax and semantics are optimized for large language model processing and generation.

**Rationale**: LLMs tokenize text differently than humans read it. Special characters like `{`, `}`, `"`, and `,` often consume full tokens, while semantic content may be split across multiple tokens. By minimizing special characters and using numeric field identifiers, LNMP reduces token consumption by 60-70% compared to JSON.

**Design Choices Influenced**:
- Semantic field IDs (F12) instead of string keys ("user_id")
- Minimal punctuation (= and ; only)
- Optional type hints that don't burden simple cases
- Unquoted strings where unambiguous

**Example Impact**:
```
JSON:  {"user_id":14532,"active":true}  (~12 tokens)
LNMP:  F12=14532;F7=1                   (~5 tokens)
```

#### Principle 2: Deterministic Canonicalization

**Statement**: Every record has exactly one canonical representation, enabling reliable checksums and caching.

**Rationale**: LLM-generated outputs may vary in formatting (field order, whitespace, number representation). Without canonicalization, semantically identical records would produce different checksums, breaking caching and drift detection. Deterministic canonicalization ensures that `F12=14532;F7=1` and `F7=1;F12=14532` both canonicalize to the same form.

**Design Choices Influenced**:
- Mandatory field ordering by FID (ascending)
- Normalized number formatting (no leading zeros, scientific notation thresholds)
- Consistent string quoting rules
- Standardized whitespace (newlines between fields in canonical form)

**Example Impact**:
```
Input:     F23=["admin","dev"];F7=1;F12=14532
Canonical: F7=1
           F12=14532
           F23=["admin","dev"]
```

#### Principle 3: Progressive Complexity

**Statement**: Simple use cases require minimal syntax, while complex scenarios are supported through optional features.

**Rationale**: Most agent-to-LLM communication involves simple field-value pairs. The protocol should not burden these common cases with complexity needed only for advanced scenarios. Optional features (type hints, checksums, nested structures) are available when needed but don't complicate basic usage.

**Design Choices Influenced**:
- Basic syntax is minimal: `F12=14532`
- Type hints are optional: `F12:i=14532`
- Checksums are optional: `F12=14532#36AAE667`
- Nested structures use clear delimiters: `F50={F12=1;F7=1}`

**Example Progression**:
```
Simple:     F12=14532
With hint:  F12:i=14532
With check: F12:i=14532#36AAE667
Nested:     F50={F12:i=14532#36AAE667;F7:b=1}
```

#### Principle 4: Dual-Format Architecture

**Statement**: The protocol provides both human-readable text and efficient binary formats with bidirectional conversion guarantees.

**Rationale**: Different use cases have different requirements. LLM processing requires readable text, while network transport benefits from binary efficiency. Rather than forcing a choice, LNMP provides both with guaranteed round-trip conversion, allowing developers to use the right format for each context.

**Design Choices Influenced**:
- Text format optimized for LLM tokenization
- Binary format optimized for size and parsing speed
- Canonical forms ensure round-trip stability
- Version byte in binary format enables evolution

**Example Round-Trip**:
```
Text:   F7=1;F12=14532
Binary: 04 00 02 07 00 03 01 0C 00 01 E4 E3 00
Text:   F7=1
        F12=14532
```

#### Principle 5: Zero-Ambiguity Parsing

**Statement**: The grammar is unambiguous, ensuring all implementations parse input identically.

**Rationale**: Ambiguous grammars lead to interoperability failures when different implementations make different parsing decisions. LNMP's grammar has no ambiguous productions, and value type precedence is explicitly defined. This ensures that `[{F1=a}]` is always parsed as a nested array, never as a string array.

**Design Choices Influenced**:
- Formal ABNF/EBNF grammar with no ambiguous productions
- Explicit value type precedence order
- Clear delimiter semantics (`;` vs newline, `{` vs `[`)
- Unambiguous checksum detection (exactly 8 hex digits after `#`)

**Example Disambiguation**:
```
{F1=a}      → Always nested record
[{F1=a}]    → Always nested array
[a,b]       → Always string array
#36AAE667   → Always checksum (8 hex digits)
#comment    → Always comment (not 8 hex digits)
```

#### Principle 6: Semantic Integrity

**Statement**: The protocol provides mechanisms to detect and prevent semantic drift in LLM-generated data.

**Rationale**: LLMs may generate outputs that are syntactically valid but semantically incorrect (wrong types, corrupted values, hallucinated fields). Optional checksums, type hints, and equivalence mappings help detect and correct these issues before they propagate through systems.

**Design Choices Influenced**:
- Optional checksums for critical fields
- Type hints for validation
- Equivalence mappings for normalization
- Strict mode for compliance testing

**Example Validation**:
```
F12:i=14532#36AAE667  → Type hint validates integer, checksum validates value
F7:b=2                → Error: boolean must be 0 or 1
F1:s=123              → Valid: string "123", not integer 123
```

#### Principle 7: Extensibility Without Breaking Changes

**Statement**: The protocol can evolve to support new features while maintaining backward compatibility.

**Rationale**: Protocols must evolve as use cases expand, but breaking changes fragment ecosystems. LNMP reserves type codes, uses version negotiation, and designs syntax extensions to be backward-compatible. A v0.3 parser can read v0.4 text output, and v0.4 parsers can read v0.3 input.

**Design Choices Influenced**:
- Reserved type tags (0x06, 0x07) for future nested structure support in binary format
- Version byte in binary format enables format evolution
- Optional features don't break parsers that don't support them
- Syntax extensions use new delimiters or prefixes

**Example Evolution**:
```
v0.3: F12=14532                    → Supported in v0.4
v0.4: F12=14532 (binary format)    → New feature, v0.3 ignores binary
v0.5: F50={F12=1} (binary nested)  → Will use reserved tag 0x06
```

#### Principle 8: Implementation Simplicity

**Statement**: The protocol can be implemented with hand-written parsers without complex tooling.

**Rationale**: Complex protocols requiring parser generators or extensive dependencies slow adoption. LNMP's grammar is simple enough for hand-written recursive descent parsers, and the binary format uses straightforward encoding (VarInt, IEEE 754). This lowers barriers to implementation across diverse languages and platforms.

**Design Choices Influenced**:
- Simple grammar suitable for recursive descent parsing
- No parser generator required
- Standard encodings (LEB128, IEEE 754, UTF-8)
- Minimal dependencies (no external libraries required)

**Example Parser Complexity**:
```rust
// Simple recursive descent parser structure
fn parse_field() -> Result<Field> {
    expect("F")?;
    let fid = parse_number()?;
    let type_hint = parse_type_hint()?;  // Optional
    expect("=")?;
    let value = parse_value()?;
    let checksum = parse_checksum()?;    // Optional
    Ok(Field { fid, type_hint, value, checksum })
}
```

### 1.3 Terminology

This section defines key terms used throughout the specification. For a complete glossary, see Appendix C.

**LNMP (Lightweight Nested Message Protocol)**  
The protocol defined in this specification, encompassing both text and binary formats.

**Record**  
A collection of fields representing a structured data object. In text format, fields are separated by semicolons or newlines. In binary format, fields are encoded as entries within a frame.

**Field**  
A single key-value pair within a record, consisting of a field identifier (FID), optional type hint, value, and optional checksum. Text format: `F<fid>[:<type_hint>]=<value>[#<checksum>]`

**FID (Field Identifier)**  
An unsigned 16-bit integer (0-65535) that uniquely identifies a field within a record. FIDs provide semantic meaning and enable deterministic field ordering. Example: `F12` has FID 12.

**Type Hint**  
An optional annotation specifying the expected type of a field value. Format: `:<code>` where code is one of `i` (integer), `f` (float), `b` (boolean), `s` (string), `sa` (string array), `r` (record), `ra` (record array). Example: `F12:i=14532`

**Canonical Form**  
The standardized, deterministic representation of a record. Canonical form requires fields sorted by FID in ascending order, normalized value formatting, and consistent whitespace. Two semantically identical records always have the same canonical form.

**Checksum**  
An optional 32-bit CRC32-based hash of a field's canonical value, used for integrity validation and semantic drift detection. Format: `#XXXXXXXX` (8 hexadecimal digits). Example: `F12=14532#36AAE667`

**VarInt (Variable-Length Integer)**  
A space-efficient integer encoding using LEB128 (Little Endian Base 128). Small integers use fewer bytes: 0-127 use 1 byte, 128-16383 use 2 bytes, etc. Used in binary format for field IDs, counts, and integer values.

**Binary Frame**  
The top-level structure of the binary format, consisting of a version byte, flags byte, entry count (VarInt), and sequence of binary entries. Example: `04 00 03 ...` (version 0x04, flags 0x00, 3 entries)

**Type Tag**  
A single byte in binary format identifying the type of a value. Tags: 0x01 (integer), 0x02 (float), 0x03 (boolean), 0x04 (string), 0x05 (string array), 0x06 (record, reserved), 0x07 (record array, reserved).

**Nested Record**  
A record contained within another record as a field value. Text format: `F50={F12=1;F7=1}`. Supported in text format (v0.3+), reserved for binary format (v0.5).

**Nested Array**  
An array of records as a field value. Text format: `F60=[{F1=alice},{F1=bob}]`. Supported in text format (v0.3+), reserved for binary format (v0.5).

**String Array**  
An array of string values. Text format: `F23=["admin","dev"]`. Binary format: count (VarInt) followed by length-prefixed UTF-8 strings.

**Escape Sequence**  
A backslash-prefixed character in quoted strings representing special characters. Supported: `\\` (backslash), `\"` (quote), `\n` (newline), `\r` (carriage return), `\t` (tab).

**Separator**  
A delimiter between fields in text format. Semicolon (`;`) for inline format, newline (`\n`) for multiline format. Both are semantically equivalent; canonical form uses newlines.

**Conformance Level**  
A tier of protocol support: Level 1 (Minimal) supports primitive types and text format; Level 2 (Standard) adds type hints and nested structures; Level 3 (Full) adds checksums and binary format.

**Round-Trip Guarantee**  
The property that converting between formats preserves semantic content. Text → Binary → Text produces canonical text. Binary → Text → Binary produces identical binary (byte-for-byte).

**Semantic ID**  
A field identifier (FID) that carries semantic meaning, enabling LLMs to learn field relationships through pattern recognition. Example: F12 might consistently represent user IDs across different records.

**LEB128 (Little Endian Base 128)**  
A variable-length encoding for integers where each byte contributes 7 bits of data and 1 continuation bit. Used for VarInt encoding in binary format. Defined in DWARF debugging format specification.

**IEEE 754**  
The standard for floating-point arithmetic. LNMP uses IEEE 754 double-precision (64-bit) format for float values in both text and binary representations.

**UTF-8**  
The character encoding standard for all string values in LNMP. Both text and binary formats use UTF-8 encoding. Invalid UTF-8 sequences are rejected during parsing.

**Strict Mode**  
A parsing mode that treats all deviations from canonical form as errors. Used for validation, compliance testing, and scenarios requiring exact format adherence.

**Equivalence Mapping**  
An optional feature that normalizes semantically equivalent values (e.g., "yes" → 1 for boolean fields, "admin" → "administrator"). Configured per-field via semantic dictionary.

---

## 2. Modular Specification Map

The detailed content that formerly filled this RFC now lives in purpose-built Markdown files inside `spec/`. Each document lists requirement IDs (REQ-*) that map directly to the code and tests exercising them. The subsections below summarize the scope of each file so implementers can jump straight to the canonical source.

### 2.1 Core & Terminology

- **File:** `spec/lnmp-core-spec.md`
- **Focus:** Record/field definitions, canonical ordering, semantic checksums, and protocol profiles (`Loose`, `Standard`, `Strict`).
- **Evidence:** `crates/lnmp-core`, `crates/lnmp-codec` builder/encoder tests, and REQ-REC / REQ-CONF families.

### 2.2 Text Format

- **File:** `spec/lnmp-text-format.md`
- **Focus:** Text grammar, separators, whitespace, type hints, checksum/comment rules, and encoder constraints.
- **Evidence:** `crates/lnmp-codec/src/parser.rs`, canonical encoder tests, and REQ-TXT/REQ-ENC compliance vectors.

### 2.3 Binary Format

- **File:** `spec/lnmp-binary-format.md`
- **Focus:** Frame layout, VarInt encoding, type tags, nested encoding rules, and decoder validation.
- **Evidence:** `crates/lnmp-codec/src/binary/*`, binary round-trip/error suites, and REQ-BIN/REQ-CAN-BIN IDs.

### 2.4 Canonicalization

- **File:** `spec/lnmp-canonicalization.md`
- **Focus:** Text/binary canonical rules, round-trip guarantees, strict-mode behavior, and fixture coverage (`spec/examples/`).
- **Evidence:** `crates/lnmp-codec/src/encoder.rs`, property tests, and REQ-CAN-* mappings.

### 2.5 Security & Compliance

- **File:** `spec/lnmp-security-compliance.md`
- **Focus:** Semantic checksums, sanitizer behavior, structural limits, schema negotiation, and compliance runner requirements.
- **Evidence:** `crates/lnmp-core/src/checksum.rs`, `crates/lnmp-sanitize`, and the Rust compliance harness.

### 2.6 Migration & Versioning

- **File:** `spec/lnmp-migration-versioning.md`
- **Focus:** Version deltas (v0.3→v0.4→v0.5), capability negotiation, compatibility matrices, and upgrade guidance.
- **Evidence:** README release notes, schema negotiation examples/tests, and backward compatibility suites.

### 2.7 Grammar & Error Classes

- **Files:** `spec/grammar.md`, `spec/error-classes.md`
- **Focus:** Formal ABNF/EBNF definitions, parser precedence, strict-mode checks, and canonical error classifications.
- **Evidence:** Parser unit tests, compliance error cases, and REQ-ERR-* references.

### 2.8 Supplemental Specifications

- **Files:** `spec/lnmp-container-format.md`, `spec/lnmp-envelope-v1.0.md`, `spec/lnmp-metadata-extension-rfc.md`, `spec/lnmp-net-v1.md`, etc.
- **Focus:** Optional envelopes, metadata, transport bindings, and other ecosystem extensions that build on the core spec set.

## 3. Governance & Living Drafts

The LNMP working group coordinates protocol work through the “Spec Refresh” GitHub Project board and the labels `spec-gap`, `needs-tests`, and `needs-doc`. Key operational practices:

- **Fixture verification:** `.github/workflows/spec-fixtures.yml` invokes `cargo run -p lnmp-compliance-tests --bin lnmp-verify-examples` on every PR to guarantee that documented fixtures continue to round-trip through the shipping parser/encoder pairings.
- **Compliance visibility:** `tests/compliance/test-cases.yaml` now carries REQ IDs, and the Rust runner appends those IDs to any failing test, keeping CI output aligned with the specs.
- **Living Draft cadence:** At the end of each quarter (or earlier for major changes) maintainers tag `spec-LD-YYYYQn` after CI succeeds. Release notes identify the exact spec files and crate revisions included in the snapshot so downstream SDKs can pin against a stable reference.
- **Change approval:** Pull requests that touch normative text must cite the relevant REQ IDs, update associated tests/fixtures, and refresh the Implementation Status table near the top of this file.

## 4. References

### 4.1 Normative References

- `spec/lnmp-core-spec.md`
- `spec/lnmp-text-format.md`
- `spec/lnmp-binary-format.md`
- `spec/lnmp-canonicalization.md`
- `spec/lnmp-security-compliance.md`
- `spec/lnmp-migration-versioning.md`
- `spec/grammar.md`
- `spec/error-classes.md`

### 4.2 Informative References

- `spec/lnmp-container-format.md`
- `spec/lnmp-envelope-v1.0.md`
- `spec/lnmp-metadata-extension-rfc.md`
- `spec/lnmp-net-v1.md`
- `README.md`
- `tests/compliance/` (REQ-tagged YAML suites and fixture verifier)
