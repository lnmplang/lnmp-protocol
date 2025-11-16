# LNMP Migration Guide

This guide helps you migrate between LNMP versions.

## Table of Contents

- [v0.3 to v0.4](#v03-to-v04)
- [v0.2 to v0.3](#v02-to-v03)
- [v0.1 to v0.2](#v01-to-v02)

---

## v0.3 to v0.4

### Overview

LNMP v0.4 introduces a binary protocol format that enables efficient serialization and bidirectional conversion between text and binary representations. This establishes LNMP as a complete transport protocol with both human-readable (text) and machine-optimized (binary) formats.

**Key Point**: v0.4 is **fully backward compatible** with v0.3 for text format. The binary format is a new addition that doesn't affect existing text-based code.

### Backward Compatibility

✅ **Guaranteed Compatible:**
- All v0.3 text format syntax remains valid in v0.4
- Existing parsers and encoders work without changes
- Nested structures continue to work in text format
- Semantic checksums remain fully supported
- All v0.3 features (SFE, SEL, LLB) continue to work

❌ **No Breaking Changes:**
- No API changes to existing text format functions
- No changes to existing value types
- No changes to parsing or encoding behavior for text format

### New Features

#### 1. Binary Protocol Format

**What's New:**
- `BinaryEncoder` - Converts text/records to binary format
- `BinaryDecoder` - Converts binary format to text/records
- `BinaryError` - Error types for binary operations
- `EncoderConfig` and `DecoderConfig` - Configuration options
- VarInt encoding for space-efficient integers
- Type tags for explicit type information

**Binary Format Structure:**
```
┌─────────┬─────────┬─────────────┬──────────────────────┐
│ VERSION │  FLAGS  │ ENTRY_COUNT │      ENTRIES...      │
│ (1 byte)│(1 byte) │  (VarInt)   │     (variable)       │
└─────────┴─────────┴─────────────┴──────────────────────┘
```

**Supported Types in Binary Format:**
- Integer (0x01): VarInt encoded signed 64-bit integers
- Float (0x02): IEEE 754 double-precision (8 bytes, little-endian)
- Boolean (0x03): Single byte (0x00 = false, 0x01 = true)
- String (0x04): Length-prefixed UTF-8
- String Array (0x05): Count-prefixed array of length-prefixed strings

**Important Limitation:**
- ⚠️ Nested structures (NestedRecord, NestedArray) are **not supported** in v0.4 binary format
- Nested structures remain fully supported in text format
- Binary support for nested structures is planned for v0.5

#### 2. Basic Binary Encoding

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

// Create a record
let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 7,
    value: LnmpValue::Bool(true),
});
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

// Encode to binary
let encoder = BinaryEncoder::new();
let binary = encoder.encode(&record).unwrap();

// Decode from binary
let decoder = BinaryDecoder::new();
let decoded_record = decoder.decode(&binary).unwrap();
```

#### 3. Text ↔ Binary Conversion

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

// Text to binary
let text = "F7=1\nF12=14532\nF23=[admin,dev]";
let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(text).unwrap();

// Binary to text
let decoder = BinaryDecoder::new();
let decoded_text = decoder.decode_to_text(&binary).unwrap();

// Round-trip maintains data integrity
assert_eq!(text, decoded_text);
```

#### 4. Configuration Options

**Encoder Configuration:**
```rust
use lnmp_codec::binary::{BinaryEncoder, EncoderConfig};

let config = EncoderConfig::new()
    .with_validate_canonical(true)  // Validate canonical form before encoding
    .with_sort_fields(true);        // Sort fields by FID (default: true)

let encoder = BinaryEncoder::with_config(config);
```

**Decoder Configuration:**
```rust
use lnmp_codec::binary::{BinaryDecoder, DecoderConfig};

let config = DecoderConfig::new()
    .with_validate_ordering(true)  // Enforce canonical field order
    .with_strict_parsing(true);    // Detect trailing data

let decoder = BinaryDecoder::with_config(config);
```

#### 5. Round-Trip Guarantees

The binary format maintains canonical form guarantees:

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

// Unsorted input becomes canonical
let unsorted_text = "F23=[admin]\nF7=1\nF12=14532";

let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(unsorted_text).unwrap();

let decoder = BinaryDecoder::new();
let canonical_text = decoder.decode_to_text(&binary).unwrap();

// Output is sorted by FID: F7=1\nF12=14532\nF23=[admin]

// Multiple round-trips produce stable output
let binary2 = encoder.encode_text(&canonical_text).unwrap();
assert_eq!(binary, binary2);
```

### API Changes

#### New Modules

```rust
// lnmp-codec/src/binary/
pub mod binary {
    pub use encoder::{BinaryEncoder, EncoderConfig};
    pub use decoder::{BinaryDecoder, DecoderConfig};
    pub use error::BinaryError;
    pub use types::{BinaryValue, TypeTag};
    pub use entry::BinaryEntry;
    pub use frame::BinaryFrame;
}
```

#### New Types

```rust
// Binary encoder
pub struct BinaryEncoder {
    config: EncoderConfig,
}

pub struct EncoderConfig {
    pub validate_canonical: bool,
    pub sort_fields: bool,
}

// Binary decoder
pub struct BinaryDecoder {
    config: DecoderConfig,
}

pub struct DecoderConfig {
    pub validate_ordering: bool,
    pub strict_parsing: bool,
}

// Error types
pub enum BinaryError {
    UnsupportedVersion { found: u8, supported: Vec<u8> },
    InvalidFID { fid: u16, reason: String },
    InvalidTypeTag { tag: u8 },
    InvalidValue { field_id: u16, type_tag: TypeTag, reason: String },
    TrailingData { bytes_remaining: usize },
    CanonicalViolation { reason: String },
    UnexpectedEof { expected: usize, found: usize },
    InvalidVarInt { reason: String },
    InvalidUtf8 { field_id: u16 },
    TextFormatError { source: LnmpError },
}
```

### Migration Steps

#### Step 1: No Changes Required for Text Format

If you're only using text format, no changes are needed:

```rust
// v0.3 code (still works in v0.4)
use lnmp_codec::{Parser, Encoder};

let text = "F7=1\nF12=14532";
let mut parser = Parser::new(text).unwrap();
let record = parser.parse_record().unwrap();

let encoder = Encoder::new();
let output = encoder.encode(&record);
```

#### Step 2: Add Binary Format Support (Optional)

To use binary format, add the binary module:

```rust
// v0.4 with binary format
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

let text = "F7=1\nF12=14532";

// Encode to binary
let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(text).unwrap();

// Decode from binary
let decoder = BinaryDecoder::new();
let decoded_text = decoder.decode_to_text(&binary).unwrap();
```

#### Step 3: Handle Nested Structures

If you use nested structures, be aware they're not supported in binary format:

```rust
use lnmp_codec::binary::BinaryEncoder;
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

let mut inner = LnmpRecord::new();
inner.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });

let mut record = LnmpRecord::new();
record.add_field(LnmpField {
    fid: 50,
    value: LnmpValue::NestedRecord(Box::new(inner)),
});

let encoder = BinaryEncoder::new();
let result = encoder.encode(&record);

// This will return an error:
// BinaryError::InvalidValue { reason: "Nested records not supported in v0.4 binary format" }
```

**Workaround**: Continue using text format for nested structures until v0.5.

### Performance Benefits

#### Space Efficiency

Binary format provides significant space savings:

```rust
use lnmp_codec::binary::BinaryEncoder;

let text = "F7=1\nF12=14532\nF23=[admin,dev]";
let encoder = BinaryEncoder::new();
let binary = encoder.encode_text(text).unwrap();

println!("Text size: {} bytes", text.len());      // ~30 bytes
println!("Binary size: {} bytes", binary.len());  // ~20 bytes
println!("Savings: ~33%");
```

**Expected Savings**: 30-50% size reduction for typical records

#### Encoding/Decoding Speed

- **Encoding**: < 1μs per field for simple types
- **Decoding**: < 1μs per field for simple types
- **Round-trip**: < 10μs for typical 10-field record

### Common Migration Patterns

#### Pattern 1: Add Binary Transport to Existing Application

```rust
// Before (v0.3) - text only
use lnmp_codec::{Parser, Encoder};

fn send_data(record: &LnmpRecord) -> String {
    let encoder = Encoder::new();
    encoder.encode(record)
}

fn receive_data(text: &str) -> LnmpRecord {
    let mut parser = Parser::new(text).unwrap();
    parser.parse_record().unwrap()
}

// After (v0.4) - with binary transport
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

fn send_data(record: &LnmpRecord) -> Vec<u8> {
    let encoder = BinaryEncoder::new();
    encoder.encode(record).unwrap()
}

fn receive_data(binary: &[u8]) -> LnmpRecord {
    let decoder = BinaryDecoder::new();
    decoder.decode(binary).unwrap()
}
```

#### Pattern 2: Support Both Text and Binary Formats

```rust
use lnmp_codec::{Parser, Encoder};
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

enum Format {
    Text,
    Binary,
}

fn encode_record(record: &LnmpRecord, format: Format) -> Vec<u8> {
    match format {
        Format::Text => {
            let encoder = Encoder::new();
            encoder.encode(record).into_bytes()
        }
        Format::Binary => {
            let encoder = BinaryEncoder::new();
            encoder.encode(record).unwrap()
        }
    }
}

fn decode_record(data: &[u8], format: Format) -> LnmpRecord {
    match format {
        Format::Text => {
            let text = std::str::from_utf8(data).unwrap();
            let mut parser = Parser::new(text).unwrap();
            parser.parse_record().unwrap()
        }
        Format::Binary => {
            let decoder = BinaryDecoder::new();
            decoder.decode(data).unwrap()
        }
    }
}
```

#### Pattern 3: Optimize Storage with Binary Format

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
use std::fs;

// Save to file in binary format
fn save_record(record: &LnmpRecord, path: &str) -> std::io::Result<()> {
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(record).unwrap();
    fs::write(path, binary)
}

// Load from file in binary format
fn load_record(path: &str) -> std::io::Result<LnmpRecord> {
    let binary = fs::read(path)?;
    let decoder = BinaryDecoder::new();
    Ok(decoder.decode(&binary).unwrap())
}
```

### Troubleshooting

#### Issue: Nested Structures Not Supported

**Cause**: v0.4 binary format doesn't support nested structures.

**Solution**: Continue using text format for nested structures:

```rust
use lnmp_codec::{Parser, Encoder};

// Use text format for nested structures
let text = "F50={F12=1;F7=1}";
let mut parser = Parser::new(text).unwrap();
let record = parser.parse_record().unwrap();

let encoder = Encoder::new();
let output = encoder.encode(&record);
```

#### Issue: Version Mismatch Error

**Cause**: Trying to decode binary data with wrong version.

**Solution**: Ensure binary data is v0.4 format (version byte 0x04):

```rust
use lnmp_codec::binary::{BinaryDecoder, BinaryError};

let decoder = BinaryDecoder::new();
match decoder.decode(&binary) {
    Ok(record) => println!("Success!"),
    Err(BinaryError::UnsupportedVersion { found, .. }) => {
        eprintln!("Unsupported version: 0x{:02X}", found);
    }
    Err(e) => eprintln!("Decode error: {}", e),
}
```

#### Issue: Trailing Data Detected

**Cause**: Extra bytes after binary frame in strict mode.

**Solution**: Either fix the data or disable strict parsing:

```rust
use lnmp_codec::binary::{BinaryDecoder, DecoderConfig};

// Option 1: Disable strict parsing
let config = DecoderConfig::new().with_strict_parsing(false);
let decoder = BinaryDecoder::with_config(config);

// Option 2: Trim data to exact frame size
// (requires knowing frame size in advance)
```

### When to Use Binary Format

✅ **Use Binary Format When:**
- Network bandwidth is limited
- Storage space is constrained
- Performance is critical
- Data doesn't contain nested structures
- Machine-to-machine communication

❌ **Use Text Format When:**
- Human readability is important
- Debugging and inspection needed
- Data contains nested structures (until v0.5)
- Interoperability with v0.3 systems
- LLM input/output (text is more token-efficient for LLMs)

### Recommended Migration Path

1. **Phase 1**: Update dependencies to v0.4 (no code changes needed)
2. **Phase 2**: Run existing tests (should pass without changes)
3. **Phase 3**: Identify use cases for binary format (transport, storage)
4. **Phase 4**: Add binary encoding/decoding for identified use cases
5. **Phase 5**: Measure performance and space improvements
6. **Phase 6**: Continue using text format for nested structures

### Summary

v0.4 is a **non-breaking** release that adds binary protocol support while maintaining full backward compatibility with v0.3 text format. You can adopt binary format incrementally for specific use cases without changing existing code.

**Key Takeaways:**
- ✅ All v0.3 text format code works without changes
- ✅ Binary format is opt-in for specific use cases
- ✅ 30-50% space savings with binary format
- ✅ Round-trip conversion maintains data integrity
- ⚠️ Nested structures not supported in binary format (use text format)
- ✅ Binary support for nested structures planned for v0.5

---

## v0.2 to v0.3

### Overview

LNMP v0.3 introduces the "protocol intelligence layer" with four major subsystems:

1. **Semantic Fidelity Engine (SFE)** - Checksums, normalization, equivalence mapping
2. **Structural Extensibility Layer (SEL)** - Nested records and arrays
3. **Zero-Ambiguity Grammar (ZAG)** - Formal PEG/EBNF specification
4. **LNMP-LLM Bridge Layer (LLB)** - Prompt optimization, explain mode, ShortForm

**Key Point**: v0.3 is **fully backward compatible** with v0.2. All existing code continues to work without changes.

### Backward Compatibility

✅ **Guaranteed Compatible:**
- All v0.2 syntax remains valid in v0.3
- Existing parsers can read v0.3 output (ignoring checksums)
- Type hints remain optional
- Field ordering behavior unchanged
- Canonical format rules unchanged (except for nested structures)

❌ **No Breaking Changes:**
- No API changes to existing functions
- No changes to existing value types
- No changes to parsing behavior for v0.2 format

### New Features

#### 1. Nested Structures

**What's New:**
- `LnmpValue::NestedRecord(Box<LnmpRecord>)` - Records within records
- `LnmpValue::NestedArray(Vec<LnmpRecord>)` - Arrays of records
- `TypeHint::Record` (`:r`) and `TypeHint::RecordArray` (`:ra`)

**Syntax:**
```rust
// Nested record: F50={F12=1;F7=1}
let input = "F50={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Nested array: F60=[{F1=alice},{F1=bob}]
let input = "F60=[{F1=alice},{F1=bob}]";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();
```

**Migration Steps:**
1. No changes required for existing code
2. To use nested structures, update your data model to use new variants
3. Update pattern matching to handle new variants:

```rust
// Before (v0.2)
match &field.value {
    LnmpValue::Int(i) => println!("Int: {}", i),
    LnmpValue::String(s) => println!("String: {}", s),
    // ... other types
}

// After (v0.3) - add new variants
match &field.value {
    LnmpValue::Int(i) => println!("Int: {}", i),
    LnmpValue::String(s) => println!("String: {}", s),
    LnmpValue::NestedRecord(rec) => println!("Nested record with {} fields", rec.fields().len()),
    LnmpValue::NestedArray(arr) => println!("Nested array with {} elements", arr.len()),
    // ... other types
}
```

#### 2. Semantic Checksums (SC32)

**What's New:**
- `SemanticChecksum` module in `lnmp-core`
- `enable_checksums` option in `EncoderConfig`
- `validate_checksums` option in `ParserConfig`
- Checksum syntax: `F12:i=14532#6A93B3F1`

**Usage:**
```rust
use lnmp_codec::{Encoder, EncoderConfig, Parser, ParserConfig};

// Encoding with checksums
let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
// Output: F12:i=14532#6A93B3F1

// Parsing with checksum validation
let config = ParserConfig {
    validate_checksums: true,
    ..Default::default()
};
let mut parser = Parser::with_config(input, config).unwrap();
let record = parser.parse_record().unwrap();  // Validates checksums
```

**Migration Steps:**
1. No changes required for existing code
2. To enable checksums, update encoder configuration
3. To validate checksums, update parser configuration
4. Checksums are optional - parsers ignore them by default

**When to Use:**
- ✅ Critical fields where drift prevention is important
- ✅ Data exchanged with LLMs over multiple turns
- ✅ Fields with semantic equivalence (use with normalization)
- ❌ High-frequency data (adds ~8 bytes per field)
- ❌ Human-edited data (checksums will break on edits)

#### 3. Value Normalization

**What's New:**
- `ValueNormalizer` in `lnmp-codec`
- `NormalizationConfig` for configurable normalization
- Canonical transformations for semantic equivalence

**Usage:**
```rust
use lnmp_codec::{EncoderConfig, NormalizationConfig, StringCaseRule};

let norm_config = NormalizationConfig {
    string_case: StringCaseRule::Lower,
    remove_trailing_zeros: true,
    float_precision: None,
};

let config = EncoderConfig {
    enable_checksums: true,
    normalization_config: Some(norm_config),
    ..Default::default()
};

let encoder = Encoder::with_config(config);
```

**Normalization Rules:**
- **Booleans**: `true/false/yes/no/1/0` → `1` or `0`
- **Floats**: `-0.0` → `0.0`, `3.140` → `3.14`
- **Strings**: Configurable case transformation

**Migration Steps:**
1. No changes required for existing code
2. To enable normalization, add `normalization_config` to encoder
3. Normalization affects checksum computation
4. Use normalization when semantic equivalence is important

#### 4. Equivalence Mapping

**What's New:**
- `EquivalenceMapper` in `lnmp-codec`
- Synonym recognition for field values
- Integration with semantic dictionary

**Usage:**
```rust
use lnmp_codec::{EquivalenceMapper, EncoderConfig};

let mut mapper = EquivalenceMapper::new();
mapper.add_mapping(7, "yes".to_string(), "1".to_string());
mapper.add_mapping(7, "true".to_string(), "1".to_string());
mapper.add_mapping(23, "admin".to_string(), "administrator".to_string());

let config = EncoderConfig {
    enable_checksums: true,
    equivalence_mapper: Some(mapper),
    ..Default::default()
};

let encoder = Encoder::with_config(config);
```

**Migration Steps:**
1. No changes required for existing code
2. To enable equivalence mapping, add `equivalence_mapper` to encoder
3. Define mappings for fields with known synonyms
4. Use with checksums to ensure semantic equivalence

#### 5. LLM-Optimized Encoding

**What's New:**
- Explain mode - human-readable comments
- ShortForm encoding - extreme token reduction
- Prompt visibility optimization

**Explain Mode:**
```rust
use lnmp_llb::{ExplainEncoder, SemanticDictionary};

let dict = SemanticDictionary::load_from_file("dictionary.yaml").unwrap();
let encoder = Encoder::new();
let explain_encoder = ExplainEncoder::new(encoder, dict);

let output = explain_encoder.encode_with_explanation(&record);
// Output:
// F12:i=14532  # user_id
// F7:b=1       # is_active
// F23:sa=[admin,dev]  # roles
```

**ShortForm Encoding:**
```rust
use lnmp_llb::{ShortFormEncoder, ShortFormConfig};

let config = ShortFormConfig {
    omit_prefix: true,
    omit_type_hints: true,
    minimal_whitespace: true,
};

let encoder = ShortFormEncoder::new(config);
let output = encoder.encode(&record);
// Output: 12=14532 7=1 23=[admin,dev]
```

**Migration Steps:**
1. Add `lnmp-llb` dependency to use LLM-optimized features
2. Use explain mode for debugging and human review
3. Use ShortForm only for LLM input (not for storage or APIs)
4. ShortForm is NOT canonical LNMP - use with caution

**When to Use:**
- ✅ Explain mode: Debugging, human review, documentation
- ✅ ShortForm: LLM input optimization, token reduction
- ❌ ShortForm: Storage, APIs, canonical format

### API Changes

#### New Types

```rust
// lnmp-core
pub enum LnmpValue {
    // ... existing variants
    NestedRecord(Box<LnmpRecord>),  // NEW
    NestedArray(Vec<LnmpRecord>),   // NEW
}

pub enum TypeHint {
    // ... existing variants
    Record,       // NEW: :r
    RecordArray,  // NEW: :ra
}

// lnmp-core/checksum.rs (NEW module)
pub struct SemanticChecksum;
impl SemanticChecksum {
    pub fn compute(fid: FieldId, type_hint: TypeHint, value: &LnmpValue) -> u32;
    pub fn validate(fid: FieldId, type_hint: TypeHint, value: &LnmpValue, checksum: u32) -> bool;
    pub fn format(checksum: u32) -> String;
}

// lnmp-codec
pub struct EncoderConfig {
    pub canonical: bool,
    pub include_type_hints: bool,
    pub enable_checksums: bool,              // NEW
    pub normalization_config: Option<NormalizationConfig>,  // NEW
    pub equivalence_mapper: Option<EquivalenceMapper>,      // NEW
}

pub struct ParserConfig {
    pub mode: ParsingMode,
    pub validate_checksums: bool,            // NEW
    pub equivalence_mapper: Option<EquivalenceMapper>,  // NEW
}

// lnmp-codec/normalizer.rs (NEW module)
pub struct ValueNormalizer {
    config: NormalizationConfig,
}

pub struct NormalizationConfig {
    pub string_case: StringCaseRule,
    pub float_precision: Option<usize>,
    pub remove_trailing_zeros: bool,
}

// lnmp-codec/equivalence.rs (NEW module)
pub struct EquivalenceMapper {
    mappings: HashMap<FieldId, HashMap<String, String>>,
}
```

#### New Methods

```rust
// LnmpValue
impl LnmpValue {
    pub fn depth(&self) -> usize;  // NEW
    pub fn validate_structure(&self) -> Result<(), LnmpError>;  // NEW
}

// TypeHint
impl TypeHint {
    pub fn validates(&self, value: &LnmpValue) -> bool;  // UPDATED for nested types
}
```

### Configuration Migration

#### Encoder Configuration

```rust
// v0.2 (still works)
let encoder = Encoder::new();

// v0.3 with new features
let config = EncoderConfig {
    canonical: true,
    include_type_hints: true,
    enable_checksums: true,                    // NEW
    normalization_config: Some(NormalizationConfig {
        string_case: StringCaseRule::Lower,
        remove_trailing_zeros: true,
        float_precision: None,
    }),
    equivalence_mapper: Some(mapper),          // NEW
};
let encoder = Encoder::with_config(config);
```

#### Parser Configuration

```rust
// v0.2 (still works)
let mut parser = Parser::new(input).unwrap();

// v0.3 with new features
let config = ParserConfig {
    mode: ParsingMode::Loose,
    validate_checksums: true,                  // NEW
    equivalence_mapper: Some(mapper),          // NEW
};
let mut parser = Parser::with_config(input, config).unwrap();
```

### Testing Migration

#### Update Pattern Matching

If you have exhaustive pattern matching on `LnmpValue`, add new variants:

```rust
// Before (v0.2)
match value {
    LnmpValue::Int(_) => { /* ... */ }
    LnmpValue::Float(_) => { /* ... */ }
    LnmpValue::Bool(_) => { /* ... */ }
    LnmpValue::String(_) => { /* ... */ }
    LnmpValue::StringArray(_) => { /* ... */ }
}

// After (v0.3)
match value {
    LnmpValue::Int(_) => { /* ... */ }
    LnmpValue::Float(_) => { /* ... */ }
    LnmpValue::Bool(_) => { /* ... */ }
    LnmpValue::String(_) => { /* ... */ }
    LnmpValue::StringArray(_) => { /* ... */ }
    LnmpValue::NestedRecord(_) => { /* ... */ }  // NEW
    LnmpValue::NestedArray(_) => { /* ... */ }   // NEW
}
```

#### Update Test Assertions

If you have tests that check encoded output with checksums enabled:

```rust
// Before (v0.2)
assert_eq!(output, "F12=14532");

// After (v0.3 with checksums)
assert!(output.starts_with("F12=14532#"));
assert_eq!(output.len(), "F12=14532#XXXXXXXX".len());
```

### Performance Considerations

#### Checksum Overhead

- Computation: <1μs per field
- Storage: +8 bytes per field (hex checksum)
- Parsing: Minimal overhead when validation disabled

**Recommendation**: Enable checksums only for critical fields.

#### Nested Structure Overhead

- Parsing: <10μs for 3-level nesting
- Memory: Proportional to structure depth
- Encoding: Minimal overhead (depth-first traversal)

**Recommendation**: Limit nesting depth to 10 levels for optimal performance.

#### Normalization Overhead

- Boolean normalization: <100ns per value
- Float normalization: <200ns per value
- String normalization: Depends on string length

**Recommendation**: Enable normalization only when semantic equivalence is required.

### Common Migration Patterns

#### Pattern 1: Add Checksums to Existing Code

```rust
// Before (v0.2)
let encoder = Encoder::new();
let output = encoder.encode(&record);

// After (v0.3)
let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
```

#### Pattern 2: Use Nested Structures

```rust
// Before (v0.2) - flat structure
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 1, value: LnmpValue::String("user".to_string()) });
record.add_field(LnmpField { fid: 10, value: LnmpValue::String("nested".to_string()) });
record.add_field(LnmpField { fid: 11, value: LnmpValue::String("data".to_string()) });

// After (v0.3) - nested structure
let mut inner = LnmpRecord::new();
inner.add_field(LnmpField { fid: 10, value: LnmpValue::String("nested".to_string()) });
inner.add_field(LnmpField { fid: 11, value: LnmpValue::String("data".to_string()) });

let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 1, value: LnmpValue::String("user".to_string()) });
record.add_field(LnmpField { fid: 2, value: LnmpValue::NestedRecord(Box::new(inner)) });
```

#### Pattern 3: Add Normalization and Equivalence

```rust
// Before (v0.2)
let encoder = Encoder::new();

// After (v0.3)
let mut mapper = EquivalenceMapper::new();
mapper.add_mapping(7, "yes".to_string(), "1".to_string());
mapper.add_mapping(7, "true".to_string(), "1".to_string());

let config = EncoderConfig {
    enable_checksums: true,
    normalization_config: Some(NormalizationConfig::default()),
    equivalence_mapper: Some(mapper),
    ..Default::default()
};
let encoder = Encoder::with_config(config);
```

### Troubleshooting

#### Issue: Checksums Don't Match

**Cause**: Value normalization not applied consistently.

**Solution**: Ensure both encoder and parser use the same `NormalizationConfig`:

```rust
let norm_config = NormalizationConfig::default();

let encoder_config = EncoderConfig {
    enable_checksums: true,
    normalization_config: Some(norm_config.clone()),
    ..Default::default()
};

let parser_config = ParserConfig {
    validate_checksums: true,
    ..Default::default()
};
```

#### Issue: Pattern Matching Exhaustiveness Error

**Cause**: New `LnmpValue` variants not handled.

**Solution**: Add new variants to match expressions:

```rust
match value {
    // ... existing variants
    LnmpValue::NestedRecord(rec) => { /* handle nested record */ }
    LnmpValue::NestedArray(arr) => { /* handle nested array */ }
}
```

#### Issue: ShortForm Parsing Fails

**Cause**: ShortForm parser not enabled.

**Solution**: Use `ShortFormParser` instead of standard `Parser`:

```rust
use lnmp_llb::{ShortFormParser, ShortFormConfig};

let config = ShortFormConfig::default();
let mut parser = ShortFormParser::new(config);
let record = parser.parse(input).unwrap();
```

### Recommended Migration Path

1. **Phase 1**: Update dependencies, run existing tests (should pass)
2. **Phase 2**: Add checksums to critical fields
3. **Phase 3**: Refactor flat structures to nested structures (if applicable)
4. **Phase 4**: Add normalization and equivalence mapping (if needed)
5. **Phase 5**: Use LLM-optimized features (explain mode, ShortForm) for specific use cases

### Summary

v0.3 is a **non-breaking** release that adds powerful new features while maintaining full backward compatibility. You can adopt new features incrementally without changing existing code.

**Key Takeaways:**
- ✅ All v0.2 code works without changes
- ✅ New features are opt-in via configuration
- ✅ Nested structures require updating pattern matching
- ✅ Checksums are optional and configurable
- ✅ LLM-optimized features are in separate crate (`lnmp-llb`)

---

## v0.1 to v0.2

### Overview

LNMP v0.2 introduced deterministic serialization and canonical format rules.

**Key Changes:**
- Fields are now sorted by FID during encoding
- Canonical format uses newlines (not semicolons)
- Type hints added (optional)
- Strict/Loose parsing modes

### Breaking Changes

#### 1. Field Ordering

**v0.1**: Fields encoded in insertion order
**v0.2**: Fields encoded sorted by FID

```rust
// v0.1 output
F100=3
F5=1
F50=2

// v0.2 output
F5=1
F50=2
F100=3
```

**Migration**: Update tests that expect specific field order.

#### 2. Canonical Format

**v0.1**: Semicolons or newlines
**v0.2**: Newlines only (canonical)

```rust
// v0.1 (still parsed in v0.2)
F1=42;F2=test;F3=1

// v0.2 canonical
F1=42
F2=test
F3=1
```

**Migration**: No code changes needed. Parser accepts both formats.

### New Features

#### Type Hints

```rust
use lnmp_codec::{Encoder, EncoderConfig};

let config = EncoderConfig {
    include_type_hints: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
// Output: F12:i=14532
```

#### Strict Mode

```rust
use lnmp_codec::{Parser, ParsingMode};

let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
// Requires canonical format
```

### Migration Steps

1. Update tests expecting specific field order
2. Use `Encoder::new()` for canonical format
3. Add type hints if needed for explicit typing
4. Use strict mode for canonical format validation

### Summary

v0.2 is mostly backward compatible for parsing, but encoding behavior changed. Update tests and use new configuration options for canonical format.

---

## Additional Resources

- **API Documentation**: Run `cargo doc --open`
- **Examples**: See `examples/` directory
- **Specification**: See `.kiro/specs/lnmp-v0.3-semantic-fidelity/`
- **Grammar**: See `spec/grammar.md`
- **Error Classes**: See `spec/error-classes.md`

## Support

For questions or issues:
- Open an issue on GitHub
- Check the examples directory
- Read the API documentation
- Review the specification documents
