# LNMP v0.3 API Reference

Complete API reference for LNMP v0.3 implementation.

## Table of Contents

- [lnmp-core](#lnmp-core)
  - [Types](#types)
  - [Records](#records)
  - [Checksums](#checksums)
- [lnmp-codec](#lnmp-codec)
  - [Parser](#parser)
  - [Encoder](#encoder)
  - [Normalizer](#normalizer)
  - [Equivalence Mapper](#equivalence-mapper)
  - [Configuration](#configuration)
- [lnmp-sfe](#lnmp-sfe)
  - [Semantic Dictionary](#semantic-dictionary)
- [lnmp-llb](#lnmp-llb)
  - [Explain Mode](#explain-mode)
  - [ShortForm](#shortform)
  - [Prompt Optimization](#prompt-optimization)

---

## lnmp-core

Core type definitions for LNMP data structures.

### Types

#### FieldId

```rust
pub type FieldId = u16;
```

Type alias for field identifiers. Range: 0-65535.

#### LnmpValue

```rust
pub enum LnmpValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    StringArray(Vec<String>),
    NestedRecord(Box<LnmpRecord>),  // v0.3
    NestedArray(Vec<LnmpRecord>),   // v0.3
}
```

Represents all supported value types in LNMP.

**Methods:**

```rust
impl LnmpValue {
    /// Returns the nesting depth (0 for primitives)
    pub fn depth(&self) -> usize;
    
    /// Validates structural integrity
    pub fn validate_structure(&self) -> Result<(), LnmpError>;
}
```

**Examples:**

```rust
use lnmp_core::LnmpValue;

// Primitive values
let int_val = LnmpValue::Int(42);
let float_val = LnmpValue::Float(3.14);
let bool_val = LnmpValue::Bool(true);
let string_val = LnmpValue::String("hello".to_string());
let array_val = LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]);

// Nested values (v0.3)
let mut inner = LnmpRecord::new();
inner.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });
let nested_val = LnmpValue::NestedRecord(Box::new(inner));

// Check depth
assert_eq!(int_val.depth(), 0);
assert_eq!(nested_val.depth(), 1);

// Validate structure
nested_val.validate_structure().unwrap();
```

#### TypeHint

```rust
pub enum TypeHint {
    Int,          // :i
    Float,        // :f
    Bool,         // :b
    String,       // :s
    StringArray,  // :sa
    Record,       // :r  (v0.3)
    RecordArray,  // :ra (v0.3)
}
```

Type annotations for explicit typing.

**Methods:**

```rust
impl TypeHint {
    /// Checks if type hint matches value type
    pub fn validates(&self, value: &LnmpValue) -> bool;
    
    /// Returns the string representation
    pub fn as_str(&self) -> &str;
    
    /// Parses from string
    pub fn from_str(s: &str) -> Option<Self>;
}
```

**Examples:**

```rust
use lnmp_core::{TypeHint, LnmpValue};

let hint = TypeHint::Int;
assert_eq!(hint.as_str(), "i");
assert!(hint.validates(&LnmpValue::Int(42)));
assert!(!hint.validates(&LnmpValue::String("test".to_string())));

let hint = TypeHint::parse("sa").unwrap();
assert_eq!(hint, TypeHint::StringArray);
```

#### LnmpField

```rust
pub struct LnmpField {
    pub fid: FieldId,
    pub value: LnmpValue,
}
```

A single field assignment consisting of a field ID and value.

**Examples:**

```rust
use lnmp_core::{LnmpField, LnmpValue};

let field = LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
};

println!("F{} = {:?}", field.fid, field.value);
```

### Records

#### LnmpRecord

```rust
pub struct LnmpRecord {
    // private fields
}
```

A collection of fields representing a complete LNMP record.

**Methods:**

```rust
impl LnmpRecord {
    /// Creates a new empty record
    pub fn new() -> Self;
    
    /// Adds a field to the record
    pub fn add_field(&mut self, field: LnmpField);
    
    /// Gets a field by ID
    pub fn get_field(&self, fid: FieldId) -> Option<&LnmpField>;
    
    /// Gets a mutable reference to a field by ID
    pub fn get_field_mut(&mut self, fid: FieldId) -> Option<&mut LnmpField>;
    
    /// Returns all fields
    pub fn fields(&self) -> &[LnmpField];
    
    /// Returns sorted fields (by FID)
    pub fn sorted_fields(&self) -> Vec<&LnmpField>;
    
    /// Consumes the record and returns fields
    pub fn into_fields(self) -> Vec<LnmpField>;
    
    /// Returns the number of fields
    pub fn len(&self) -> usize;
    
    /// Checks if the record is empty
    pub fn is_empty(&self) -> bool;
}
```

**Examples:**

```rust
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

// Create a new record
let mut record = LnmpRecord::new();

// Add fields
record.add_field(LnmpField {
    fid: 12,
    value: LnmpValue::Int(14532),
});

record.add_field(LnmpField {
    fid: 7,
    value: LnmpValue::Bool(true),
});

// Get a field
if let Some(field) = record.get_field(12) {
    println!("User ID: {:?}", field.value);
}

// Iterate over fields
for field in record.fields() {
    println!("F{} = {:?}", field.fid, field.value);
}

// Get sorted fields
let sorted = record.sorted_fields();
assert_eq!(sorted[0].fid, 7);  // Sorted by FID
assert_eq!(sorted[1].fid, 12);

// Check size
println!("Record has {} fields", record.len());
```

### Checksums

#### SemanticChecksum

```rust
pub struct SemanticChecksum;
```

Compute and validate semantic checksums (SC32) for drift prevention.

**Methods:**

```rust
impl SemanticChecksum {
    /// Computes SC32 checksum for a field
    pub fn compute(fid: FieldId, type_hint: TypeHint, value: &LnmpValue) -> u32;
    
    /// Validates checksum against field
    pub fn validate(
        fid: FieldId,
        type_hint: TypeHint,
        value: &LnmpValue,
        checksum: u32
    ) -> bool;
    
    /// Formats checksum as 8-character hex string
    pub fn format(checksum: u32) -> String;
    
    /// Parses checksum from hex string
    pub fn parse(s: &str) -> Result<u32, LnmpError>;
}
```

**Examples:**

```rust
use lnmp_core::{SemanticChecksum, TypeHint, LnmpValue};

// Compute checksum
let checksum = SemanticChecksum::compute(
    12,
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
assert!(is_valid);

// Format as hex
let hex = SemanticChecksum::format(checksum);
println!("Checksum: {}", hex);  // "36AAE667"

// Parse from hex
let parsed = SemanticChecksum::parse("36AAE667").unwrap();
assert_eq!(parsed, checksum);
```

---

## lnmp-codec

Parser and encoder implementations for LNMP text format.

### Parser

#### Parser

```rust
pub struct Parser<'a> {
    // private fields
}
```

Parses LNMP text format into structured data.

**Methods:**

```rust
impl<'a> Parser<'a> {
    /// Creates a new parser with default configuration
    pub fn new(input: &'a str) -> Result<Self, LnmpError>;
    
    /// Creates a parser with parsing mode
    pub fn with_mode(input: &'a str, mode: ParsingMode) -> Result<Self, LnmpError>;
    
    /// Creates a parser with full configuration
    pub fn with_config(input: &'a str, config: ParserConfig) -> Result<Self, LnmpError>;
    
    /// Parses a complete record
    pub fn parse_record(&mut self) -> Result<LnmpRecord, LnmpError>;
    
    /// Parses a single field
    pub fn parse_field(&mut self) -> Result<LnmpField, LnmpError>;
}
```

**Examples:**

```rust
use lnmp_codec::{Parser, ParsingMode, ParserConfig};

// Basic parsing
let input = "F12=14532\nF7=1";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();

// Strict mode
let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
let record = parser.parse_record().unwrap();

// With configuration
let config = ParserConfig {
    mode: ParsingMode::Loose,
    validate_checksums: true,
    equivalence_mapper: None,
};
let mut parser = Parser::with_config(input, config).unwrap();
let record = parser.parse_record().unwrap();

// Parse nested structures
let input = "F50={F12=1;F7=1}";
let mut parser = Parser::new(input).unwrap();
let record = parser.parse_record().unwrap();
```

#### ParsingMode

```rust
pub enum ParsingMode {
    Strict,  // Requires canonical format
    Loose,   // Accepts format variations (default)
}
```

**Examples:**

```rust
use lnmp_codec::{Parser, ParsingMode};

// Loose mode accepts variations
let input = "F3=test;F1=42";  // Unsorted, semicolons
let mut parser = Parser::with_mode(input, ParsingMode::Loose).unwrap();
let record = parser.parse_record().unwrap();

// Strict mode requires canonical format
let input = "F1=42\nF3=test";  // Sorted, newlines
let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
let record = parser.parse_record().unwrap();
```

### Encoder

#### Encoder

```rust
pub struct Encoder {
    // private fields
}
```

Encodes structured data into LNMP text format.

**Methods:**

```rust
impl Encoder {
    /// Creates a new encoder with default configuration
    pub fn new() -> Self;
    
    /// Creates an encoder with configuration
    pub fn with_config(config: EncoderConfig) -> Self;
    
    /// Encodes a record to LNMP text
    pub fn encode(&self, record: &LnmpRecord) -> String;
    
    /// Encodes a single field
    pub fn encode_field(&self, field: &LnmpField) -> String;
}
```

**Examples:**

```rust
use lnmp_codec::{Encoder, EncoderConfig};
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

// Basic encoding
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });

let encoder = Encoder::new();
let output = encoder.encode(&record);
println!("{}", output);  // F12=14532

// With type hints
let config = EncoderConfig {
    include_type_hints: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
println!("{}", output);  // F12:i=14532

// With checksums
let config = EncoderConfig {
    enable_checksums: true,
    ..Default::default()
};
let encoder = Encoder::with_config(config);
let output = encoder.encode(&record);
println!("{}", output);  // F12=14532#36AAE667
```

### Normalizer

#### ValueNormalizer

```rust
pub struct ValueNormalizer {
    // private fields
}
```

Applies canonical transformations to values for semantic equivalence.

**Methods:**

```rust
impl ValueNormalizer {
    /// Creates a new normalizer with configuration
    pub fn new(config: NormalizationConfig) -> Self;
    
    /// Normalizes a value to canonical form
    pub fn normalize(&self, value: &LnmpValue) -> LnmpValue;
}
```

**Examples:**

```rust
use lnmp_codec::{ValueNormalizer, NormalizationConfig, StringCaseRule};
use lnmp_core::LnmpValue;

let config = NormalizationConfig {
    string_case: StringCaseRule::Lower,
    remove_trailing_zeros: true,
    float_precision: None,
};

let normalizer = ValueNormalizer::new(config);

// Normalize boolean
let val = LnmpValue::String("yes".to_string());
let normalized = normalizer.normalize(&val);  // LnmpValue::Bool(true)

// Normalize float
let val = LnmpValue::Float(3.140);
let normalized = normalizer.normalize(&val);  // LnmpValue::Float(3.14)

// Normalize string case
let val = LnmpValue::String("HELLO".to_string());
let normalized = normalizer.normalize(&val);  // LnmpValue::String("hello")
```

#### NormalizationConfig

```rust
pub struct NormalizationConfig {
    pub string_case: StringCaseRule,
    pub float_precision: Option<usize>,
    pub remove_trailing_zeros: bool,
}
```

**StringCaseRule:**

```rust
pub enum StringCaseRule {
    None,   // No transformation
    Lower,  // Convert to lowercase
    Upper,  // Convert to uppercase
}
```

**Examples:**

```rust
use lnmp_codec::{NormalizationConfig, StringCaseRule};

// Default configuration
let config = NormalizationConfig::default();

// Custom configuration
let config = NormalizationConfig {
    string_case: StringCaseRule::Lower,
    float_precision: Some(2),  // Round to 2 decimal places
    remove_trailing_zeros: true,
};
```

### Equivalence Mapper

#### EquivalenceMapper

```rust
pub struct EquivalenceMapper {
    // private fields
}
```

Maps synonyms and related terms to canonical forms.

**Methods:**

```rust
impl EquivalenceMapper {
    /// Creates a new empty mapper
    pub fn new() -> Self;
    
    /// Loads equivalence mappings from dictionary
    pub fn load_from_dict(&mut self, dict: &SemanticDictionary) -> Result<(), LnmpError>;
    
    /// Maps value to canonical form for a field
    pub fn map(&self, fid: FieldId, value: &str) -> Option<String>;
    
    /// Adds custom mapping for a field
    pub fn add_mapping(&mut self, fid: FieldId, from: String, to: String);
    
    /// Checks if mapping exists
    pub fn has_mapping(&self, fid: FieldId, value: &str) -> bool;
}
```

**Examples:**

```rust
use lnmp_codec::EquivalenceMapper;

let mut mapper = EquivalenceMapper::new();

// Add mappings
mapper.add_mapping(7, "yes".to_string(), "1".to_string());
mapper.add_mapping(7, "true".to_string(), "1".to_string());
mapper.add_mapping(23, "admin".to_string(), "administrator".to_string());

// Map values
assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
assert_eq!(mapper.map(7, "true"), Some("1".to_string()));
assert_eq!(mapper.map(23, "admin"), Some("administrator".to_string()));
assert_eq!(mapper.map(7, "unknown"), None);

// Check if mapping exists
assert!(mapper.has_mapping(7, "yes"));
assert!(!mapper.has_mapping(7, "unknown"));
```

### Configuration

#### EncoderConfig

```rust
pub struct EncoderConfig {
    pub canonical: bool,
    pub include_type_hints: bool,
    pub enable_checksums: bool,
    pub normalization_config: Option<NormalizationConfig>,
    pub equivalence_mapper: Option<EquivalenceMapper>,
}
```

**Examples:**

```rust
use lnmp_codec::{EncoderConfig, NormalizationConfig, EquivalenceMapper};

// Default configuration
let config = EncoderConfig::default();

// Custom configuration
let mut mapper = EquivalenceMapper::new();
mapper.add_mapping(7, "yes".to_string(), "1".to_string());

let config = EncoderConfig {
    canonical: true,
    include_type_hints: true,
    enable_checksums: true,
    normalization_config: Some(NormalizationConfig::default()),
    equivalence_mapper: Some(mapper),
};
```

#### ParserConfig

```rust
pub struct ParserConfig {
    pub mode: ParsingMode,
    pub validate_checksums: bool,
    pub equivalence_mapper: Option<EquivalenceMapper>,
}
```

**Examples:**

```rust
use lnmp_codec::{ParserConfig, ParsingMode, EquivalenceMapper};

// Default configuration
let config = ParserConfig::default();

// Custom configuration
let mut mapper = EquivalenceMapper::new();
mapper.add_mapping(7, "yes".to_string(), "1".to_string());

let config = ParserConfig {
    mode: ParsingMode::Strict,
    validate_checksums: true,
    equivalence_mapper: Some(mapper),
};
```

---

## lnmp-sfe

Semantic Fidelity Engine - semantic dictionary and equivalence mapping.

### Semantic Dictionary

#### SemanticDictionary

```rust
pub struct SemanticDictionary {
    // private fields
}
```

Stores field names and equivalence mappings.

**Methods:**

```rust
impl SemanticDictionary {
    /// Creates a new empty dictionary
    pub fn new() -> Self;
    
    /// Loads dictionary from YAML file
    pub fn load_from_file(path: &Path) -> Result<Self, LnmpError>;
    
    /// Gets field name by ID
    pub fn get_field_name(&self, fid: FieldId) -> Option<&str>;
    
    /// Gets equivalence mapping for a field value
    pub fn get_equivalence(&self, fid: FieldId, value: &str) -> Option<&str>;
    
    /// Adds field name
    pub fn add_field_name(&mut self, fid: FieldId, name: String);
    
    /// Adds equivalence mapping
    pub fn add_equivalence(&mut self, fid: FieldId, from: String, to: String);
}
```

**Dictionary Format (YAML):**

```yaml
fields:
  12:
    name: user_id
    type: integer
    
  7:
    name: is_active
    type: boolean
    equivalences:
      yes: "1"
      true: "1"
      no: "0"
      false: "0"
      
  23:
    name: roles
    type: string_array
    equivalences:
      admin: administrator
      dev: developer
```

**Examples:**

```rust
use lnmp_sfe::SemanticDictionary;
use std::path::Path;

// Load from file
let dict = SemanticDictionary::load_from_file(Path::new("dictionary.yaml")).unwrap();

// Get field name
assert_eq!(dict.get_field_name(12), Some("user_id"));

// Get equivalence
assert_eq!(dict.get_equivalence(7, "yes"), Some("1"));

// Create programmatically
let mut dict = SemanticDictionary::new();
dict.add_field_name(12, "user_id".to_string());
dict.add_equivalence(7, "yes".to_string(), "1".to_string());
```

---

## lnmp-llb

LNMP-LLM Bridge Layer - prompt optimization, explain mode, and ShortForm encoding.

### Explain Mode

#### ExplainEncoder

```rust
pub struct ExplainEncoder {
    // private fields
}
```

Encodes with human-readable comments for debugging.

**Methods:**

```rust
impl ExplainEncoder {
    /// Creates a new explain encoder
    pub fn new(encoder: Encoder, dictionary: SemanticDictionary) -> Self;
    
    /// Encodes with inline explanations
    pub fn encode_with_explanation(&self, record: &LnmpRecord) -> String;
}
```

**Examples:**

```rust
use lnmp_llb::ExplainEncoder;
use lnmp_codec::Encoder;
use lnmp_sfe::SemanticDictionary;
use std::path::Path;

// Load dictionary
let dict = SemanticDictionary::load_from_file(Path::new("dictionary.yaml")).unwrap();

// Create explain encoder
let encoder = Encoder::new();
let explain_encoder = ExplainEncoder::new(encoder, dict);

// Encode with explanations
let output = explain_encoder.encode_with_explanation(&record);
// Output:
// F12:i=14532  # user_id
// F7:b=1       # is_active
// F23:sa=[admin,dev]  # roles
```

### ShortForm

#### ShortFormEncoder

```rust
pub struct ShortFormEncoder {
    // private fields
}
```

Encodes in ShortForm format for extreme token reduction.

**Methods:**

```rust
impl ShortFormEncoder {
    /// Creates a new ShortForm encoder
    pub fn new(config: ShortFormConfig) -> Self;
    
    /// Encodes in ShortForm format
    pub fn encode(&self, record: &LnmpRecord) -> String;
}
```

**Examples:**

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

#### ShortFormParser

```rust
pub struct ShortFormParser {
    // private fields
}
```

Parses ShortForm input.

**Methods:**

```rust
impl ShortFormParser {
    /// Creates a new ShortForm parser
    pub fn new(config: ShortFormConfig) -> Self;
    
    /// Parses ShortForm input
    pub fn parse(&mut self, input: &str) -> Result<LnmpRecord, LnmpError>;
}
```

**Examples:**

```rust
use lnmp_llb::{ShortFormParser, ShortFormConfig};

let config = ShortFormConfig::default();
let mut parser = ShortFormParser::new(config);

let input = "12=14532 7=1 23=[admin,dev]";
let record = parser.parse(input).unwrap();
```

#### ShortFormConfig

```rust
pub struct ShortFormConfig {
    pub omit_prefix: bool,
    pub omit_type_hints: bool,
    pub minimal_whitespace: bool,
}
```

**Examples:**

```rust
use lnmp_llb::ShortFormConfig;

// Default configuration
let config = ShortFormConfig::default();

// Custom configuration
let config = ShortFormConfig {
    omit_prefix: true,
    omit_type_hints: true,
    minimal_whitespace: true,
};
```

### Prompt Optimization

#### PromptOptimizer

```rust
pub struct PromptOptimizer {
    // private fields
}
```

Optimizes encoding for LLM tokenization efficiency.

**Methods:**

```rust
impl PromptOptimizer {
    /// Creates a new prompt optimizer
    pub fn new(config: PromptOptConfig) -> Self;
    
    /// Optimizes field encoding for tokenization
    pub fn optimize_field(&self, field: &LnmpField) -> String;
    
    /// Optimizes array encoding
    pub fn optimize_array(&self, arr: &[String]) -> String;
}
```

**Examples:**

```rust
use lnmp_llb::{PromptOptimizer, PromptOptConfig};

let config = PromptOptConfig {
    minimize_symbols: true,
    align_token_boundaries: true,
    optimize_arrays: true,
};

let optimizer = PromptOptimizer::new(config);
let optimized = optimizer.optimize_field(&field);
```

#### PromptOptConfig

```rust
pub struct PromptOptConfig {
    pub minimize_symbols: bool,
    pub align_token_boundaries: bool,
    pub optimize_arrays: bool,
}
```

---

## Error Handling

### LnmpError

```rust
pub enum LnmpError {
    // Lexical errors
    InvalidCharacter { char: char, line: usize, column: usize },
    UnterminatedString { line: usize, column: usize },
    
    // Syntactic errors
    UnexpectedToken { expected: String, found: String, line: usize, column: usize },
    InvalidFieldId { value: String, line: usize, column: usize },
    
    // Semantic errors
    TypeHintMismatch { field_id: FieldId, expected_type: String, actual_value: String },
    ChecksumMismatch { field_id: FieldId, expected: u32, actual: u32 },
    
    // Structural errors
    NestingTooDeep { max_depth: usize, actual_depth: usize },
    InvalidNestedStructure { reason: String },
    
    // Strict mode violations
    StrictModeViolation { reason: String },
}
```

**Methods:**

```rust
impl LnmpError {
    /// Adds context to error
    pub fn with_context(self, context: ErrorContext) -> Self;
    
    /// Formats error with source snippet
    pub fn format_with_source(&self) -> String;
}
```

**Examples:**

```rust
use lnmp_codec::Parser;

let input = "F12=invalid";
match Parser::new(input) {
    Ok(mut parser) => {
        match parser.parse_record() {
            Ok(record) => println!("Success"),
            Err(e) => eprintln!("Parse error: {}", e.format_with_source()),
        }
    }
    Err(e) => eprintln!("Lexer error: {}", e),
}
```

---

## Complete Example

```rust
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
use lnmp_codec::{Parser, Encoder, EncoderConfig, NormalizationConfig, EquivalenceMapper};
use lnmp_sfe::SemanticDictionary;
use lnmp_llb::ExplainEncoder;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
    record.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });
    
    // Configure encoder with checksums and normalization
    let mut mapper = EquivalenceMapper::new();
    mapper.add_mapping(7, "yes".to_string(), "1".to_string());
    
    let config = EncoderConfig {
        canonical: true,
        include_type_hints: true,
        enable_checksums: true,
        normalization_config: Some(NormalizationConfig::default()),
        equivalence_mapper: Some(mapper),
    };
    
    // Encode
    let encoder = Encoder::with_config(config);
    let output = encoder.encode(&record);
    println!("Encoded: {}", output);
    
    // Parse
    let mut parser = Parser::new(&output)?;
    let parsed_record = parser.parse_record()?;
    
    // Explain mode
    let dict = SemanticDictionary::load_from_file(Path::new("dictionary.yaml"))?;
    let explain_encoder = ExplainEncoder::new(Encoder::new(), dict);
    let explained = explain_encoder.encode_with_explanation(&parsed_record);
    println!("Explained:\n{}", explained);
    
    Ok(())
}
```

---

## Additional Resources

- **Examples**: See `examples/` directory for complete working examples
- **Specification**: See `.kiro/specs/lnmp-v0.3-semantic-fidelity/` for formal requirements and design
- **Grammar**: See `spec/grammar.md` for PEG/EBNF specification
- **Migration Guide**: See `MIGRATION.md` for version migration instructions
