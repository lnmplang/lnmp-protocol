//! Encoder for converting structured records into LNMP text format.

use crate::config::EncoderConfig;
use lnmp_core::checksum::SemanticChecksum;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue, TypeHint};

/// Encodes a quantized embedding into compact text format
///
/// Format: `QV[scheme,scale,zp,min,hex_data]`
/// Example: `QV[QInt8,0.001568,0,-0.5,a1b2c3d4...]`
fn encode_quantized_embedding(qv: &lnmp_quant::QuantizedVector) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(32 + qv.data.len() * 2);

    // Format: QV[scheme,scale,zero_point,min_val,data_hex]
    write!(
        &mut result,
        "QV[{:?},{},{},{},",
        qv.scheme, qv.scale, qv.zero_point, qv.min_val
    )
    .unwrap();

    // Append hex-encoded data
    for byte in &qv.data {
        write!(&mut result, "{:02x}", byte).unwrap();
    }

    result.push(']');
    result
}

/// Encoder for LNMP text format
pub struct Encoder {
    use_semicolons: bool,
    config: EncoderConfig,
    normalizer: Option<crate::normalizer::ValueNormalizer>,
}

impl Encoder {
    /// Creates a new encoder with default settings (canonical format)
    pub fn new() -> Self {
        Self {
            // Canonical format uses newlines between top-level fields
            use_semicolons: false,
            config: EncoderConfig::default(),
            normalizer: None,
        }
    }

    /// Creates a new encoder with custom configuration
    pub fn with_config(config: EncoderConfig) -> Self {
        let normalizer = config.semantic_dictionary.as_ref().map(|dict| {
            crate::normalizer::ValueNormalizer::new(crate::normalizer::NormalizationConfig {
                semantic_dictionary: Some(dict.clone()),
                ..crate::normalizer::NormalizationConfig::default()
            })
        });

        Self {
            // If canonical is enabled, prefer newlines; otherwise use semicolons for inline format
            use_semicolons: !config.canonical,
            config,
            normalizer,
        }
    }

    /// Creates a new encoder with specified format (deprecated - use new() for canonical format)
    #[deprecated(
        note = "Use new() for canonical format. Semicolons are not part of v0.2 canonical format."
    )]
    pub fn with_semicolons(use_semicolons: bool) -> Self {
        Self {
            use_semicolons,
            config: EncoderConfig::default(),
            normalizer: None,
        }
    }

    /// Encodes a complete record into LNMP text format (canonical format with sorted fields)
    pub fn encode(&self, record: &LnmpRecord) -> String {
        // Canonicalize the record first (sorts fields and nested structures)
        let canonical = canonicalize_record(record);

        let fields: Vec<String> = canonical
            .fields()
            .iter()
            .map(|field| {
                let normalized_value = if let Some(norm) = &self.normalizer {
                    norm.normalize_with_fid(Some(field.fid), &field.value)
                } else {
                    field.value.clone()
                };
                let normalized_field = LnmpField {
                    fid: field.fid,
                    value: normalized_value,
                };
                self.encode_field(&normalized_field)
            })
            .collect();

        if self.use_semicolons {
            fields.join(";")
        } else {
            fields.join("\n")
        }
    }

    /// Encodes a single field (F<fid>=<value> or F<fid>:<type>=<value> or with checksum)
    fn encode_field(&self, field: &LnmpField) -> String {
        let type_hint = if self.config.include_type_hints {
            Some(self.get_type_hint(&field.value))
        } else {
            None
        };

        let base = if let Some(hint) = type_hint {
            format!(
                "F{}:{}={}",
                field.fid,
                hint.as_str(),
                self.encode_value(&field.value)
            )
        } else {
            format!("F{}={}", field.fid, self.encode_value(&field.value))
        };

        // Append checksum if enabled
        if self.config.enable_checksums {
            let checksum = SemanticChecksum::compute(field.fid, type_hint, &field.value);
            let checksum_str = SemanticChecksum::format(checksum);
            format!("{}#{}", base, checksum_str)
        } else {
            base
        }
    }

    /// Gets the type hint for a value
    fn get_type_hint(&self, value: &LnmpValue) -> TypeHint {
        match value {
            LnmpValue::Int(_) => TypeHint::Int,
            LnmpValue::Float(_) => TypeHint::Float,
            LnmpValue::Bool(_) => TypeHint::Bool,
            LnmpValue::String(_) => TypeHint::String,
            LnmpValue::StringArray(_) => TypeHint::StringArray,
            LnmpValue::NestedRecord(_) => TypeHint::Record,
            LnmpValue::NestedArray(_) => TypeHint::RecordArray,
            LnmpValue::Embedding(_) => TypeHint::Embedding,
            LnmpValue::EmbeddingDelta(_) => TypeHint::Embedding,
            LnmpValue::QuantizedEmbedding(_) => TypeHint::QuantizedEmbedding,
        }
    }

    /// Encodes a value based on its type
    fn encode_value(&self, value: &LnmpValue) -> String {
        match value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Float(f) => f.to_string(),
            LnmpValue::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            LnmpValue::String(s) => self.encode_string(s),
            LnmpValue::StringArray(arr) => {
                // Canonical format: no spaces after commas
                let items: Vec<String> = arr.iter().map(|s| self.encode_string(s)).collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::NestedRecord(record) => self.encode_nested_record(record),
            LnmpValue::NestedArray(records) => self.encode_nested_array(records),
            LnmpValue::Embedding(vec) => {
                // Text format representation for embeddings is not yet standardized.
                // We use a placeholder format that indicates the dimension.
                format!("[vector dim={}]", vec.dim)
            }
            LnmpValue::EmbeddingDelta(delta) => {
                // Text format representation for embedding deltas is not yet standardized.
                // We use a placeholder format that indicates the number of changes.
                format!("[vector_delta changes={}]", delta.changes.len())
            }
            LnmpValue::QuantizedEmbedding(qv) => {
                // Compact text format: QV[scheme,scale,zp,min,hex_data]
                encode_quantized_embedding(qv)
            }
        }
    }

    /// Encodes a nested record {F<fid>=<value>;F<fid>=<value>}
    fn encode_nested_record(&self, record: &LnmpRecord) -> String {
        if record.fields().is_empty() {
            return "{}".to_string();
        }

        let fields: Vec<String> = record
            .fields()
            .iter()
            .map(|field| self.encode_field_without_checksum(field))
            .collect();

        // Nested records always use semicolons as separators
        format!("{{{}}}", fields.join(";"))
    }

    /// Encodes a field without checksum (for use in nested structures)
    fn encode_field_without_checksum(&self, field: &LnmpField) -> String {
        if self.config.include_type_hints {
            let type_hint = self.get_type_hint(&field.value);
            format!(
                "F{}:{}={}",
                field.fid,
                type_hint.as_str(),
                self.encode_value(&field.value)
            )
        } else {
            format!("F{}={}", field.fid, self.encode_value(&field.value))
        }
    }

    /// Encodes a nested array [{...},{...}]
    fn encode_nested_array(&self, records: &[LnmpRecord]) -> String {
        if records.is_empty() {
            return "[]".to_string();
        }

        let encoded_records: Vec<String> = records
            .iter()
            .map(|record| self.encode_nested_record(record))
            .collect();

        // Canonical format: no spaces after commas
        format!("[{}]", encoded_records.join(","))
    }

    /// Encodes a string, adding quotes and escapes if needed
    fn encode_string(&self, s: &str) -> String {
        if self.needs_quoting(s) {
            format!("\"{}\"", self.escape_string(s))
        } else {
            s.to_string()
        }
    }

    /// Checks if a string needs quoting
    fn needs_quoting(&self, s: &str) -> bool {
        if s.is_empty() || looks_like_literal(s) {
            return true;
        }

        for ch in s.chars() {
            if !is_safe_unquoted_char(ch) {
                return true;
            }
        }

        false
    }

    /// Escapes special characters in a string
    fn escape_string(&self, s: &str) -> String {
        let mut result = String::new();
        for ch in s.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(ch),
            }
        }
        result
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonicalizes a record by recursively sorting fields and normalizing nested structures
///
/// This function ensures deterministic encoding by:
/// - Sorting fields by FID at every nesting level (depth-first)
/// - Recursively canonicalizing nested records and arrays
/// - Omitting redundant empty fields (empty strings, empty arrays, empty nested structures)
/// - Maintaining structural integrity
pub fn canonicalize_record(record: &LnmpRecord) -> LnmpRecord {
    let mut canonical = LnmpRecord::new();

    // Sort fields by FID (stable sort preserves insertion order for duplicates)
    let sorted = record.sorted_fields();

    for field in sorted {
        let canonical_value = canonicalize_value(&field.value);

        // Omit redundant empty fields
        if !is_empty_value(&canonical_value) {
            canonical.add_field(LnmpField {
                fid: field.fid,
                value: canonical_value,
            });
        }
    }

    canonical
}

/// Canonicalizes a value by recursively processing nested structures
fn canonicalize_value(value: &LnmpValue) -> LnmpValue {
    match value {
        // Primitive values are already canonical
        LnmpValue::Int(i) => LnmpValue::Int(*i),
        LnmpValue::Float(f) => LnmpValue::Float(*f),
        LnmpValue::Bool(b) => LnmpValue::Bool(*b),
        LnmpValue::String(s) => LnmpValue::String(s.clone()),
        LnmpValue::StringArray(arr) => LnmpValue::StringArray(arr.clone()),

        // Recursively canonicalize nested record
        LnmpValue::NestedRecord(nested) => {
            let canonical_nested = canonicalize_record(nested);
            LnmpValue::NestedRecord(Box::new(canonical_nested))
        }

        // Recursively canonicalize each record in nested array
        LnmpValue::NestedArray(arr) => {
            let canonical_arr: Vec<LnmpRecord> = arr.iter().map(canonicalize_record).collect();
            LnmpValue::NestedArray(canonical_arr)
        }
        // Embeddings are already canonical (binary data)
        LnmpValue::Embedding(vec) => LnmpValue::Embedding(vec.clone()),
        LnmpValue::EmbeddingDelta(delta) => LnmpValue::EmbeddingDelta(delta.clone()),
        LnmpValue::QuantizedEmbedding(qv) => LnmpValue::QuantizedEmbedding(qv.clone()),
    }
}

/// Checks if a value is considered "empty" and should be omitted during canonicalization
///
/// Empty values include:
/// - Empty strings
/// - Empty string arrays
/// - Empty nested records (records with no fields)
/// - Empty nested arrays (arrays with no elements)
fn is_empty_value(value: &LnmpValue) -> bool {
    match value {
        LnmpValue::String(s) => s.is_empty(),
        LnmpValue::StringArray(arr) => arr.is_empty(),
        LnmpValue::NestedRecord(record) => record.fields().is_empty(),
        LnmpValue::NestedArray(arr) => arr.is_empty(),
        // Embeddings are never considered empty even if dimension is 0 (which shouldn't happen)
        LnmpValue::Embedding(_) => false,
        LnmpValue::EmbeddingDelta(_) => false,
        LnmpValue::QuantizedEmbedding(_) => false,
        // Non-empty primitive values are never considered empty
        LnmpValue::Int(_) | LnmpValue::Float(_) | LnmpValue::Bool(_) => false,
    }
}

/// Validates that canonicalization is idempotent (round-trip stable)
///
/// Verifies that canonicalize(canonicalize(x)) == canonicalize(x)
/// This ensures that the canonicalization process is stable and deterministic.
///
/// Returns true if the record is round-trip stable, false otherwise.
pub fn validate_round_trip_stability(record: &LnmpRecord) -> bool {
    let canonical_once = canonicalize_record(record);
    let canonical_twice = canonicalize_record(&canonical_once);
    canonical_once == canonical_twice
}

/// Checks if a character is safe for unquoted strings
fn is_safe_unquoted_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'
}

fn looks_like_literal(s: &str) -> bool {
    if s.trim().is_empty() {
        return true;
    }

    let lower = s.to_ascii_lowercase();
    matches!(lower.as_str(), "true" | "false" | "yes" | "no")
        || s.parse::<i64>().is_ok()
        || s.parse::<f64>().is_ok()
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    use lnmp_core::LnmpField;

    #[test]
    fn test_encode_integer() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F1=42");
    }

    #[test]
    fn test_encode_negative_integer() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(-123),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F1=-123");
    }

    #[test]
    fn test_encode_float() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F2=3.14");
    }

    #[test]
    fn test_encode_bool_true() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(true),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F3=1");
    }

    #[test]
    fn test_encode_bool_false() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(false),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F3=0");
    }

    #[test]
    fn test_encode_unquoted_string() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("test_value".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F4=test_value");
    }

    #[test]
    fn test_encode_quoted_string() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("hello world".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, r#"F4="hello world""#);
    }

    #[test]
    fn test_encode_string_with_escapes() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("hello \"world\"".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, r#"F4="hello \"world\"""#);
    }

    #[test]
    fn test_encode_string_with_newline() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("line1\nline2".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, r#"F4="line1\nline2""#);
    }

    #[test]
    fn test_encode_string_with_backslash() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("back\\slash".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, r#"F4="back\\slash""#);
    }

    #[test]
    fn test_encode_string_array() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec![
                "admin".to_string(),
                "dev".to_string(),
                "user".to_string(),
            ]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F5=[admin,dev,user]");
    }

    #[test]
    fn test_encode_string_array_with_quoted_strings() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["hello world".to_string(), "test".to_string()]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, r#"F5=["hello world",test]"#);
    }

    #[test]
    fn test_encode_empty_string_array() {
        // Empty string arrays are omitted during canonicalization (Requirement 9.3)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec![]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, ""); // Empty field is omitted
    }

    #[test]
    fn test_encode_multiline_format() {
        let mut record = LnmpRecord::new();
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

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        // Canonical format: fields are sorted by FID
        assert_eq!(output, "F7=1\nF12=14532\nF20=Halil");
    }

    #[test]
    #[allow(deprecated)]
    fn test_encode_inline_format() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let encoder = Encoder::with_semicolons(true);
        let output = encoder.encode(&record);
        // Fields are sorted even with semicolons
        assert_eq!(output, "F7=1;F12=14532;F23=[admin,dev]");
    }

    #[test]
    fn test_encode_empty_record() {
        let record = LnmpRecord::new();
        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "");
    }

    #[test]
    #[allow(deprecated)]
    fn test_encode_spec_example() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let encoder = Encoder::with_semicolons(true);
        let output = encoder.encode(&record);
        // Fields are sorted by FID
        assert_eq!(output, "F7=1;F12=14532;F23=[admin,dev]");
    }

    #[test]
    fn test_needs_quoting() {
        let encoder = Encoder::new();

        assert!(!encoder.needs_quoting("simple"));
        assert!(!encoder.needs_quoting("test_value"));
        assert!(!encoder.needs_quoting("file-name"));
        assert!(!encoder.needs_quoting("version1.2.3"));

        assert!(encoder.needs_quoting("hello world"));
        assert!(encoder.needs_quoting("test@example"));
        assert!(encoder.needs_quoting(""));
        assert!(encoder.needs_quoting("test;value"));
    }

    #[test]
    fn test_escape_string() {
        let encoder = Encoder::new();

        assert_eq!(encoder.escape_string("simple"), "simple");
        assert_eq!(
            encoder.escape_string("hello \"world\""),
            r#"hello \"world\""#
        );
        assert_eq!(encoder.escape_string("back\\slash"), r#"back\\slash"#);
        assert_eq!(encoder.escape_string("line1\nline2"), r#"line1\nline2"#);
        assert_eq!(encoder.escape_string("tab\there"), r#"tab\there"#);
        assert_eq!(encoder.escape_string("return\rhere"), r#"return\rhere"#);
    }

    #[test]
    #[allow(deprecated)]
    fn test_round_trip() {
        use crate::parser::Parser;

        let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let encoder = Encoder::with_semicolons(true);
        let output = encoder.encode(&record);

        // Parse again to verify
        let mut parser2 = Parser::new(&output).unwrap();
        let record2 = parser2.parse_record().unwrap();

        assert_eq!(record.fields().len(), record2.fields().len());
        assert_eq!(
            record.get_field(12).unwrap().value,
            record2.get_field(12).unwrap().value
        );
        assert_eq!(
            record.get_field(7).unwrap().value,
            record2.get_field(7).unwrap().value
        );
        assert_eq!(
            record.get_field(23).unwrap().value,
            record2.get_field(23).unwrap().value
        );
    }

    #[test]
    fn test_deterministic_field_sorting() {
        // Create record with fields in random order
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(2),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // Fields should be sorted by FID
        assert_eq!(output, "F5=1\nF50=2\nF100=3");
    }

    #[test]
    fn test_deterministic_sorting_with_duplicates() {
        // Test that stable sort preserves insertion order for duplicate FIDs
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("first".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("second".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // F5 first, then both F10s in insertion order
        assert_eq!(output, "F5=1\nF10=first\nF10=second");
    }

    #[test]
    fn test_canonical_whitespace_formatting() {
        // Test that there's no whitespace around equals signs
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // No spaces around =
        assert_eq!(output, "F1=42");
        assert!(!output.contains(" = "));
        assert!(!output.contains("= "));
        assert!(!output.contains(" ="));
    }

    #[test]
    fn test_array_formatting_no_spaces() {
        // Test that arrays have no spaces after commas
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // No spaces after commas in array
        assert_eq!(output, "F1=[one,two,three]");
        assert!(!output.contains(", "));
    }

    #[test]
    fn test_encoding_with_type_hints() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Float(3.14),
        });

        let config = EncoderConfig::new().with_type_hints(true);
        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Fields sorted with type hints
        assert_eq!(output, "F5:f=3.14\nF7:b=1\nF12:i=14532");
    }

    #[test]
    fn test_encoding_without_type_hints() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let config = EncoderConfig::new();
        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Fields sorted without type hints
        assert_eq!(output, "F7=1\nF12=14532");
        assert!(!output.contains(':'));
    }

    #[test]
    fn test_all_type_hints() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("test".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        });

        let config = EncoderConfig::new().with_type_hints(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        assert_eq!(output, "F1:i=42\nF2:f=3.14\nF3:b=1\nF4:s=test\nF5:sa=[a,b]");
    }

    #[test]
    fn test_multiple_encode_cycles_identical() {
        // Test that encoding multiple times produces identical output
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let encoder = Encoder::new();
        let output1 = encoder.encode(&record);
        let output2 = encoder.encode(&record);
        let output3 = encoder.encode(&record);

        assert_eq!(output1, output2);
        assert_eq!(output2, output3);
    }

    #[test]
    fn test_canonicalize_record_basic() {
        // Test basic field sorting
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(2),
        });

        let canonical = canonicalize_record(&record);
        let fields = canonical.fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].fid, 5);
        assert_eq!(fields[1].fid, 50);
        assert_eq!(fields[2].fid, 100);
    }

    #[test]
    fn test_canonicalize_record_with_nested_record() {
        // Test nested record canonicalization
        let mut inner_record = LnmpRecord::new();
        inner_record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        inner_record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let mut outer_record = LnmpRecord::new();
        outer_record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner_record)),
        });
        outer_record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("test".to_string()),
        });

        let canonical = canonicalize_record(&outer_record);
        let fields = canonical.fields();

        // Outer fields should be sorted
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].fid, 10);
        assert_eq!(fields[1].fid, 50);

        // Inner fields should also be sorted
        if let LnmpValue::NestedRecord(nested) = &fields[1].value {
            let nested_fields = nested.fields();
            assert_eq!(nested_fields.len(), 2);
            assert_eq!(nested_fields[0].fid, 7);
            assert_eq!(nested_fields[1].fid, 12);
        } else {
            panic!("Expected nested record");
        }
    }

    #[test]
    fn test_canonicalize_record_with_nested_array() {
        // Test nested array canonicalization
        let mut record1 = LnmpRecord::new();
        record1.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });
        record1.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });

        let mut record2 = LnmpRecord::new();
        record2.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(4),
        });
        record2.add_field(LnmpField {
            fid: 15,
            value: LnmpValue::Int(3),
        });

        let mut outer_record = LnmpRecord::new();
        outer_record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![record1, record2]),
        });

        let canonical = canonicalize_record(&outer_record);
        let fields = canonical.fields();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].fid, 60);

        // Each record in the array should have sorted fields
        if let LnmpValue::NestedArray(arr) = &fields[0].value {
            assert_eq!(arr.len(), 2);

            let arr_fields1 = arr[0].fields();
            assert_eq!(arr_fields1[0].fid, 10);
            assert_eq!(arr_fields1[1].fid, 20);

            let arr_fields2 = arr[1].fields();
            assert_eq!(arr_fields2[0].fid, 15);
            assert_eq!(arr_fields2[1].fid, 30);
        } else {
            panic!("Expected nested array");
        }
    }

    #[test]
    fn test_canonicalize_deeply_nested_structure() {
        // Test 3-level deep nesting
        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Int(3),
        });
        level3.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });
        level2.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(2),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });
        level1.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::String("test".to_string()),
        });

        let canonical = canonicalize_record(&level1);
        let fields = canonical.fields();

        // Level 1 should be sorted
        assert_eq!(fields[0].fid, 50);
        assert_eq!(fields[1].fid, 100);

        // Level 2 should be sorted
        if let LnmpValue::NestedRecord(level2_rec) = &fields[1].value {
            let level2_fields = level2_rec.fields();
            assert_eq!(level2_fields[0].fid, 10);
            assert_eq!(level2_fields[1].fid, 20);

            // Level 3 should be sorted
            if let LnmpValue::NestedRecord(level3_rec) = &level2_fields[1].value {
                let level3_fields = level3_rec.fields();
                assert_eq!(level3_fields[0].fid, 1);
                assert_eq!(level3_fields[1].fid, 3);
            } else {
                panic!("Expected level 3 nested record");
            }
        } else {
            panic!("Expected level 2 nested record");
        }
    }

    #[test]
    fn test_canonicalize_empty_record() {
        let record = LnmpRecord::new();
        let canonical = canonicalize_record(&record);
        assert_eq!(canonical.fields().len(), 0);
    }

    #[test]
    fn test_canonicalize_empty_nested_structures() {
        // Empty nested structures should be omitted during canonicalization (Requirement 9.3)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(LnmpRecord::new())),
        });
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![]),
        });

        let canonical = canonicalize_record(&record);
        let fields = canonical.fields();

        // Empty nested structures should be omitted
        assert_eq!(fields.len(), 0);
    }

    #[test]
    fn test_canonicalize_preserves_values() {
        // Test that canonicalization doesn't modify values
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14159),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("test value".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        });

        let canonical = canonicalize_record(&record);

        assert_eq!(canonical.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(
            canonical.get_field(2).unwrap().value,
            LnmpValue::Float(3.14159)
        );
        assert_eq!(canonical.get_field(3).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            canonical.get_field(4).unwrap().value,
            LnmpValue::String("test value".to_string())
        );
        assert_eq!(
            canonical.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_canonicalize_idempotent() {
        // Test that canonicalizing twice produces the same result
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(2),
        });

        let canonical1 = canonicalize_record(&record);
        let canonical2 = canonicalize_record(&canonical1);

        assert_eq!(canonical1, canonical2);
    }

    #[test]
    fn test_canonicalize_mixed_nested_structures() {
        // Test record with both nested records and nested arrays
        let mut inner_record = LnmpRecord::new();
        inner_record.add_field(LnmpField {
            fid: 15,
            value: LnmpValue::Int(5),
        });
        inner_record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(3),
        });

        let mut array_record = LnmpRecord::new();
        array_record.add_field(LnmpField {
            fid: 25,
            value: LnmpValue::Int(7),
        });
        array_record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(6),
        });

        let mut outer_record = LnmpRecord::new();
        outer_record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedRecord(Box::new(inner_record)),
        });
        outer_record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedArray(vec![array_record]),
        });
        outer_record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("test".to_string()),
        });

        let canonical = canonicalize_record(&outer_record);
        let fields = canonical.fields();

        // Outer fields sorted
        assert_eq!(fields[0].fid, 10);
        assert_eq!(fields[1].fid, 50);
        assert_eq!(fields[2].fid, 100);

        // Nested array fields sorted
        if let LnmpValue::NestedArray(arr) = &fields[1].value {
            let arr_fields = arr[0].fields();
            assert_eq!(arr_fields[0].fid, 20);
            assert_eq!(arr_fields[1].fid, 25);
        } else {
            panic!("Expected nested array");
        }

        // Nested record fields sorted
        if let LnmpValue::NestedRecord(nested) = &fields[2].value {
            let nested_fields = nested.fields();
            assert_eq!(nested_fields[0].fid, 5);
            assert_eq!(nested_fields[1].fid, 15);
        } else {
            panic!("Expected nested record");
        }
    }

    // Nested structure encoding tests
    #[test]
    fn test_encode_simple_nested_record() {
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        inner.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // Fields should be sorted within nested record
        assert_eq!(output, "F50={F7=1;F12=1}");
    }

    #[test]
    fn test_encode_empty_nested_record() {
        // Empty nested records are omitted during canonicalization (Requirement 9.3)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(LnmpRecord::new())),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, ""); // Empty field is omitted
    }

    #[test]
    fn test_encode_nested_record_with_various_types() {
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        inner.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        inner.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // Fields sorted: F7, F12, F23
        assert_eq!(output, "F50={F7=1;F12=14532;F23=[admin,dev]}");
    }

    #[test]
    fn test_encode_deeply_nested_record() {
        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("deep".to_string()),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&level1);
        assert_eq!(output, "F3={F2={F1=deep}}");
    }

    #[test]
    fn test_encode_simple_nested_array() {
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut rec3 = LnmpRecord::new();
        rec3.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(3),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![rec1, rec2, rec3]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F60=[{F12=1},{F12=2},{F12=3}]");
    }

    #[test]
    fn test_encode_empty_nested_array() {
        // Empty nested arrays are omitted during canonicalization (Requirement 9.3)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, ""); // Empty field is omitted
    }

    #[test]
    fn test_encode_nested_array_with_multiple_fields() {
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("alice".to_string()),
        });
        rec1.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("admin".to_string()),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("bob".to_string()),
        });
        rec2.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("user".to_string()),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::NestedArray(vec![rec1, rec2]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F200=[{F1=alice;F2=admin},{F1=bob;F2=user}]");
    }

    #[test]
    fn test_encode_nested_array_preserves_order() {
        // Requirement 5.3: Element order must be preserved
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("first".to_string()),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("second".to_string()),
        });

        let mut rec3 = LnmpRecord::new();
        rec3.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("third".to_string()),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![rec1, rec2, rec3]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        assert_eq!(output, "F60=[{F1=first},{F1=second},{F1=third}]");
    }

    #[test]
    fn test_encode_nested_array_fields_sorted() {
        // Fields within each nested record should be sorted
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });
        rec1.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![rec1]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        // Fields should be sorted: F10, F20
        assert_eq!(output, "F60=[{F10=1;F20=2}]");
    }

    #[test]
    fn test_encode_mixed_nested_structures() {
        // Test record with both nested record and nested array
        let mut inner_record = LnmpRecord::new();
        inner_record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("nested".to_string()),
        });

        let mut array_rec = LnmpRecord::new();
        array_rec.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(42),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner_record)),
        });
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![array_rec]),
        });
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("top".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);
        // Fields sorted: F10, F50, F60
        assert_eq!(output, "F10=top\nF50={F1=nested}\nF60=[{F2=42}]");
    }

    #[test]
    fn test_encode_nested_record_with_type_hints() {
        use crate::config::EncoderConfig;

        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        inner.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let config = EncoderConfig::new().with_type_hints(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Type hints should be included
        assert_eq!(output, "F50:r={F7:b=1;F12:i=14532}");
    }

    #[test]
    fn test_encode_nested_array_with_type_hints() {
        use crate::config::EncoderConfig;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![rec1, rec2]),
        });

        let config = EncoderConfig::new().with_type_hints(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Type hints should be included
        assert_eq!(output, "F60:ra=[{F12:i=1},{F12:i=2}]");
    }

    #[test]
    fn test_round_trip_nested_record() {
        use crate::parser::Parser;

        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        inner.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        let mut parser = Parser::new(&output).unwrap();
        let parsed = parser.parse_record().unwrap();

        // Compare canonical versions since encoder sorts fields
        let canonical_original = canonicalize_record(&record);
        assert_eq!(canonical_original, parsed);
    }

    #[test]
    fn test_round_trip_nested_array() {
        use crate::parser::Parser;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("alice".to_string()),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("bob".to_string()),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::NestedArray(vec![rec1, rec2]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        let mut parser = Parser::new(&output).unwrap();
        let parsed = parser.parse_record().unwrap();

        assert_eq!(record, parsed);
    }

    #[test]
    fn test_round_trip_complex_nested_structure() {
        use crate::parser::Parser;

        // Create a complex structure with multiple levels of nesting
        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("nested".to_string()),
        });
        level2.add_field(LnmpField {
            fid: 11,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });

        let mut array_rec = LnmpRecord::new();
        array_rec.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Bool(true),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });
        record.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::NestedArray(vec![array_rec]),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        let mut parser = Parser::new(&output).unwrap();
        let parsed = parser.parse_record().unwrap();

        assert_eq!(record, parsed);
    }

    // Checksum encoding tests
    #[test]
    fn test_encode_with_checksum() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let config = EncoderConfig::new().with_checksums(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Should include checksum
        assert!(output.contains('#'));
        assert!(output.starts_with("F12=14532#"));

        // Checksum should be 8 hex characters
        let parts: Vec<&str> = output.split('#').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_encode_with_checksum_and_type_hints() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let config = EncoderConfig::new()
            .with_type_hints(true)
            .with_checksums(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Should include both type hint and checksum
        assert!(output.contains(':'));
        assert!(output.contains('#'));
        assert!(output.starts_with("F12:i=14532#"));
    }

    #[test]
    fn test_encode_without_checksum() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let config = EncoderConfig::new();

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Should not include checksum
        assert!(!output.contains('#'));
        assert_eq!(output, "F12=14532");
    }

    #[test]
    fn test_encode_multiple_fields_with_checksums() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let config = EncoderConfig::new()
            .with_type_hints(true)
            .with_checksums(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Each field should have a checksum
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains('#'));
        assert!(lines[1].contains('#'));
    }

    #[test]
    fn test_checksum_deterministic() {
        use crate::config::EncoderConfig;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let config = EncoderConfig::new().with_type_hints(true);

        let encoder = Encoder::with_config(config);

        // Encode multiple times
        let output1 = encoder.encode(&record);
        let output2 = encoder.encode(&record);
        let output3 = encoder.encode(&record);

        // All outputs should be identical
        assert_eq!(output1, output2);
        assert_eq!(output2, output3);
    }

    #[test]
    fn test_encode_nested_record_with_checksum() {
        use crate::config::EncoderConfig;

        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let config = EncoderConfig::new()
            .with_type_hints(true)
            .with_checksums(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Should include checksum for the nested record field
        assert!(output.contains('#'));
        // The nested record value includes type hints for inner fields
        assert!(output.starts_with("F50:r={F12:i=1}#"));

        // Verify checksum is 8 hex characters
        let parts: Vec<&str> = output.split('#').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_encode_nested_array_with_checksum() {
        use crate::config::EncoderConfig;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![rec1]),
        });

        let config = EncoderConfig::new()
            .with_type_hints(true)
            .with_checksums(true);

        let encoder = Encoder::with_config(config);
        let output = encoder.encode(&record);

        // Should include checksum for the nested array field
        assert!(output.contains('#'));
        assert!(output.starts_with("F60:ra=[{F12:i=1}]#"));

        // Verify checksum is 8 hex characters
        let parts: Vec<&str> = output.split('#').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_checksum_different_for_different_values() {
        use crate::config::EncoderConfig;

        let mut record1 = LnmpRecord::new();
        record1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let mut record2 = LnmpRecord::new();
        record2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14533),
        });

        let config = EncoderConfig::new()
            .with_type_hints(true)
            .with_checksums(true);

        let encoder = Encoder::with_config(config);

        let output1 = encoder.encode(&record1);
        let output2 = encoder.encode(&record2);

        // Checksums should be different
        assert_ne!(output1, output2);

        // Extract checksums
        let checksum1 = output1.split('#').nth(1).unwrap();
        let checksum2 = output2.split('#').nth(1).unwrap();
        assert_ne!(checksum1, checksum2);
    }

    // Canonicalization tests for v0.5 requirements

    #[test]
    fn test_canonicalize_field_ordering_multiple_levels() {
        // Test field ordering at multiple nesting levels (Requirement 9.1)
        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(3),
        });
        level3.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });
        level3.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 300,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });
        level2.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(10),
        });
        level2.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::Int(20),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 3000,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });
        level1.add_field(LnmpField {
            fid: 1000,
            value: LnmpValue::Int(100),
        });
        level1.add_field(LnmpField {
            fid: 2000,
            value: LnmpValue::Int(200),
        });

        let canonical = canonicalize_record(&level1);
        let fields = canonical.fields();

        // Level 1 fields should be sorted
        assert_eq!(fields[0].fid, 1000);
        assert_eq!(fields[1].fid, 2000);
        assert_eq!(fields[2].fid, 3000);

        // Level 2 fields should be sorted
        if let LnmpValue::NestedRecord(level2_rec) = &fields[2].value {
            let level2_fields = level2_rec.fields();
            assert_eq!(level2_fields[0].fid, 100);
            assert_eq!(level2_fields[1].fid, 200);
            assert_eq!(level2_fields[2].fid, 300);

            // Level 3 fields should be sorted
            if let LnmpValue::NestedRecord(level3_rec) = &level2_fields[2].value {
                let level3_fields = level3_rec.fields();
                assert_eq!(level3_fields[0].fid, 10);
                assert_eq!(level3_fields[1].fid, 20);
                assert_eq!(level3_fields[2].fid, 30);
            } else {
                panic!("Expected level 3 nested record");
            }
        } else {
            panic!("Expected level 2 nested record");
        }
    }

    #[test]
    fn test_canonicalize_array_record_ordering() {
        // Test that records within nested arrays have sorted fields (Requirement 9.2)
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(5),
        });
        rec1.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });
        rec1.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(3),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 80,
            value: LnmpValue::Int(8),
        });
        rec2.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });
        rec2.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::Int(6),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedArray(vec![rec1, rec2]),
        });

        let canonical = canonicalize_record(&outer);
        let fields = canonical.fields();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].fid, 100);

        // Each record in the array should have sorted fields
        if let LnmpValue::NestedArray(arr) = &fields[0].value {
            assert_eq!(arr.len(), 2);

            // First record fields should be sorted
            let rec1_fields = arr[0].fields();
            assert_eq!(rec1_fields[0].fid, 10);
            assert_eq!(rec1_fields[1].fid, 30);
            assert_eq!(rec1_fields[2].fid, 50);

            // Second record fields should be sorted
            let rec2_fields = arr[1].fields();
            assert_eq!(rec2_fields[0].fid, 20);
            assert_eq!(rec2_fields[1].fid, 60);
            assert_eq!(rec2_fields[2].fid, 80);
        } else {
            panic!("Expected nested array");
        }
    }

    #[test]
    fn test_canonicalize_empty_field_omission() {
        // Test that empty fields are omitted (Requirement 9.3)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::String("".to_string()), // Empty string
        });
        record.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::StringArray(vec![]), // Empty array
        });
        record.add_field(LnmpField {
            fid: 40,
            value: LnmpValue::NestedRecord(Box::new(LnmpRecord::new())), // Empty nested record
        });
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedArray(vec![]), // Empty nested array
        });
        record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::String("not_empty".to_string()),
        });

        let canonical = canonicalize_record(&record);
        let fields = canonical.fields();

        // Only non-empty fields should remain
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].fid, 10);
        assert_eq!(fields[0].value, LnmpValue::Int(42));
        assert_eq!(fields[1].fid, 60);
        assert_eq!(fields[1].value, LnmpValue::String("not_empty".to_string()));
    }

    #[test]
    fn test_canonicalize_empty_field_omission_nested() {
        // Test that empty fields are omitted in nested structures
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        inner.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("".to_string()), // Empty string
        });
        inner.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::String("value".to_string()),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });
        outer.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::StringArray(vec![]), // Empty array
        });

        let canonical = canonicalize_record(&outer);
        let fields = canonical.fields();

        // Only the nested record should remain (empty array omitted)
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].fid, 100);

        // Check nested record has empty field omitted
        if let LnmpValue::NestedRecord(nested) = &fields[0].value {
            let nested_fields = nested.fields();
            assert_eq!(nested_fields.len(), 2); // Only F1 and F3, F2 omitted
            assert_eq!(nested_fields[0].fid, 1);
            assert_eq!(nested_fields[1].fid, 3);
        } else {
            panic!("Expected nested record");
        }
    }

    #[test]
    fn test_canonicalize_round_trip_stability_simple() {
        // Test round-trip stability for simple record (Requirement 9.4)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(2),
        });

        assert!(validate_round_trip_stability(&record));
    }

    #[test]
    fn test_canonicalize_round_trip_stability_nested() {
        // Test round-trip stability for nested structures
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(3),
        });
        inner.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });
        inner.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 300,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });
        outer.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::Int(10),
        });
        outer.add_field(LnmpField {
            fid: 200,
            value: LnmpValue::Int(20),
        });

        assert!(validate_round_trip_stability(&outer));
    }

    #[test]
    fn test_canonicalize_round_trip_stability_deeply_nested() {
        // Test round-trip stability with deeply nested structures (depth 5)
        let mut level5 = LnmpRecord::new();
        level5.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(5),
        });
        level5.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });

        let mut level4 = LnmpRecord::new();
        level4.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::NestedRecord(Box::new(level5)),
        });

        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::NestedRecord(Box::new(level4)),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });

        assert!(validate_round_trip_stability(&level1));
    }

    #[test]
    fn test_canonicalize_round_trip_stability_with_arrays() {
        // Test round-trip stability with nested arrays
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(5),
        });
        rec1.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(1),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 80,
            value: LnmpValue::Int(8),
        });
        rec2.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(2),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedArray(vec![rec1, rec2]),
        });
        outer.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::String("test".to_string()),
        });

        assert!(validate_round_trip_stability(&outer));
    }

    #[test]
    fn test_canonicalize_round_trip_stability_with_empty_fields() {
        // Test round-trip stability with empty fields that should be omitted
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::String("".to_string()), // Empty string
        });
        record.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::StringArray(vec![]), // Empty array
        });

        assert!(validate_round_trip_stability(&record));
    }

    #[test]
    fn test_canonicalize_round_trip_stability_mixed_structures() {
        // Test round-trip stability with mixed nested structures
        let mut inner_record = LnmpRecord::new();
        inner_record.add_field(LnmpField {
            fid: 15,
            value: LnmpValue::Int(5),
        });
        inner_record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(3),
        });

        let mut array_record = LnmpRecord::new();
        array_record.add_field(LnmpField {
            fid: 25,
            value: LnmpValue::Int(7),
        });
        array_record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(6),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 100,
            value: LnmpValue::NestedRecord(Box::new(inner_record)),
        });
        outer.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedArray(vec![array_record]),
        });
        outer.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("test".to_string()),
        });

        assert!(validate_round_trip_stability(&outer));
    }

    #[test]
    fn test_is_empty_value() {
        // Test the is_empty_value helper function
        assert!(is_empty_value(&LnmpValue::String("".to_string())));
        assert!(!is_empty_value(&LnmpValue::String("not_empty".to_string())));

        assert!(is_empty_value(&LnmpValue::StringArray(vec![])));
        assert!(!is_empty_value(&LnmpValue::StringArray(vec![
            "item".to_string()
        ])));

        assert!(is_empty_value(&LnmpValue::NestedRecord(Box::new(
            LnmpRecord::new()
        ))));
        let mut non_empty_record = LnmpRecord::new();
        non_empty_record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        assert!(!is_empty_value(&LnmpValue::NestedRecord(Box::new(
            non_empty_record
        ))));

        assert!(is_empty_value(&LnmpValue::NestedArray(vec![])));
        let mut rec = LnmpRecord::new();
        rec.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        assert!(!is_empty_value(&LnmpValue::NestedArray(vec![rec])));

        // Primitive values are never empty
        assert!(!is_empty_value(&LnmpValue::Int(0)));
        assert!(!is_empty_value(&LnmpValue::Int(42)));
        assert!(!is_empty_value(&LnmpValue::Float(0.0)));
        assert!(!is_empty_value(&LnmpValue::Float(3.14)));
        assert!(!is_empty_value(&LnmpValue::Bool(true)));
        assert!(!is_empty_value(&LnmpValue::Bool(false)));
    }
}
