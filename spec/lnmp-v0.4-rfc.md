# LNMP v0.4 Protocol Specification

**Version:** 0.4  
**Date:** November 16, 2025  
**Authors:** LNMP Protocol Working Group  
**Status:** Draft Specification

---

## Abstract

LNMP (Lightweight Nested Message Protocol) is a minimal, semantic-ID-based data format designed for efficient communication between AI agents and large language models (LLMs). This specification defines version 0.4 of the protocol, which introduces a binary transport format alongside the existing human-readable text format. LNMP achieves 7-12× token reduction compared to JSON while maintaining readability and providing deterministic canonicalization for semantic integrity. The dual-format architecture enables both human-debuggable text representation and space-efficient binary encoding with bidirectional conversion guarantees.

---

## Status of This Memo

This document specifies the Lightweight Nested Message Protocol version 0.4 for the Internet community. It defines the text format syntax, binary format encoding, type system, canonicalization rules, and interoperability requirements. This specification is intended for protocol implementers creating LNMP libraries across different programming languages and platforms.

This is a draft specification and may be updated based on implementation experience and community feedback. Implementations should be prepared for potential refinements in future versions while maintaining backward compatibility with the text format.

Distribution of this document is unlimited.

---

## Table of Contents

1. [Introduction](#1-introduction)
   - 1.1 [Motivation](#11-motivation)
   - 1.2 [Design Principles](#12-design-principles)
   - 1.3 [Terminology](#13-terminology)

2. [Text Format Specification](#2-text-format-specification)
   - 2.1 [Syntax Overview](#21-syntax-overview)
   - 2.2 [Field Structure](#22-field-structure)
   - 2.3 [Value Types](#23-value-types)
   - 2.4 [Nested Structures](#24-nested-structures)
   - 2.5 [Escape Sequences](#25-escape-sequences)
   - 2.6 [Comments and Whitespace](#26-comments-and-whitespace)

3. [Binary Format Specification](#3-binary-format-specification)
   - 3.1 [Frame Structure](#31-frame-structure)
   - 3.2 [VarInt Encoding](#32-varint-encoding)
   - 3.3 [Type Tags](#33-type-tags)
   - 3.4 [Value Encoding](#34-value-encoding)
   - 3.5 [Byte Order](#35-byte-order)
   - 3.6 [Complete Binary Example](#36-complete-binary-example)

4. [Canonicalization](#4-canonicalization)
   - 4.1 [Text Canonical Form](#41-text-canonical-form)
   - 4.2 [Binary Canonical Form](#42-binary-canonical-form)
   - 4.3 [Round-Trip Guarantees](#43-round-trip-guarantees)

5. [Type System](#5-type-system)
   - 5.1 [Primitive Types](#51-primitive-types)
   - 5.2 [Composite Types](#52-composite-types)
   - 5.3 [Type Hints](#53-type-hints)
   - 5.4 [Type Coercion](#54-type-coercion)

6. [Formal Grammar](#6-formal-grammar)
   - 6.1 [ABNF Grammar](#61-abnf-grammar)
   - 6.2 [EBNF Grammar](#62-ebnf-grammar)
   - 6.3 [Parsing Precedence](#63-parsing-precedence)

7. [Error Classes](#7-error-classes)
   - 7.1 [Error Categories](#71-error-categories)
   - 7.2 [Error Codes](#72-error-codes)
   - 7.3 [Error Handling](#73-error-handling)

8. [Security Considerations](#8-security-considerations)
   - 8.1 [Buffer Overflow Protection](#81-buffer-overflow-protection)
   - 8.2 [Denial of Service](#82-denial-of-service)
   - 8.3 [Resource Limits](#83-resource-limits)
   - 8.4 [String Injection](#84-string-injection)
   - 8.5 [Checksum Validation](#85-checksum-validation)
   - 8.6 [Version Negotiation](#86-version-negotiation)

9. [Interoperability](#9-interoperability)
   - 9.1 [Conformance Levels](#91-conformance-levels)
   - 9.2 [Version Negotiation](#92-version-negotiation)
   - 9.3 [Test Vectors](#93-test-vectors)

10. [Migration Guide](#10-migration-guide)
    - 10.1 [From v0.3 to v0.4](#101-from-v03-to-v04)
    - 10.2 [Backward Compatibility](#102-backward-compatibility)

11. [References](#11-references)
    - 11.1 [Normative References](#111-normative-references)
    - 11.2 [Informative References](#112-informative-references)

12. [Appendices](#12-appendices)
    - Appendix A: [Complete Examples](#appendix-a-complete-examples)
    - Appendix B: [Compliance Checklist](#appendix-b-compliance-checklist)
    - Appendix C: [Glossary](#appendix-c-glossary)

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
- Checksums for integrity (`#6A93B3F1`)
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
F10=user_query;F11="What is the weather?";F20=location;F21="San Francisco";F30=units;F31=celsius
```

The semantic IDs (F10, F11, F20, etc.) help the LLM understand field relationships.

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
F1=create_user;F10=alice;F11=alice@example.com;F12=["admin","dev"]
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
- Checksums are optional: `F12=14532#6A93B3F1`
- Nested structures use clear delimiters: `F50={F12=1;F7=1}`

**Example Progression**:
```
Simple:     F12=14532
With hint:  F12:i=14532
With check: F12:i=14532#6A93B3F1
Nested:     F50={F12:i=14532#6A93B3F1;F7:b=1}
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
#6A93B3F1   → Always checksum (8 hex digits)
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
F12:i=14532#6A93B3F1  → Type hint validates integer, checksum validates value
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
An optional 32-bit CRC32-based hash of a field's canonical value, used for integrity validation and semantic drift detection. Format: `#XXXXXXXX` (8 hexadecimal digits). Example: `F12=14532#6A93B3F1`

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

## 2. Text Format Specification

The text format is the human-readable representation of LNMP data, optimized for LLM processing while maintaining debuggability. This section defines the complete syntax, value types, and encoding rules for the text format.

### 2.1 Syntax Overview

The LNMP text format uses a minimal syntax based on field-value pairs with semantic field identifiers.

#### Basic Syntax

The fundamental syntax pattern is:

```
F<fid>=<value>
```

Where:
- `F` is the field prefix (uppercase, required)
- `<fid>` is the field identifier, an unsigned integer from 0 to 65535
- `=` is the assignment operator
- `<value>` is the field value (type determined by syntax)

#### Simple Examples

```
F12=14532
F7=1
F1=alice
F23=["admin","dev"]
```

These examples demonstrate:
- `F12=14532` - Integer field with FID 12
- `F7=1` - Boolean field with FID 7 (1 = true)
- `F1=alice` - String field with FID 1 (unquoted)
- `F23=["admin","dev"]` - String array field with FID 23

#### Field Separators

Fields can be separated using either semicolons or newlines. Both separators are semantically equivalent, though canonical form uses newlines.

**Inline Format (Semicolon Separator)**

```
F7=1;F12=14532;F23=["admin","dev"]
```

Multiple fields on a single line, separated by semicolons. Useful for compact representation and single-line transmission.

**Multiline Format (Newline Separator)**

```
F7=1
F12=14532
F23=["admin","dev"]
```

One field per line. This is the canonical form and is preferred for readability and version control.

**Mixed Format**

```
F7=1;F12=14532
F23=["admin","dev"]
```

Semicolons and newlines can be mixed. Parsers MUST treat both separators identically. During canonicalization, all separators are normalized to newlines.

#### Parsing Rules

1. **Whitespace**: Spaces and tabs around separators and operators are ignored (except within quoted strings)
2. **Field Order**: Fields may appear in any order in input, but canonical form requires ascending FID order
3. **Duplicate Fields**: Multiple fields with the same FID are allowed in input but may be rejected in strict mode
4. **Empty Records**: A record with zero fields is valid (empty string or whitespace-only)
5. **Trailing Separators**: Trailing semicolons or newlines are permitted and ignored

#### Examples with Whitespace Variations

All of the following are semantically equivalent:

```
F12=14532
F12 = 14532
F12  =  14532
F12= 14532
```

However, canonical form uses no whitespace around the equals sign: `F12=14532`

### 2.2 Field Structure

The complete field syntax includes optional components for type validation and integrity checking.

#### Complete Syntax

```
F<fid>[:<type_hint>]=<value>[#<checksum>]
```

Components (in order):
1. **Field Prefix**: `F` (uppercase, required)
2. **Field ID**: Unsigned integer 0-65535 (required)
3. **Type Hint**: Optional type annotation (format: `:<code>`)
4. **Assignment**: `=` operator (required)
5. **Value**: Field value, type determined by syntax (required)
6. **Checksum**: Optional 32-bit integrity hash (format: `#XXXXXXXX`)

#### Component Details

**Field Prefix (`F`)**

- Always uppercase `F`
- Distinguishes fields from other syntax elements
- Required for all fields
- Example: `F12=14532` (not `f12=14532` or `12=14532`)

**Field ID (`<fid>`)**

- Unsigned 16-bit integer: 0 to 65535
- No leading zeros (except for `0` itself)
- Provides semantic meaning (e.g., F12 might always represent user IDs)
- Used for field ordering in canonical form
- Examples: `F0`, `F12`, `F999`, `F65535`
- Invalid: `F012` (leading zero), `F99999` (out of range)

**Type Hint (`:<type_hint>`)**

- Optional annotation specifying expected value type
- Format: colon followed by type code
- Type codes:
  - `:i` - Integer (signed 64-bit)
  - `:f` - Float (IEEE 754 double)
  - `:b` - Boolean (0 or 1)
  - `:s` - String (UTF-8)
  - `:sa` - String Array
  - `:r` - Nested Record
  - `:ra` - Nested Record Array
- Used for validation and disambiguation
- Examples: `F12:i=14532`, `F7:b=1`, `F1:s=123`

**Assignment Operator (`=`)**

- Single equals sign
- Required between field identifier and value
- Whitespace around operator is ignored (but not in canonical form)
- Example: `F12=14532` (not `F12:14532` or `F12==14532`)

**Value (`<value>`)**

- The field's data content
- Type determined by syntax (see Section 2.3)
- Can be primitive (integer, float, boolean, string) or composite (array, nested record)
- Examples: `14532`, `3.14`, `1`, `"hello"`, `["a","b"]`, `{F1=x}`

**Checksum (`#<checksum>`)**

- Optional 32-bit CRC32-based hash
- Format: `#` followed by exactly 8 hexadecimal digits (uppercase or lowercase)
- Computed over canonical representation of the value
- Used for integrity validation and drift detection
- Examples: `F12=14532#6A93B3F1`, `F1=alice#A1B2C3D4`
- Not a checksum: `#ABC` (too short), `#comment` (not hex)

#### Examples of Each Variation

**Basic Field (No Optional Components)**

```
F12=14532
```

Minimal syntax: field prefix, ID, equals, value.

**Field with Type Hint**

```
F12:i=14532
```

Explicitly declares the value as an integer. Useful for validation and disambiguation (e.g., distinguishing integer `0` from string `"0"`).

**Field with Checksum**

```
F12=14532#6A93B3F1
```

Includes integrity hash. Parser can validate that the value matches the checksum.

**Field with Type Hint and Checksum**

```
F12:i=14532#6A93B3F1
```

Complete field with all optional components. Provides maximum validation and integrity checking.

**String Field with Type Hint**

```
F1:s=123
```

Forces interpretation as string `"123"` rather than integer `123`. Type hint disambiguates.

**Boolean Field with Type Hint**

```
F7:b=1
```

Explicitly marks value as boolean. Validates that value is `0` or `1`.

**Array Field with Type Hint**

```
F23:sa=["admin","dev"]
```

Declares field as string array. Validates array structure and element types.

**Nested Record with Type Hint**

```
F50:r={F12=1;F7=1}
```

Declares field as nested record. Validates nested structure.

#### Checksum vs Comment Disambiguation

The `#` character can introduce either a checksum or a comment. Parsers MUST use the following rule:

- If `#` is followed by exactly 8 hexadecimal digits (0-9, A-F, a-f), it is a checksum
- Otherwise, it is a comment (continues to end of line)

**Examples:**

```
F12=14532#6A93B3F1      # Checksum (8 hex digits)
F12=14532#ABCDEF01      # Checksum (8 hex digits)
F12=14532#ABC           # Comment (not 8 hex digits)
F12=14532#comment text  # Comment (not 8 hex digits)
F12=14532 #6A93B3F1     # Comment (space before #)
```

Note: Whitespace before `#` makes it a comment, not a checksum. Checksums must immediately follow the value with no intervening whitespace.

### 2.3 Value Types

LNMP supports seven value types: five primitive types and two composite types. The type of a value is determined by its syntax.

#### Primitive Types

| Type | Syntax Pattern | Examples | Range/Constraints |
|------|----------------|----------|-------------------|
| Integer | `-?[0-9]+` | `42`, `-123`, `0` | -2^63 to 2^63-1 (i64) |
| Float | `-?[0-9]+\.[0-9]+` | `3.14`, `-2.5`, `0.0` | IEEE 754 double-precision |
| Boolean | `0` \| `1` | `0` (false), `1` (true) | Exactly 0 or 1 |
| String | `"..."` or `[A-Za-z0-9_.-]+` | `"hello"`, `simple` | UTF-8 encoded, max 1MB recommended |
| String Array | `[str,str,...]` | `["a","b"]`, `[]` | Comma-separated strings, max 10K elements recommended |

#### Composite Types

| Type | Syntax Pattern | Examples | Constraints |
|------|----------------|----------|-------------|
| Nested Record | `{F<fid>=<val>;...}` | `{F12=1;F7=1}` | Max depth 10 recommended |
| Nested Array | `[{...},{...}]` | `[{F1=a},{F1=b}]` | Max 1K elements recommended |

#### Integer Type

**Syntax**: Optional minus sign followed by one or more digits.

**Pattern**: `-?[0-9]+`

**Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 (signed 64-bit)

**Canonical Form**:
- No leading zeros (except for `0` itself)
- Minus sign directly adjacent to first digit (no space)

**Examples**:
```
F12=0           # Zero
F12=42          # Positive integer
F12=-123        # Negative integer
F12=9223372036854775807  # Maximum i64
```

**Invalid**:
```
F12=042         # Leading zero (except for "0")
F12=- 123       # Space after minus sign
F12=1.0         # Decimal point makes it a float
```

#### Float Type

**Syntax**: Optional minus sign, digits, decimal point, digits.

**Pattern**: `-?[0-9]+\.[0-9]+`

**Format**: IEEE 754 double-precision (64-bit)

**Special Values**:
- `NaN` - Not a Number (text representation: `"NaN"`)
- `Infinity` - Positive infinity (text representation: `"Infinity"`)
- `-Infinity` - Negative infinity (text representation: `"-Infinity"`)

**Canonical Form**:
- At least one digit before and after decimal point
- No trailing zeros after decimal point (except one zero if needed: `1.0`)
- Scientific notation for |x| ≥ 1e15 or |x| < 1e-6
- Minus sign directly adjacent to first digit

**Examples**:
```
F12=3.14        # Standard float
F12=-2.5        # Negative float
F12=0.0         # Zero
F12=1.23e-10    # Scientific notation
F12=1.5e15      # Large number
```

**Invalid**:
```
F12=3.          # Missing digits after decimal
F12=.14         # Missing digits before decimal
F12=3.140       # Trailing zero (not canonical)
```

#### Boolean Type

**Syntax**: Exactly `0` or `1`.

**Values**:
- `0` - False
- `1` - True

**Canonical Form**: Always `0` or `1` (no other representations).

**Examples**:
```
F7=0            # False
F7=1            # True
```

**Invalid**:
```
F7=true         # String, not boolean
F7=false        # String, not boolean
F7=2            # Out of range
```

**Note**: Without a type hint, `0` and `1` are interpreted as booleans by default. Use type hints to disambiguate:
```
F7:b=1          # Explicitly boolean
F7:i=1          # Explicitly integer
F7:s=1          # Explicitly string "1"
```

#### String Type

Strings can be represented in two forms: quoted and unquoted.

**Quoted Strings**

**Syntax**: Double quotes with optional escape sequences.

**Pattern**: `"[^"\\]*(\\.[^"\\]*)*"`

**Encoding**: UTF-8

**Use When**:
- String contains spaces or special characters
- String contains escape sequences
- String starts with a digit but should be interpreted as string
- String could be confused with other types

**Examples**:
```
F1="hello world"        # Contains space
F1="line\nbreak"        # Contains escape sequence
F1="123"                # Starts with digit
F1="true"               # Could be confused with boolean
F1="unicode: 🎯"        # Unicode characters
```

**Unquoted Strings**

**Syntax**: Alphanumeric characters, underscore, hyphen, dot.

**Pattern**: `[A-Za-z0-9_.-]+`

**Use When**:
- String contains only allowed characters
- String doesn't start with a digit
- String cannot be confused with other types

**Examples**:
```
F1=alice                # Simple identifier
F1=user_name            # With underscore
F1=api-key              # With hyphen
F1=version.1.0          # With dots
```

**Invalid Unquoted**:
```
F1=hello world          # Space requires quotes
F1=user@example         # @ requires quotes
F1=123abc               # Starts with digit, requires quotes
```

**String Encoding Rules**

1. All strings MUST be valid UTF-8
2. Invalid UTF-8 sequences MUST be rejected
3. No byte order mark (BOM) in strings
4. Maximum length: implementation-defined (1MB recommended)
5. Empty strings are valid: `F1=""`

#### String Array Type

**Syntax**: Square brackets containing comma-separated strings.

**Pattern**: `\[([^,\]]+,)*[^,\]]*\]`

**Elements**: Each element follows string rules (quoted or unquoted)

**Separator**: Comma (`,`) between elements

**Canonical Form**:
- No spaces after commas
- Elements in original order (not sorted)
- Quoted strings use minimal quoting

**Examples**:
```
F23=["admin","dev"]             # Two quoted strings
F23=[admin,dev]                 # Two unquoted strings
F23=[]                          # Empty array
F23=[single]                    # Single element
F23=["a","b","c","d"]           # Multiple elements
F23=["hello world",simple]      # Mixed quoting
```

**Invalid**:
```
F23=[admin, dev]                # Space after comma (not canonical)
F23=["admin" "dev"]             # Missing comma
F23=[admin;dev]                 # Wrong separator
```

**Constraints**:
- Maximum elements: implementation-defined (10,000 recommended)
- Each element subject to string length limits
- Duplicate elements are allowed
- Order is preserved

### 2.4 Nested Structures

LNMP supports two types of nested structures: nested records and nested arrays. These enable hierarchical data representation while maintaining the protocol's simplicity.

#### Nested Records

A nested record is a complete LNMP record contained within a field value.

**Syntax**:
```
F<fid>={F<fid>=<value>;F<fid>=<value>}
```

**Structure**:
- Opening brace `{` starts nested context
- Fields separated by semicolons (required, newlines not allowed in nested context)
- Fields follow same syntax as top-level fields
- Closing brace `}` ends nested context

**Canonical Form Rules**:
- Fields sorted by FID in ascending order
- No spaces around braces or semicolons
- Semicolon required between fields (even in canonical form)
- No trailing semicolon before closing brace

**Simple Examples**:
```
F50={F12=1;F7=1}
F100={F1=alice;F2=admin}
F200={F10=nested;F11=data;F12=here}
```

**Multiple Nesting Levels**:
```
F100={F1=user;F2={F10=nested;F11=data}}
F200={F1=dept;F2={F10=eng;F11={F20=team;F21=backend}}}
```

**Nested Records with Arrays**:
```
F200={F1=alice;F2=["admin","dev"]}
F300={F1=user;F2=["role1","role2"];F3={F10=meta}}
```

**Empty Nested Record**:
```
F50={}
```

**Nesting Depth**:
- Arbitrary depth supported by grammar
- Recommended maximum: 10 levels
- Implementations SHOULD enforce depth limits to prevent stack overflow
- Exceeding depth limit SHOULD produce error code 4001 (NestingTooDeep)

**Example with Maximum Recommended Depth**:
```
F1={F2={F3={F4={F5={F6={F7={F8={F9={F10=deep}}}}}}}}}
```

#### Nested Arrays

A nested array is an array of records as a field value.

**Syntax**:
```
F<fid>=[{F<fid>=<value>},{F<fid>=<value>}]
```

**Structure**:
- Opening bracket `[` starts array context
- Each element is a complete record in `{...}` format
- Elements separated by commas
- Closing bracket `]` ends array context

**Canonical Form Rules**:
- No spaces after commas
- Each record follows nested record canonical rules
- Elements in original order (not sorted)
- Empty arrays allowed: `[]`

**Simple Examples**:
```
F60=[{F12=1},{F12=2},{F12=3}]
F200=[{F1=alice;F2=admin},{F1=bob;F2=user}]
```

**Empty Nested Array**:
```
F60=[]
```

**Single Element**:
```
F60=[{F1=only}]
```

**Multiple Fields Per Record**:
```
F200=[
  {F1=alice;F2=admin;F3=active},
  {F1=bob;F2=user;F3=inactive},
  {F1=carol;F2=admin;F3=active}
]
```

Note: The multiline formatting above is for readability only. Canonical form would be single-line with no spaces.

**Nested Arrays with Nested Records**:
```
F400=[
  {F1=dept;F2={F10=eng;F11=dev}},
  {F1=dept;F2={F10=sales;F11=west}}
]
```

**Mixed Nesting**:
```
F500={
  F1=company;
  F2=[{F10=dept1},{F10=dept2}];
  F3={F20=meta;F21=data}
}
```

**Constraints**:
- Maximum elements: implementation-defined (1,000 recommended)
- Each record subject to nesting depth limits
- Total size limits apply to entire structure

#### Nesting Recommendations

**Performance Considerations**:
1. Deep nesting increases parsing complexity
2. Wide arrays increase memory usage
3. Combined depth and width multiply resource requirements

**Best Practices**:
1. Keep nesting depth ≤ 5 for optimal performance
2. Limit array sizes to hundreds, not thousands
3. Consider flattening deeply nested structures
4. Use top-level fields with semantic IDs instead of deep nesting

**Example - Prefer Flat Structure**:

Instead of:
```
F1={F2={F3={F4={F5=value}}}}
```

Prefer:
```
F1=value
F2=context1
F3=context2
F4=context3
```

### 2.5 Escape Sequences

Quoted strings support escape sequences for representing special characters that cannot be directly included in the string.

#### Supported Escape Sequences

| Escape | Character | Unicode | Description |
|--------|-----------|---------|-------------|
| `\\` | `\` | U+005C | Backslash |
| `\"` | `"` | U+0022 | Double quote |
| `\n` | LF | U+000A | Line feed (newline) |
| `\r` | CR | U+000D | Carriage return |
| `\t` | TAB | U+0009 | Horizontal tab |

#### Escape Sequence Rules

1. Escape sequences are ONLY recognized in quoted strings
2. Unquoted strings cannot contain escape sequences
3. Invalid escape sequences MUST produce error code 1003 (InvalidEscapeSequence)
4. Backslash followed by any other character is invalid
5. In canonical form, only necessary escapes are used

#### Examples

**Newline**:
```
F1="line1\nline2"
```

Represents:
```
line1
line2
```

**Backslash**:
```
F1="path\\to\\file"
```

Represents: `path\to\file`

**Double Quote**:
```
F1="say \"hello\""
```

Represents: `say "hello"`

**Tab**:
```
F1="column1\tcolumn2"
```

Represents: `column1	column2` (with tab character)

**Carriage Return**:
```
F1="line1\r\nline2"
```

Represents Windows-style line ending: `line1` + CR + LF + `line2`

**Multiple Escapes**:
```
F1="path: \"C:\\Users\\Alice\"\nstatus: active"
```

Represents:
```
path: "C:\Users\Alice"
status: active
```

**Unicode Characters**:
```
F1="emoji: 🎯 unicode: ñ"
```

Unicode characters are included directly (UTF-8 encoded), not escaped. No `\u` or `\x` escape sequences are supported.

#### Invalid Escape Sequences

The following escape sequences are NOT supported and MUST produce errors:

```
F1="test\x41"           # \x not supported (error 1003)
F1="test\u0041"         # \u not supported (error 1003)
F1="test\0"             # \0 not supported (error 1003)
F1="test\a"             # \a not supported (error 1003)
F1="test\b"             # \b not supported (error 1003)
F1="test\f"             # \f not supported (error 1003)
F1="test\v"             # \v not supported (error 1003)
```

**Rationale**: LNMP supports only the five most common escape sequences to keep parsing simple. Unicode characters should be included directly as UTF-8, not via escape sequences.

#### Canonical Form Escaping

In canonical form, the following escaping rules apply:

1. **Always Escape**:
   - Backslash: `\` → `\\`
   - Double quote: `"` → `\"`

2. **Conditionally Escape** (only if present):
   - Newline: LF → `\n`
   - Carriage return: CR → `\r`
   - Tab: TAB → `\t`

3. **Never Escape**:
   - Printable ASCII characters (U+0020 to U+007E, except `\` and `"`)
   - Unicode characters (U+0080 and above)

**Example Canonicalization**:

Input (with actual tab and newline characters):
```
F1="hello	world
next line"
```

Canonical form:
```
F1="hello\tworld\nnext line"
```

### 2.6 Comments and Whitespace

LNMP supports comments for documentation and whitespace for formatting, both of which are ignored during parsing.

#### Whitespace Rules

**Ignored Whitespace**:
- Spaces (U+0020) and tabs (U+0009) between tokens
- Newlines (U+000A) as field separators
- Leading and trailing whitespace in the record

**Preserved Whitespace**:
- Whitespace within quoted strings
- Whitespace is part of the string value

**Canonical Form**:
- No spaces or tabs between tokens
- Newlines only as field separators (one per field)
- No leading or trailing whitespace

**Examples**:

All of these are equivalent:
```
F12=14532
F12 = 14532
F12  =  14532
  F12=14532
F12=14532  
```

Canonical form: `F12=14532`

**Whitespace in Strings**:
```
F1="hello world"        # Space is part of the value
F1=hello world          # Invalid: unquoted strings can't contain spaces
```

#### Comment Rules

**Comment Syntax**:
- Comments start with `#` and continue to the end of the line
- Comments are completely ignored by the parser
- Comments can appear on their own line or after a field

**Comment vs Checksum Disambiguation**:

The parser MUST distinguish between checksums and comments using this rule:

- `#` followed by exactly 8 hexadecimal digits (0-9, A-F, a-f) with no intervening whitespace → **checksum**
- Any other `#` → **comment**

**Examples**:

```
# This is a comment
F12=14532

F12=14532  # This is an inline comment

F12=14532#6A93B3F1      # This is a checksum (8 hex digits)
F12=14532#ABC           # This is a comment (not 8 hex digits)
F12=14532#comment text  # This is a comment (not hex)
F12=14532 #6A93B3F1     # This is a comment (space before #)
```

**Important**: Checksums must immediately follow the value with no whitespace. Any whitespace before `#` makes it a comment, not a checksum.

#### Comment Placement

**Full-Line Comments**:
```
# Configuration for user profile
F12=14532
F7=1
# Roles assigned to user
F23=["admin","dev"]
```

**Inline Comments**:
```
F12=14532  # User ID
F7=1       # Active status
F23=["admin","dev"]  # User roles
```

**Multiple Comment Lines**:
```
# This is a multi-line comment
# explaining the purpose of this field
# and its expected values
F12=14532
```

#### Canonical Form

In canonical form:
- All comments are removed
- All whitespace (except newline separators) is removed
- Fields are separated by single newlines
- No leading or trailing whitespace

**Input**:
```
# User profile
F23=["admin","dev"];  F7=1  ;  F12=14532  # User ID
```

**Canonical Output**:
```
F7=1
F12=14532
F23=["admin","dev"]
```

#### Whitespace and Comment Interaction

```
F12=14532  # Comment with spaces before
F12=14532# Comment with no space before
F12=14532#6A93B3F1  # Checksum, then comment
F12=14532#6A93B3F1# Another comment
```

All are valid. The parser:
1. Identifies checksums first (8 hex digits immediately after value)
2. Treats remaining `#` as comment start
3. Ignores all content from comment start to end of line

---

## 3. Binary Format Specification

The binary format provides a space-efficient, machine-optimized encoding of LNMP data for network transmission and storage. This section defines the complete binary format structure, encoding rules, and type system.

The binary format maintains semantic equivalence with the text format while optimizing for size and parsing speed. All binary-encoded data can be converted to canonical text format and vice versa with guaranteed round-trip stability.

### 3.1 Frame Structure

The binary format organizes data into frames, which are self-contained units that can be transmitted or stored independently.

#### Binary Frame Layout

```
┌─────────┬─────────┬─────────────┬──────────────────────┐
│ VERSION │  FLAGS  │ ENTRY_COUNT │      ENTRIES...      │
│ (1 byte)│(1 byte) │  (VarInt)   │     (variable)       │
└─────────┴─────────┴─────────────┴──────────────────────┘
```

#### Frame Field Descriptions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| VERSION | 1 byte | u8 | Protocol version identifier (0x04 for v0.4) |
| FLAGS | 1 byte | u8 | Reserved flags for future use (0x00 in v0.4) |
| ENTRY_COUNT | Variable | VarInt | Number of field entries in this frame |
| ENTRIES | Variable | Entry[] | Sequence of binary-encoded field entries |

#### VERSION Byte

The VERSION byte identifies the binary format version, enabling version negotiation and format evolution.

**v0.4 Version Byte**: `0x04`

**Version Negotiation Rules**:
- Decoders MUST check the VERSION byte before parsing
- Decoders MUST reject frames with unsupported VERSION values
- Decoders SHOULD produce error code 5001 (UnsupportedVersion) for unknown versions
- Future versions will use different VERSION values (0x05, 0x06, etc.)

**Example**:
```
04 ...    # Valid: v0.4 frame
05 ...    # Invalid in v0.4 decoder: unsupported version
```

**Rationale**: Explicit version identification allows the protocol to evolve while maintaining backward compatibility. Decoders can gracefully reject unsupported versions rather than producing corrupt data.

#### FLAGS Byte

The FLAGS byte is reserved for future protocol extensions and optional features.

**v0.4 FLAGS Byte**: `0x00` (all bits zero)

**Reserved for Future Use**:
- Bit 0: Compression enabled
- Bit 1: Schema reference included
- Bit 2: Extended type tags
- Bit 3: Checksum included
- Bits 4-7: Reserved

**Decoder Requirements**:
- Decoders MUST accept FLAGS value 0x00
- Decoders MAY accept other FLAGS values if they support the indicated features
- Decoders SHOULD ignore unknown flag bits rather than rejecting the frame
- Future specifications will define flag bit meanings

**Example**:
```
04 00 ...    # Valid: v0.4 with no flags
04 01 ...    # May be valid in future: compression flag
04 FF ...    # May be valid in future: multiple flags
```

**Rationale**: Reserved flags enable backward-compatible feature additions. Older decoders can ignore unknown flags, while newer decoders can enable optional features.

#### ENTRY_COUNT Field

The ENTRY_COUNT field specifies how many field entries follow in the frame.

**Encoding**: VarInt (LEB128) - see Section 3.2 for encoding details

**Range**: 0 to 2^64-1 (practical limit much lower)

**Purpose**:
- Allows decoders to pre-allocate storage
- Enables frame validation before processing entries
- Supports streaming parsers that need to know entry count

**Examples**:
```
00          # 0 entries (empty frame)
01          # 1 entry
03          # 3 entries
80 01       # 128 entries (VarInt encoding)
E4 E3 00    # 14532 entries (VarInt encoding)
```

**Validation**:
- Decoders MUST verify that exactly ENTRY_COUNT entries are present
- Fewer entries than ENTRY_COUNT MUST produce error code 2002 (UnexpectedEof)
- More entries than ENTRY_COUNT MUST produce error code 2001 (UnexpectedToken)

#### ENTRIES Field

The ENTRIES field contains a sequence of binary-encoded field entries.

**Structure**: Each entry encodes one field from the LNMP record

**Encoding**: See Section 3.4 for complete entry encoding specification

**Ordering**: In canonical form, entries MUST be sorted by FID in ascending order

**Example Frame**:
```
04                          # VERSION = 0x04
00                          # FLAGS = 0x00
03                          # ENTRY_COUNT = 3 (VarInt)

# Entry 1: F7=1
07 00                       # FID = 7 (u16 little-endian)
03                          # TAG = Bool (0x03)
01                          # VALUE = true (0x01)

# Entry 2: F12=14532
0C 00                       # FID = 12 (u16 little-endian)
01                          # TAG = Int (0x01)
E4 E3 00                    # VALUE = 14532 (VarInt)

# Entry 3: F23=["admin","dev"]
17 00                       # FID = 23 (u16 little-endian)
05                          # TAG = StringArray (0x05)
02                          # COUNT = 2 (VarInt)
05 61 64 6D 69 6E          # "admin" (length 5 + UTF-8)
03 64 65 76                # "dev" (length 3 + UTF-8)
```

**Total Frame Size**: 28 bytes (compared to 33 bytes for equivalent text format)


### 3.2 VarInt Encoding

LNMP uses VarInt (Variable-Length Integer) encoding based on LEB128 (Little Endian Base 128) for space-efficient integer representation. Small integers use fewer bytes, making the binary format compact for typical use cases.

#### LEB128 Encoding Algorithm

VarInt encoding represents integers using 7 bits of data per byte, with the 8th bit indicating whether more bytes follow.

**Encoding Steps**:

1. Take the lowest 7 bits of the integer
2. If the remaining integer is non-zero, set bit 7 to 1 (continuation bit)
3. If the remaining integer is zero, set bit 7 to 0 (final byte)
4. Output the byte
5. Shift the integer right by 7 bits
6. Repeat from step 1 until the integer is zero

**Pseudocode**:
```
function encode_varint(value):
    while true:
        byte = value & 0x7F          # Take lowest 7 bits
        value = value >> 7            # Shift right by 7 bits
        if value != 0:
            byte = byte | 0x80        # Set continuation bit
        output(byte)
        if value == 0:
            break
```

#### LEB128 Decoding Algorithm

**Decoding Steps**:

1. Read a byte
2. Extract bits 0-6 as data (mask with 0x7F)
3. Add data to result, shifted by (7 × byte_index)
4. If bit 7 is 1, read next byte and continue
5. If bit 7 is 0, decoding is complete

**Pseudocode**:
```
function decode_varint():
    result = 0
    shift = 0
    while true:
        byte = read_byte()
        result = result | ((byte & 0x7F) << shift)
        if (byte & 0x80) == 0:
            break
        shift = shift + 7
    return result
```

#### VarInt Encoding Examples

| Value (Decimal) | Value (Hex) | Encoded Bytes (Hex) | Binary Representation | Byte Count |
|-----------------|-------------|---------------------|----------------------|------------|
| 0 | 0x00 | `00` | `00000000` | 1 |
| 1 | 0x01 | `01` | `00000001` | 1 |
| 127 | 0x7F | `7F` | `01111111` | 1 |
| 128 | 0x80 | `80 01` | `10000000 00000001` | 2 |
| 255 | 0xFF | `FF 01` | `11111111 00000001` | 2 |
| 256 | 0x100 | `80 02` | `10000000 00000010` | 2 |
| 14532 | 0x38C4 | `C4 71` | `11000100 01110001` | 2 |
| 16383 | 0x3FFF | `FF 7F` | `11111111 01111111` | 2 |
| 16384 | 0x4000 | `80 80 01` | `10000000 10000000 00000001` | 3 |
| 2097151 | 0x1FFFFF | `FF FF 7F` | `11111111 11111111 01111111` | 3 |
| 2097152 | 0x200000 | `80 80 80 01` | `10000000 10000000 10000000 00000001` | 4 |

**Encoding Explanation for 14532**:

```
14532 (decimal) = 0x38C4 (hex) = 0011100011000100 (binary)

Step 1: Take lowest 7 bits: 1000100 (0x44)
        Remaining: 0011100 (non-zero, so set continuation bit)
        Output: 11000100 (0xC4)

Step 2: Take lowest 7 bits: 0011100 (0x1C) 
        Remaining: 0 (zero, so no continuation bit)
        Output: 01110001 (0x71)

Result: C4 71
```

**Decoding Explanation for C4 71**:

```
Byte 1: 0xC4 = 11000100
        Data bits (0-6): 1000100 = 0x44 = 68
        Continuation bit (7): 1 (more bytes follow)
        Result so far: 68

Byte 2: 0x71 = 01110001
        Data bits (0-6): 1110001 = 0x71 = 113
        Continuation bit (7): 0 (final byte)
        Result: 68 + (113 << 7) = 68 + 14464 = 14532
```

#### Zigzag Encoding for Signed Integers

VarInt encoding is most efficient for small positive integers. For signed integers, LNMP uses zigzag encoding to map signed values to unsigned values, ensuring small negative numbers also encode efficiently.

**Zigzag Mapping**:

```
Signed → Unsigned
0      → 0
-1     → 1
1      → 2
-2     → 3
2      → 4
-3     → 5
3      → 6
...
```

**Zigzag Encoding Formula**:

```
zigzag(n) = (n << 1) ^ (n >> 63)    # For 64-bit signed integers
```

Where:
- `<<` is left shift
- `>>` is arithmetic right shift (sign-extending)
- `^` is XOR

**Zigzag Decoding Formula**:

```
unzigzag(n) = (n >> 1) ^ -(n & 1)
```

**Zigzag Examples**:

| Signed Value | Zigzag Encoded | VarInt Bytes | Explanation |
|--------------|----------------|--------------|-------------|
| 0 | 0 | `00` | (0 << 1) ^ (0 >> 63) = 0 |
| -1 | 1 | `01` | (-1 << 1) ^ (-1 >> 63) = -2 ^ -1 = 1 |
| 1 | 2 | `02` | (1 << 1) ^ (1 >> 63) = 2 ^ 0 = 2 |
| -2 | 3 | `03` | (-2 << 1) ^ (-2 >> 63) = -4 ^ -1 = 3 |
| 2 | 4 | `04` | (2 << 1) ^ (2 >> 63) = 4 ^ 0 = 4 |
| -64 | 127 | `7F` | (-64 << 1) ^ (-64 >> 63) = -128 ^ -1 = 127 |
| -65 | 129 | `81 01` | (-65 << 1) ^ (-65 >> 63) = -130 ^ -1 = 129 |

**Rationale**: Without zigzag encoding, -1 would encode as 0xFFFFFFFFFFFFFFFF (10 bytes in VarInt). With zigzag, -1 encodes as 1 (1 byte), making small negative numbers as efficient as small positive numbers.

#### VarInt Properties

**Size Ranges**:

| Value Range | Byte Count | Percentage of i64 Range |
|-------------|------------|------------------------|
| 0 to 127 | 1 byte | 0.0000000014% |
| 128 to 16,383 | 2 bytes | 0.00000018% |
| 16,384 to 2,097,151 | 3 bytes | 0.000023% |
| 2,097,152 to 268,435,455 | 4 bytes | 0.0029% |
| 268,435,456 to 34,359,738,367 | 5 bytes | 0.37% |
| 34,359,738,368 to 4,398,046,511,103 | 6 bytes | 48% |
| 4,398,046,511,104 to 562,949,953,421,311 | 7 bytes | 6,144% |
| 562,949,953,421,312 to 72,057,594,037,927,935 | 8 bytes | 786,432% |
| 72,057,594,037,927,936 to 9,223,372,036,854,775,807 | 9 bytes | 100,663,296% |
| Full i64 range (with sign bit) | 10 bytes | Maximum |

**Maximum Bytes**: 10 bytes for full i64 range (-2^63 to 2^63-1)

**Efficiency**: 
- Values 0-127: 1 byte (vs 8 bytes for fixed i64)
- Values 128-16383: 2 bytes (vs 8 bytes for fixed i64)
- Typical LNMP field IDs (0-65535): 1-3 bytes
- Typical integer values in applications: 1-4 bytes

**Canonical Form**: VarInt encoding is inherently canonical - each value has exactly one valid encoding. Decoders MUST reject non-minimal encodings (e.g., encoding 0 as `80 00` instead of `00`).

**Error Handling**:
- Decoders MUST reject VarInt values that exceed 10 bytes
- Decoders MUST reject non-minimal encodings
- Decoders MUST reject truncated VarInt values (continuation bit set on last byte)
- Errors SHOULD produce error code 5002 (InvalidVarInt)

**Reference**: LEB128 encoding is defined in the DWARF Debugging Information Format specification (Section 7.6).


### 3.3 Type Tags

Type tags are single-byte identifiers that specify the encoding format of field values in the binary format. Each type tag corresponds to one of the LNMP value types.

#### Type Tag Table

| Type | Tag Value | Hex | Binary | Description | Supported in v0.4 |
|------|-----------|-----|--------|-------------|-------------------|
| Integer | 1 | `0x01` | `00000001` | Signed 64-bit integer (VarInt encoded) | ✓ |
| Float | 2 | `0x02` | `00000010` | IEEE 754 double-precision (8 bytes) | ✓ |
| Boolean | 3 | `0x03` | `00000011` | Single byte (0x00 or 0x01) | ✓ |
| String | 4 | `0x04` | `00000100` | Length-prefixed UTF-8 string | ✓ |
| String Array | 5 | `0x05` | `00000101` | Count-prefixed array of strings | ✓ |
| Record | 6 | `0x06` | `00000110` | Nested record (reserved for v0.5) | ✗ |
| Record Array | 7 | `0x07` | `00000111` | Array of nested records (reserved for v0.5) | ✗ |
| Reserved | 8-255 | `0x08-0xFF` | Various | Reserved for future types | ✗ |

#### Type Tag Descriptions

**Integer (0x01)**

Represents a signed 64-bit integer value encoded using VarInt with zigzag encoding.

- **Range**: -2^63 to 2^63-1
- **Encoding**: Zigzag + VarInt (1-10 bytes)
- **Use cases**: Counters, IDs, timestamps, quantities

**Float (0x02)**

Represents an IEEE 754 double-precision floating-point value.

- **Size**: Always 8 bytes
- **Encoding**: Little-endian IEEE 754 binary64
- **Special values**: NaN, Infinity, -Infinity (all supported)
- **Use cases**: Measurements, percentages, scientific data

**Boolean (0x03)**

Represents a boolean (true/false) value.

- **Size**: Always 1 byte
- **Values**: 0x00 (false), 0x01 (true)
- **Invalid**: Any value other than 0x00 or 0x01 MUST produce error code 3003 (InvalidValue)
- **Use cases**: Flags, status indicators, binary choices

**String (0x04)**

Represents a UTF-8 encoded text string.

- **Encoding**: Length (VarInt) + UTF-8 bytes
- **Length**: Number of bytes (not characters)
- **Validation**: MUST be valid UTF-8, no BOM
- **Empty strings**: Length 0 is valid
- **Use cases**: Names, descriptions, identifiers, text content

**String Array (0x05)**

Represents an ordered array of UTF-8 strings.

- **Encoding**: Count (VarInt) + repeated (Length + UTF-8 bytes)
- **Count**: Number of elements in array
- **Empty arrays**: Count 0 is valid
- **Element order**: Preserved from input
- **Use cases**: Tags, roles, lists of identifiers

#### Reserved Type Tags

**Record (0x06) - Reserved for v0.5**

Will represent nested record structures in binary format.

- **Status**: Reserved, not supported in v0.4
- **Planned encoding**: Entry count + repeated entries
- **v0.4 behavior**: Decoders MUST reject tag 0x06 with error code 5003 (UnsupportedTypeTag)
- **Text format**: Nested records ARE supported in text format (v0.3+)

**Record Array (0x07) - Reserved for v0.5**

Will represent arrays of nested records in binary format.

- **Status**: Reserved, not supported in v0.4
- **Planned encoding**: Array count + repeated (entry count + entries)
- **v0.4 behavior**: Decoders MUST reject tag 0x07 with error code 5003 (UnsupportedTypeTag)
- **Text format**: Nested arrays ARE supported in text format (v0.3+)

**Future Types (0x08-0xFF)**

All type tags from 0x08 to 0xFF are reserved for future protocol extensions.

- **Possible future types**: Binary blobs, timestamps, UUIDs, decimal numbers, maps, sets
- **v0.4 behavior**: Decoders MUST reject unknown type tags with error code 5003 (UnsupportedTypeTag)
- **Forward compatibility**: Future versions may define additional type tags

#### Type Tag Usage

**In Binary Entries**:

Each binary entry includes a type tag that specifies how to decode the value bytes:

```
┌──────────┬──────────┬──────────────────┐
│   FID    │  THTAG   │      VALUE       │
│ (2 bytes)│ (1 byte) │   (variable)     │
└──────────┴──────────┴──────────────────┘
```

The THTAG (Type Hint Tag) byte contains the type tag value.

**Type Tag Selection**:

When encoding text format to binary format, the encoder determines the type tag based on the value syntax:

| Text Value | Type Tag | Rationale |
|------------|----------|-----------|
| `42` | 0x01 (Integer) | Matches integer pattern |
| `3.14` | 0x02 (Float) | Contains decimal point |
| `0` or `1` | 0x03 (Boolean) | Single digit 0 or 1 |
| `"hello"` | 0x04 (String) | Quoted string |
| `simple` | 0x04 (String) | Unquoted string |
| `["a","b"]` | 0x05 (String Array) | Array syntax |
| `{F1=x}` | N/A | Not supported in v0.4 binary |
| `[{F1=x}]` | N/A | Not supported in v0.4 binary |

**Type Hint Validation**:

If the text format includes a type hint (e.g., `F12:i=14532`), the encoder MUST verify that the type hint matches the selected type tag. Mismatches MUST produce error code 3001 (TypeHintMismatch).

**Decoder Requirements**:

- Decoders MUST check the type tag before decoding the value
- Decoders MUST reject unknown type tags (error code 5003)
- Decoders MUST reject reserved type tags 0x06 and 0x07 in v0.4 (error code 5003)
- Decoders MUST validate that value bytes match the type tag specification

**Example Type Tag Usage**:

```
# Integer field: F12=14532
0C 00           # FID = 12
01              # Type tag = Integer
C4 71           # Value = 14532 (VarInt)

# String field: F1="alice"
01 00           # FID = 1
04              # Type tag = String
05              # Length = 5
61 6C 69 63 65  # UTF-8 bytes for "alice"
```


### 3.4 Value Encoding

This section defines the complete encoding format for each value type in binary entries.

#### Binary Entry Structure

Each field in the binary format is encoded as an entry with the following structure:

```
┌──────────┬──────────┬──────────────────┐
│   FID    │  THTAG   │      VALUE       │
│ (2 bytes)│ (1 byte) │   (variable)     │
└──────────┴──────────┴──────────────────┘
```

**Components**:

1. **FID (Field Identifier)**: 2 bytes, unsigned 16-bit integer, little-endian
2. **THTAG (Type Hint Tag)**: 1 byte, type tag value (0x01-0x05 in v0.4)
3. **VALUE**: Variable length, encoding depends on type tag

#### Integer Encoding (Tag 0x01)

**Format**: Zigzag-encoded VarInt

**Encoding Steps**:
1. Apply zigzag encoding to signed integer: `zigzag(n) = (n << 1) ^ (n >> 63)`
2. Encode result as VarInt (LEB128)

**Size**: 1-10 bytes (depending on magnitude)

**Examples**:

| Text Value | Signed i64 | Zigzag | VarInt Bytes | Complete Entry (Hex) |
|------------|------------|--------|--------------|---------------------|
| `F12=0` | 0 | 0 | `00` | `0C 00 01 00` |
| `F12=1` | 1 | 2 | `02` | `0C 00 01 02` |
| `F12=-1` | -1 | 1 | `01` | `0C 00 01 01` |
| `F12=127` | 127 | 254 | `FE 01` | `0C 00 01 FE 01` |
| `F12=14532` | 14532 | 29064 | `C8 E3 01` | `0C 00 01 C8 E3 01` |
| `F12=-14532` | -14532 | 29063 | `C7 E3 01` | `0C 00 01 C7 E3 01` |

**Byte Layout Diagram for F12=14532**:

```
0C 00           ← FID = 12 (little-endian u16)
01              ← Type tag = Integer
C8 E3 01        ← Value = 14532 (zigzag + VarInt)
```

#### Float Encoding (Tag 0x02)

**Format**: IEEE 754 double-precision (binary64), little-endian

**Size**: Always 8 bytes

**Encoding**: Direct binary representation of IEEE 754 double

**Special Values**:
- **NaN**: `00 00 00 00 00 00 F8 7F` (canonical NaN)
- **Infinity**: `00 00 00 00 00 00 F0 7F`
- **-Infinity**: `00 00 00 00 00 00 F0 FF`
- **Zero**: `00 00 00 00 00 00 00 00`
- **-Zero**: `00 00 00 00 00 00 00 80`

**Examples**:

| Text Value | Float Value | IEEE 754 Bytes (Hex) | Complete Entry (Hex) |
|------------|-------------|----------------------|---------------------|
| `F12=0.0` | 0.0 | `00 00 00 00 00 00 00 00` | `0C 00 02 00 00 00 00 00 00 00 00` |
| `F12=1.0` | 1.0 | `00 00 00 00 00 00 F0 3F` | `0C 00 02 00 00 00 00 00 00 F0 3F` |
| `F12=3.14` | 3.14 | `1F 85 EB 51 B8 1E 09 40` | `0C 00 02 1F 85 EB 51 B8 1E 09 40` |
| `F12=-2.5` | -2.5 | `00 00 00 00 00 00 04 C0` | `0C 00 02 00 00 00 00 00 00 04 C0` |

**Byte Layout Diagram for F12=3.14**:

```
0C 00                       ← FID = 12 (little-endian u16)
02                          ← Type tag = Float
1F 85 EB 51 B8 1E 09 40    ← Value = 3.14 (IEEE 754 little-endian)
```

**Canonical Form**: IEEE 754 normalized form (no denormalized numbers in canonical output)

#### Boolean Encoding (Tag 0x03)

**Format**: Single byte

**Size**: Always 1 byte

**Values**:
- **False**: `0x00`
- **True**: `0x01`

**Validation**: Any value other than 0x00 or 0x01 MUST produce error code 3003 (InvalidValue)

**Examples**:

| Text Value | Boolean | Byte | Complete Entry (Hex) |
|------------|---------|------|---------------------|
| `F7=0` | false | `00` | `07 00 03 00` |
| `F7=1` | true | `01` | `07 00 03 01` |

**Byte Layout Diagram for F7=1**:

```
07 00           ← FID = 7 (little-endian u16)
03              ← Type tag = Boolean
01              ← Value = true
```

#### String Encoding (Tag 0x04)

**Format**: Length (VarInt) + UTF-8 bytes

**Encoding Steps**:
1. Encode string as UTF-8 bytes
2. Count byte length (not character count)
3. Encode length as VarInt
4. Output length followed by UTF-8 bytes

**Size**: Variable (1+ bytes for length, N bytes for content)

**Validation**:
- MUST be valid UTF-8
- MUST NOT include byte order mark (BOM)
- Empty strings (length 0) are valid

**Examples**:

| Text Value | String | Length | UTF-8 Bytes (Hex) | Complete Entry (Hex) |
|------------|--------|--------|-------------------|---------------------|
| `F1=""` | (empty) | 0 | (none) | `01 00 04 00` |
| `F1="a"` | "a" | 1 | `61` | `01 00 04 01 61` |
| `F1="alice"` | "alice" | 5 | `61 6C 69 63 65` | `01 00 04 05 61 6C 69 63 65` |
| `F1="hello world"` | "hello world" | 11 | `68 65 6C 6C 6F 20 77 6F 72 6C 64` | `01 00 04 0B 68 65 6C 6C 6F 20 77 6F 72 6C 64` |
| `F1="🎯"` | "🎯" | 4 | `F0 9F 8E AF` | `01 00 04 04 F0 9F 8E AF` |

**Byte Layout Diagram for F1="alice"**:

```
01 00               ← FID = 1 (little-endian u16)
04                  ← Type tag = String
05                  ← Length = 5 bytes (VarInt)
61 6C 69 63 65      ← UTF-8 bytes for "alice"
```

**Unicode Handling**:

Unicode characters are encoded as UTF-8 multi-byte sequences. The length field represents byte count, not character count.

Example: "🎯" (U+1F3AF)
- Character count: 1
- Byte count: 4 (UTF-8: F0 9F 8E AF)
- Length field: 4

**Escape Sequences**:

In text format, escape sequences like `\n` are represented as actual characters in binary format:

| Text Format | Binary Encoding |
|-------------|-----------------|
| `F1="line1\nline2"` | Length=12, bytes: `6C 69 6E 65 31 0A 6C 69 6E 65 32` |
| `F1="path\\file"` | Length=10, bytes: `70 61 74 68 5C 66 69 6C 65` |
| `F1="say \"hi\""` | Length=8, bytes: `73 61 79 20 22 68 69 22` |

#### String Array Encoding (Tag 0x05)

**Format**: Count (VarInt) + repeated (Length (VarInt) + UTF-8 bytes)

**Encoding Steps**:
1. Count number of elements in array
2. Encode count as VarInt
3. For each element:
   - Encode string length as VarInt
   - Output UTF-8 bytes

**Size**: Variable (1+ bytes for count, then length+bytes for each element)

**Empty Arrays**: Count 0 is valid (no element data follows)

**Examples**:

**Empty Array: F23=[]**

```
17 00           ← FID = 23 (little-endian u16)
05              ← Type tag = String Array
00              ← Count = 0 (no elements)
```

**Single Element: F23=["admin"]**

```
17 00                   ← FID = 23
05                      ← Type tag = String Array
01                      ← Count = 1
05 61 64 6D 69 6E      ← Element 0: length=5, "admin"
```

**Two Elements: F23=["admin","dev"]**

```
17 00                   ← FID = 23
05                      ← Type tag = String Array
02                      ← Count = 2
05 61 64 6D 69 6E      ← Element 0: length=5, "admin"
03 64 65 76            ← Element 1: length=3, "dev"
```

**Multiple Elements: F23=["a","b","c","d"]**

```
17 00           ← FID = 23
05              ← Type tag = String Array
04              ← Count = 4
01 61           ← Element 0: length=1, "a"
01 62           ← Element 1: length=1, "b"
01 63           ← Element 2: length=1, "c"
01 64           ← Element 3: length=1, "d"
```

**Byte Layout Diagram for F23=["admin","dev"]**:

```
17 00                   ← FID = 23 (little-endian u16)
05                      ← Type tag = String Array
02                      ← Count = 2 elements (VarInt)
05                      ← Element 0 length = 5 (VarInt)
61 64 6D 69 6E         ← Element 0 UTF-8: "admin"
03                      ← Element 1 length = 3 (VarInt)
64 65 76               ← Element 1 UTF-8: "dev"
```

**Element Ordering**: Elements are encoded in the order they appear in the text format. Order is preserved during round-trip conversion.

**Element Validation**: Each element MUST be a valid UTF-8 string. Invalid UTF-8 in any element MUST produce an error.

#### Nested Structures (Not Supported in v0.4 Binary)

**Nested Records** (type tag 0x06) and **Nested Arrays** (type tag 0x07) are reserved for future versions and are NOT supported in v0.4 binary format.

**v0.4 Behavior**:
- Text format: Nested structures ARE fully supported
- Binary format: Nested structures are NOT supported
- Encoding text with nested structures to binary MUST produce error code 5004 (NestedStructuresNotSupported)
- Decoding binary with type tags 0x06 or 0x07 MUST produce error code 5003 (UnsupportedTypeTag)

**Workaround for v0.4**:

Applications needing nested structures should:
1. Use text format for records with nested structures
2. Flatten nested structures into top-level fields with semantic FIDs
3. Wait for v0.5 binary format support

**Example Flattening**:

Instead of:
```
F50={F1=alice;F2=admin}
```

Use:
```
F50=alice
F51=admin
```

**v0.5 Planned Support**:

Future versions will support nested structures in binary format with the following encoding:

**Nested Record (0x06)**: Entry count (VarInt) + repeated entries
**Nested Array (0x07)**: Array count (VarInt) + repeated (entry count + entries)

#### Value Encoding Summary Table

| Type | Tag | Size | Encoding Format |
|------|-----|------|-----------------|
| Integer | 0x01 | 1-10 bytes | Zigzag + VarInt |
| Float | 0x02 | 8 bytes | IEEE 754 little-endian |
| Boolean | 0x03 | 1 byte | 0x00 or 0x01 |
| String | 0x04 | Variable | Length (VarInt) + UTF-8 |
| String Array | 0x05 | Variable | Count (VarInt) + repeated strings |
| Record | 0x06 | N/A | Reserved for v0.5 |
| Record Array | 0x07 | N/A | Reserved for v0.5 |


### 3.5 Byte Order

LNMP binary format uses little-endian byte order for all multi-byte values. This section defines the byte ordering rules and provides examples.

#### Little-Endian Byte Order

**Definition**: In little-endian byte order, the least significant byte (LSB) is stored at the lowest memory address, and the most significant byte (MSB) is stored at the highest memory address.

**Rationale**: Little-endian is the native byte order on most modern processor architectures (x86, x86-64, ARM in most configurations), providing better performance by avoiding byte-swapping operations during encoding and decoding.

#### Multi-Byte Values in LNMP

The following value types use multi-byte representations:

1. **Field IDs (FID)**: 2 bytes, unsigned 16-bit integer
2. **Float Values**: 8 bytes, IEEE 754 double-precision
3. **VarInt Values**: Variable bytes (inherently little-endian due to LEB128)

#### Field ID Byte Order

Field IDs are encoded as 2-byte unsigned integers in little-endian order.

**Examples**:

| FID (Decimal) | FID (Hex) | Little-Endian Bytes | Big-Endian (Not Used) |
|---------------|-----------|---------------------|----------------------|
| 0 | 0x0000 | `00 00` | `00 00` |
| 1 | 0x0001 | `01 00` | `00 01` |
| 7 | 0x0007 | `07 00` | `00 07` |
| 12 | 0x000C | `0C 00` | `00 0C` |
| 23 | 0x0017 | `17 00` | `00 17` |
| 255 | 0x00FF | `FF 00` | `00 FF` |
| 256 | 0x0100 | `00 01` | `01 00` |
| 4660 | 0x1234 | `34 12` | `12 34` |
| 65535 | 0xFFFF | `FF FF` | `FF FF` |

**Encoding Process for FID 12 (0x000C)**:

```
Hex value: 0x000C
Binary:    00000000 00001100

Byte 0 (LSB): 00001100 = 0x0C
Byte 1 (MSB): 00000000 = 0x00

Little-endian output: 0C 00
```

**Decoding Process for bytes 0C 00**:

```
Byte 0: 0x0C = 00001100
Byte 1: 0x00 = 00000000

Combine: (0x00 << 8) | 0x0C = 0x000C = 12
```

#### Float Byte Order

IEEE 754 double-precision floats are encoded as 8 bytes in little-endian order.

**Examples**:

| Float Value | IEEE 754 (Hex) | Little-Endian Bytes | Big-Endian (Not Used) |
|-------------|----------------|---------------------|----------------------|
| 0.0 | 0x0000000000000000 | `00 00 00 00 00 00 00 00` | `00 00 00 00 00 00 00 00` |
| 1.0 | 0x3FF0000000000000 | `00 00 00 00 00 00 F0 3F` | `3F F0 00 00 00 00 00 00` |
| 3.14 | 0x40091EB851EB851F | `1F 85 EB 51 B8 1E 09 40` | `40 09 1E B8 51 EB 85 1F` |
| -2.5 | 0xC004000000000000 | `00 00 00 00 00 00 04 C0` | `C0 04 00 00 00 00 00 00` |

**Encoding Process for 3.14**:

```
IEEE 754 representation: 0x40091EB851EB851F

Bytes (MSB to LSB):
Byte 7: 0x40
Byte 6: 0x09
Byte 5: 0x1E
Byte 4: 0xB8
Byte 3: 0x51
Byte 2: 0xEB
Byte 1: 0x85
Byte 0: 0x1F

Little-endian output: 1F 85 EB 51 B8 1E 09 40
```

**Decoding Process for bytes 1F 85 EB 51 B8 1E 09 40**:

```
Read bytes in order:
Byte 0: 0x1F
Byte 1: 0x85
Byte 2: 0xEB
Byte 3: 0x51
Byte 4: 0xB8
Byte 5: 0x1E
Byte 6: 0x09
Byte 7: 0x40

Reconstruct IEEE 754: 0x40091EB851EB851F = 3.14
```

#### VarInt Byte Order

VarInt encoding (LEB128) is inherently little-endian. The first byte contains the least significant 7 bits, the second byte contains the next 7 bits, and so on.

**Example for value 14532**:

```
Value: 14532 (decimal) = 0x38C4 (hex)

VarInt encoding: C4 71

Byte 0: 0xC4 = 11000100
        Data bits (0-6): 1000100 = 0x44 = 68 (LSB)
        Continuation bit (7): 1

Byte 1: 0x71 = 01110001
        Data bits (0-6): 1110001 = 0x71 = 113
        Continuation bit (7): 0

Reconstruction: 68 + (113 << 7) = 68 + 14464 = 14532
```

The least significant bits appear in the first byte, making VarInt naturally little-endian.

#### Implementation Considerations

**Native Byte Order**:

On little-endian architectures (x86, x86-64, most ARM), multi-byte values can be read and written directly from/to memory without byte swapping:

```c
// On little-endian architecture
uint16_t fid = 12;
write_bytes(&fid, 2);  // Writes 0C 00 directly

double value = 3.14;
write_bytes(&value, 8);  // Writes 1F 85 EB 51 B8 1E 09 40 directly
```

**Big-Endian Architectures**:

On big-endian architectures (some MIPS, PowerPC, older ARM), implementations MUST perform byte swapping:

```c
// On big-endian architecture
uint16_t fid = 12;
uint16_t fid_le = swap_bytes_16(fid);  // Convert to little-endian
write_bytes(&fid_le, 2);  // Writes 0C 00

double value = 3.14;
double value_le = swap_bytes_64(value);  // Convert to little-endian
write_bytes(&value_le, 8);  // Writes 1F 85 EB 51 B8 1E 09 40
```

**Portability**:

Implementations SHOULD use platform-independent byte order conversion functions:

- C/C++: `htole16()`, `htole64()`, `le16toh()`, `le64toh()`
- Rust: `u16::to_le_bytes()`, `u16::from_le_bytes()`, `f64::to_le_bytes()`, `f64::from_le_bytes()`
- Python: `struct.pack('<H', value)`, `struct.pack('<d', value)`
- JavaScript: `DataView` with `setUint16(offset, value, true)` (true = little-endian)

#### Byte Order Summary

| Value Type | Size | Byte Order | Notes |
|------------|------|------------|-------|
| Field ID | 2 bytes | Little-endian | u16, LSB first |
| Float | 8 bytes | Little-endian | IEEE 754, LSB first |
| VarInt | Variable | Little-endian | LEB128, inherently LE |
| Type Tag | 1 byte | N/A | Single byte, no ordering |
| Boolean | 1 byte | N/A | Single byte, no ordering |
| String Length | Variable | Little-endian | VarInt encoding |
| Array Count | Variable | Little-endian | VarInt encoding |

**Validation**: Decoders MUST interpret all multi-byte values as little-endian. Incorrect byte order interpretation will produce corrupt data.


### 3.6 Complete Binary Example

This section provides a complete end-to-end example of LNMP binary encoding, showing the text format, binary encoding process, and size comparison.

#### Example Record

**Text Format (Canonical)**:

```
F7=1
F12=14532
F23=["admin","dev"]
```

**Semantic Meaning**:
- F7: Active status (boolean, true)
- F12: User ID (integer, 14532)
- F23: User roles (string array, ["admin", "dev"])

#### Binary Encoding Process

**Step 1: Frame Header**

```
VERSION:      04              # Protocol version 0.4
FLAGS:        00              # No flags set
ENTRY_COUNT:  03              # 3 field entries
```

**Step 2: Entry 1 (F7=1)**

```
FID:    07 00                 # Field ID 7 (little-endian u16)
TAG:    03                    # Type tag: Boolean
VALUE:  01                    # Boolean value: true (0x01)
```

**Step 3: Entry 2 (F12=14532)**

```
FID:    0C 00                 # Field ID 12 (little-endian u16)
TAG:    01                    # Type tag: Integer
VALUE:  C8 E3 01              # Integer value: 14532 (zigzag + VarInt)
```

Encoding calculation for 14532:
- Zigzag: (14532 << 1) ^ (14532 >> 63) = 29064
- VarInt of 29064: C8 E3 01

**Step 4: Entry 3 (F23=["admin","dev"])**

```
FID:    17 00                 # Field ID 23 (little-endian u16)
TAG:    05                    # Type tag: String Array
COUNT:  02                    # Array count: 2 elements
ELEM0:  05 61 64 6D 69 6E    # "admin" (length 5 + UTF-8)
ELEM1:  03 64 65 76           # "dev" (length 3 + UTF-8)
```

Element encoding:
- "admin": length=5, UTF-8 bytes: 61 64 6D 69 6E
- "dev": length=3, UTF-8 bytes: 64 65 76

#### Complete Binary Frame

**Hexadecimal Representation with Annotations**:

```
04                          # VERSION = 0x04
00                          # FLAGS = 0x00
03                          # ENTRY_COUNT = 3

# Entry 1: F7=1 (Boolean)
07 00                       # FID = 7
03                          # TAG = Boolean
01                          # VALUE = true

# Entry 2: F12=14532 (Integer)
0C 00                       # FID = 12
01                          # TAG = Integer
C8 E3 01                    # VALUE = 14532 (VarInt)

# Entry 3: F23=["admin","dev"] (String Array)
17 00                       # FID = 23
05                          # TAG = String Array
02                          # COUNT = 2
05 61 64 6D 69 6E          # "admin" (length 5 + bytes)
03 64 65 76                # "dev" (length 3 + bytes)
```

**Continuous Hexadecimal (No Spaces)**:

```
0400030700030100010C00C8E3011700050205616464696E03646576
```

**Byte Array (28 bytes total)**:

```
[0x04, 0x00, 0x03, 0x07, 0x00, 0x03, 0x01, 0x0C, 0x00, 0x01, 0xC8, 0xE3, 0x01, 0x17, 0x00, 0x05, 0x02, 0x05, 0x61, 0x64, 0x6D, 0x69, 0x6E, 0x03, 0x64, 0x65, 0x76]
```

#### Byte-by-Byte Breakdown

| Offset | Byte | Hex | Binary | Description |
|--------|------|-----|--------|-------------|
| 0 | 4 | 0x04 | 00000100 | VERSION: v0.4 |
| 1 | 0 | 0x00 | 00000000 | FLAGS: none |
| 2 | 3 | 0x03 | 00000011 | ENTRY_COUNT: 3 |
| 3 | 7 | 0x07 | 00000111 | Entry 1 FID LSB: 7 |
| 4 | 0 | 0x00 | 00000000 | Entry 1 FID MSB: 0 |
| 5 | 3 | 0x03 | 00000011 | Entry 1 TAG: Boolean |
| 6 | 1 | 0x01 | 00000001 | Entry 1 VALUE: true |
| 7 | 12 | 0x0C | 00001100 | Entry 2 FID LSB: 12 |
| 8 | 0 | 0x00 | 00000000 | Entry 2 FID MSB: 0 |
| 9 | 1 | 0x01 | 00000001 | Entry 2 TAG: Integer |
| 10 | 200 | 0xC8 | 11001000 | Entry 2 VALUE byte 0 (VarInt) |
| 11 | 227 | 0xE3 | 11100011 | Entry 2 VALUE byte 1 (VarInt) |
| 12 | 1 | 0x01 | 00000001 | Entry 2 VALUE byte 2 (VarInt) |
| 13 | 23 | 0x17 | 00010111 | Entry 3 FID LSB: 23 |
| 14 | 0 | 0x00 | 00000000 | Entry 3 FID MSB: 0 |
| 15 | 5 | 0x05 | 00000101 | Entry 3 TAG: String Array |
| 16 | 2 | 0x02 | 00000010 | Entry 3 COUNT: 2 |
| 17 | 5 | 0x05 | 00000101 | Element 0 length: 5 |
| 18 | 97 | 0x61 | 01100001 | Element 0 byte 0: 'a' |
| 19 | 100 | 0x64 | 01100100 | Element 0 byte 1: 'd' |
| 20 | 109 | 0x6D | 01101101 | Element 0 byte 2: 'm' |
| 21 | 105 | 0x69 | 01101001 | Element 0 byte 3: 'i' |
| 22 | 110 | 0x6E | 01101110 | Element 0 byte 4: 'n' |
| 23 | 3 | 0x03 | 00000011 | Element 1 length: 3 |
| 24 | 100 | 0x64 | 01100100 | Element 1 byte 0: 'd' |
| 25 | 101 | 0x65 | 01100101 | Element 1 byte 1: 'e' |
| 26 | 118 | 0x76 | 01110110 | Element 1 byte 2: 'v' |

#### Size Comparison

**Text Format (Canonical)**:

```
F7=1
F12=14532
F23=["admin","dev"]
```

- Character count: 33 characters (including newlines)
- Byte count (UTF-8): 33 bytes
- Token count (estimated): ~12 tokens for typical LLM tokenizer

**Binary Format**:

```
04 00 03 07 00 03 01 0C 00 01 C8 E3 01 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76
```

- Byte count: 28 bytes
- Size reduction: 5 bytes (15% smaller)
- Token count: N/A (binary not tokenized)

**Size Breakdown by Component**:

| Component | Text Bytes | Binary Bytes | Savings |
|-----------|------------|--------------|---------|
| Frame header | 0 | 3 | -3 |
| F7=1 | 4 | 4 | 0 |
| Separator | 1 | 0 | +1 |
| F12=14532 | 9 | 6 | +3 |
| Separator | 1 | 0 | +1 |
| F23=["admin","dev"] | 18 | 15 | +3 |
| **Total** | **33** | **28** | **+5 (15%)** |

**Observations**:

1. **Frame overhead**: Binary format adds 3 bytes for frame header (VERSION, FLAGS, ENTRY_COUNT)
2. **Field encoding**: Binary format saves bytes on field syntax (no "F", "=", separators)
3. **Integer encoding**: VarInt is more efficient for small integers (14532 → 3 bytes vs 5 characters)
4. **String arrays**: Binary format saves bytes on array syntax (no brackets, quotes, commas)
5. **Overall**: Binary format is 15% smaller for this example

**Typical Size Reductions**:

- Small records (3-5 fields): 10-20% smaller
- Medium records (10-20 fields): 30-40% smaller
- Large records (50+ fields): 40-50% smaller
- Records with many integers: 50-60% smaller
- Records with long strings: 5-15% smaller

#### Round-Trip Verification

**Binary → Text Conversion**:

Input binary (hex):
```
04 00 03 07 00 03 01 0C 00 01 C8 E3 01 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76
```

Decoded text (canonical):
```
F7=1
F12=14532
F23=["admin","dev"]
```

**Text → Binary Conversion**:

Input text (any format):
```
F23=["admin","dev"];F7=1;F12=14532
```

Encoded binary (hex):
```
04 00 03 07 00 03 01 0C 00 01 C8 E3 01 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76
```

Note: Fields are reordered by FID during canonicalization (F7, F12, F23).

**Round-Trip Guarantee**:

```
Text (any format) → Binary → Text (canonical) → Binary
```

The final binary MUST be byte-for-byte identical to the first binary encoding.

#### Implementation Example (Pseudocode)

**Encoding**:

```python
def encode_binary(text):
    # Parse text format
    fields = parse_text(text)
    
    # Sort by FID (canonicalization)
    fields.sort(key=lambda f: f.fid)
    
    # Write frame header
    output = bytearray()
    output.append(0x04)  # VERSION
    output.append(0x00)  # FLAGS
    output.extend(encode_varint(len(fields)))  # ENTRY_COUNT
    
    # Write each entry
    for field in fields:
        output.extend(encode_u16_le(field.fid))  # FID
        output.append(field.type_tag)  # TAG
        output.extend(encode_value(field.value, field.type_tag))  # VALUE
    
    return bytes(output)
```

**Decoding**:

```python
def decode_binary(data):
    offset = 0
    
    # Read frame header
    version = data[offset]; offset += 1
    if version != 0x04:
        raise UnsupportedVersion(version)
    
    flags = data[offset]; offset += 1
    entry_count, bytes_read = decode_varint(data[offset:])
    offset += bytes_read
    
    # Read entries
    fields = []
    for _ in range(entry_count):
        fid = decode_u16_le(data[offset:offset+2]); offset += 2
        tag = data[offset]; offset += 1
        value, bytes_read = decode_value(data[offset:], tag)
        offset += bytes_read
        fields.append(Field(fid, tag, value))
    
    return fields
```

This completes the Binary Format Specification section.

---

## 4. Canonicalization

Canonicalization is the process of converting an LNMP record into its standardized, deterministic representation. Every LNMP record has exactly one canonical form, regardless of how it was originally encoded. This property enables reliable checksums, semantic caching, drift detection, and consistent comparison of semantically equivalent records.

Canonicalization applies to both text and binary formats, with format-specific rules for each. The canonical form is the foundation for LNMP's semantic integrity features and ensures interoperability across implementations.

### 4.1 Text Canonical Form

The text canonical form defines the standardized representation of LNMP records in text format. All implementations MUST produce canonical output when encoding, and parsers SHOULD normalize input to canonical form during processing.

#### Canonicalization Rules

The text canonical form is defined by eight mandatory rules:

**Rule 1: Field Ordering**

Fields MUST be sorted by FID (Field Identifier) in ascending numerical order.

Non-canonical:
```
F23=["admin","dev"]
F7=1
F12=14532
```

Canonical:
```
F7=1
F12=14532
F23=["admin","dev"]
```

**Rationale**: Deterministic field ordering ensures that semantically identical records always produce the same text representation, regardless of the order in which fields were added or transmitted.

**Rule 2: Field Separators**

Fields MUST be separated by newline characters (`\n`). Semicolon separators are not used in canonical form.

Non-canonical:
```
F7=1;F12=14532;F23=["admin","dev"]
```

Canonical:
```
F7=1
F12=14532
F23=["admin","dev"]
```

**Rationale**: Newline separators improve readability, work better with version control systems (line-based diffs), and provide consistent formatting.

**Rule 3: Whitespace**

No whitespace is permitted around operators (`=`, `:`) or within field syntax, except:
- Spaces within quoted strings are preserved
- Spaces within string arrays between elements are not permitted

Non-canonical:
```
F7 = 1
F12 : i = 14532
F23 = [ "admin" , "dev" ]
```

Canonical:
```
F7=1
F12:i=14532
F23=["admin","dev"]
```

**Rationale**: Eliminating optional whitespace reduces ambiguity and ensures byte-for-byte identical output across implementations.

**Rule 4: String Quoting**

Strings MUST use the minimal quoting necessary:
- Unquoted form for strings matching `[A-Za-z0-9_.-]+`
- Quoted form for all other strings (containing spaces, special characters, or starting with digits)

Non-canonical:
```
F1="simple"
F2=hello world
F3=123
```

Canonical:
```
F1=simple
F2="hello world"
F3="123"
```

**Rationale**: Unquoted strings reduce token consumption for simple identifiers, while quoted strings handle complex content. The rule is deterministic: implementations can always determine which form to use.

**Rule 5: Number Formatting**

**Integers**:
- No leading zeros (except for the value `0` itself)
- No positive sign (`+`)
- Negative sign (`-`) for negative values

Non-canonical:
```
F1=007
F2=+42
F3=-0
```

Canonical:
```
F1=7
F2=42
F3=0
```

**Floats**:
- No trailing zeros after decimal point (except one zero if no fractional part: `1.0`)
- No leading zeros before decimal point (except `0.x`)
- Scientific notation for |x| ≥ 10^15 or 0 < |x| < 10^-6
- Lowercase `e` for exponent

Non-canonical:
```
F1=3.140000
F2=01.5
F3=0.0000001
F4=1000000000000000
```

Canonical:
```
F1=3.14
F2=1.5
F3=1e-7
F4=1e15
```

**Rationale**: Normalized number formatting eliminates representation ambiguity and reduces size. Scientific notation prevents extremely long number strings.

**Rule 6: Boolean Representation**

Booleans MUST be represented as `0` (false) or `1` (true). String representations like `"true"` or `"false"` are not canonical.

Non-canonical:
```
F7=true
F8=false
F9="1"
```

Canonical:
```
F7=1
F8=0
F9="1"
```

Note: `F9="1"` is canonical because the type hint or context indicates it's a string, not a boolean.

**Rationale**: Numeric boolean representation is more token-efficient and unambiguous.

**Rule 7: Array Formatting**

String arrays MUST:
- Have no spaces after commas
- Use minimal string quoting (per Rule 4)
- Maintain element order (arrays are ordered)

Non-canonical:
```
F23=[ "admin" , "dev" , "guest" ]
F24=[a, b, c]
```

Canonical:
```
F23=["admin","dev","guest"]
F24=[a,b,c]
```

**Rationale**: Compact array formatting reduces size and eliminates whitespace ambiguity.

**Rule 8: Nested Structure Ordering**

Within nested records and nested arrays:
- Fields MUST be sorted by FID at every nesting level
- Semicolons MUST separate fields within nested records (newlines not used)
- No spaces around operators or separators

Non-canonical:
```
F50={F12=1; F7=1}
F60=[{F2=bob; F1=user}, {F2=alice; F1=admin}]
```

Canonical:
```
F50={F7=1;F12=1}
F60=[{F1=user;F2=bob},{F1=admin;F2=alice}]
```

**Rationale**: Consistent ordering at all nesting levels ensures deterministic representation of complex structures. Semicolons are required in nested contexts because newlines would break the nested syntax.

#### Complete Canonicalization Examples

**Example 1: Simple Fields**

Non-canonical input:
```
F23=["admin","dev"];F7=1;F12=14532
```

Canonical output:
```
F7=1
F12=14532
F23=["admin","dev"]
```

Changes applied:
- Fields reordered by FID (7, 12, 23)
- Semicolons replaced with newlines

**Example 2: Whitespace and Formatting**

Non-canonical input:
```
F12 : i = 014532
F7 : b = 1
F1 = "simple"
```

Canonical output:
```
F1=simple
F7:b=1
F12:i=14532
```

Changes applied:
- Fields reordered by FID (1, 7, 12)
- Whitespace removed
- Leading zero removed from integer
- Unnecessary quotes removed from string

**Example 3: Numbers and Booleans**

Non-canonical input:
```
F10=3.140000
F11=+42
F12=0.0000001
F13=true
F14=false
```

Canonical output:
```
F10=3.14
F11=42
F12=1e-7
F13=1
F14=0
```

Changes applied:
- Trailing zeros removed from float
- Positive sign removed from integer
- Scientific notation applied to small float
- Boolean strings converted to numeric form

**Example 4: Nested Structures**

Non-canonical input:
```
F60=[{F2=bob;F1=user},{F2=alice;F1=admin}]
F50={F12=1;F7=1;F2=test}
```

Canonical output:
```
F50={F2=test;F7=1;F12=1}
F60=[{F1=user;F2=bob},{F1=admin;F2=alice}]
```

Changes applied:
- Top-level fields reordered (F50 before F60)
- Nested fields reordered within F50 (2, 7, 12)
- Nested fields reordered within F60 array elements (1, 2)

**Example 5: Complex Record**

Non-canonical input:
```
F100 : r = { F3 = "value" ; F1 = 42 ; F2 : f = 3.140000 }
F50 : sa = [ "item1" , "item2" , "item3" ]
F10 = +123
```

Canonical output:
```
F10=123
F50:sa=["item1","item2","item3"]
F100:r={F1=42;F2:f=3.14;F3=value}
```

Changes applied:
- Fields reordered (10, 50, 100)
- Whitespace removed throughout
- Positive sign removed from F10
- Spaces removed from array in F50
- Nested fields reordered in F100 (1, 2, 3)
- Trailing zeros removed from float in F100
- Unnecessary quotes removed from F3 value

#### Canonicalization Algorithm

Implementations SHOULD use the following algorithm to produce canonical text:

1. **Parse** the input record into an internal representation (AST or data structure)
2. **Sort** all fields by FID in ascending order (recursively for nested structures)
3. **Normalize** each value:
   - Integers: Remove leading zeros, remove positive sign
   - Floats: Remove trailing zeros, apply scientific notation thresholds
   - Booleans: Convert to `0` or `1`
   - Strings: Apply minimal quoting rules
   - Arrays: Remove spaces, normalize elements
   - Nested: Recursively apply canonicalization
4. **Format** the output:
   - Use newlines between top-level fields
   - Use semicolons within nested structures
   - No whitespace around operators
   - Preserve type hints and checksums if present
5. **Validate** (optional): Verify the output matches canonical form rules

**Pseudocode**:

```python
def canonicalize_text(record):
    # Parse input
    fields = parse(record)
    
    # Sort by FID
    fields.sort(key=lambda f: f.fid)
    
    # Normalize and format each field
    lines = []
    for field in fields:
        line = f"F{field.fid}"
        if field.type_hint:
            line += f":{field.type_hint}"
        line += "="
        line += normalize_value(field.value)
        if field.checksum:
            line += f"#{field.checksum}"
        lines.append(line)
    
    # Join with newlines
    return "\n".join(lines)

def normalize_value(value):
    if isinstance(value, int):
        return str(value)  # No leading zeros
    elif isinstance(value, float):
        s = format_float(value)  # Remove trailing zeros, apply sci notation
        return s
    elif isinstance(value, bool):
        return "1" if value else "0"
    elif isinstance(value, str):
        return quote_if_needed(value)
    elif isinstance(value, list):
        if all(isinstance(x, str) for x in value):
            # String array
            elements = [quote_if_needed(x) for x in value]
            return f"[{','.join(elements)}]"
        else:
            # Nested array
            records = [canonicalize_nested(r) for r in value]
            return f"[{','.join(records)}]"
    elif isinstance(value, dict):
        # Nested record
        return canonicalize_nested(value)
```

#### Strict Mode

Parsers MAY implement a "strict mode" that rejects non-canonical input. In strict mode:
- Fields not in FID order → Error
- Semicolon separators at top level → Error
- Whitespace around operators → Error
- Non-minimal string quoting → Error
- Non-normalized numbers → Error
- Any deviation from canonical rules → Error

Strict mode is useful for:
- Compliance testing
- Validating encoder implementations
- Enforcing canonical format in specific contexts

**Example Strict Mode Behavior**:

```python
# Non-canonical input
input = "F12=14532;F7=1"

# Loose mode (default)
result = parse(input, strict=False)
# Success: Parses and normalizes to canonical form

# Strict mode
result = parse(input, strict=True)
# Error: Fields not in FID order (F12 before F7)
```

### 4.2 Binary Canonical Form

The binary canonical form defines the standardized representation of LNMP records in binary format. Binary canonicalization is simpler than text canonicalization because the binary format has less flexibility in representation.

#### Binary Canonicalization Rules

**Rule 1: Field Ordering**

Binary entries MUST be sorted by FID in ascending order, identical to text format.

**Rationale**: Consistent field ordering across formats enables reliable round-trip conversion and checksums.

**Rule 2: VarInt Encoding**

VarInt values MUST use the minimal byte representation. No unnecessary continuation bytes are permitted.

Example:
- Value 127: `7F` (1 byte) ✓ Canonical
- Value 127: `FF 00` (2 bytes) ✗ Non-canonical (unnecessary byte)

**Rationale**: Minimal encoding reduces size and eliminates representation ambiguity.

**Rule 3: Float Encoding**

Float values MUST use IEEE 754 normalized form:
- No denormalized numbers (use normalized representation)
- Consistent NaN representation (quiet NaN: `7FF8000000000000`)
- Consistent infinity representation (+Inf: `7FF0000000000000`, -Inf: `FFF0000000000000`)

**Rationale**: Normalized floats ensure consistent binary representation across platforms and implementations.

**Rule 4: String Encoding**

String values MUST:
- Use UTF-8 encoding without BOM (Byte Order Mark)
- Use minimal length encoding (no padding)
- Preserve exact byte content (no normalization of Unicode forms)

**Rationale**: UTF-8 without BOM is the standard encoding. No Unicode normalization preserves exact string content.

**Rule 5: Version and Flags**

- VERSION byte MUST be `0x04` for v0.4
- FLAGS byte MUST be `0x00` in v0.4 (all flags reserved)

**Rationale**: Consistent version and flags enable reliable version detection and future extensibility.

**Rule 6: Entry Count**

The ENTRY_COUNT field MUST accurately reflect the number of entries in the frame. It MUST use minimal VarInt encoding.

**Rationale**: Accurate count enables efficient parsing and validation.

#### Binary Canonicalization Examples

**Example 1: Field Ordering**

Non-canonical (fields out of order):
```
04 00 02
0C 00 01 E4 E3 00    # F12=14532
07 00 03 01          # F7=1
```

Canonical (fields in FID order):
```
04 00 02
07 00 03 01          # F7=1
0C 00 01 E4 E3 00    # F12=14532
```

**Example 2: VarInt Encoding**

Non-canonical (unnecessary continuation byte):
```
04 00 01
07 00 01 FF 00       # F7=127 (2 bytes for VarInt)
```

Canonical (minimal VarInt):
```
04 00 01
07 00 01 7F          # F7=127 (1 byte for VarInt)
```

**Example 3: Complete Frame**

Non-canonical:
```
04 00 03
17 00 05 02 05 61 64 6D 69 6E 03 64 65 76    # F23=["admin","dev"]
0C 00 01 E4 E3 00                            # F12=14532
07 00 03 01                                  # F7=1
```

Canonical:
```
04 00 03
07 00 03 01                                  # F7=1
0C 00 01 E4 E3 00                            # F12=14532
17 00 05 02 05 61 64 6D 69 6E 03 64 65 76    # F23=["admin","dev"]
```

Changes applied:
- Entries reordered by FID (7, 12, 23)

#### Binary Canonicalization Algorithm

Implementations SHOULD use the following algorithm:

1. **Parse** the binary frame into an internal representation
2. **Sort** all entries by FID in ascending order
3. **Validate** encoding:
   - VarInt values use minimal bytes
   - Float values are normalized
   - String values are valid UTF-8 without BOM
4. **Re-encode** the frame:
   - Write VERSION (0x04)
   - Write FLAGS (0x00)
   - Write ENTRY_COUNT (minimal VarInt)
   - Write entries in FID order
5. **Verify** the output is byte-for-byte canonical

**Pseudocode**:

```python
def canonicalize_binary(frame):
    # Parse frame
    version, flags, entries = parse_binary(frame)
    
    # Validate version and flags
    assert version == 0x04
    assert flags == 0x00
    
    # Sort entries by FID
    entries.sort(key=lambda e: e.fid)
    
    # Build canonical frame
    output = bytearray()
    output.append(0x04)  # VERSION
    output.append(0x00)  # FLAGS
    output.extend(encode_varint(len(entries)))  # ENTRY_COUNT
    
    for entry in entries:
        output.extend(encode_u16_le(entry.fid))  # FID
        output.append(entry.tag)  # TYPE_TAG
        output.extend(encode_value(entry.value, entry.tag))  # VALUE
    
    return bytes(output)
```

#### Binary Strict Mode

Binary parsers MAY implement strict mode that rejects non-canonical binary:
- Entries not in FID order → Error
- Non-minimal VarInt encoding → Error
- Denormalized floats → Error
- Invalid UTF-8 or UTF-8 with BOM → Error
- Incorrect VERSION or FLAGS → Error

### 4.3 Round-Trip Guarantees

LNMP provides strong guarantees for round-trip conversion between text and binary formats. These guarantees ensure that semantic content is preserved and that canonical forms are stable across conversions.

#### Text → Binary → Text Guarantee

**Statement**: Converting text to binary and back to text produces canonical text output.

**Formal Definition**:

```
∀ text_input ∈ ValidLNMP:
  canonical(text_input) = decode_binary(encode_text(text_input))
```

Where:
- `text_input` is any valid LNMP text input (canonical or non-canonical)
- `encode_text()` converts text to binary format
- `decode_binary()` converts binary to canonical text format
- `canonical()` converts text to canonical text format

**Example**:

```
Input text (non-canonical):
F23=["admin","dev"];F7=1;F12=14532

↓ encode_text()

Binary:
04 00 03 07 00 03 01 0C 00 01 E4 E3 00 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76

↓ decode_binary()

Output text (canonical):
F7=1
F12=14532
F23=["admin","dev"]
```

**Verification**:

```python
def verify_text_binary_text(text_input):
    # Convert to canonical form
    canonical_text = canonicalize_text(text_input)
    
    # Round-trip through binary
    binary = encode_text(text_input)
    output_text = decode_binary(binary)
    
    # Verify equality
    assert output_text == canonical_text
    return True
```

**Properties**:
- Field ordering is normalized (by FID)
- Value formatting is normalized (numbers, strings, booleans)
- Whitespace is normalized (newlines, no spaces)
- Semantic content is preserved exactly

#### Binary → Text → Binary Guarantee

**Statement**: Converting binary to text and back to binary produces identical binary output (byte-for-byte).

**Formal Definition**:

```
∀ binary_input ∈ CanonicalBinary:
  binary_input = encode_text(decode_binary(binary_input))
```

Where:
- `binary_input` is canonical binary format
- `decode_binary()` converts binary to canonical text
- `encode_text()` converts text to binary format

**Example**:

```
Input binary:
04 00 03 07 00 03 01 0C 00 01 E4 E3 00 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76

↓ decode_binary()

Text (canonical):
F7=1
F12=14532
F23=["admin","dev"]

↓ encode_text()

Output binary:
04 00 03 07 00 03 01 0C 00 01 E4 E3 00 17 00 05 02 05 61 64 6D 69 6E 03 64 65 76
```

**Verification**:

```python
def verify_binary_text_binary(binary_input):
    # Round-trip through text
    text = decode_binary(binary_input)
    output_binary = encode_text(text)
    
    # Verify byte-for-byte equality
    assert output_binary == binary_input
    return True
```

**Properties**:
- Binary representation is preserved exactly
- No information loss during conversion
- Field ordering is maintained
- Value encoding is maintained

#### Stability After First Canonicalization

**Statement**: Once a record is canonicalized, repeated conversions produce identical output.

**Formal Definition**:

```
∀ record ∈ ValidLNMP:
  canonical(record) = canonical(canonical(record))
  
∀ binary ∈ CanonicalBinary:
  binary = encode_text(decode_binary(binary))
```

**Example - Text Stability**:

```
Input (non-canonical):
F12=14532;F7=1

↓ First canonicalization

Canonical:
F7=1
F12=14532

↓ Second canonicalization

Canonical (unchanged):
F7=1
F12=14532

↓ Third canonicalization

Canonical (unchanged):
F7=1
F12=14532
```

**Example - Binary Stability**:

```
Input binary (canonical):
04 00 02 07 00 03 01 0C 00 01 E4 E3 00

↓ decode_binary() → encode_text()

Output binary (identical):
04 00 02 07 00 03 01 0C 00 01 E4 E3 00

↓ decode_binary() → encode_text()

Output binary (identical):
04 00 02 07 00 03 01 0C 00 01 E4 E3 00
```

**Verification**:

```python
def verify_stability(input_data):
    # Text stability
    if isinstance(input_data, str):
        canonical1 = canonicalize_text(input_data)
        canonical2 = canonicalize_text(canonical1)
        canonical3 = canonicalize_text(canonical2)
        assert canonical1 == canonical2 == canonical3
    
    # Binary stability
    elif isinstance(input_data, bytes):
        text = decode_binary(input_data)
        binary1 = encode_text(text)
        text2 = decode_binary(binary1)
        binary2 = encode_text(text2)
        assert binary1 == binary2 == input_data
    
    return True
```

#### Checksum Preservation

**Statement**: Checksums remain valid through format conversions when computed over canonical values.

**Example**:

```
Input text with checksum:
F12:i=14532#6A93B3F1

↓ encode_text()

Binary (checksum not encoded in v0.4):
04 00 01 0C 00 01 E4 E3 00

↓ decode_binary()

Output text (checksum recomputed):
F12:i=14532#6A93B3F1
```

**Note**: In v0.4, checksums are not encoded in binary format. When decoding binary to text, implementations MAY recompute checksums if configured to do so. The checksum will match the original because the canonical value is preserved.

#### Implementation Examples

**Example 1: Text Round-Trip**

```rust
use lnmp_codec::{Parser, Encoder, BinaryEncoder, BinaryDecoder};

fn test_text_roundtrip() {
    let input = "F23=[admin,dev];F7=1;F12=14532";
    
    // Parse and canonicalize
    let record = Parser::new().parse(input).unwrap();
    let canonical = Encoder::new().encode(&record).unwrap();
    
    // Convert to binary and back
    let binary = BinaryEncoder::new().encode(&record).unwrap();
    let decoded = BinaryDecoder::new().decode(&binary).unwrap();
    let output = Encoder::new().encode(&decoded).unwrap();
    
    // Verify
    assert_eq!(output, canonical);
    assert_eq!(output, "F7=1\nF12=14532\nF23=[admin,dev]");
}
```

**Example 2: Binary Round-Trip**

```rust
fn test_binary_roundtrip() {
    let input_binary = vec![
        0x04, 0x00, 0x02,           // Header
        0x07, 0x00, 0x03, 0x01,     // F7=1
        0x0C, 0x00, 0x01, 0xE4, 0xE3, 0x00,  // F12=14532
    ];
    
    // Decode to text
    let decoder = BinaryDecoder::new();
    let record = decoder.decode(&input_binary).unwrap();
    
    // Encode back to binary
    let encoder = BinaryEncoder::new();
    let output_binary = encoder.encode(&record).unwrap();
    
    // Verify byte-for-byte equality
    assert_eq!(output_binary, input_binary);
}
```

**Example 3: Multiple Round-Trips**

```rust
fn test_multiple_roundtrips() {
    let input = "F12=14532;F7=1";
    
    let mut current_text = input.to_string();
    
    // Perform 10 round-trips
    for _ in 0..10 {
        let record = Parser::new().parse(&current_text).unwrap();
        let binary = BinaryEncoder::new().encode(&record).unwrap();
        let decoded = BinaryDecoder::new().decode(&binary).unwrap();
        current_text = Encoder::new().encode(&decoded).unwrap();
    }
    
    // Verify stability
    let canonical = "F7=1\nF12=14532";
    assert_eq!(current_text, canonical);
}
```

#### Guarantees Summary

| Conversion | Input Format | Output Format | Guarantee |
|------------|--------------|---------------|-----------|
| Text → Binary → Text | Any valid text | Canonical text | Semantic preservation, canonical form |
| Binary → Text → Binary | Canonical binary | Identical binary | Byte-for-byte equality |
| Text → Text | Any valid text | Canonical text | Idempotent after first canonicalization |
| Binary → Binary | Canonical binary | Identical binary | Idempotent (no change) |

**Key Properties**:
- **Semantic Preservation**: All field values and types are preserved exactly
- **Canonical Stability**: Repeated canonicalization produces identical output
- **Deterministic**: Same input always produces same output
- **Reversible**: Binary ↔ Text conversions are lossless (except checksums in v0.4)
- **Interoperable**: All implementations produce identical canonical forms

This completes the Canonicalization section.

---

## 5. Type System

The LNMP type system defines all supported data types, their representations in text and binary formats, constraints, and validation rules. The type system is designed to be simple yet expressive, supporting both primitive types for common data and composite types for structured information.

LNMP's type system is explicitly typed through syntax rather than schema. The type of a value is determined by its syntactic form in text format and by its type tag in binary format. Optional type hints provide additional validation and disambiguation.

### 5.1 Primitive Types

LNMP supports five primitive types: integer, float, boolean, string, and string array. Each primitive type has specific syntax, encoding rules, and constraints.

#### Integer Type

**Description**: Signed 64-bit integer values.

**Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 (i64)

**Text Format Syntax**: `-?[0-9]+`
- Optional negative sign
- One or more decimal digits
- No leading zeros (except for `0` itself)
- No positive sign (`+`)
- No decimal point

**Binary Format Encoding**:
- Type tag: `0x01`
- Value encoding: VarInt (LEB128) with zigzag encoding for negative values
- Size: 1-10 bytes depending on magnitude

**Type Hint Code**: `:i`

**Examples**:

```
# Text format
F12=14532
F13=-42
F14=0
F15=9223372036854775807

# With type hint
F12:i=14532
F13:i=-42

# Binary format (hexadecimal)
0C 00 01 E4 E3 00           # F12=14532 (FID=12, tag=0x01, value=14532 as VarInt)
0D 00 01 54                 # F13=-42 (FID=13, tag=0x01, value=-42 zigzag encoded)
```

**Constraints**:
- Values outside the i64 range are invalid
- Leading zeros are not permitted (except for `0`)
- Positive sign (`+`) is not permitted
- Decimal points are not permitted (use float type)

**Canonical Form**:
- No leading zeros: `007` → `7`
- No positive sign: `+42` → `42`
- Negative zero normalized: `-0` → `0`

**Invalid Examples**:

```
F12=007                     # Error: leading zeros
F12=+42                     # Error: positive sign
F12=9223372036854775808     # Error: exceeds i64 max
F12=3.14                    # Error: decimal point (use float)
```

#### Float Type

**Description**: IEEE 754 double-precision floating-point values.

**Range**: Approximately ±1.7 × 10^308 with 15-17 significant decimal digits

**Text Format Syntax**: `-?[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?`
- Optional negative sign
- One or more decimal digits
- Decimal point (required)
- One or more fractional digits
- Optional scientific notation exponent

**Binary Format Encoding**:
- Type tag: `0x02`
- Value encoding: IEEE 754 double-precision (8 bytes, little-endian)
- Size: 8 bytes (fixed)

**Type Hint Code**: `:f`

**Examples**:

```
# Text format
F20=3.14
F21=-2.5
F22=0.0
F23=1.23e-10
F24=1.5e15

# With type hint
F20:f=3.14
F21:f=-2.5

# Binary format (hexadecimal)
14 00 02 1F 85 EB 51 B8 1E 09 40    # F20=3.14 (FID=20, tag=0x02, IEEE 754 bytes)
15 00 02 00 00 00 00 00 00 04 C0    # F21=-2.5
```

**Special Values**:

LNMP supports IEEE 754 special values with specific text representations:

| Value | Text Format | Binary Format | Description |
|-------|-------------|---------------|-------------|
| Positive Infinity | `Infinity` | `7FF0000000000000` | Result of overflow or division by zero |
| Negative Infinity | `-Infinity` | `FFF0000000000000` | Negative overflow |
| Not a Number | `NaN` | `7FF8000000000000` | Invalid operation result |
| Positive Zero | `0.0` | `0000000000000000` | Zero |
| Negative Zero | `-0.0` | `8000000000000000` | Negative zero (distinct in IEEE 754) |

**Examples with Special Values**:

```
F30=Infinity
F31=-Infinity
F32=NaN
F33=0.0
F34=-0.0
```

**Constraints**:
- Decimal point is required (distinguishes from integer)
- At least one digit before and after decimal point
- Scientific notation uses lowercase `e` in canonical form
- Denormalized numbers are normalized in canonical form

**Canonical Form**:
- Remove trailing zeros after decimal: `3.140000` → `3.14`
- Remove leading zeros before decimal: `01.5` → `1.5`
- Use scientific notation for |x| ≥ 10^15: `1000000000000000` → `1e15`
- Use scientific notation for 0 < |x| < 10^-6: `0.0000001` → `1e-7`
- Lowercase `e` for exponent: `1.5E10` → `1.5e10`
- No positive sign on exponent: `1e+10` → `1e10`

**Invalid Examples**:

```
F20=3                       # Error: no decimal point (use integer)
F20=.5                      # Error: no digit before decimal
F20=5.                      # Error: no digit after decimal
F20=3,14                    # Error: comma instead of decimal point
```

#### Boolean Type

**Description**: Logical true/false values.

**Values**: `0` (false) or `1` (true)

**Text Format Syntax**: `0|1`
- Single digit: `0` or `1`
- No other representations permitted in canonical form

**Binary Format Encoding**:
- Type tag: `0x03`
- Value encoding: Single byte (`0x00` for false, `0x01` for true)
- Size: 1 byte (fixed)

**Type Hint Code**: `:b`

**Examples**:

```
# Text format
F7=1                        # true
F8=0                        # false

# With type hint
F7:b=1
F8:b=0

# Binary format (hexadecimal)
07 00 03 01                 # F7=1 (FID=7, tag=0x03, value=0x01)
08 00 03 00                 # F8=0 (FID=8, tag=0x03, value=0x00)
```

**Constraints**:
- Only `0` and `1` are valid boolean values
- String representations like `"true"` or `"false"` are not booleans (they are strings)
- Any value other than `0` or `1` is invalid when type hint is `:b`

**Canonical Form**:
- Always `0` or `1` (numeric form)
- String representations are converted: `"true"` → `1`, `"false"` → `0` (only with equivalence mapping)

**Invalid Examples**:

```
F7:b=2                      # Error: only 0 or 1 permitted
F7:b=true                   # Error: string representation not allowed
F7:b="1"                    # Error: quoted string, not boolean
```

**Disambiguation**:

Without a type hint, the value `0` or `1` may be ambiguous (could be integer, boolean, or string). Type hints resolve this:

```
F7=1                        # Ambiguous: could be int, bool, or string "1"
F7:b=1                      # Explicitly boolean
F7:i=1                      # Explicitly integer
F7:s=1                      # Error: type mismatch (1 is not a string)
F7="1"                      # Explicitly string "1"
```

#### String Type

**Description**: UTF-8 encoded text values.

**Encoding**: UTF-8 (Unicode Transformation Format, 8-bit)

**Text Format Syntax**:
- **Quoted**: `"..."` with escape sequences
- **Unquoted**: `[A-Za-z0-9_.-]+` (alphanumeric, underscore, hyphen, dot)

**Binary Format Encoding**:
- Type tag: `0x04`
- Value encoding: Length (VarInt) + UTF-8 bytes
- Size: Variable (1+ bytes for length, N bytes for content)

**Type Hint Code**: `:s`

**Examples**:

```
# Text format - unquoted
F1=alice
F2=user_123
F3=192.168.1.1

# Text format - quoted
F4="hello world"
F5="line1\nline2"
F6="path\\to\\file"
F7="say \"hello\""

# With type hint
F1:s=alice
F4:s="hello world"

# Binary format (hexadecimal)
01 00 04 05 61 6C 69 63 65              # F1=alice (length=5, UTF-8 bytes)
04 00 04 0B 68 65 6C 6C 6F 20 77 6F 72 6C 64  # F4="hello world" (length=11)
```

**Quoting Rules**:

Strings MUST be quoted if they:
- Contain spaces or special characters (except `_`, `-`, `.`)
- Contain escape sequences
- Start with a digit (to avoid confusion with numbers)
- Could be confused with other types (e.g., `"0"` vs boolean `0`)
- Are empty (`""`)

Strings MAY be unquoted if they:
- Match the pattern `[A-Za-z0-9_.-]+`
- Start with a letter, underscore, or hyphen
- Do not conflict with reserved words (none in LNMP)

**Canonical Form**:
- Use unquoted form when possible: `"simple"` → `simple`
- Use quoted form when required: `hello world` → `"hello world"`
- Normalize escape sequences: `\x` → invalid (only `\\`, `\"`, `\n`, `\r`, `\t` supported)

**Escape Sequences**:

Supported escape sequences in quoted strings:

| Escape | Character | Unicode | Description |
|--------|-----------|---------|-------------|
| `\\` | `\` | U+005C | Backslash |
| `\"` | `"` | U+0022 | Double quote |
| `\n` | LF | U+000A | Newline (line feed) |
| `\r` | CR | U+000D | Carriage return |
| `\t` | TAB | U+0009 | Horizontal tab |

**Examples with Escapes**:

```
F10="hello\nworld"          # Newline
F11="path\\to\\file"        # Backslashes
F12="say \"hello\""         # Quotes
F13="tab\there"             # Tab
F14="line1\r\nline2"        # Windows line ending
```

**Constraints**:
- Strings MUST be valid UTF-8
- Invalid UTF-8 sequences are rejected
- No BOM (Byte Order Mark) in binary format
- Maximum length: Implementation-defined (recommended: 1 MB)
- Unsupported escape sequences (e.g., `\x`, `\u`, `\0`) are invalid

**Invalid Examples**:

```
F1="test\x41"               # Error: \x escape not supported
F2="test\u0041"             # Error: \u escape not supported
F3="test\0"                 # Error: \0 escape not supported
F4=hello world              # Error: unquoted string with space
```

#### String Array Type

**Description**: Ordered collection of string values.

**Elements**: UTF-8 encoded strings

**Text Format Syntax**: `[str,str,...]`
- Opening bracket `[`
- Comma-separated string elements
- Closing bracket `]`
- Empty arrays permitted: `[]`

**Binary Format Encoding**:
- Type tag: `0x05`
- Value encoding: Count (VarInt) + repeated (Length (VarInt) + UTF-8 bytes)
- Size: Variable

**Type Hint Code**: `:sa`

**Examples**:

```
# Text format
F23=["admin","dev","guest"]
F24=[alice,bob,charlie]
F25=[]
F26=["single"]

# With type hint
F23:sa=["admin","dev","guest"]

# Binary format (hexadecimal)
17 00 05 03                             # F23, tag=0x05, count=3
  05 61 64 6D 69 6E                     # "admin" (length=5)
  03 64 65 76                           # "dev" (length=3)
  05 67 75 65 73 74                     # "guest" (length=5)

19 00 05 00                             # F25=[] (count=0, empty array)
```

**Constraints**:
- All elements MUST be strings
- Elements maintain insertion order
- Duplicate elements are permitted
- Maximum elements: Implementation-defined (recommended: 10,000)
- Each element subject to string constraints (UTF-8, max length)

**Canonical Form**:
- No spaces after commas: `[a, b, c]` → `[a,b,c]`
- Use minimal string quoting: `["simple"]` → `[simple]`
- Preserve element order (arrays are ordered)

**Invalid Examples**:

```
F23=[1,2,3]                 # Error: elements must be strings, not integers
F23=["admin",123]           # Error: mixed types not permitted
F23=[admin dev]             # Error: missing comma separator
```

**Empty Arrays**:

```
F25=[]                      # Valid: empty array
F25:sa=[]                   # Valid: empty array with type hint
```

**Quoting in Arrays**:

Each element follows string quoting rules independently:

```
F23=[simple,"with space","123"]
```

- `simple` - unquoted (matches pattern)
- `"with space"` - quoted (contains space)
- `"123"` - quoted (starts with digit)

#### Primitive Types Summary Table

| Type | Text Syntax | Binary Tag | Binary Size | Type Hint | Range/Constraints |
|------|-------------|------------|-------------|-----------|-------------------|
| Integer | `-?[0-9]+` | `0x01` | 1-10 bytes | `:i` | -2^63 to 2^63-1 |
| Float | `-?[0-9]+\.[0-9]+` | `0x02` | 8 bytes | `:f` | IEEE 754 double |
| Boolean | `0\|1` | `0x03` | 1 byte | `:b` | 0 or 1 only |
| String | `"..."\|[A-Za-z0-9_.-]+` | `0x04` | Variable | `:s` | UTF-8, max 1MB |
| String Array | `[str,...]` | `0x05` | Variable | `:sa` | Max 10K elements |

This completes the Primitive Types subsection.

### 5.2 Composite Types

LNMP supports two composite types for representing structured and hierarchical data: nested records and nested arrays. These types enable complex data modeling while maintaining the protocol's simplicity and determinism.

**Note**: Composite types are fully supported in text format (v0.3+) but are reserved for future implementation in binary format (v0.5). In v0.4, binary format supports only primitive types.

#### Nested Record Type

**Description**: A record contained within another record as a field value, enabling hierarchical data structures.

**Structure**: Collection of fields with their own FIDs, type hints, and values

**Text Format Syntax**: `{F<fid>=<value>;...}`
- Opening brace `{` starts nested context
- Fields separated by semicolons (required, newlines not used)
- Fields sorted by FID in canonical form
- Closing brace `}` ends nested context
- Empty nested records permitted: `{}`

**Binary Format Encoding**:
- Type tag: `0x06` (reserved for v0.5)
- Not implemented in v0.4
- Implementations MUST reject binary frames with tag `0x06`

**Type Hint Code**: `:r`

**Examples**:

```
# Simple nested record
F50={F12=1;F7=1}

# Nested record with multiple fields
F100={F1=alice;F2=admin;F3=active}

# Nested record with type hints
F200={F12:i=14532;F7:b=1;F1:s=test}
```

# Multi-level nesting
F300={F1=outer;F2={F10=inner;F11=nested}}

# Nested record with arrays
F400={F1=user;F2=["admin","dev"]}

# Empty nested record
F500={}

# With type hint
F50:r={F12=1;F7=1}
```

**Constraints**:
- Fields within nested records follow all standard field rules
- FIDs are scoped to their containing record (F1 in outer record is distinct from F1 in nested record)
- Maximum nesting depth: Implementation-defined (recommended: 10 levels)
- Semicolons MUST separate fields (newlines not permitted within nested records)
- Fields MUST be sorted by FID in canonical form

**Canonical Form**:
- Fields sorted by FID at every nesting level
- No spaces around operators or separators
- Semicolons between fields (not newlines)

**Examples**:

Non-canonical:
```
F50={F12=1; F7=1}
F100={ F2=bob ; F1=alice }
```

Canonical:
```
F50={F7=1;F12=1}
F100={F1=alice;F2=bob}
```

**Nesting Depth**:

Implementations SHOULD support at least 10 levels of nesting. Deeper nesting MAY be rejected:

```
# Level 1
F1={F2=value}

# Level 2
F1={F2={F3=value}}

# Level 3
F1={F2={F3={F4=value}}}

# ... up to recommended maximum of 10 levels
```

**Invalid Examples**:

```
F50={F12=1 F7=1}            # Error: missing semicolon separator
F50={F12=1;F7=1;            # Error: trailing semicolon (allowed but not canonical)
F50={F12=}                  # Error: missing value
F50={12=1}                  # Error: missing F prefix
```

**Use Cases**:

Nested records are useful for:
- User profiles with embedded address information
- Configuration objects with subsections
- Hierarchical data models
- Structured metadata

**Example - User Profile**:

```
F100={F1=alice;F2=alice@example.com;F10={F20=123 Main St;F21=San Francisco;F22=CA}}
```

This represents:
- F100: User record
  - F1: Username "alice"
  - F2: Email "alice@example.com"
  - F10: Address record (nested)
    - F20: Street "123 Main St"
    - F21: City "San Francisco"
    - F22: State "CA"

#### Nested Array Type

**Description**: An array of records as a field value, enabling collections of structured data.

**Elements**: Complete records in `{...}` format

**Text Format Syntax**: `[{F<fid>=<value>},...{F<fid>=<value>}]`
- Opening bracket `[` starts array context
- Each element is a complete record in `{...}` format
- Elements separated by commas
- Closing bracket `]` ends array context
- Empty arrays permitted: `[]`

**Binary Format Encoding**:
- Type tag: `0x07` (reserved for v0.5)
- Not implemented in v0.4
- Implementations MUST reject binary frames with tag `0x07`

**Type Hint Code**: `:ra`

**Examples**:

```
# Array of simple records
F60=[{F12=1},{F12=2},{F12=3}]

# Array with multiple fields per record
F200=[{F1=alice;F2=admin},{F1=bob;F2=user},{F1=charlie;F2=guest}]

# Array with type hints
F300=[{F1:s=alice;F7:b=1},{F1:s=bob;F7:b=0}]

# Empty array
F400=[]

# Array with nested structures
F500=[{F1=dept;F2={F10=eng;F11=dev}},{F1=dept;F2={F10=sales;F11=west}}]

# With type hint
F60:ra=[{F12=1},{F12=2}]
```

**Constraints**:
- All elements MUST be records (enclosed in `{...}`)
- Elements maintain insertion order
- Each record follows standard field rules (FID ordering, etc.)
- Maximum elements: Implementation-defined (recommended: 1,000)
- Maximum nesting depth applies to nested structures within array elements

**Canonical Form**:
- No spaces after commas
- Fields within each record sorted by FID
- Element order preserved (arrays are ordered)

**Examples**:

Non-canonical:
```
F60=[{F12=1}, {F12=2}, {F12=3}]
F200=[{F2=bob;F1=user},{F2=alice;F1=admin}]
```

Canonical:
```
F60=[{F12=1},{F12=2},{F12=3}]
F200=[{F1=user;F2=bob},{F1=admin;F2=alice}]
```

**Invalid Examples**:

```
F60=[{F12=1} {F12=2}]       # Error: missing comma separator
F60=[F12=1,F12=2]           # Error: elements must be records (missing braces)
F60=[{F12=1},{F7=true}]     # Error: "true" is not valid (use 1)
F60=[{},{},{}]              # Valid: array of empty records
```

**Use Cases**:

Nested arrays are useful for:
- Lists of users with attributes
- Collections of configuration items
- Query results with multiple rows
- Batch operations with structured data

**Example - User List**:

```
F100=[
  {F1=alice;F2=alice@example.com;F3=["admin","dev"]},
  {F1=bob;F2=bob@example.com;F3=["user"]},
  {F1=charlie;F2=charlie@example.com;F3=["guest"]}
]
```

This represents an array of user records, each with:
- F1: Username
- F2: Email
- F3: Roles (string array)

Note: In canonical form, this would be on a single line with no spaces.

#### v0.4 Binary Format Limitations

**Important**: In LNMP v0.4, the binary format does NOT support nested records or nested arrays. These types are reserved for v0.5.

**Current Support Matrix**:

| Type | Text Format (v0.3+) | Binary Format (v0.4) | Binary Format (v0.5+) |
|------|---------------------|----------------------|-----------------------|
| Integer | ✓ Supported | ✓ Supported (0x01) | ✓ Supported |
| Float | ✓ Supported | ✓ Supported (0x02) | ✓ Supported |
| Boolean | ✓ Supported | ✓ Supported (0x03) | ✓ Supported |
| String | ✓ Supported | ✓ Supported (0x04) | ✓ Supported |
| String Array | ✓ Supported | ✓ Supported (0x05) | ✓ Supported |
| Nested Record | ✓ Supported | ✗ Reserved (0x06) | ✓ Planned |
| Nested Array | ✓ Supported | ✗ Reserved (0x07) | ✓ Planned |

**Implications**:

1. **Text-Only Nested Structures**: Records containing nested structures can only be represented in text format in v0.4
2. **Binary Conversion Limitation**: Text records with nested structures CANNOT be converted to v0.4 binary format
3. **Error Handling**: Implementations MUST return an error when attempting to encode nested structures to v0.4 binary
4. **Forward Compatibility**: Type tags 0x06 and 0x07 are reserved; v0.4 decoders MUST reject frames containing these tags

**Example Error Scenario**:

```rust
// Text with nested record
let text = "F50={F12=1;F7=1}";

// Parse successfully
let record = Parser::new().parse(text).unwrap();

// Attempt binary encoding
let result = BinaryEncoder::new().encode(&record);

// Error: nested structures not supported in v0.4 binary format
assert!(result.is_err());
```

**Workarounds for v0.4**:

Until v0.5 binary support is available, applications needing binary encoding of nested structures can:

1. **Flatten structures**: Convert nested records to top-level fields with naming conventions
2. **Use text format**: Transmit nested structures in text format only
3. **Serialize separately**: Encode nested structures as JSON/CBOR strings within LNMP string fields
4. **Wait for v0.5**: Plan migration to v0.5 when nested binary support is available

**Example Flattening**:

```
# Original nested structure
F50={F12=1;F7=1}

# Flattened for binary compatibility
F50_12=1
F50_7=1
```

#### Composite Types Summary Table

| Type | Text Syntax | Binary Tag | Text Support | Binary Support (v0.4) | Type Hint |
|------|-------------|------------|--------------|----------------------|-----------|
| Nested Record | `{F<fid>=<val>;...}` | `0x06` | ✓ v0.3+ | ✗ Reserved for v0.5 | `:r` |
| Nested Array | `[{...},{...}]` | `0x07` | ✓ v0.3+ | ✗ Reserved for v0.5 | `:ra` |

This completes the Composite Types subsection.

### 5.3 Type Hints

Type hints are optional annotations that explicitly specify the expected type of a field value. They provide validation, disambiguation, and documentation benefits without being required for basic protocol operation.

#### Purpose and Benefits

Type hints serve several purposes:

1. **Validation**: Ensure values match expected types before processing
2. **Disambiguation**: Resolve ambiguous values (e.g., `0` as integer vs boolean vs string)
3. **Documentation**: Make field types explicit for human readers and tools
4. **Schema Enforcement**: Enable schema-like validation without requiring separate schema files
5. **Error Detection**: Catch type mismatches early in parsing

#### Syntax

Type hints use the format `:<code>` immediately after the field ID and before the equals sign:

```
F<fid>:<type_hint>=<value>
```

#### Type Hint Codes

LNMP defines seven type hint codes corresponding to the seven supported types:

| Type Hint | Code | Type | Description | Use Cases |
|-----------|------|------|-------------|-----------|
| Integer | `:i` | Integer (i64) | Signed 64-bit integer | Disambiguate `0` from boolean, validate numeric fields |
| Float | `:f` | Float (f64) | IEEE 754 double | Force float interpretation, validate decimal values |
| Boolean | `:b` | Boolean | True (1) or false (0) | Validate 0/1 as boolean, distinguish from integer |
| String | `:s` | String | UTF-8 text | Force string interpretation of numeric-looking values |
| String Array | `:sa` | String Array | Array of strings | Validate array structure and element types |
| Record | `:r` | Nested Record | Nested record structure | Validate nested record syntax |
| Record Array | `:ra` | Nested Array | Array of records | Validate array of records structure |

#### When to Use Type Hints

Type hints are OPTIONAL but RECOMMENDED in the following scenarios:

**1. Ambiguous Values**

When a value could be interpreted as multiple types:

```
F7=0                        # Ambiguous: integer 0, boolean false, or string "0"?
F7:b=0                      # Explicit: boolean false
F7:i=0                      # Explicit: integer 0
F7:s=0                      # Error: type mismatch (0 is not a string)
```

**2. Validation Requirements**

When strict type checking is needed:

```
F12:i=14532                 # Validate that value is an integer
F7:b=1                      # Validate that value is 0 or 1
F23:sa=["admin","dev"]      # Validate that value is a string array
```

**3. Schema Enforcement**

When implementing schema-like validation without separate schema files:

```
# User record with type hints
F1:s=alice
F2:s=alice@example.com
F7:b=1
F12:i=14532
F23:sa=["admin","dev"]
```

**4. Documentation**

When making field types explicit for human readers:

```
# Configuration with documented types
F100:i=3600                 # Timeout in seconds (integer)
F101:f=0.95                 # Confidence threshold (float)
F102:b=1                    # Enable feature (boolean)
F103:s=production           # Environment name (string)
```

**5. LLM-Generated Output**

When validating LLM-generated LNMP to catch hallucinations or type errors:

```
# Expected: F7:b=1
# LLM generates: F7:b=true
# Error caught: "true" is not a valid boolean (must be 0 or 1)
```

#### When NOT to Use Type Hints

Type hints are NOT REQUIRED in the following scenarios:

**1. Unambiguous Values**

When the value syntax clearly indicates the type:

```
F1=alice                    # Clearly a string (unquoted identifier)
F2="hello world"            # Clearly a string (quoted)
F3=3.14                     # Clearly a float (has decimal point)
F4=["a","b"]                # Clearly a string array (bracket syntax)
F5={F1=x}                   # Clearly a nested record (brace syntax)
```

**2. Simple Use Cases**

When validation is not critical and types are obvious from context:

```
F12=14532
F1=alice
F23=[admin,dev]
```

**3. Token Efficiency Priority**

When minimizing token count is more important than validation:

```
# Without type hints (fewer tokens)
F7=1;F12=14532;F23=[admin,dev]

# With type hints (more tokens)
F7:b=1;F12:i=14532;F23:sa=[admin,dev]
```

#### Type Hint Examples

**Example 1: Integer Type Hint**

```
F12:i=14532                 # Valid: 14532 is an integer
F12:i=-42                   # Valid: negative integer
F12:i=0                     # Valid: zero
F12:i=3.14                  # Error: 3.14 is a float, not an integer
F12:i="123"                 # Error: "123" is a string, not an integer
```

**Example 2: Float Type Hint**

```
F20:f=3.14                  # Valid: 3.14 is a float
F20:f=-2.5                  # Valid: negative float
F20:f=1.0                   # Valid: float with .0
F20:f=42                    # Error: 42 is an integer, not a float
F20:f=Infinity              # Valid: special float value
```

**Example 3: Boolean Type Hint**

```
F7:b=1                      # Valid: 1 is boolean true
F7:b=0                      # Valid: 0 is boolean false
F7:b=2                      # Error: only 0 or 1 permitted
F7:b=true                   # Error: "true" is not valid (use 1)
F7:b="1"                    # Error: "1" is a string, not a boolean
```

**Example 4: String Type Hint**

```
F1:s=alice                  # Valid: alice is a string
F1:s="hello world"          # Valid: quoted string
F1:s=123                    # Error: 123 is an integer, not a string
F1:s="123"                  # Valid: "123" is a string
F1:s=0                      # Error: 0 is an integer/boolean, not a string
```

**Example 5: String Array Type Hint**

```
F23:sa=["admin","dev"]      # Valid: string array
F23:sa=[alice,bob]          # Valid: unquoted strings
F23:sa=[]                   # Valid: empty array
F23:sa=[1,2,3]              # Error: elements must be strings
F23:sa="admin,dev"          # Error: string, not array
```

**Example 6: Nested Record Type Hint**

```
F50:r={F12=1;F7=1}          # Valid: nested record
F50:r={}                    # Valid: empty nested record
F50:r={F12=1}               # Valid: single field
F50:r=[{F12=1}]             # Error: array, not record
F50:r="record"              # Error: string, not record
```

**Example 7: Nested Array Type Hint**

```
F60:ra=[{F12=1},{F12=2}]    # Valid: array of records
F60:ra=[]                   # Valid: empty array
F60:ra=[{}]                 # Valid: array with empty record
F60:ra={F12=1}              # Error: record, not array
F60:ra=["a","b"]            # Error: string array, not record array
```

#### Type Hint Validation Rules

Implementations MUST enforce the following validation rules when type hints are present:

1. **Type Match**: The value MUST match the type specified by the hint
2. **Syntax Check**: The value syntax MUST conform to the type's syntax rules
3. **Constraint Check**: The value MUST satisfy the type's constraints (range, length, etc.)
4. **Error on Mismatch**: Type mismatches MUST produce an error (not a warning)

**Validation Algorithm**:

```python
def validate_type_hint(field):
    if not field.type_hint:
        return True  # No validation needed
    
    expected_type = field.type_hint
    actual_type = infer_type(field.value)
    
    if actual_type != expected_type:
        raise TypeHintMismatch(
            f"Field F{field.fid} has type hint {expected_type} "
            f"but value is {actual_type}"
        )
    
    return True
```

#### Type Hint Preservation

Type hints are preserved through format conversions:

**Text → Binary → Text**:
- Type hints are NOT encoded in v0.4 binary format
- When decoding, implementations MAY infer type hints from value syntax
- Original type hints are lost in round-trip

**Example**:

```
Input text:
F12:i=14532

↓ encode_text()

Binary:
0C 00 01 E4 E3 00           # No type hint encoded

↓ decode_binary()

Output text:
F12=14532                   # Type hint not preserved
```

**Note**: Future versions may encode type hints in binary format using the FLAGS byte or extended entry format.

#### Complete Example with Type Hints

```
# User record with comprehensive type hints
F1:s=alice
F2:s=alice@example.com
F7:b=1
F12:i=14532
F20:f=3.14
F23:sa=["admin","dev"]
F50:r={F1:s=metadata;F2:i=42}
F60:ra=[{F1:s=item1;F2:i=1},{F1:s=item2;F2:i=2}]
```

This record demonstrates:
- String fields with `:s` hints (F1, F2)
- Boolean field with `:b` hint (F7)
- Integer field with `:i` hint (F12)
- Float field with `:f` hint (F20)
- String array with `:sa` hint (F23)
- Nested record with `:r` hint (F50)
- Nested array with `:ra` hint (F60)

#### Type Hints Summary

| Aspect | Details |
|--------|---------|
| **Required?** | No, optional |
| **Syntax** | `:<code>` after FID, before `=` |
| **Codes** | `:i`, `:f`, `:b`, `:s`, `:sa`, `:r`, `:ra` |
| **Validation** | Enforced when present |
| **Binary Encoding** | Not encoded in v0.4 |
| **Use Cases** | Disambiguation, validation, documentation, schema enforcement |

This completes the Type Hints subsection.

### 5.4 Type Coercion

Type coercion is the automatic conversion of values from one type to another. LNMP takes a strict, explicit approach to typing: **automatic type coercion is NOT performed by the protocol**.

#### No Automatic Coercion

LNMP does NOT automatically convert between types. Each value has exactly one type determined by its syntax, and type hints (when present) MUST match that type exactly.

**Rationale**:

1. **Predictability**: No surprises from implicit conversions
2. **Semantic Integrity**: Values mean exactly what they say
3. **Error Detection**: Type mismatches are caught immediately
4. **LLM Clarity**: LLMs learn precise type syntax without ambiguity
5. **Interoperability**: All implementations behave identically

#### Valid Type Matching

When type hints are present, the value syntax MUST match the expected type:

**Valid Examples**:

```
F12:i=14532                 # ✓ Integer value, integer hint
F20:f=3.14                  # ✓ Float value, float hint
F7:b=1                      # ✓ Boolean value, boolean hint
F1:s=alice                  # ✓ String value, string hint
F23:sa=["admin","dev"]      # ✓ String array value, string array hint
F50:r={F12=1}               # ✓ Nested record value, record hint
F60:ra=[{F12=1}]            # ✓ Nested array value, array hint
```

#### Invalid Type Mismatches

The following are type mismatches and MUST produce errors:

**Invalid Examples**:

```
F12:i="14532"               # ✗ String value, integer hint expected
F20:f=3                     # ✗ Integer value, float hint expected
F7:b=true                   # ✗ String "true", boolean hint expected
F7:b=2                      # ✗ Integer 2, boolean must be 0 or 1
F1:s=123                    # ✗ Integer value, string hint expected
F23:sa="admin,dev"          # ✗ String value, array hint expected
F50:r=[{F12=1}]             # ✗ Array value, record hint expected
```

#### Error Behavior

When a type mismatch is detected, implementations MUST:

1. **Reject the field**: Do not accept the value
2. **Return an error**: Provide clear error message with field ID and type mismatch details
3. **Stop processing**: Do not continue parsing (unless in lenient mode)

**Example Error Messages**:

```
Error: Type hint mismatch for field F12
  Expected: integer (:i)
  Actual: string ("14532")
  Location: line 3, column 5

Error: Type hint mismatch for field F7
  Expected: boolean (:b)
  Actual: integer (2)
  Note: Boolean values must be 0 or 1

Error: Type hint mismatch for field F20
  Expected: float (:f)
  Actual: integer (42)
  Suggestion: Use 42.0 for float or remove :f hint
```

#### No Implicit Conversions

LNMP does NOT perform common implicit conversions found in other languages:

**String to Number**:

```
F12:i="123"                 # ✗ NOT converted to integer 123
```

**Number to String**:

```
F1:s=123                    # ✗ NOT converted to string "123"
```

**Integer to Float**:

```
F20:f=42                    # ✗ NOT converted to float 42.0
```

**Float to Integer**:

```
F12:i=42.0                  # ✗ NOT converted to integer 42
```

**Boolean to Integer**:

```
F12:i=1                     # Ambiguous without context
F12:i=1                     # If parsed as boolean, ✗ NOT converted to integer
```

**String to Boolean**:

```
F7:b=true                   # ✗ NOT converted to boolean 1
F7:b="1"                    # ✗ NOT converted to boolean 1
```

#### Equivalence Mapping (Optional Feature)

While LNMP does NOT perform automatic type coercion, implementations MAY support **equivalence mapping** as an optional, explicitly configured feature for semantic normalization.

**Definition**: Equivalence mapping is a user-configured feature that normalizes semantically equivalent values to a canonical form based on field-specific rules.

**Key Characteristics**:

1. **Opt-In**: Must be explicitly enabled and configured
2. **Field-Specific**: Rules apply to specific FIDs, not globally
3. **Pre-Parsing**: Applied before type validation
4. **Semantic Dictionary**: Configured via external semantic dictionary file
5. **Not Part of Core Protocol**: Optional extension, not required for conformance

**Use Cases**:

- Normalizing boolean representations: `"yes"` → `1`, `"true"` → `1`
- Standardizing terminology: `"admin"` → `"administrator"`
- Handling legacy data: `"Y"` → `1`, `"N"` → `0`
- LLM output normalization: Various boolean representations to canonical form

**Example Configuration** (semantic dictionary):

```yaml
equivalence_mappings:
  F7:  # Boolean field
    "yes": 1
    "true": 1
    "on": 1
    "no": 0
    "false": 0
    "off": 0
  
  F23:  # Role field
    "admin": "administrator"
    "dev": "developer"
    "qa": "quality_assurance"
```

**Example Usage**:

```
# Input (with equivalence mapping enabled)
F7=yes
F23=admin

# After equivalence mapping (before parsing)
F7=1
F23=administrator

# Parsed result
F7:b=1
F23:s=administrator
```

**Important Distinctions**:

| Feature | Type Coercion | Equivalence Mapping |
|---------|---------------|---------------------|
| **Automatic?** | Yes (if supported) | No, explicit config required |
| **Scope** | All fields | Specific FIDs only |
| **Part of Protocol?** | No, LNMP prohibits | No, optional extension |
| **When Applied?** | During parsing | Before parsing |
| **Configurable?** | Usually not | Yes, per-field rules |
| **LNMP Support** | ✗ Not supported | ✓ Optional feature |

**Equivalence Mapping Algorithm**:

```python
def apply_equivalence_mapping(field_text, semantic_dict):
    # Parse field to extract FID and value
    fid, value = extract_fid_and_value(field_text)
    
    # Check if FID has equivalence rules
    if fid not in semantic_dict:
        return field_text  # No mapping
    
    # Check if value has a mapping
    mappings = semantic_dict[fid]
    if value in mappings:
        # Replace value with mapped equivalent
        new_value = mappings[value]
        return reconstruct_field(fid, new_value)
    
    return field_text  # No mapping for this value

# Example usage
semantic_dict = {
    7: {"yes": "1", "true": "1", "no": "0", "false": "0"}
}

input_text = "F7=yes"
mapped_text = apply_equivalence_mapping(input_text, semantic_dict)
# Result: "F7=1"

# Now parse normally
record = parse(mapped_text)
# F7 is now boolean 1
```

**Equivalence Mapping Best Practices**:

1. **Document Mappings**: Clearly document all equivalence rules
2. **Validate Targets**: Ensure mapped values are valid for the field type
3. **Avoid Ambiguity**: Don't map multiple values to the same target if they have different semantics
4. **Test Thoroughly**: Verify mappings don't introduce unexpected behavior
5. **Version Control**: Track changes to semantic dictionaries
6. **Minimal Mappings**: Only map when necessary for interoperability

#### Explicit Conversion (Application Layer)

While LNMP does not perform type coercion, applications MAY perform explicit conversions at the application layer after parsing:

**Example**:

```rust
// Parse LNMP
let record = Parser::new().parse("F12=14532").unwrap();

// Application-layer conversion
let value = record.get_field(12).unwrap();
match value {
    Value::Integer(i) => {
        // Explicitly convert to string if needed
        let s = i.to_string();
        println!("User ID as string: {}", s);
    }
    _ => panic!("Expected integer"),
}
```

**Key Point**: Conversions happen AFTER parsing, in application code, not during protocol processing.

#### Summary: Type Coercion Policy

| Scenario | LNMP Behavior | Rationale |
|----------|---------------|-----------|
| Automatic type coercion | ✗ Not supported | Maintains semantic integrity |
| Type hint validation | ✓ Enforced strictly | Catches errors early |
| Implicit conversions | ✗ Not performed | Predictable behavior |
| Equivalence mapping | ✓ Optional feature | Semantic normalization |
| Application conversions | ✓ Allowed | Application-specific needs |

**Core Principle**: LNMP values have exactly one type determined by syntax. Type hints validate this type but do not convert it. Applications may convert values after parsing, but the protocol itself performs no coercion.

This completes the Type Coercion subsection and the entire Type System section.

---



## 6. Formal Grammar

The LNMP text format is defined using formal grammar notations to ensure unambiguous parsing across all implementations. This section provides complete grammar specifications in both ABNF (Augmented Backus-Naur Form) and EBNF (Extended Backus-Naur Form) notations, along with parsing precedence rules that eliminate ambiguity.

The grammar is designed to be:
- **Unambiguous**: Every valid input has exactly one parse tree
- **Deterministic**: Parsing behavior is identical across all implementations
- **Complete**: All valid LNMP v0.4 syntax is covered
- **Implementable**: Suitable for hand-written recursive descent parsers

### 6.1 ABNF Grammar

This section provides the complete ABNF grammar for the LNMP text format, following RFC 5234 [RFC5234] conventions. ABNF (Augmented Backus-Naur Form) is a widely-used notation for defining syntax in Internet protocols.

#### Core Rules

The following core rules from RFC 5234 are used throughout this grammar:

```abnf
ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
DIGIT          =  %x30-39             ; 0-9
HEXDIG         =  DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
DQUOTE         =  %x22                ; " (double quote)
SP             =  %x20                ; space
HTAB           =  %x09                ; horizontal tab
LF             =  %x0A                ; line feed
CR             =  %x0D                ; carriage return

```

#### Top-Level Rules

```abnf
; A complete LNMP record
lnmp-record     = [field-list]

; List of fields separated by semicolons or newlines
field-list      = field *(separator field) [separator]

; Field separator (semicolon or newline)
separator       = ";" / LF

; A single field assignment
field           = field-prefix field-id [type-hint] "=" value [checksum]

; Field prefix (always uppercase F)
field-prefix    = "F"

; Field identifier (0-65535)
field-id        = 1*5DIGIT
                  ; Maximum value 65535 (5 digits)
                  ; No leading zeros except for "0" itself

; Optional type hint
type-hint       = ":" type-code

; Type codes
type-code       = "ra" / "sa" / "i" / "f" / "b" / "s" / "r"
                  ; Order matters: longer codes first to avoid ambiguity
                  ; ra = record array, sa = string array
                  ; i = integer, f = float, b = boolean
                  ; s = string, r = record

```

#### Value Type Rules

```abnf
; Any value type (ordered by precedence)
value           = nested-record / nested-array / string-array /
                  boolean / number / string

; Nested record: {F1=value;F2=value}
nested-record   = "{" [nested-field-list] "}"

; Field list within nested record (semicolon-separated only)
nested-field-list = nested-field *(";" nested-field) [";"]

; Field within nested record
nested-field    = field-prefix field-id [type-hint] "=" value [checksum]

; Nested array of records: [{F1=v},{F2=v}]
nested-array    = "[" [record-list] "]"

; List of records within array
record-list     = nested-record *("," nested-record)

; String array: [str1,str2,str3]
string-array    = "[" [string-list] "]"

; List of strings
string-list     = string *("," string)

```

#### Primitive Type Rules

```abnf
; String value (quoted or unquoted)
string          = quoted-string / unquoted-string

; Quoted string with escape sequences
quoted-string   = DQUOTE *quoted-char DQUOTE

; Characters allowed in quoted strings
quoted-char     = escape-sequence / 
                  %x20-21 /           ; SP-!
                  %x23-5B /           ; #-[
                  %x5D-10FFFF         ; ]-end of Unicode
                  ; Excludes: " (0x22) and \ (0x5C)

; Escape sequences
escape-sequence = "\" ("\" / DQUOTE / "n" / "r" / "t")
                  ; \\ = backslash
                  ; \" = double quote
                  ; \n = newline (LF)
                  ; \r = carriage return (CR)
                  ; \t = horizontal tab

; Unquoted string (alphanumeric, underscore, hyphen, dot)
unquoted-string = 1*(ALPHA / DIGIT / "_" / "-" / ".")

; Numeric value (integer or float)
number          = ["-"] 1*DIGIT ["." 1*DIGIT] [exponent]

; Scientific notation exponent
exponent        = ("e" / "E") [("+" / "-")] 1*DIGIT

; Boolean value (0 or 1)
boolean         = "0" / "1"

```

#### Checksum and Whitespace Rules

```abnf
; Semantic checksum: #XXXXXXXX (8 hex digits)
checksum        = "#" 8HEXDIG

; Whitespace (spaces and tabs, ignored between tokens)
WSP             = SP / HTAB

; Comments (# followed by text until newline, ignored)
; Note: Checksums take precedence (parsed first)
comment         = "#" *(WSP / %x21-10FFFF) LF
                  ; Any characters except newline
```

#### Complete ABNF Grammar Summary

The complete ABNF grammar for LNMP v0.4 text format is:

```abnf
; ============================================================================
; LNMP v0.4 Text Format - Complete ABNF Grammar
; Reference: RFC 5234 (ABNF Specification)
; ============================================================================

; Top-Level
lnmp-record     = [field-list]
field-list      = field *(separator field) [separator]
separator       = ";" / LF
field           = field-prefix field-id [type-hint] "=" value [checksum]

; Field Components
field-prefix    = "F"
field-id        = 1*5DIGIT
type-hint       = ":" type-code
type-code       = "ra" / "sa" / "i" / "f" / "b" / "s" / "r"

; Value Types
value           = nested-record / nested-array / string-array /
                  boolean / number / string


; Nested Structures
nested-record   = "{" [nested-field-list] "}"
nested-field-list = nested-field *(";" nested-field) [";"]
nested-field    = field-prefix field-id [type-hint] "=" value [checksum]
nested-array    = "[" [record-list] "]"
record-list     = nested-record *("," nested-record)

; Primitive Types
string-array    = "[" [string-list] "]"
string-list     = string *("," string)
string          = quoted-string / unquoted-string
quoted-string   = DQUOTE *quoted-char DQUOTE
quoted-char     = escape-sequence / %x20-21 / %x23-5B / %x5D-10FFFF
escape-sequence = "\" ("\" / DQUOTE / "n" / "r" / "t")
unquoted-string = 1*(ALPHA / DIGIT / "_" / "-" / ".")
number          = ["-"] 1*DIGIT ["." 1*DIGIT] [exponent]
exponent        = ("e" / "E") [("+" / "-")] 1*DIGIT
boolean         = "0" / "1"

; Checksum
checksum        = "#" 8HEXDIG

; Core Rules (from RFC 5234)
ALPHA           = %x41-5A / %x61-7A
DIGIT           = %x30-39
HEXDIG          = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
DQUOTE          = %x22
SP              = %x20
HTAB            = %x09
LF              = %x0A
```


#### Grammar Notes

**Field ID Constraints**: The `field-id` rule allows 1-5 digits, but implementations MUST validate that the numeric value is in the range 0-65535. Values outside this range are invalid.

**Type Code Ordering**: The `type-code` rule lists longer codes first (`ra`, `sa`) to ensure correct parsing. ABNF uses ordered choice semantics in some implementations, so this ordering prevents ambiguity.

**Value Precedence**: The `value` rule lists alternatives in precedence order. Parsers MUST attempt to match alternatives in the order specified to ensure deterministic parsing.

**Whitespace Handling**: Whitespace (spaces and tabs) between tokens is implicitly ignored in ABNF. Implementations SHOULD skip whitespace during lexical analysis.

**Comment Disambiguation**: The `checksum` rule takes precedence over comments. A `#` followed by exactly 8 hexadecimal digits is always parsed as a checksum, not a comment.

**Escape Sequences**: Only five escape sequences are supported: `\\`, `\"`, `\n`, `\r`, `\t`. Any other escape sequence (e.g., `\x`, `\u`) is invalid and MUST be rejected.

**Number Format**: The `number` rule matches both integers and floats. Integers have no decimal point; floats MUST have a decimal point. Scientific notation is supported for both.

**Boolean Disambiguation**: The `boolean` rule matches only `0` and `1`. When a type hint is present (`:b`), implementations MUST validate that the value is exactly `0` or `1`.

This completes the ABNF Grammar subsection.


### 6.2 EBNF Grammar

This section provides the complete EBNF grammar for the LNMP text format, following ISO/IEC 14977 [ISO-14977] conventions. EBNF (Extended Backus-Naur Form) is an alternative notation that uses different syntax conventions than ABNF.

#### EBNF Notation Conventions

The following EBNF notation is used throughout this grammar:

| Notation | Meaning | Example |
|----------|---------|---------|
| `=` | Definition | `rule = expression ;` |
| `,` | Concatenation | `a , b` (a followed by b) |
| `\|` | Alternation | `a \| b` (a or b) |
| `[ ]` | Optional | `[ a ]` (zero or one occurrence) |
| `{ }` | Repetition | `{ a }` (zero or more occurrences) |
| `( )` | Grouping | `( a \| b )` |
| `" "` | Terminal string | `"F"` |
| `' '` | Terminal string | `'F'` |
| `(* *)` | Comment | `(* comment *)` |
| `;` | Rule terminator | End of rule definition |

#### Top-Level Rules

```ebnf
(* ========================================================================== *)
(* LNMP v0.4 Text Format - Complete EBNF Grammar                             *)
(* Reference: ISO/IEC 14977 (EBNF Specification)                             *)
(* ========================================================================== *)

(* A complete LNMP record *)
lnmp_record = [ field_list ] ;

(* List of fields separated by semicolons or newlines *)
field_list = field , { separator , field } , [ separator ] ;

(* Field separator *)
separator = ";" | newline ;

(* A single field assignment *)
field = field_prefix , field_id , [ type_hint ] , "=" , value , [ checksum ] ;


(* Field components *)
field_prefix = "F" ;

field_id = digit , { digit } ;
           (* 0-65535, no leading zeros except for "0" *)

type_hint = ":" , type_code ;

type_code = "ra" | "sa" | "i" | "f" | "b" | "s" | "r" ;
            (* ra = record array, sa = string array *)
            (* i = integer, f = float, b = boolean *)
            (* s = string, r = record *)
```

#### Value Type Rules

```ebnf
(* Any value type - ordered by precedence *)
value = nested_record 
      | nested_array 
      | string_array 
      | boolean 
      | number 
      | string ;

(* Nested record: {F1=value;F2=value} *)
nested_record = "{" , [ nested_field_list ] , "}" ;

nested_field_list = nested_field , { ";" , nested_field } , [ ";" ] ;

nested_field = field_prefix , field_id , [ type_hint ] , "=" , 
               value , [ checksum ] ;

(* Nested array of records: [{F1=v},{F2=v}] *)
nested_array = "[" , [ record_list ] , "]" ;

record_list = nested_record , { "," , nested_record } ;

(* String array: [str1,str2,str3] *)
string_array = "[" , [ string_list ] , "]" ;

string_list = string , { "," , string } ;
```


#### Primitive Type Rules

```ebnf
(* String value *)
string = quoted_string | unquoted_string ;

(* Quoted string with escape sequences *)
quoted_string = '"' , { quoted_char } , '"' ;

quoted_char = escape_sequence 
            | unicode_char - ( '"' | "\" ) ;
            (* Any Unicode character except quote and backslash *)

escape_sequence = "\" , ( "\" | '"' | "n" | "r" | "t" ) ;
                  (* \\ = backslash, \" = quote *)
                  (* \n = newline, \r = carriage return, \t = tab *)

(* Unquoted string *)
unquoted_string = ( letter | digit | "_" | "-" | "." ) ,
                  { letter | digit | "_" | "-" | "." } ;

(* Numeric value *)
number = [ "-" ] , digit , { digit } , [ "." , digit , { digit } ] ,
         [ exponent ] ;

exponent = ( "e" | "E" ) , [ ( "+" | "-" ) ] , digit , { digit } ;

(* Boolean value *)
boolean = "0" | "1" ;
```

#### Checksum and Character Class Rules

```ebnf
(* Semantic checksum *)
checksum = "#" , hex_digit , hex_digit , hex_digit , hex_digit ,
           hex_digit , hex_digit , hex_digit , hex_digit ;
           (* Exactly 8 hexadecimal digits *)

(* Character classes *)
letter = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" |
         "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" |
         "U" | "V" | "W" | "X" | "Y" | "Z" |
         "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" |
         "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" |
         "u" | "v" | "w" | "x" | "y" | "z" ;

digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

hex_digit = digit | "A" | "B" | "C" | "D" | "E" | "F" |
            "a" | "b" | "c" | "d" | "e" | "f" ;

unicode_char = ? any valid UTF-8 character ? ;

newline = ? line feed character (U+000A) ? ;
```


#### Complete EBNF Grammar Summary

The complete EBNF grammar for LNMP v0.4 text format is provided below in a consolidated form:

```ebnf
(* ========================================================================== *)
(* LNMP v0.4 Text Format - Complete EBNF Grammar                             *)
(* ========================================================================== *)

(* Top-Level *)
lnmp_record       = [ field_list ] ;
field_list        = field , { separator , field } , [ separator ] ;
separator         = ";" | newline ;
field             = field_prefix , field_id , [ type_hint ] , "=" , 
                    value , [ checksum ] ;

(* Field Components *)
field_prefix      = "F" ;
field_id          = digit , { digit } ;
type_hint         = ":" , type_code ;
type_code         = "ra" | "sa" | "i" | "f" | "b" | "s" | "r" ;

(* Value Types *)
value             = nested_record | nested_array | string_array 
                  | boolean | number | string ;

(* Nested Structures *)
nested_record     = "{" , [ nested_field_list ] , "}" ;
nested_field_list = nested_field , { ";" , nested_field } , [ ";" ] ;
nested_field      = field_prefix , field_id , [ type_hint ] , "=" , 
                    value , [ checksum ] ;
nested_array      = "[" , [ record_list ] , "]" ;
record_list       = nested_record , { "," , nested_record } ;

(* Primitive Types *)
string_array      = "[" , [ string_list ] , "]" ;
string_list       = string , { "," , string } ;
string            = quoted_string | unquoted_string ;
quoted_string     = '"' , { quoted_char } , '"' ;
quoted_char       = escape_sequence | unicode_char - ( '"' | "\" ) ;
escape_sequence   = "\" , ( "\" | '"' | "n" | "r" | "t" ) ;
unquoted_string   = ( letter | digit | "_" | "-" | "." ) ,
                    { letter | digit | "_" | "-" | "." } ;
number            = [ "-" ] , digit , { digit } , 
                    [ "." , digit , { digit } ] , [ exponent ] ;
exponent          = ( "e" | "E" ) , [ ( "+" | "-" ) ] , digit , { digit } ;
boolean           = "0" | "1" ;

(* Checksum *)
checksum          = "#" , hex_digit , hex_digit , hex_digit , hex_digit ,
                    hex_digit , hex_digit , hex_digit , hex_digit ;

(* Character Classes *)
letter            = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" |
                    "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" |
                    "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" |
                    "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" |
                    "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" |
                    "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" ;
digit             = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
hex_digit         = digit | "A" | "B" | "C" | "D" | "E" | "F" |
                    "a" | "b" | "c" | "d" | "e" | "f" ;
unicode_char      = ? any valid UTF-8 character ? ;
newline           = ? line feed character (U+000A) ? ;
```


#### Grammar Notes

**Concatenation vs Alternation**: EBNF uses `,` for concatenation (sequence) and `|` for alternation (choice). For example, `a , b` means "a followed by b", while `a | b` means "a or b".

**Repetition**: EBNF uses `{ }` for zero or more repetitions and `[ ]` for optional (zero or one). For example, `{ digit }` means zero or more digits, while `[ "-" ]` means an optional minus sign.

**Character Classes**: The `letter` and `digit` rules explicitly enumerate all valid characters. Implementations MAY use character ranges (e.g., `[A-Za-z]`) for efficiency.

**Unicode Characters**: The `unicode_char` rule represents any valid UTF-8 character. Implementations MUST validate UTF-8 encoding and reject invalid sequences.

**Whitespace**: EBNF does not have implicit whitespace handling. Implementations SHOULD skip whitespace (spaces and tabs) between tokens during lexical analysis.

**Comments**: Comments in EBNF are enclosed in `(* *)`. In LNMP syntax, comments start with `#` and continue to end of line, but checksums (8 hex digits after `#`) take precedence.

**Ambiguity Resolution**: The `value` rule lists alternatives in precedence order. Implementations MUST attempt to match alternatives in the order specified to ensure deterministic parsing.

**Field ID Range**: The `field_id` rule allows any sequence of digits, but implementations MUST validate that the numeric value is in the range 0-65535.

**Type Code Ordering**: The `type_code` rule lists longer codes (`ra`, `sa`) before shorter codes to ensure correct parsing in implementations that use longest-match semantics.

This completes the EBNF Grammar subsection.


### 6.3 Parsing Precedence

This section defines the precedence rules for parsing LNMP values, ensuring unambiguous interpretation across all implementations. The precedence order eliminates ambiguity when multiple value types could potentially match the same input.

#### Value Type Precedence Order

When parsing a value, implementations MUST attempt to match value types in the following order (highest to lowest precedence):

1. **Nested Record** (`{...}`)
2. **Nested Array** (`[{...}]`)
3. **String Array** (`[...]`)
4. **Boolean** (`0` or `1`)
5. **Number** (`-?[0-9]+(\.[0-9]+)?`)
6. **String** (quoted or unquoted)

This ordering ensures that:
- Structured types (nested records and arrays) are recognized before primitive types
- Boolean values are distinguished from integer values when appropriate
- Numeric values are distinguished from string values
- Ambiguous inputs are resolved deterministically

#### Precedence Rationale

**1. Nested Record (Highest Precedence)**

Nested records start with `{` and end with `}`. This syntax is unambiguous and cannot be confused with any other value type.

**Example:**
```
F50={F12=1;F7=1}
```

The opening brace `{` immediately identifies this as a nested record. No other value type uses braces.

**Why highest precedence?** Braces are unique to nested records, so they must be checked first to avoid misinterpretation.


**2. Nested Array**

Nested arrays start with `[` followed by `{`, distinguishing them from string arrays which start with `[` followed by a string.

**Example:**
```
F60=[{F12=1},{F12=2}]
```

The sequence `[{` unambiguously identifies a nested array. The opening brace after the bracket distinguishes this from a string array.

**Why second precedence?** Must be checked before string arrays to correctly identify arrays of records vs arrays of strings.

**Disambiguation Example:**
```
F60=[{F12=1}]      → Nested array (contains record)
F60=[admin,dev]    → String array (contains strings)
```

**3. String Array**

String arrays start with `[` and contain comma-separated strings. They are distinguished from nested arrays by the absence of braces after the opening bracket.

**Example:**
```
F23=["admin","dev","guest"]
F24=[alice,bob,charlie]
F25=[]
```

**Why third precedence?** Must be checked after nested arrays but before primitive types to correctly identify array structures.

**Disambiguation Example:**
```
[{F1=a}]           → Nested array (starts with [{)
[a,b,c]            → String array (starts with [ but not [{)
```


**4. Boolean**

Boolean values are `0` or `1`. They must be checked before numbers to ensure that `0` and `1` are interpreted as booleans when appropriate.

**Example:**
```
F7=1               → Could be boolean true or integer 1
F7:b=1             → Explicitly boolean true
F7:i=1             → Explicitly integer 1
```

**Why fourth precedence?** Without type hints, `0` and `1` are ambiguous. By checking boolean before number, implementations can provide consistent default behavior.

**Disambiguation Strategy:**

Without type hints:
- `0` and `1` are treated as booleans by default (when boolean is checked first)
- Other integers are treated as numbers

With type hints:
- `:b` forces boolean interpretation (only `0` or `1` valid)
- `:i` forces integer interpretation
- `:s` forces string interpretation (e.g., `"1"` as string)

**Note:** Some implementations MAY choose to treat `0` and `1` as integers by default. The grammar allows both interpretations, but type hints provide explicit disambiguation.

**5. Number**

Numeric values include integers and floats. They are distinguished by the presence or absence of a decimal point.

**Examples:**
```
F12=14532          → Integer
F20=3.14           → Float
F21=-42            → Negative integer
F22=1.5e10         → Float with scientific notation
```

**Why fifth precedence?** Must be checked after booleans to handle `0` and `1` correctly, but before strings to ensure numeric values are not misinterpreted as strings.

**Disambiguation Example:**
```
42                 → Number (no quotes, no decimal → integer)
3.14               → Number (has decimal → float)
"42"               → String (has quotes)
```


**6. String (Lowest Precedence)**

String values can be quoted or unquoted. They are the fallback type when no other type matches.

**Examples:**
```
F1=alice           → Unquoted string
F2="hello world"   → Quoted string
F3="123"           → Quoted string (not a number)
F4=user_123        → Unquoted string
```

**Why lowest precedence?** Strings are the most flexible type and can represent almost any value. By checking strings last, we ensure that more specific types (numbers, booleans, arrays) are recognized first.

**Disambiguation Example:**
```
alice              → String (unquoted, matches [A-Za-z0-9_.-]+)
"alice"            → String (quoted)
123                → Number (not string, no quotes)
"123"              → String (quoted, forces string interpretation)
```

#### Precedence in Practice

**Example 1: Distinguishing Arrays**

Input: `[{F1=a}]`

Parsing steps:
1. Check nested record? No (doesn't start with `{`)
2. Check nested array? Yes (starts with `[{`) → **Match: Nested Array**

Input: `[a,b,c]`

Parsing steps:
1. Check nested record? No (doesn't start with `{`)
2. Check nested array? No (starts with `[` but not `[{`)
3. Check string array? Yes (starts with `[` and contains strings) → **Match: String Array**

**Example 2: Distinguishing Numbers and Booleans**

Input: `0`

Parsing steps (without type hint):
1. Check nested record? No
2. Check nested array? No
3. Check string array? No
4. Check boolean? Yes (`0` matches boolean pattern) → **Match: Boolean**

Input: `0` with type hint `:i`

Type hint forces integer interpretation → **Match: Integer**

Input: `42`

Parsing steps:
1. Check nested record? No
2. Check nested array? No
3. Check string array? No
4. Check boolean? No (`42` is not `0` or `1`)
5. Check number? Yes → **Match: Number (Integer)**


**Example 3: Distinguishing Numbers and Strings**

Input: `123`

Parsing steps:
1. Check nested record? No
2. Check nested array? No
3. Check string array? No
4. Check boolean? No
5. Check number? Yes → **Match: Number (Integer)**

Input: `"123"`

Parsing steps:
1. Check nested record? No
2. Check nested array? No
3. Check string array? No
4. Check boolean? No
5. Check number? No (has quotes)
6. Check string? Yes → **Match: String**

**Example 4: Complex Nested Structure**

Input: `{F1=alice;F2=[{F10=admin},{F10=user}]}`

Parsing steps:
1. Check nested record? Yes (starts with `{`) → **Match: Nested Record**
   - Parse nested fields:
     - F1: Check types → String "alice"
     - F2: Check types → Nested Array `[{F10=admin},{F10=user}]`

#### Type Hint Override

Type hints override the default precedence by explicitly specifying the expected type. When a type hint is present, implementations MUST validate that the value matches the specified type.

**Examples:**

```
F7:b=1             → Type hint forces boolean interpretation
F7:i=1             → Type hint forces integer interpretation
F1:s=123           → Error: 123 is a number, not a string
F1:s="123"         → Valid: "123" is a string
F12:i=hello        → Error: hello is a string, not an integer
```

Type hints provide explicit disambiguation when the default precedence is not desired or when validation is required.


#### Implementation Guidelines

**Recursive Descent Parsing**

The precedence order naturally maps to a recursive descent parser structure:

```python
def parse_value():
    # Try each type in precedence order
    if peek() == '{':
        return parse_nested_record()
    elif peek() == '[' and peek_ahead() == '{':
        return parse_nested_array()
    elif peek() == '[':
        return parse_string_array()
    elif current_token in ['0', '1'] and not has_decimal():
        return parse_boolean()
    elif is_numeric(current_token):
        return parse_number()
    else:
        return parse_string()
```

**Lookahead Requirements**

Most value types can be identified with single-character lookahead:
- `{` → Nested record
- `[` → Array (requires one more lookahead to distinguish nested vs string)
- `0` or `1` → Boolean (unless followed by more digits or decimal point)
- `-` or digit → Number (unless quoted)
- `"` → Quoted string
- Letter, `_`, `-`, `.` → Unquoted string

**Error Handling**

If no value type matches, implementations MUST report a syntax error:

```
Error: Expected value at line 2, column 8
  |
2 | F12=@invalid
  |     ^^^^^^^^ Invalid value syntax
  |
Hint: Values must be nested records, arrays, booleans, numbers, or strings
```

#### Precedence Summary Table

| Precedence | Type | Start Pattern | Distinguishing Feature |
|------------|------|---------------|------------------------|
| 1 (Highest) | Nested Record | `{` | Opening brace |
| 2 | Nested Array | `[{` | Bracket followed by brace |
| 3 | String Array | `[` (not `[{`) | Bracket not followed by brace |
| 4 | Boolean | `0` or `1` | Single digit 0 or 1 |
| 5 | Number | `-?[0-9]` | Digits with optional minus and decimal |
| 6 (Lowest) | String | `"` or `[A-Za-z0-9_.-]` | Quoted or unquoted identifier |

This completes the Parsing Precedence subsection and the entire Formal Grammar section.

---
