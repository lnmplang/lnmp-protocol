//! Prompt visibility optimization for LLM tokenization efficiency.
//!
//! This module provides optimization strategies to maximize LLM prompt visibility
//! and token efficiency when encoding LNMP data.

use lnmp_core::{LnmpField, LnmpValue};

/// Configuration for prompt visibility optimization
#[derive(Debug, Clone)]
pub struct PromptOptConfig {
    /// Minimize unnecessary symbols (e.g., quotes, whitespace)
    pub minimize_symbols: bool,

    /// Optimize patterns to align with common tokenizer boundaries
    pub align_token_boundaries: bool,

    /// Use compact format for arrays
    pub optimize_arrays: bool,
}

impl Default for PromptOptConfig {
    fn default() -> Self {
        Self {
            minimize_symbols: true,
            align_token_boundaries: true,
            optimize_arrays: true,
        }
    }
}

/// Prompt visibility optimizer for LNMP encoding
pub struct PromptOptimizer {
    config: PromptOptConfig,
}

impl PromptOptimizer {
    /// Creates a new prompt optimizer with the given configuration
    pub fn new(config: PromptOptConfig) -> Self {
        Self { config }
    }

    /// Creates a new prompt optimizer with default configuration
    /// This is available through the `Default` trait implementation.
    /// Optimizes field encoding for tokenization efficiency
    ///
    /// This method applies various optimization strategies based on the configuration:
    /// - Symbol minimization: removes unnecessary quotes and whitespace
    /// - Token alignment: formats values to align with common tokenizer boundaries
    /// - Array optimization: uses compact list format for string arrays
    ///
    /// # Arguments
    ///
    /// * `field` - The field to optimize
    ///
    /// # Returns
    ///
    /// An optimized string representation of the field value
    pub fn optimize_field(&self, field: &LnmpField) -> String {
        match &field.value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Float(f) => self.optimize_float(*f),
            LnmpValue::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            LnmpValue::String(s) => self.optimize_string(s),
            LnmpValue::StringArray(arr) => self.optimize_array(arr),
            LnmpValue::Embedding(_) => {
                String::new() // Embeddings are not text, so they don't contribute to prompts
            }
            LnmpValue::EmbeddingDelta(_) => {
                String::new() // Deltas are not text
            }
            LnmpValue::NestedRecord(_) | LnmpValue::NestedArray(_) => {
                // Nested structures are handled by the encoder
                // This is a placeholder for future optimization
                String::new()
            }
        }
    }

    /// Optimizes array encoding for tokenization efficiency
    ///
    /// This method creates a compact, zero-ambiguity list format for string arrays:
    /// - Removes quotes when strings don't contain special characters
    /// - Uses comma separators without spaces
    /// - Maintains bracket delimiters for clarity
    ///
    /// # Arguments
    ///
    /// * `arr` - The string array to optimize
    ///
    /// # Returns
    ///
    /// An optimized string representation of the array
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_llb::{PromptOptimizer, PromptOptConfig};
    ///
    /// let optimizer = PromptOptimizer::default();
    /// let arr = vec!["admin".to_string(), "developer".to_string(), "user".to_string()];
    /// let result = optimizer.optimize_array(&arr);
    /// assert_eq!(result, "[admin,developer,user]");
    /// ```
    pub fn optimize_array(&self, arr: &[String]) -> String {
        if !self.config.optimize_arrays {
            // Standard format with quotes
            let items: Vec<String> = arr
                .iter()
                .map(|s| format!("\"{}\"", escape_string(s)))
                .collect();
            return format!("[{}]", items.join(","));
        }

        // Optimized format: remove quotes when safe
        let items: Vec<String> = arr
            .iter()
            .map(|s| {
                if needs_quotes(s) {
                    format!("\"{}\"", escape_string(s))
                } else {
                    s.clone()
                }
            })
            .collect();

        if self.config.minimize_symbols {
            // No spaces after commas
            format!("[{}]", items.join(","))
        } else {
            // Standard spacing
            format!("[{}]", items.join(", "))
        }
    }

    /// Optimizes string encoding
    fn optimize_string(&self, s: &str) -> String {
        if !self.config.minimize_symbols || needs_quotes(s) {
            format!("\"{}\"", escape_string(s))
        } else {
            s.to_string()
        }
    }

    /// Optimizes float encoding
    fn optimize_float(&self, f: f64) -> String {
        if self.config.align_token_boundaries {
            // Remove trailing zeros and unnecessary decimal points
            let s = format!("{}", f);
            if s.contains('.') {
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            } else {
                s
            }
        } else {
            format!("{}", f)
        }
    }
}

impl Default for PromptOptimizer {
    fn default() -> Self {
        Self::new(PromptOptConfig::default())
    }
}

/// Checks if a string needs quotes in LNMP format
///
/// A string needs quotes if it:
/// - Is empty
/// - Contains whitespace
/// - Contains special characters (comma, semicolon, brackets, quotes, etc.)
/// - Starts with a digit (to avoid ambiguity with numbers)
fn needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    // Check if starts with digit
    if s.chars().next().unwrap().is_ascii_digit() {
        return true;
    }

    // Check for special characters
    for ch in s.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' | ',' | ';' | '[' | ']' | '{' | '}' | '"' | '\\' | '#'
            | '=' => {
                return true;
            }
            _ => {}
        }
    }

    false
}

/// Escapes special characters in a string for LNMP format
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    // FieldId import removed - not required in these tests

    #[test]
    fn test_optimize_field_int() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        };
        assert_eq!(optimizer.optimize_field(&field), "14532");
    }

    #[test]
    fn test_optimize_field_negative_int() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::Int(-42),
        };
        assert_eq!(optimizer.optimize_field(&field), "-42");
    }

    #[test]
    fn test_optimize_field_float() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14),
        };
        assert_eq!(optimizer.optimize_field(&field), "3.14");
    }

    #[test]
    fn test_optimize_field_float_trailing_zeros() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.140),
        };
        let result = optimizer.optimize_field(&field);
        // Should remove trailing zeros
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_optimize_field_float_whole_number() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 2,
            value: LnmpValue::Float(5.0),
        };
        let result = optimizer.optimize_field(&field);
        // Should remove decimal point for whole numbers
        assert_eq!(result, "5");
    }

    #[test]
    fn test_optimize_field_bool_true() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        };
        assert_eq!(optimizer.optimize_field(&field), "1");
    }

    #[test]
    fn test_optimize_field_bool_false() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 7,
            value: LnmpValue::Bool(false),
        };
        assert_eq!(optimizer.optimize_field(&field), "0");
    }

    #[test]
    fn test_optimize_field_string_simple() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("admin".to_string()),
        };
        // Simple alphanumeric string doesn't need quotes
        assert_eq!(optimizer.optimize_field(&field), "admin");
    }

    #[test]
    fn test_optimize_field_string_with_spaces() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("hello world".to_string()),
        };
        // String with spaces needs quotes
        assert_eq!(optimizer.optimize_field(&field), "\"hello world\"");
    }

    #[test]
    fn test_optimize_field_string_with_special_chars() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("test,value".to_string()),
        };
        // String with comma needs quotes
        assert_eq!(optimizer.optimize_field(&field), "\"test,value\"");
    }

    #[test]
    fn test_optimize_field_string_empty() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("".to_string()),
        };
        // Empty string needs quotes
        assert_eq!(optimizer.optimize_field(&field), "\"\"");
    }

    #[test]
    fn test_optimize_array_simple() {
        let optimizer = PromptOptimizer::default();
        let arr = vec![
            "admin".to_string(),
            "developer".to_string(),
            "user".to_string(),
        ];
        assert_eq!(optimizer.optimize_array(&arr), "[admin,developer,user]");
    }

    #[test]
    fn test_optimize_array_with_spaces() {
        let optimizer = PromptOptimizer::default();
        let arr = vec!["hello world".to_string(), "test".to_string()];
        assert_eq!(optimizer.optimize_array(&arr), "[\"hello world\",test]");
    }

    #[test]
    fn test_optimize_array_empty() {
        let optimizer = PromptOptimizer::default();
        let arr: Vec<String> = vec![];
        assert_eq!(optimizer.optimize_array(&arr), "[]");
    }

    #[test]
    fn test_optimize_array_single_element() {
        let optimizer = PromptOptimizer::default();
        let arr = vec!["admin".to_string()];
        assert_eq!(optimizer.optimize_array(&arr), "[admin]");
    }

    #[test]
    fn test_optimize_array_with_special_chars() {
        let optimizer = PromptOptimizer::default();
        let arr = vec!["test,value".to_string(), "normal".to_string()];
        assert_eq!(optimizer.optimize_array(&arr), "[\"test,value\",normal]");
    }

    #[test]
    fn test_optimize_array_disabled() {
        let config = PromptOptConfig {
            minimize_symbols: false,
            align_token_boundaries: false,
            optimize_arrays: false,
        };
        let optimizer = PromptOptimizer::new(config);
        let arr = vec!["admin".to_string(), "user".to_string()];
        // Should use standard format with quotes
        assert_eq!(optimizer.optimize_array(&arr), "[\"admin\",\"user\"]");
    }

    #[test]
    fn test_optimize_array_with_spacing() {
        let config = PromptOptConfig {
            minimize_symbols: false,
            align_token_boundaries: true,
            optimize_arrays: true,
        };
        let optimizer = PromptOptimizer::new(config);
        let arr = vec!["admin".to_string(), "user".to_string()];
        // Should include spaces after commas
        assert_eq!(optimizer.optimize_array(&arr), "[admin, user]");
    }

    #[test]
    fn test_needs_quotes_simple() {
        assert!(!needs_quotes("admin"));
        assert!(!needs_quotes("user123"));
        assert!(!needs_quotes("test_value"));
        assert!(!needs_quotes("my-value"));
    }

    #[test]
    fn test_needs_quotes_empty() {
        assert!(needs_quotes(""));
    }

    #[test]
    fn test_needs_quotes_whitespace() {
        assert!(needs_quotes("hello world"));
        assert!(needs_quotes("test\tvalue"));
        assert!(needs_quotes("line\nbreak"));
    }

    #[test]
    fn test_needs_quotes_special_chars() {
        assert!(needs_quotes("test,value"));
        assert!(needs_quotes("test;value"));
        assert!(needs_quotes("test[value"));
        assert!(needs_quotes("test]value"));
        assert!(needs_quotes("test{value"));
        assert!(needs_quotes("test}value"));
        assert!(needs_quotes("test\"value"));
        assert!(needs_quotes("test\\value"));
        assert!(needs_quotes("test#value"));
        assert!(needs_quotes("test=value"));
    }

    #[test]
    fn test_needs_quotes_starts_with_digit() {
        assert!(needs_quotes("123abc"));
        assert!(needs_quotes("0test"));
        assert!(!needs_quotes("abc123"));
    }

    #[test]
    fn test_escape_string_no_escapes() {
        assert_eq!(escape_string("admin"), "admin");
        assert_eq!(escape_string("hello world"), "hello world");
    }

    #[test]
    fn test_escape_string_backslash() {
        assert_eq!(escape_string("test\\value"), "test\\\\value");
    }

    #[test]
    fn test_escape_string_quote() {
        assert_eq!(escape_string("test\"value"), "test\\\"value");
    }

    #[test]
    fn test_escape_string_newline() {
        assert_eq!(escape_string("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn test_escape_string_carriage_return() {
        assert_eq!(escape_string("line\rbreak"), "line\\rbreak");
    }

    #[test]
    fn test_escape_string_tab() {
        assert_eq!(escape_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_escape_string_multiple() {
        assert_eq!(
            escape_string("test\"with\\escapes\n"),
            "test\\\"with\\\\escapes\\n"
        );
    }

    #[test]
    fn test_config_default() {
        let config = PromptOptConfig::default();
        assert!(config.minimize_symbols);
        assert!(config.align_token_boundaries);
        assert!(config.optimize_arrays);
    }

    #[test]
    fn test_config_custom() {
        let config = PromptOptConfig {
            minimize_symbols: false,
            align_token_boundaries: true,
            optimize_arrays: false,
        };
        assert!(!config.minimize_symbols);
        assert!(config.align_token_boundaries);
        assert!(!config.optimize_arrays);
    }

    #[test]
    fn test_optimizer_with_custom_config() {
        let config = PromptOptConfig {
            minimize_symbols: false,
            align_token_boundaries: false,
            optimize_arrays: false,
        };
        let optimizer = PromptOptimizer::new(config);

        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("admin".to_string()),
        };
        // With minimize_symbols=false, should add quotes
        assert_eq!(optimizer.optimize_field(&field), "\"admin\"");
    }

    #[test]
    fn test_optimize_field_string_array() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "developer".to_string()]),
        };
        assert_eq!(optimizer.optimize_field(&field), "[admin,developer]");
    }

    #[test]
    fn test_optimize_string_with_underscore() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("user_name".to_string()),
        };
        assert_eq!(optimizer.optimize_field(&field), "user_name");
    }

    #[test]
    fn test_optimize_string_with_hyphen() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("user-name".to_string()),
        };
        assert_eq!(optimizer.optimize_field(&field), "user-name");
    }

    #[test]
    fn test_optimize_string_with_dot() {
        let optimizer = PromptOptimizer::default();
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("file.txt".to_string()),
        };
        assert_eq!(optimizer.optimize_field(&field), "file.txt");
    }
}
