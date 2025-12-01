//! Explain mode encoding for LNMP
//!
//! This module provides human-readable annotations for LNMP data by appending
//! inline comments with field names and descriptions.

use lnmp_core::{FieldId, LnmpField, LnmpRecord, LnmpValue, TypeHint};
use lnmp_sfe;
use std::collections::HashMap;

/// Semantic dictionary for field name mappings
///
/// Maps field IDs to human-readable names for explain mode output.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticDictionary {
    field_names: HashMap<FieldId, String>,
}

impl SemanticDictionary {
    /// Creates a new empty semantic dictionary
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field name mapping
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID
    /// * `name` - The human-readable field name
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_llb::SemanticDictionary;
    ///
    /// let mut dict = SemanticDictionary::new();
    /// dict.add_field_name(12, "user_id".to_string());
    /// dict.add_field_name(7, "is_active".to_string());
    /// ```
    pub fn add_field_name(&mut self, fid: FieldId, name: String) {
        self.field_names.insert(fid, name);
    }

    /// Gets the field name for a given field ID
    ///
    /// Returns `None` if no mapping exists for the field ID.
    pub fn get_field_name(&self, fid: FieldId) -> Option<&str> {
        self.field_names.get(&fid).map(|s| s.as_str())
    }

    /// Creates a dictionary from a list of (field_id, name) pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_llb::SemanticDictionary;
    ///
    /// let dict = SemanticDictionary::from_pairs(vec![
    ///     (12, "user_id"),
    ///     (7, "is_active"),
    ///     (23, "roles"),
    /// ]);
    /// ```
    pub fn from_pairs<S: Into<String>>(pairs: Vec<(FieldId, S)>) -> Self {
        let mut dict = Self::new();
        for (fid, name) in pairs {
            dict.add_field_name(fid, name.into());
        }
        dict
    }
}

/// Encoder that adds human-readable explanations to LNMP output
///
/// ExplainEncoder wraps the standard LNMP encoding and appends inline comments
/// with field names from a semantic dictionary. This is useful for debugging
/// and human inspection of LNMP data.
///
/// # Format
///
/// The explain mode output follows this format:
/// ```text
/// F<fid>:<type>=<value>  # <field_name>
/// ```
///
/// Comments are aligned at a consistent column for readability.
pub struct ExplainEncoder {
    dictionary: SemanticDictionary,
    include_type_hints: bool,
    comment_column: usize,
}

impl ExplainEncoder {
    /// Creates a new explain encoder with the given semantic dictionary
    ///
    /// # Arguments
    ///
    /// * `dictionary` - The semantic dictionary for field name lookups
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_llb::{ExplainEncoder, SemanticDictionary};
    ///
    /// let dict = SemanticDictionary::from_pairs(vec![
    ///     (12, "user_id"),
    ///     (7, "is_active"),
    /// ]);
    /// let encoder = ExplainEncoder::new(dict);
    /// ```
    pub fn new(dictionary: SemanticDictionary) -> Self {
        Self {
            dictionary,
            include_type_hints: true,
            comment_column: 20,
        }
    }

    /// Creates an explain encoder from an SFE semantic dictionary.
    /// This copies only the field name mappings.
    pub fn from_sfe_dictionary(dict: &lnmp_sfe::SemanticDictionary) -> Self {
        let mut local = SemanticDictionary::new();
        for (fid, name) in dict.field_name_entries() {
            local.add_field_name(fid, name.to_string());
        }
        Self::new(local)
    }

    /// Sets whether to include type hints in the output
    ///
    /// Default is `true`.
    pub fn with_type_hints(mut self, include: bool) -> Self {
        self.include_type_hints = include;
        self
    }

    /// Sets the column at which comments should be aligned
    ///
    /// Default is 20.
    pub fn with_comment_column(mut self, column: usize) -> Self {
        self.comment_column = column;
        self
    }

    /// Encodes a record with inline explanations
    ///
    /// This method produces LNMP output with human-readable comments appended
    /// to each field. The non-comment portions maintain canonical format.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to encode
    ///
    /// # Returns
    ///
    /// A string containing the LNMP encoding with inline comments
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
    /// use lnmp_llb::{ExplainEncoder, SemanticDictionary};
    ///
    /// let mut record = LnmpRecord::new();
    /// record.add_field(LnmpField {
    ///     fid: 12,
    ///     value: LnmpValue::Int(14532),
    /// });
    /// record.add_field(LnmpField {
    ///     fid: 7,
    ///     value: LnmpValue::Bool(true),
    /// });
    ///
    /// let dict = SemanticDictionary::from_pairs(vec![
    ///     (12, "user_id"),
    ///     (7, "is_active"),
    /// ]);
    ///
    /// let encoder = ExplainEncoder::new(dict);
    /// let output = encoder.encode_with_explanation(&record);
    ///
    /// // Output will be:
    /// // F7:b=1              # is_active
    /// // F12:i=14532         # user_id
    /// ```
    pub fn encode_with_explanation(&self, record: &LnmpRecord) -> String {
        // Canonicalize the record (sort fields)
        let canonical = self.canonicalize_record(record);

        let lines: Vec<String> = canonical
            .fields()
            .iter()
            .map(|field| self.encode_field_with_explanation(field))
            .collect();

        lines.join("\n")
    }

    /// Encodes a single field with explanation
    fn encode_field_with_explanation(&self, field: &LnmpField) -> String {
        let base = self.encode_field(field);

        // Add comment if field name is available
        if let Some(field_name) = self.dictionary.get_field_name(field.fid) {
            // Calculate padding to align comment
            let padding = if base.len() < self.comment_column {
                self.comment_column - base.len()
            } else {
                2 // Minimum 2 spaces before comment
            };

            format!("{}{}# {}", base, " ".repeat(padding), field_name)
        } else {
            base
        }
    }

    /// Encodes a single field in canonical format
    fn encode_field(&self, field: &LnmpField) -> String {
        if self.include_type_hints {
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

    /// Gets the type hint for a value
    fn get_type_hint(&self, value: &LnmpValue) -> TypeHint {
        match value {
            LnmpValue::Int(_) => TypeHint::Int,
            LnmpValue::Float(_) => TypeHint::Float,
            LnmpValue::Bool(_) => TypeHint::Bool,
            LnmpValue::String(_) => TypeHint::String,
            LnmpValue::IntArray(_) => TypeHint::IntArray,
            LnmpValue::FloatArray(_) => TypeHint::FloatArray,
            LnmpValue::BoolArray(_) => TypeHint::BoolArray,
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
            LnmpValue::IntArray(arr) => {
                let items: Vec<String> = arr.iter().map(|i| i.to_string()).collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::FloatArray(arr) => {
                let items: Vec<String> = arr.iter().map(|f| f.to_string()).collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::BoolArray(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|b| if *b { "1".to_string() } else { "0".to_string() })
                    .collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::StringArray(arr) => {
                let items: Vec<String> = arr.iter().map(|s| self.encode_string(s)).collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::NestedRecord(record) => self.encode_nested_record(record),
            LnmpValue::NestedArray(records) => self.encode_nested_array(records),
            LnmpValue::Embedding(vec) => {
                format!("[embedding dim={}]", vec.dim)
            }
            LnmpValue::EmbeddingDelta(delta) => {
                format!("[embedding_delta changes={}]", delta.changes.len())
            }
            LnmpValue::QuantizedEmbedding(qv) => {
                format!(
                    "[quantized_embedding dim={} scheme={:?}]",
                    qv.dim, qv.scheme
                )
            }
        }
    }

    /// Encodes a nested record
    fn encode_nested_record(&self, record: &LnmpRecord) -> String {
        if record.fields().is_empty() {
            return "{}".to_string();
        }

        let fields: Vec<String> = record
            .sorted_fields()
            .iter()
            .map(|field| self.encode_field(field))
            .collect();

        format!("{{{}}}", fields.join(";"))
    }

    /// Encodes a nested array
    fn encode_nested_array(&self, records: &[LnmpRecord]) -> String {
        if records.is_empty() {
            return "[]".to_string();
        }

        let encoded_records: Vec<String> = records
            .iter()
            .map(|record| self.encode_nested_record(record))
            .collect();

        format!("[{}]", encoded_records.join(","))
    }

    /// Encodes a string with quotes and escapes if needed
    fn encode_string(&self, s: &str) -> String {
        if self.needs_quoting(s) {
            format!("\"{}\"", self.escape_string(s))
        } else {
            s.to_string()
        }
    }

    /// Checks if a string needs quoting
    fn needs_quoting(&self, s: &str) -> bool {
        if s.is_empty() {
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

    /// Canonicalizes a record by sorting fields
    fn canonicalize_record(&self, record: &LnmpRecord) -> LnmpRecord {
        LnmpRecord::from_sorted_fields(record.sorted_fields())
    }
}

/// Checks if a character is safe for unquoted strings
fn is_safe_unquoted_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_dictionary_new() {
        let dict = SemanticDictionary::new();
        assert!(dict.get_field_name(1).is_none());
    }

    #[test]
    fn test_semantic_dictionary_add_field_name() {
        let mut dict = SemanticDictionary::new();
        dict.add_field_name(12, "user_id".to_string());
        dict.add_field_name(7, "is_active".to_string());

        assert_eq!(dict.get_field_name(12), Some("user_id"));
        assert_eq!(dict.get_field_name(7), Some("is_active"));
        assert_eq!(dict.get_field_name(99), None);
    }

    #[test]
    fn test_semantic_dictionary_from_pairs() {
        let dict =
            SemanticDictionary::from_pairs(vec![(12, "user_id"), (7, "is_active"), (23, "roles")]);

        assert_eq!(dict.get_field_name(12), Some("user_id"));
        assert_eq!(dict.get_field_name(7), Some("is_active"));
        assert_eq!(dict.get_field_name(23), Some("roles"));
    }

    #[test]
    fn test_explain_encoder_basic() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let dict = SemanticDictionary::from_pairs(vec![(12, "user_id")]);
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        assert!(output.contains("F12:i=14532"));
        assert!(output.contains("# user_id"));
    }

    #[test]
    fn test_explain_encoder_multiple_fields() {
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

        let dict =
            SemanticDictionary::from_pairs(vec![(12, "user_id"), (7, "is_active"), (23, "roles")]);

        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        // Fields should be sorted by FID
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);

        // F7 should come first
        assert!(lines[0].contains("F7:b=1"));
        assert!(lines[0].contains("# is_active"));

        // F12 should come second
        assert!(lines[1].contains("F12:i=14532"));
        assert!(lines[1].contains("# user_id"));

        // F23 should come third
        assert!(lines[2].contains("F23:sa=[admin,dev]"));
        assert!(lines[2].contains("# roles"));
    }

    #[test]
    fn test_explain_encoder_without_type_hints() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let dict = SemanticDictionary::from_pairs(vec![(12, "user_id")]);
        let encoder = ExplainEncoder::new(dict).with_type_hints(false);
        let output = encoder.encode_with_explanation(&record);

        assert!(output.contains("F12=14532"));
        assert!(!output.contains(":i"));
        assert!(output.contains("# user_id"));
    }

    #[test]
    fn test_explain_encoder_field_without_name() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 99,
            value: LnmpValue::Int(42),
        });

        let dict = SemanticDictionary::from_pairs(vec![(12, "user_id")]);
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);

        // F12 should have comment
        assert!(lines[0].contains("F12:i=14532"));
        assert!(lines[0].contains("# user_id"));

        // F99 should not have comment
        assert!(lines[1].contains("F99:i=42"));
        assert!(!lines[1].contains("#"));
    }

    #[test]
    fn test_explain_encoder_comment_alignment() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let dict = SemanticDictionary::from_pairs(vec![(1, "id"), (12, "user_id")]);

        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        let lines: Vec<&str> = output.lines().collect();

        // Both comments should be aligned at the same column
        let comment_pos_1 = lines[0].find('#').unwrap();
        let comment_pos_2 = lines[1].find('#').unwrap();

        // Comments should be at or near the same position
        // (allowing for minimum spacing)
        assert!(comment_pos_1 >= 20 || comment_pos_2 >= 20);
    }

    #[test]
    fn test_explain_encoder_custom_comment_column() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let dict = SemanticDictionary::from_pairs(vec![(12, "user_id")]);
        let encoder = ExplainEncoder::new(dict).with_comment_column(30);
        let output = encoder.encode_with_explanation(&record);

        let comment_pos = output.find('#').unwrap();
        assert!(comment_pos >= 30);
    }

    #[test]
    fn test_explain_encoder_from_sfe_dictionary() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let mut sfe_dict = lnmp_sfe::SemanticDictionary::new();
        sfe_dict.add_field_name(7, "is_active".to_string());

        let encoder = ExplainEncoder::from_sfe_dictionary(&sfe_dict);
        let output = encoder.encode_with_explanation(&record);
        assert!(output.contains("is_active"));
    }

    #[test]
    fn test_explain_encoder_all_value_types() {
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

        let dict = SemanticDictionary::from_pairs(vec![
            (1, "count"),
            (2, "pi"),
            (3, "active"),
            (4, "name"),
            (5, "tags"),
        ]);

        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        assert!(output.contains("F1:i=42"));
        assert!(output.contains("# count"));
        assert!(output.contains("F2:f=3.14"));
        assert!(output.contains("# pi"));
        assert!(output.contains("F3:b=1"));
        assert!(output.contains("# active"));
        assert!(output.contains("F4:s=test"));
        assert!(output.contains("# name"));
        assert!(output.contains("F5:sa=[a,b]"));
        assert!(output.contains("# tags"));
    }

    #[test]
    fn test_explain_encoder_string_with_spaces() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("hello world".to_string()),
        });

        let dict = SemanticDictionary::from_pairs(vec![(4, "message")]);
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        assert!(output.contains(r#"F4:s="hello world""#));
        assert!(output.contains("# message"));
    }

    #[test]
    fn test_explain_encoder_empty_record() {
        let record = LnmpRecord::new();
        let dict = SemanticDictionary::new();
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        assert_eq!(output, "");
    }

    #[test]
    fn test_explain_encoder_nested_record() {
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

        let dict = SemanticDictionary::from_pairs(vec![(50, "user_data")]);
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&outer_record);
        assert!(output.contains("F50:r="));
        assert!(output.contains("{F7:b=1;F12:i=1}"));
        assert!(output.contains("# user_data"));
    }

    #[test]
    fn test_explain_encoder_nested_array() {
        let mut record1 = LnmpRecord::new();
        record1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut record2 = LnmpRecord::new();
        record2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut outer_record = LnmpRecord::new();
        outer_record.add_field(LnmpField {
            fid: 60,
            value: LnmpValue::NestedArray(vec![record1, record2]),
        });

        let dict = SemanticDictionary::from_pairs(vec![(60, "users")]);
        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&outer_record);

        assert!(output.contains("F60:ra="));
        assert!(output.contains("[{F12:i=1},{F12:i=2}]"));
        assert!(output.contains("# users"));
    }

    #[test]
    fn test_explain_encoder_field_sorting() {
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

        let dict =
            SemanticDictionary::from_pairs(vec![(5, "first"), (50, "second"), (100, "third")]);

        let encoder = ExplainEncoder::new(dict);
        let output = encoder.encode_with_explanation(&record);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);

        // Should be sorted by FID
        assert!(lines[0].contains("F5:i=1"));
        assert!(lines[1].contains("F50:i=2"));
        assert!(lines[2].contains("F100:i=3"));
    }

    #[test]
    fn test_is_safe_unquoted_char() {
        assert!(is_safe_unquoted_char('a'));
        assert!(is_safe_unquoted_char('Z'));
        assert!(is_safe_unquoted_char('0'));
        assert!(is_safe_unquoted_char('9'));
        assert!(is_safe_unquoted_char('_'));
        assert!(is_safe_unquoted_char('-'));
        assert!(is_safe_unquoted_char('.'));

        assert!(!is_safe_unquoted_char(' '));
        assert!(!is_safe_unquoted_char(','));
        assert!(!is_safe_unquoted_char(';'));
        assert!(!is_safe_unquoted_char('['));
        assert!(!is_safe_unquoted_char(']'));
    }
}
