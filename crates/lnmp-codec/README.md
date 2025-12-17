# lnmp-codec

Parser and encoder implementations for LNMP (LLM Native Minimal Protocol) v0.5 text format and binary format.

> **FID Registry:** All examples in this documentation use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

> Maintainers note: when shipping codec changes, remember to run the workspace Release workflow (see `CONTRIBUTING.md`) so the meta crate picks up the new version.

## Features

### Text Format (v0.3)
- **Deterministic serialization**: Fields always sorted by FID for consistent output
- **Canonical format**: Newline-separated, no extra whitespace (v0.2)
- **Type hints**: Optional type annotations (`:i`, `:f`, `:b`, `:s`, `:sa`, `:r`, `:ra`)
- **Generic Array Support**: `IntArray`, `FloatArray`, `BoolArray` handling in parsing/encoding
- **Strict Profile Integration**: `LnmpProfile` (Loose, Standard, Strict) for validation and canonical enforcement
- **Nested structures**: Parse and encode nested records and arrays (v0.3)
- **Semantic checksums**: Optional SC32 checksums for drift prevention (v0.3)
- **Value normalization**: Canonical value transformations (v0.3)
- **Equivalence mapping**: Synonym recognition (v0.3)
- **Semantic dictionary (optional)**: Apply `lnmp-sfe` dictionaries during parse/encode to map values to canonical equivalents
- **Strict mode**: Validates canonical format compliance
- **Loose mode**: Accepts format variations (default)
- **Lenient sanitizer**: Optional pre-parse repair layer shared with `lnmp-sanitize` for LLM-facing inputs
- **Round-trip stability**: `parse(encode(parse(x))) == parse(encode(x))`

### Binary Format (v0.4)
- **Efficient encoding**: 30-50% size reduction compared to text format
- **Zero-copy design**: Fast serialization and deserialization
- **Bidirectional conversion**: Seamless text ↔ binary conversion
- **Canonical binary**: Fields sorted by FID, deterministic encoding
- **VarInt encoding**: Space-efficient integer representation
- **Type safety**: Explicit type tags for all values
- **Version validation**: Protocol version checking
- **Interoperability**: Compatible with v0.3 text format for supported types

## Quick Start

### Text Format

```rust
use lnmp_codec::{Parser, Encoder};

// Parse LNMP text
let input = "F12=14532\nF7=1\nF23=[admin,dev]";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Encode to canonical format
let encoder = Encoder::new();
let output = encoder.encode(&record);
// Output: F7=1\nF12=14532\nF23=[admin,dev]  (sorted by FID)
```

### Semantic Dictionary Normalization (optional)

```rust
use lnmp_codec::{Parser, Encoder};
use lnmp_sfe::SemanticDictionary;

// Build a dictionary: map Admin/ADMIN -> admin for field 23
let mut dict = SemanticDictionary::new();
dict.add_equivalence(23, "Admin".to_string(), "admin".to_string());

// Parse with dictionary (applies equivalence during parse)
let mut parser = Parser::with_config(
    "F23=[Admin]",
    lnmp_codec::config::ParserConfig {
        semantic_dictionary: Some(dict.clone()),
        ..Default::default()
    },
)
.unwrap();
let record = parser.parse_record().unwrap();

// Encode with the same dictionary (ensures canonical output)
let encoder = Encoder::with_config(
    lnmp_codec::config::EncoderConfig::new().with_semantic_dictionary(dict),
);
let output = encoder.encode(&record);
assert_eq!(output, "F23=[admin]");
```

### Binary Format (v0.4)

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

// Encode text to binary
let text = "F7=1\nF12=14532\nF23=[admin,dev]";
let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(text).unwrap();

// Decode binary to text
let decoder = BinaryDecoder::new();
let decoded_text = decoder.decode_to_text(&binary).unwrap();
// Output: F7=1\nF12=14532\nF23=[admin,dev]  (canonical format)

// Round-trip conversion maintains data integrity
assert_eq!(text, decoded_text);
```

### v0.3 Quick Start - Nested Structures

```rust
use lnmp_codec::{Parser, Encoder};

// Parse nested record (F70=nested_data from registry)
let input = "F70={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Parse record array (F71=record_list from registry)
let input = "F71=[{F20=alice},{F20=bob}]";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Encode with checksums
use lnmp_codec::EncoderConfig;
let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
// Output: F12=14532#36AAE667  (with checksum)
```

### Lenient LLM-Friendly Parsing

```rust
use lnmp_codec::{Parser, TextInputMode, ParsingMode};
use lnmp_codec::binary::BinaryEncoder;

let messy = r#"F20=hello "world"; F40 = 3.14;F30=00042"#;

// Parser profile geared for LLM output
let mut parser = Parser::with_config(
    messy,
    lnmp_codec::config::ParserConfig {
        text_input_mode: TextInputMode::Lenient,
        mode: ParsingMode::Loose,
        normalize_values: true,
        ..Default::default()
    },
).unwrap();
let record = parser.parse_record().unwrap();

// Binary encoder also provides lenient/strict helpers
let encoder = BinaryEncoder::new();
let bytes = encoder.encode_text_llm_profile(messy).unwrap();

// For M2M strict flows use `Parser::new_strict` or `encode_text_strict_profile`.
```

## LNMP v0.2 Features

### Deterministic Serialization

Fields are always sorted by FID, ensuring consistent output:

```rust
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 30, value: LnmpValue::Int(3) });  // F30=count
record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });   // F7=is_active
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) });  // F12=user_id

let encoder = Encoder::new();
let output = encoder.encode(&record);
// Output: F7=1\nF12=2\nF30=3  (sorted by FID)
```

- **Arrays**: `[...]`
  > **Note**: Text parsing now honors typed hints—`:ia`, `:fa`, and `:ba` force integer, float, and boolean arrays respectively. Without a hint, the parser treats the array as `StringArray`, mirroring the binary encoder/decoder semantics.

### Type Hints

Optional type annotations for explicit typing:

```rust
use lnmp_codec::{Encoder, EncoderConfig};

let config = EncoderConfig {
    include_type_hints: true,
    canonical: true,
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
// Output: F12:i=14532\nF5:f=3.14\nF7:b=1
```

### Strict vs Loose Parsing

```rust
use lnmp_codec::{Parser, ParsingMode};

// Loose mode (default): accepts format variations
let mut parser = Parser::new("F20=test;F7=1").unwrap();  // Unsorted, semicolons OK

// Strict mode: requires canonical format  
let mut parser = Parser::with_mode("F7=1\nF20=test", ParsingMode::Strict).unwrap();

// Strict input mode (no sanitizer)
let mut strict_input_parser = Parser::new_strict("F7=1\nF20=test").unwrap();
```

## v0.3 Features

### Nested Structures

Parse and encode hierarchical data:

```rust
use lnmp_codec::{Parser, Encoder};

// Nested record: F70={F12=1;F7=1} (F70=nested_data)
let input = "F70={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Record array: F71=[{F20=alice},{F20=bob}] (F71=record_list, F20=name)
let input = "F71=[{F20=alice},{F20=bob}]";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Deep nesting (F70=nested_data)
let input = "F70={F20=user;F70={F30=nested;F31=data}}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();
```

### Compliance & Lenient Test Suite

- `tests/compliance/rust` contains the cross-language suite for strict flows.
- `tests/compliance/rust/test-cases-lenient.yaml` mirrors the shared sanitizer behavior (auto-quote, comment trimming, nested repairs).
- Run `cargo test -p lnmp-codec --tests test-driver -- --nocapture` to execute both strict and lenient suites.

The lenient path uses the `lnmp-sanitize` crate under the hood so SDKs (Rust/TS/Go/Python) can apply identical repair logic before calling strict parsers.

### Recommended SDK Profiles

| Profile | Parser Config | Binary Encoder | Intended Use |
|---------|---------------|----------------|--------------|
| **LLM-facing** | `text_input_mode = Lenient`, `mode = ParsingMode::Loose`, `normalize_values = true` | `encode_text_llm_profile` | Repair user/LLM text before strict parsing |
| **M2M strict** | `Parser::new_strict()` or `ParserConfig { text_input_mode = Strict, mode = ParsingMode::Strict }` | `encode_text_strict_profile` | Deterministic machine-to-machine pipelines |

- Rust exposes helpers (`Parser::new_lenient`, `Parser::new_strict`, binary profile methods).
- TypeScript/Go/Python SDKs mirror the same defaults: `LLMProfile` (Lenient+Loose) for agent/model traffic and `M2MProfile` (Strict+Strict) for canonical pipelines.
- All SDKs rely on the same sanitizer rules from `lnmp-sanitize`, ensuring identical repairs across languages.

**Nested Structure Rules:**
- Nested records use `{...}` syntax with semicolon separators
- Nested arrays use `[{...},{...}]` syntax
- Fields sorted by FID at every nesting level
- Arbitrary nesting depth supported

### Semantic Checksums (SC32)

Enable checksums for drift prevention:

```rust
use lnmp_codec::{Encoder, EncoderConfig};

let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
// Output: F12:i=14532#36AAE667

// Parse and validate checksums
use lnmp_codec::{Parser, ParserConfig};
let config = ParserConfig {
    validate_checksums: true,
    ..Default::default()
};
let mut parser = Parser::with_config(input, config).unwrap();
let record = parser.parse_record().unwrap();  // Validates checksums
```

### Value Normalization

Canonical value transformations:

```rust
use lnmp_codec::{ValueNormalizer, NormalizationConfig};

let config = NormalizationConfig {
    string_case: StringCaseRule::Lower,
    remove_trailing_zeros: true,
    ..Default::default()
};
let normalizer = ValueNormalizer::new(config);

// Normalizes: true → 1, -0.0 → 0.0, 3.140 → 3.14
let normalized = normalizer.normalize(&value);
```

### Equivalence Mapping

Synonym recognition:

```rust
use lnmp_codec::EquivalenceMapper;

let mut mapper = EquivalenceMapper::new();
mapper.add_mapping(7, "yes".to_string(), "1".to_string());
mapper.add_mapping(7, "true".to_string(), "1".to_string());

// Maps "yes" → "1" for field 7
let canonical = mapper.map(7, "yes");  // Some("1")
```

## Canonical Format Rules

v0.3 canonical format:
- ✓ Fields sorted by FID at all nesting levels
- ✓ Newline-separated (no semicolons at top level)
- ✓ Semicolons required in nested records
- ✓ No whitespace around equals signs
- ✓ No spaces after commas in arrays
- ✓ No comments (except in explain mode)
- ✓ Checksums appended as `#XXXXXXXX` when enabled

## Configuration Options

### EncoderConfig

```rust
pub struct EncoderConfig {
    pub canonical: bool,              // Use canonical format
    pub include_type_hints: bool,     // Add type hints
    pub enable_checksums: bool,       // Append SC32 checksums (v0.3)
    pub normalization_config: Option<NormalizationConfig>,  // Value normalization (v0.3)
    pub equivalence_mapper: Option<EquivalenceMapper>,      // Synonym mapping (v0.3)
}
```

### ParserConfig

```rust
pub struct ParserConfig {
    pub mode: ParsingMode,            // Strict or Loose
    pub validate_checksums: bool,     // Validate SC32 checksums (v0.3)
    pub equivalence_mapper: Option<EquivalenceMapper>,  // Synonym mapping (v0.3)
}
```

### NormalizationConfig

```rust
pub struct NormalizationConfig {
    pub string_case: StringCaseRule,      // Lower, Upper, None
    pub float_precision: Option<usize>,   // Decimal places
    pub remove_trailing_zeros: bool,      // Remove trailing zeros
}
```

## Migration from v0.2

v0.3 is backward compatible with v0.2. New features:

| Feature | v0.2 | v0.3 |
|---------|------|------|
| Nested structures | Not supported | Supported |
| Checksums | Not supported | Optional SC32 |
| Value normalization | Not supported | Configurable |
| Equivalence mapping | Not supported | Configurable |
| Type hints | `:i`, `:f`, `:b`, `:s`, `:sa` | + `:r`, `:ra` |

### Migration Guide

1. **Parsing**: No changes needed - v0.3 parser accepts v0.2 format
2. **Encoding**: New optional features (checksums, normalization)
3. **Nested structures**: Use new `NestedRecord` and `NestedArray` variants
4. **Tests**: Update for new value types if using nested structures

```rust
// v0.2 code (still works)
let encoder = Encoder::new();

// v0.3 code with new features
let config = EncoderConfig {
    enable_checksums: true,
    normalization_config: Some(NormalizationConfig::default()),
    ..Default::default()
};
let encoder = Encoder::with_config(config);
```

## Performance Notes

- **Sorting overhead**: Minimal - uses stable sort on encode
- **Memory**: Sorted fields are cloned, original record unchanged
- **Parsing**: Loose mode has same performance as v0.1

## Binary Format Details (v0.4)

### Binary Frame Structure

```
┌─────────┬─────────┬─────────────┬──────────────────────┐
│ VERSION │  FLAGS  │ ENTRY_COUNT │      ENTRIES...      │
│ (1 byte)│(1 byte) │  (VarInt)   │     (variable)       │
└─────────┴─────────┴─────────────┴──────────────────────┘
```

Each entry contains:
```
┌──────────┬──────────┬──────────────────┐
│   FID    │  THTAG   │      VALUE       │
│ (2 bytes)│ (1 byte) │   (variable)     │
└──────────┴──────────┴──────────────────┘
```

### Supported Types

- **Integer** (0x01): VarInt encoded signed 64-bit integers
- **Float** (0x02): IEEE 754 double-precision (8 bytes, little-endian)
- **Boolean** (0x03): Single byte (0x00 = false, 0x01 = true)
- **String** (0x04): Length-prefixed UTF-8 (length as VarInt + bytes)
- **String Array** (0x05): Count-prefixed array of length-prefixed strings

### Binary Encoding Example

```rust
use lnmp_codec::binary::BinaryEncoder;
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 7,
    value: LnmpValue::Bool(true),
});
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

let encoder = BinaryEncoder::new();
let binary = encoder.encode(&record).unwrap();
// Binary format: [0x04, 0x00, 0x02, ...] (version, flags, entry count, entries)
```

### Configuration Options

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder, EncoderConfig, DecoderConfig};

// Encoder configuration
let encoder_config = EncoderConfig::new()
    .with_validate_canonical(true)
    .with_sort_fields(true);
let encoder = BinaryEncoder::with_config(encoder_config);

// Decoder configuration
let decoder_config = DecoderConfig::new()
    .with_validate_ordering(true)  // Enforce canonical field order
    .with_strict_parsing(true);    // Detect trailing data
let decoder = BinaryDecoder::with_config(decoder_config);
```

### Performance Characteristics

- **Space Efficiency**: 30-50% size reduction compared to text format
- **Encoding Speed**: < 1μs per field for simple types
- **Decoding Speed**: < 1μs per field for simple types
- **Round-trip**: < 10μs for typical 10-field record

## v0.5.14 Features

### Dynamic FID Discovery Protocol

Query and synchronize FID registries between peers at runtime:

```rust
use lnmp_codec::binary::{SchemaNegotiator, FidDefinition, FidDefStatus, TypeTag};

// Create registry-aware negotiator
let mut negotiator = SchemaNegotiator::v0_5()
    .with_registry_version("1.0.0".into());

// Request peer's FID registry
let request = negotiator.request_registry(None);

// Handle registry response
let fids = vec![
    FidDefinition {
        fid: 12,
        name: "user_id".into(),
        type_tag: TypeTag::Int,
        status: FidDefStatus::Active,
        since: "0.1.0".into(),
    },
];
negotiator.handle_registry_response("1.0.0".into(), fids);

// Check FID support
if negotiator.peer_supports_fid(12) {
    println!("Peer understands user_id field");
}
```

**New Message Types:**
- `RequestRegistry` - Query peer FID definitions
- `RegistryResponse` - Full registry response
- `RegistryDelta` - Incremental sync

### Encoder FID Validation

Validate fields before encoding:

```rust
use lnmp_codec::{Encoder, EncoderConfig};
use lnmp_core::registry::{embedded_registry, ValidationMode};

let config = EncoderConfig::new()
    .with_fid_registry(embedded_registry())
    .with_fid_validation_mode(ValidationMode::Error);

let encoder = Encoder::with_config(config);
let result = encoder.encode_validated(&record);  // Returns error on invalid FID
```

## Examples

This crate includes several examples in the `examples/` directory:

- **[parse_simple](./examples/parse_simple.rs)**: Basic parsing of LNMP text
- **[encode_with_hints](./examples/encode_with_hints.rs)**: Encoding with type hints and checksums

Run examples with:
```bash
cargo run --example parse_simple -p lnmp-codec
cargo run --example encode_with_hints -p lnmp-codec
```

See the root `examples/` directory for integration examples and v0.4 binary format demos.

## License

MIT OR Apache-2.0
