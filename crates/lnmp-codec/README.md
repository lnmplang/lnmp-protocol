# lnmp-codec

Parser and encoder implementations for LNMP (LLM Native Minimal Protocol) v0.3 text format and v0.4 binary format.

## Features

### Text Format (v0.3)
- **Deterministic serialization**: Fields always sorted by FID for consistent output
- **Canonical format**: Newline-separated, no extra whitespace (v0.2)
- **Type hints**: Optional type annotations (`:i`, `:f`, `:b`, `:s`, `:sa`, `:r`, `:ra`)
- **Nested structures**: Parse and encode nested records and arrays (v0.3)
- **Semantic checksums**: Optional SC32 checksums for drift prevention (v0.3)
- **Value normalization**: Canonical value transformations (v0.3)
- **Equivalence mapping**: Synonym recognition (v0.3)
- **Strict mode**: Validates canonical format compliance
- **Loose mode**: Accepts format variations (default)
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

// Parse nested record
let input = "F50={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Parse nested array
let input = "F60=[{F1=alice},{F1=bob}]";
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
// Output: F12=14532#6A93B3F1  (with checksum)
```

## LNMP v0.2 Features

### Deterministic Serialization

Fields are always sorted by FID, ensuring consistent output:

```rust
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 100, value: LnmpValue::Int(3) });
record.add_field(LnmpField { fid: 5, value: LnmpValue::Int(1) });
record.add_field(LnmpField { fid: 50, value: LnmpValue::Int(2) });

let encoder = Encoder::new();
let output = encoder.encode(&record);
// Output: F5=1\nF50=2\nF100=3  (sorted)
```

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
let mut parser = Parser::new("F3=test;F1=42").unwrap();  // Unsorted, semicolons OK

// Strict mode: requires canonical format
let mut parser = Parser::with_mode("F1=42\nF3=test", ParsingMode::Strict).unwrap();
```

## v0.3 Features

### Nested Structures

Parse and encode hierarchical data:

```rust
use lnmp_codec::{Parser, Encoder};

// Nested record: F50={F12=1;F7=1}
let input = "F50={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Nested array: F60=[{F1=alice},{F1=bob}]
let input = "F60=[{F1=alice},{F1=bob}]";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Deep nesting
let input = "F100={F1=user;F2={F10=nested;F11=data}}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();
```

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
// Output: F12:i=14532#6A93B3F1

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

## Examples

See the `examples/` directory for complete examples:

**v0.2 Examples:**
- `type_hints.rs`: Type hint usage
- `strict_vs_loose.rs`: Parsing mode comparison
- `deterministic_serialization.rs`: Canonical format demo

**v0.3 Examples:**
- `nested_structures.rs`: Nested records and arrays
- `semantic_checksums.rs`: SC32 checksum usage
- `explain_mode.rs`: Explain mode encoding
- `shortform.rs`: ShortForm encoding/parsing
- `structural_canonicalization.rs`: Structural canonicalization

**v0.4 Examples (Binary Format):**
- `binary_encoding.rs`: Basic binary encoding and decoding
- `binary_roundtrip.rs`: Round-trip conversion and data integrity

Run examples:
```bash
# v0.2 examples
cargo run --example type_hints
cargo run --example strict_vs_loose
cargo run --example deterministic_serialization

# v0.3 examples
cargo run --example nested_structures
cargo run --example semantic_checksums
cargo run --example explain_mode
cargo run --example shortform
cargo run --example structural_canonicalization

# v0.4 examples (binary format)
cargo run --example binary_encoding
cargo run --example binary_roundtrip
```

## License

MIT OR Apache-2.0
