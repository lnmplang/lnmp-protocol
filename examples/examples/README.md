# LNMP Examples

This directory contains examples demonstrating the features of the LNMP (LLM Native Minimal Protocol) implementation.

## Overview

LNMP is a data serialization format optimized for LLM (Large Language Model) contexts. It provides:
- Minimal token usage compared to JSON
- Type safety with optional type hints
- Deterministic serialization for consistency
- Semantic checksums to prevent LLM input drift
- Support for nested structures
- LLM-optimized encoding strategies

## Running Examples

To run any example:

```bash
cargo run --example <example_name>
```

If you're using `lnmp-examples` as a separate repository (recommended), we
depend on the `lnmp` repository as a git dependency by default. That means
the examples will use the crates from `lnmp` as provided from the `main`
branch. If you want to use a specific release of the protocol repo, edit
`examples/Cargo.toml` to set `tag = "vX.Y.Z"`.

For example:

```bash
cargo run --example semantic_checksums
cargo run --example nested_structures
cargo run --example explain_mode
```

### Local development with a sibling `lnmp` checkout

If you clone `lnmp` next to `lnmp-examples` on your machine (i.e.
they are sibling directories under the same root), you can use local path
overrides so that examples use the local crates while you iterate.

Create (or use the provided) `.cargo/config.toml` with a `patch.crates-io`
section. An example file is present in the repository and uses the following
paths relative to this repo root:

```
[patch.crates-io]
lnmp-core = { path = "../lnmp/crates/lnmp-core" }
lnmp-codec = { path = "../lnmp/crates/lnmp-codec" }
lnmp-llb = { path = "../lnmp/crates/lnmp-llb" }
```

This keeps `lnmp-examples` a clean separate repo while still allowing fast
iteration against a local `lnmp` checkout.

## Examples by Feature

### Core Features (v0.2)

#### `type_hints.rs`
Demonstrates the use of type hints in LNMP encoding and parsing.

**Features shown:**
- Encoding with and without type hints
- Type hint validation during parsing
- All supported type hints: `:i` (int), `:f` (float), `:b` (bool), `:s` (string), `:sa` (string array)

**Run:** `cargo run --example type_hints`

#### `strict_vs_loose.rs`
Shows the difference between strict (canonical) and loose parsing modes.

**Features shown:**
- Canonical format requirements
- Loose mode flexibility
- Field ordering and whitespace handling

**Run:** `cargo run --example strict_vs_loose`

#### `deterministic_serialization.rs`
Demonstrates how LNMP ensures deterministic output for consistent serialization.

**Features shown:**
- Automatic field sorting by Field ID
- Round-trip stability
- Canonical format normalization

**Run:** `cargo run --example deterministic_serialization`

#### `structural_canonicalization.rs`
Shows structural canonicalization rules for deterministic encoding.

**Features shown:**
- Field ordering rules
- Whitespace normalization
- Canonical format guarantees

**Run:** `cargo run --example structural_canonicalization`

### v0.3 Features - Semantic Fidelity Engine (SFE)

#### `semantic_checksums.rs`
Demonstrates semantic checksums (SC32) for preventing LLM input drift.

**Features shown:**
- Computing SC32 checksums for field values
- Checksum validation
- Checksum formatting (8-character hex)
- Semantic equivalence (e.g., -0.0 and 0.0 produce same checksum)
- Checksums for all value types

**Run:** `cargo run --example semantic_checksums`

**Key concepts:**
- SC32 = 32-bit semantic checksum
- Computed from: Field ID + Type Hint + Normalized Value
- Prevents LLM hallucination and input drift
- Format: `F12:i=14532#36AAE667`

### v0.3 Features - Structural Extensibility Layer (SEL)

#### `nested_structures.rs`
Shows how to encode and parse nested records and arrays.

**Features shown:**
- Simple nested records: `F50={F12=1;F7=1}`
- Nested arrays of records: `F60=[{F12=1},{F12=2}]`
- Complex multi-level nesting
- Structural canonicalization at all nesting levels
- Nested structures with checksums

**Run:** `cargo run --example nested_structures`

**Key concepts:**
- Nested records use `{...}` syntax
- Nested arrays use `[{...},{...}]` syntax
- Fields sorted by FID at every nesting level
- Arbitrary nesting depth supported

### v0.3 Features - LNMP-LLM Bridge Layer (LLB)

#### `explain_mode.rs`
Demonstrates explain mode encoding with human-readable annotations.

**Features shown:**
- Adding inline comments with field names
- Using semantic dictionaries for field name lookups
- Comment alignment for readability
- Explain mode with and without type hints
- Nested structures with explain mode

**Run:** `cargo run --example explain_mode`

**Key concepts:**
- Format: `F12:i=14532  # user_id`
- Comments aligned at consistent column
- Parser ignores explain mode comments
- Useful for debugging and human inspection

**Example output:**
```
F7:b=1              # is_active
F12:i=14532         # user_id
F23:sa=[admin,dev]  # roles
```

#### `shortform.rs`
Shows LNMP-ShortForm encoding for extreme token reduction.

**Features shown:**
- ShortForm syntax (omits 'F' prefix)
- Token reduction metrics vs JSON and standard LNMP
- ShortForm with nested structures
- When to use (and not use) ShortForm

**Run:** `cargo run --example shortform`

**Key concepts:**
- ShortForm: `12=14532 7=1 23=[admin,dev]`
- 7-12× token reduction vs JSON
- ~15% reduction vs standard LNMP
- **WARNING:** ShortForm is UNSAFE - use only for LLM input optimization
- Never use for storage, APIs, or canonical format requirements

**Example comparison:**
```
JSON:          {"user_id":14532,"is_active":true,"roles":["admin","dev"]}
Standard LNMP: F12:i=14532;F7:b=1;F23:sa=[admin,dev]
ShortForm:     12=14532 7=1 23=[admin,dev]
```

### v0.4 Features - Binary Protocol

#### `binary_encoding.rs`
Demonstrates binary encoding and decoding for efficient transport.

**Features shown:**
- Binary encoding of LNMP records
- Binary decoding back to records
- Type tag system for binary values
- VarInt encoding for compact integers

**Run:** `cargo run --example binary_encoding`

#### `binary_roundtrip.rs`
Shows round-trip conversion between text and binary formats.

**Features shown:**
- Text → Binary → Text conversion
- Binary → Text → Binary conversion
- Canonical form preservation
- Size comparison between formats

**Run:** `cargo run --example binary_roundtrip`

### v0.5 Features - Advanced Protocol

#### `v05_nested_binary.rs`
Demonstrates binary encoding of nested structures (v0.5).

**Features shown:**
- Binary nested records (TypeTag 0x06)
- Binary nested arrays (TypeTag 0x07)
- Multi-level nesting with depth validation
- Canonical ordering at all nesting levels
- Size and depth limit enforcement

**Run:** `cargo run --example v05_nested_binary`

**Key concepts:**
- Nested structures in binary format
- Depth limits (default 32 levels)
- Size limits for security
- Automatic canonical ordering

#### `v05_streaming.rs`
Shows the Streaming Frame Layer (SFL) for large payloads.

**Features shown:**
- Frame types: BEGIN, CHUNK, END, ERROR
- Chunked transmission with configurable chunk size
- XOR checksum validation
- Backpressure flow control
- Error recovery and incomplete stream detection

**Run:** `cargo run --example v05_streaming`

**Key concepts:**
- Default chunk size: 4KB
- Frame-based transmission
- Integrity checking with checksums
- Flow control for large data

#### `v05_schema_negotiation.rs`
Demonstrates the Schema Negotiation Layer (SNL).

**Features shown:**
- Client-server capability exchange
- Feature flag negotiation
- FID conflict detection
- Type mismatch detection
- Protocol version negotiation

**Run:** `cargo run --example v05_schema_negotiation`

**Key concepts:**
- Capability negotiation before data exchange
- Feature intersection (agreed features)
- Schema compatibility validation
- Graceful degradation

#### `v05_delta_encoding.rs`
Shows the Delta Encoding & Partial Update Layer (DPL).

**Features shown:**
- Computing deltas between records
- Delta operations: SET_FIELD, DELETE_FIELD, UPDATE_FIELD, MERGE_RECORD
- Bandwidth savings measurement
- Incremental updates for nested structures

**Run:** `cargo run --example v05_delta_encoding`

**Key concepts:**
- 50%+ bandwidth savings for typical updates
- Only changed fields transmitted
- Nested record merging
- Incremental update chains

#### `v05_llb2_binary.rs`
Demonstrates LLB2 integration with binary format.

**Features shown:**
- Binary ↔ ShortForm conversion
- Binary ↔ FullText conversion
- Flattening nested binary structures
- Semantic hints in binary context
- Collision-safe ID generation

**Run:** `cargo run --example v05_llb2_binary`

**Key concepts:**
- Optimal LLM token usage
- Binary format optimization
- Nested structure flattening
- Semantic hint embedding

## Feature Matrix

| Feature | Version | Examples |
|---------|---------|----------|
| Type Hints | v0.2 | `type_hints.rs` |
| Deterministic Serialization | v0.2 | `deterministic_serialization.rs`, `structural_canonicalization.rs` |
| Strict vs Loose Parsing | v0.2 | `strict_vs_loose.rs` |
| Semantic Checksums (SC32) | v0.3 | `semantic_checksums.rs` |
| Nested Records | v0.3 | `nested_structures.rs` |
| Nested Arrays | v0.3 | `nested_structures.rs` |
| Explain Mode | v0.3 | `explain_mode.rs` |
| ShortForm Encoding | v0.3 | `shortform.rs` |
| Binary Protocol | v0.4 | `binary_encoding.rs`, `binary_roundtrip.rs` |
| Binary Nested Structures | v0.5 | `v05_nested_binary.rs` |
| Streaming Frame Layer | v0.5 | `v05_streaming.rs` |
| Schema Negotiation | v0.5 | `v05_schema_negotiation.rs` |
| Delta Encoding | v0.5 | `v05_delta_encoding.rs` |
| LLB2 Binary Integration | v0.5 | `v05_llb2_binary.rs` |

## LNMP v0.3 Architecture

LNMP v0.3 introduces the "protocol intelligence layer" with four major subsystems:

### 1. Semantic Fidelity Engine (SFE)
- **Semantic Checksums (SC32):** 32-bit checksums to prevent LLM input drift
- **Value Normalization:** Canonical transformations for semantic equivalence
- **Equivalence Mapping:** Synonym and related term recognition

### 2. Structural Extensibility Layer (SEL)
- **Nested Records:** Hierarchical data structures
- **Nested Arrays:** Collections of structured data
- **Structural Canonicalization:** Deterministic encoding at all nesting levels

### 3. Zero-Ambiguity Grammar (ZAG)
- **Formal PEG Grammar:** Ensures deterministic parsing across implementations
- **Error Classification:** Formal error classes for all parsing failures
- **Multi-language Compliance:** Test suite for cross-language consistency

### 4. LNMP-LLM Bridge Layer (LLB)
- **Prompt Visibility Optimization:** Maximize LLM tokenization efficiency
- **Explain Mode:** Human-readable annotations for debugging
- **ShortForm Encoding:** Extreme token reduction for LLM contexts

## Token Efficiency

LNMP is designed to minimize token usage in LLM contexts:

| Format | Example | Tokens (approx) |
|--------|---------|-----------------|
| JSON | `{"user_id":14532,"is_active":true,"roles":["admin","dev"]}` | ~15-20 |
| Standard LNMP | `F12=14532\nF7=1\nF23=[admin,dev]` | ~8-10 |
| LNMP ShortForm | `12=14532 7=1 23=[admin,dev]` | ~7-9 |

**Token reduction:** 7-12× compared to JSON

## Best Practices

### When to Use Type Hints
- ✓ Use when type safety is important
- ✓ Use with checksums for maximum fidelity
- ✗ Skip for minimal token usage (if types are known)

### When to Use Checksums
- ✓ Use for LLM input to prevent drift
- ✓ Use for critical data integrity
- ✗ Skip for human-readable output
- ✗ Skip when token count is critical

### When to Use Explain Mode
- ✓ Use for debugging and development
- ✓ Use for human inspection of data
- ✗ Don't use in production (adds tokens)
- ✗ Don't use for LLM input (unnecessary)

### When to Use ShortForm
- ✓ Use for LLM prompt input (token optimization)
- ✓ Use for temporary data in LLM contexts
- ✗ **NEVER** use for storage or persistence
- ✗ **NEVER** use for APIs or interoperability
- ✗ **NEVER** use when checksums are needed

## Additional Resources

- **Specification:** See `spec/` directory for formal grammar and error classes
- **Compliance Tests:** See `tests/compliance/` for multi-language test suite
- **Design Document:** See `.kiro/specs/lnmp-v0.3-semantic-fidelity/design.md`
- **Requirements:** See `.kiro/specs/lnmp-v0.3-semantic-fidelity/requirements.md`

## Contributing

When adding new examples:
1. Follow the existing example structure
2. Include clear comments explaining each feature
3. Add the example to `examples/Cargo.toml`
4. Update this README with the new example
5. Ensure the example compiles and runs successfully

## License

See the root LICENSE file for license information.
