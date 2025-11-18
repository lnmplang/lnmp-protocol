//! Value normalization system for semantic equivalence.
//!
//! This module provides value normalization to ensure semantically equivalent values
//! produce identical checksums. Normalization rules include:
//! - Boolean: Convert all representations (true/false, yes/no, 1/0) to canonical form
//! - Float: Convert -0.0 to 0.0, remove trailing zeros
//! - String: Apply case transformation based on configuration

use lnmp_core::LnmpValue;
use lnmp_sfe::SemanticDictionary;

/// String case transformation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringCaseRule {
    /// Convert to lowercase
    Lower,
    /// Convert to uppercase
    Upper,
    /// No case transformation
    #[default]
    None,
}

// Default implementation derived via #[derive(Default)]

/// Configuration for value normalization
#[derive(Debug, Clone)]
pub struct NormalizationConfig {
    /// String case transformation rule
    pub string_case: StringCaseRule,
    /// Optional decimal precision for floats
    pub float_precision: Option<usize>,
    /// Whether to remove trailing zeros from floats
    pub remove_trailing_zeros: bool,
    /// Optional semantic dictionary for equivalence normalization
    pub semantic_dictionary: Option<SemanticDictionary>,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            string_case: StringCaseRule::None,
            float_precision: None,
            remove_trailing_zeros: true,
            semantic_dictionary: None,
        }
    }
}

/// Value normalizer for semantic equivalence
#[derive(Debug)]
pub struct ValueNormalizer {
    config: NormalizationConfig,
}

impl ValueNormalizer {
    /// Creates a new normalizer with the given configuration
    pub fn new(config: NormalizationConfig) -> Self {
        Self { config }
    }

    /// Normalizes a value to its canonical form (no field context).
    pub fn normalize(&self, value: &LnmpValue) -> LnmpValue {
        self.normalize_with_fid(None, value)
    }

    /// Normalizes a value with field context for dictionary-based mapping.
    pub fn normalize_with_fid(&self, fid: Option<u16>, value: &LnmpValue) -> LnmpValue {
        match value {
            LnmpValue::Int(i) => LnmpValue::Int(*i),
            LnmpValue::Float(f) => LnmpValue::Float(self.normalize_float(*f)),
            LnmpValue::Bool(b) => LnmpValue::Bool(*b),
            LnmpValue::String(s) => LnmpValue::String(self.normalize_string_for(fid, s)),
            LnmpValue::StringArray(arr) => {
                LnmpValue::StringArray(
                    arr.iter()
                        .map(|s| self.normalize_string_for(fid, s))
                        .collect(),
                )
            }
            LnmpValue::NestedRecord(record) => LnmpValue::NestedRecord(record.clone()),
            LnmpValue::NestedArray(records) => LnmpValue::NestedArray(records.clone()),
        }
    }

    /// Normalizes boolean representations to canonical form
    ///
    /// Converts common boolean representations:
    /// - "true", "yes", "1" → true
    /// - "false", "no", "0" → false
    pub fn normalize_bool(&self, value: &str) -> Option<bool> {
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Some(true),
            "false" | "no" | "0" => Some(false),
            _ => None,
        }
    }

    /// Normalizes float representations
    ///
    /// - Converts -0.0 to 0.0
    /// - Removes trailing zeros after decimal point (if configured)
    /// - Applies precision rounding (if configured)
    fn normalize_float(&self, f: f64) -> f64 {
        // Convert -0.0 to 0.0
        let mut normalized = if f == 0.0 { 0.0 } else { f };

        // Apply precision if configured
        if let Some(precision) = self.config.float_precision {
            let multiplier = 10_f64.powi(precision as i32);
            normalized = (normalized * multiplier).round() / multiplier;
        }

        normalized
    }

    /// Normalizes string representations
    ///
    /// Applies case transformation based on configuration
    fn normalize_string_for(&self, fid: Option<u16>, s: &str) -> String {
        if let (Some(dict), Some(fid)) = (&self.config.semantic_dictionary, fid) {
            if let Some(eq) = dict.get_equivalence(fid, s) {
                return eq.to_string();
            }
            if let Some(eq) = dict.get_equivalence_normalized(fid, s) {
                return eq.to_string();
            }
        }

        match self.config.string_case {
            StringCaseRule::Lower => s.to_lowercase(),
            StringCaseRule::Upper => s.to_uppercase(),
            StringCaseRule::None => s.to_string(),
        }
    }

    /// Formats a normalized float as a string with trailing zeros removed
    pub fn format_float(&self, f: f64) -> String {
        if !self.config.remove_trailing_zeros {
            return f.to_string();
        }

        let s = f.to_string();

        // If there's no decimal point, return as-is
        if !s.contains('.') {
            return s;
        }

        // Remove trailing zeros after decimal point
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}

impl Default for ValueNormalizer {
    fn default() -> Self {
        Self::new(NormalizationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NormalizationConfig::default();
        assert_eq!(config.string_case, StringCaseRule::None);
        assert_eq!(config.float_precision, None);
        assert!(config.remove_trailing_zeros);
    }

    #[test]
    fn test_normalize_int() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::Int(42);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Int(42));
    }

    #[test]
    fn test_normalize_bool() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::Bool(true);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Bool(true));
    }

    #[test]
    fn test_normalize_bool_from_string() {
        let normalizer = ValueNormalizer::default();

        assert_eq!(normalizer.normalize_bool("true"), Some(true));
        assert_eq!(normalizer.normalize_bool("True"), Some(true));
        assert_eq!(normalizer.normalize_bool("TRUE"), Some(true));
        assert_eq!(normalizer.normalize_bool("yes"), Some(true));
        assert_eq!(normalizer.normalize_bool("Yes"), Some(true));
        assert_eq!(normalizer.normalize_bool("1"), Some(true));

        assert_eq!(normalizer.normalize_bool("false"), Some(false));
        assert_eq!(normalizer.normalize_bool("False"), Some(false));
        assert_eq!(normalizer.normalize_bool("FALSE"), Some(false));
        assert_eq!(normalizer.normalize_bool("no"), Some(false));
        assert_eq!(normalizer.normalize_bool("No"), Some(false));
        assert_eq!(normalizer.normalize_bool("0"), Some(false));

        assert_eq!(normalizer.normalize_bool("invalid"), None);
        assert_eq!(normalizer.normalize_bool(""), None);
    }

    #[test]
    fn test_normalize_float_negative_zero() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::Float(-0.0);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Float(0.0));
    }

    #[test]
    fn test_normalize_float_positive_zero() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::Float(0.0);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Float(0.0));
    }

    #[test]
    fn test_normalize_float_regular() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::Float(3.14);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_normalize_float_with_precision() {
        let config = NormalizationConfig {
            string_case: StringCaseRule::None,
            float_precision: Some(2),
            remove_trailing_zeros: true,
            semantic_dictionary: None,
        };
        let normalizer = ValueNormalizer::new(config);

        let value = LnmpValue::Float(3.14159);
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_format_float_remove_trailing_zeros() {
        let normalizer = ValueNormalizer::default();

        assert_eq!(normalizer.format_float(3.140), "3.14");
        assert_eq!(normalizer.format_float(3.100), "3.1");
        assert_eq!(normalizer.format_float(3.000), "3");
        assert_eq!(normalizer.format_float(3.14), "3.14");
        assert_eq!(normalizer.format_float(0.0), "0");
    }

    #[test]
    fn test_format_float_keep_trailing_zeros() {
        let config = NormalizationConfig {
            string_case: StringCaseRule::None,
            float_precision: None,
            remove_trailing_zeros: false,
            semantic_dictionary: None,
        };
        let normalizer = ValueNormalizer::new(config);

        let formatted = normalizer.format_float(3.14);
        assert!(formatted.starts_with("3.14"));
    }

    #[test]
    fn test_normalize_string_no_case() {
        let normalizer = ValueNormalizer::default();
        let value = LnmpValue::String("Test".to_string());
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::String("Test".to_string()));
    }

    #[test]
    fn test_normalize_string_lowercase() {
        let config = NormalizationConfig {
            string_case: StringCaseRule::Lower,
            float_precision: None,
            remove_trailing_zeros: true,
            semantic_dictionary: None,
        };
        let normalizer = ValueNormalizer::new(config);

        let value = LnmpValue::String("TeSt".to_string());
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::String("test".to_string()));
    }

    #[test]
    fn test_normalize_string_uppercase() {
        let config = NormalizationConfig {
            string_case: StringCaseRule::Upper,
            float_precision: None,
            remove_trailing_zeros: true,
            semantic_dictionary: None,
        };
        let normalizer = ValueNormalizer::new(config);

        let value = LnmpValue::String("TeSt".to_string());
        let normalized = normalizer.normalize(&value);
        assert_eq!(normalized, LnmpValue::String("TEST".to_string()));
    }

    #[test]
    fn test_normalize_string_array() {
        let config = NormalizationConfig {
            string_case: StringCaseRule::Lower,
            float_precision: None,
            remove_trailing_zeros: true,
            semantic_dictionary: None,
        };
        let normalizer = ValueNormalizer::new(config);

        let value = LnmpValue::StringArray(vec![
            "Admin".to_string(),
            "Developer".to_string(),
            "USER".to_string(),
        ]);
        let normalized = normalizer.normalize(&value);

        assert_eq!(
            normalized,
            LnmpValue::StringArray(vec![
                "admin".to_string(),
                "developer".to_string(),
                "user".to_string(),
            ])
        );
    }

    #[test]
    fn test_normalize_nested_record() {
        use lnmp_core::{LnmpField, LnmpRecord};

        let normalizer = ValueNormalizer::default();

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let value = LnmpValue::NestedRecord(Box::new(record.clone()));
        let normalized = normalizer.normalize(&value);

        // Nested records are not modified by normalization
        assert_eq!(normalized, LnmpValue::NestedRecord(Box::new(record)));
    }

    #[test]
    fn test_normalize_nested_array() {
        use lnmp_core::{LnmpField, LnmpRecord};

        let normalizer = ValueNormalizer::default();

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let value = LnmpValue::NestedArray(vec![record.clone()]);
        let normalized = normalizer.normalize(&value);

        // Nested arrays are not modified by normalization
        assert_eq!(normalized, LnmpValue::NestedArray(vec![record]));
    }

    #[test]
    fn test_string_case_rule_default() {
        assert_eq!(StringCaseRule::default(), StringCaseRule::None);
    }
}
