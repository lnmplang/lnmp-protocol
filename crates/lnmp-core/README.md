# lnmp-core

Core type definitions for LNMP (LLM Native Minimal Protocol) v0.5.4.

## Overview

This crate provides the fundamental data structures for representing LNMP data:

- `FieldId` - Type alias for field identifiers (u16, range 0-65535)
- `LnmpValue` - Enum representing all supported value types (including nested structures and generic arrays)
- `RecordBuilder` - Fluent API for constructing canonical records with automatic field sorting
- `LnmpField` - A field ID and value pair
- `LnmpRecord` - A collection of fields representing a complete record
- `SemanticChecksum` - SC32 checksum computation for semantic fidelity (v0.3)
- `TypeHint` - Type annotations for explicit typing (including nested types and generic arrays)
- `LnmpProfile` - Strictness profiles (Loose, Standard, Strict) for validation and canonical enforcement
- `StrictDeterministicConfig` - Fine-grained control over determinism requirements

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
lnmp-core = { path = "path/to/lnmp-core" }
```

## Examples

This crate includes several examples in the `examples/` directory:

- **[basic_record](./examples/basic_record.rs)**: Creating and using simple records
- **[nested_structures](./examples/nested_structures.rs)**: Working with nested records and arrays

Run examples with:
```bash
cargo run --example basic_record -p lnmp-core
cargo run --example nested_structures -p lnmp-core
```

## Quick Start

```rust
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// Create a new record
let mut record = LnmpRecord::new();

// Add fields with different value types
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

record.add_field(LnmpField {
    fid: 7,
    value: LnmpValue::Bool(true),
});

record.add_field(LnmpField {
    fid: 20,
    value: LnmpValue::String("Halil".to_string()),
});

record.add_field(LnmpField {
    fid: 23,
    value: LnmpValue::StringArray(vec![
        "admin".to_string(),
        "dev".to_string(),
    ]),
});

// Access fields
if let Some(field) = record.get_field(12) {
    println!("User ID: {:?}", field.value);
}

// Iterate over all fields
for field in record.fields() {
    println!("F{} = {:?}", field.fid, field.value);
}

// Get field count
println!("Total fields: {}", record.fields().len());
```

## Types

### LnmpValue

Represents all supported value types in LNMP v0.3:

```rust
pub enum LnmpValue {
    Int(i64),                      // Integer values
    Float(f64),                    // Floating-point values
    Bool(bool),                    // Boolean values (true/false)
    String(String),                // String values
    StringArray(Vec<String>),      // Arrays of strings
    IntArray(Vec<i64>),            // Arrays of integers (v0.5.4)
    FloatArray(Vec<f64>),          // Arrays of floats (v0.5.4)
    BoolArray(Vec<bool>),          // Arrays of booleans (v0.5.4)
    NestedRecord(Box<LnmpRecord>), // Nested records (v0.3)
    NestedArray(Vec<LnmpRecord>),  // Arrays of records (v0.3)
}
```

**v0.3 Nested Structure Support:**

```rust
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

// Create a nested record
let mut inner_record = LnmpRecord::new();
inner_record.add_field(LnmpField {
    fid: 10,
    value: LnmpValue::String("nested".to_string()),
});

let mut outer_record = LnmpRecord::new();
outer_record.add_field(LnmpField {
    fid: 50,
    value: LnmpValue::NestedRecord(Box::new(inner_record)),
});

// Create a nested array
let mut record1 = LnmpRecord::new();
record1.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });

let mut record2 = LnmpRecord::new();
record2.add_field(LnmpField { fid: 1, value: LnmpValue::Int(2) });

let mut parent = LnmpRecord::new();
parent.add_field(LnmpField {
    fid: 60,
    value: LnmpValue::NestedArray(vec![record1, record2]),
});
```

### LnmpField

A single field assignment consisting of a field ID and value:

```rust
pub struct LnmpField {
    pub fid: FieldId,      // Field identifier (0-65535)
    pub value: LnmpValue,  // Field value
}
```

### LnmpRecord

A collection of fields representing a complete LNMP record:

```rust
impl LnmpRecord {
    pub fn new() -> Self;
    pub fn add_field(&mut self, field: LnmpField);
    pub fn get_field(&self, fid: FieldId) -> Option<&LnmpField>;
    pub fn fields(&self) -> &[LnmpField];
    pub fn into_fields(self) -> Vec<LnmpField>;
}
```

### RecordBuilder (v0.5.4)

Fluent API for constructing records with automatic canonical ordering:

```rust
use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};

// Builder automatically sorts fields by FID
let record = RecordBuilder::new()
    .add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) })
    .add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) })
    .add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) })
    .build();

// Fields are stored in canonical order: 7, 12, 23
assert_eq!(record.fields()[0].fid, 7);

// Alternative: construct from unsorted fields
let fields = vec![
    LnmpField { fid: 100, value: LnmpValue::Int(1) },
    LnmpField { fid: 50, value: LnmpValue::Int(2) },
];
let record = RecordBuilder::from_fields(fields);
```

### TypeHint

Type annotations for explicit typing (v0.2+):

```rust
pub enum TypeHint {
    Int,          // :i
    Float,        // :f
    Bool,         // :b
    String,       // :s
    StringArray,  // :sa
    IntArray,     // :ia (v0.5.4)
    FloatArray,   // :fa (v0.5.4)
    BoolArray,    // :ba (v0.5.4)
    Record,       // :r  (v0.3)
    RecordArray,  // :ra (v0.3)
}
```

### SemanticChecksum (v0.3)

Compute and validate semantic checksums (SC32) for drift prevention:

```rust
use lnmp_core::{SemanticChecksum, TypeHint, LnmpValue};

// Compute checksum
let checksum = SemanticChecksum::compute(
    12,  // field ID
    TypeHint::Int,
    &LnmpValue::Int(14532)
);

// Validate checksum
let is_valid = SemanticChecksum::validate(
    12,
    TypeHint::Int,
    &LnmpValue::Int(14532),
    checksum
);

// Format as hex string
let hex = SemanticChecksum::format(checksum);  // "36AAE667"
```

## v0.3 Features

### Nested Structures

Support for hierarchical data modeling:

- **Nested Records**: `F50={F12=1;F7=1}` - Records within records
- **Nested Arrays**: `F60=[{F1=1},{F1=2}]` - Arrays of records
- **Arbitrary Depth**: Limited only by available memory
- **Structural Validation**: `value.validate_structure()` ensures integrity

### Semantic Checksums (SC32)

32-bit checksums for preventing LLM input drift:

- **Deterministic**: Same value always produces same checksum
- **Semantic**: Based on FID + type hint + normalized value
- **Optional**: Can be enabled/disabled via configuration
- **Fast**: CRC32-based algorithm (<1μs per field)

### Value Normalization

Canonical transformations for semantic equivalence:

- **Booleans**: `true/false/yes/no/1/0` → `1` or `0`
- **Floats**: `-0.0` → `0.0`, `3.140` → `3.14`
- **Strings**: Configurable case normalization

## Deterministic Guarantees

LNMP-Core provides strong deterministic guarantees to prevent LLM drift and ensure semantic consistency:

### Canonical Field Ordering

The `sorted_fields()` method provides **canonical representation** where fields are sorted by FID (Field ID):

```rust
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) });
record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) });

// Insertion order: 23, 7, 12
assert_eq!(record.fields()[0].fid, 23);
assert_eq!(record.fields()[1].fid, 7);

// Canonical order (sorted by FID): 7, 12, 23
let sorted = record.sorted_fields();
assert_eq!(sorted[0].fid, 7);
assert_eq!(sorted[1].fid, 12);
assert_eq!(sorted[2].fid, 23);
```

**Key Points:**
- `fields()` returns fields in insertion order
- `sorted_fields()` returns fields in canonical order (sorted by FID)
- All encoders (`lnmp-codec`) use `sorted_fields()` for output
- `SemanticChecksum` uses `sorted_fields()` for deterministic hashing

### Deterministic SemanticChecksum (SC32)

The `SemanticChecksum` system ensures the same semantic data always produces the same checksum, **regardless of field insertion order**:

```rust
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue, TypeHint};
use lnmp_core::checksum::SemanticChecksum;

// Record 1: fields added in order 12, 7
let mut rec1 = LnmpRecord::new();
rec1.add_field(LnmpField { fid: 12, value: LnmpValue::Int(100) });
rec1.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });

// Record 2: fields added in order 7, 12 (different!)
let mut rec2 = LnmpRecord::new();
rec2.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });
rec2.add_field(LnmpField { fid: 12, value: LnmpValue::Int(100) });

// When serialized for checksum, both use sorted_fields internally
// So nested records with different insertion order produce SAME checksum
let val1 = LnmpValue::NestedRecord(Box::new(rec1));
let val2 = LnmpValue::NestedRecord(Box::new(rec2));

let cs1 = SemanticChecksum::compute(50, Some(TypeHint::Record), &val1);
let cs2 = SemanticChecksum::compute(50, Some(TypeHint::Record), &val2);

assert_eq!(cs1, cs2); // ✅ Same checksum despite different insertion order!
```

**Normalization Rules:**
- **Booleans**: Always serialized as `1` (true) or `0` (false)
- **Floats**: `-0.0` normalized to `0.0`, trailing zeros removed (e.g., `3.140` → `3.14`)
- **Nested Records**: Fields sorted by FID before serialization
- **Arrays**: Order preserved (array element order is semantic)

### Canonical Encoding

All encoders in `lnmp-codec` produce canonical output:

```rust
use lnmp_codec::Encoder;

// Fields added in non-canonical order
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) });
record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });

let encoder = Encoder::new();
let output = encoder.encode(&record);

// Output is ALWAYS sorted by FID (canonical):
// F7=1
// F23=3
```

**Binary Encoding:**
- Binary encoders also use `sorted_fields()`
- Binary format can optionally validate canonical field ordering
- Round-trip (text → binary → text) produces canonical sorted output

### Best Practices

**For Deterministic Behavior:**
1. Use `sorted_fields()` when comparing records semantically
2. Use `SemanticChecksum` to detect drift
3. Rely on encoder output (always canonical)

**When Order Matters:**
1. Use `fields()` to preserve insertion order
2. Note: `PartialEq` compares in insertion order (structural equality)

#### Canonical Equality (v0.5.4)

For semantic comparison that ignores field order:

```rust
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

let mut rec1 = RecordBuilder::new()
    .add_field(LnmpField { fid: 12, value: LnmpValue::Int(100) })
    .add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) })
    .build();

let mut rec2 = RecordBuilder::new()
    .add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) })
    .add_field(LnmpField { fid: 12, value: LnmpValue::Int(100) })
    .build();

// Structural equality: false (different field order)
assert_ne!(rec1, rec2);

// Canonical equality: true (same fields, semantically)
assert!(rec1.canonical_eq(&rec2));
```

## Features

- **Zero dependencies** - Pure Rust implementation (except CRC32 for checksums)
- **Type safety** - Strong typing for all value types
- **Efficient storage** - Fields stored in a Vec for cache-friendly access
- **Flexible access** - Get fields by ID or iterate over all fields
- **Deterministic checksums** - Field-order-independent SC32 for drift prevention
- **Canonical encoding** - Encoders always produce sorted output
- **Nested structures** - Support for hierarchical data in text format (v0.3)
- **Semantic checksums** - Drift prevention with SC32 (v0.3)
- **Binary format support** - Compatible with v0.4 binary protocol (via lnmp-codec)

## Binary Format Support (v0.4)

The core types are fully compatible with the v0.4 binary protocol format:

- All primitive types (Int, Float, Bool, String, StringArray) can be encoded to binary
- Binary encoding provides 30-50% size reduction compared to text format
- Round-trip conversion (text ↔ binary) maintains data integrity
- **Note**: Nested structures (NestedRecord, NestedArray) are not yet supported in v0.4 binary format
  - Nested structures remain fully supported in text format
  - Binary support for nested structures is planned for v0.5

For binary encoding/decoding, use the `lnmp-codec` crate:

```rust
use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

let mut record = LnmpRecord::new();
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

## Migration from v0.2

v0.3 is backward compatible with v0.2. New features:

- `LnmpValue::NestedRecord` and `LnmpValue::NestedArray` variants
- `TypeHint::Record` and `TypeHint::RecordArray` variants
- `SemanticChecksum` module for checksum computation
- `depth()` and `validate_structure()` methods on `LnmpValue`

v0.4 adds binary protocol support (via lnmp-codec) with no changes to core types.

Existing v0.2 code continues to work without changes.

## License

MIT OR Apache-2.0
