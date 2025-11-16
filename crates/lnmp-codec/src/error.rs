//! Error types for LNMP parsing and encoding operations.

use crate::lexer::Token;

/// Context information for error reporting with source snippets
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// Line number where the error occurred
    pub line: usize,
    /// Column number where the error occurred
    pub column: usize,
    /// Source snippet showing 3 lines of context around the error
    pub source_snippet: String,
    /// Optional source file name
    pub source_file: Option<String>,
}

impl ErrorContext {
    /// Creates a new error context
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            source_snippet: String::new(),
            source_file: None,
        }
    }

    /// Creates an error context with source snippet
    pub fn with_snippet(line: usize, column: usize, source: &str) -> Self {
        let snippet = Self::extract_snippet(source, line);
        Self {
            line,
            column,
            source_snippet: snippet,
            source_file: None,
        }
    }

    /// Creates an error context with source snippet and file name
    pub fn with_file(line: usize, column: usize, source: &str, file: String) -> Self {
        let snippet = Self::extract_snippet(source, line);
        Self {
            line,
            column,
            source_snippet: snippet,
            source_file: Some(file),
        }
    }

    /// Extracts 3 lines of context around the error line
    fn extract_snippet(source: &str, error_line: usize) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let start = error_line.saturating_sub(2);
        let end = (error_line + 1).min(lines.len());

        lines[start..end].join("\n")
    }
}

/// Error type for LNMP parsing and encoding operations
#[derive(Debug, Clone, PartialEq)]
pub enum LnmpError {
    /// Invalid character encountered during lexical analysis
    InvalidCharacter {
        /// The invalid character
        char: char,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Unterminated string literal
    UnterminatedString {
        /// Line number where the string started
        line: usize,
        /// Column number where the string started
        column: usize,
    },
    /// Unexpected token encountered during parsing
    UnexpectedToken {
        /// What token was expected
        expected: String,
        /// What token was actually found
        found: Token,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Invalid field ID (not a valid u16 or out of range)
    InvalidFieldId {
        /// The invalid value that was encountered
        value: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Invalid value for a field
    InvalidValue {
        /// The field ID where the error occurred
        field_id: u16,
        /// Reason why the value is invalid
        reason: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Unexpected end of file
    UnexpectedEof {
        /// Line number where EOF was encountered
        line: usize,
        /// Column number where EOF was encountered
        column: usize,
    },
    /// Invalid escape sequence in a string
    InvalidEscapeSequence {
        /// The invalid escape sequence
        sequence: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Strict mode violation
    StrictModeViolation {
        /// Description of the violation
        reason: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Type hint mismatch
    TypeHintMismatch {
        /// The field ID where the error occurred
        field_id: u16,
        /// Expected type from hint
        expected_type: String,
        /// Actual value that was parsed
        actual_value: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Invalid type hint
    InvalidTypeHint {
        /// The invalid type hint
        hint: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Checksum mismatch (v0.3)
    ChecksumMismatch {
        /// The field ID where the error occurred
        field_id: u16,
        /// Expected checksum
        expected: String,
        /// Found checksum
        found: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Nesting depth exceeds maximum allowed (v0.3)
    NestingTooDeep {
        /// Maximum allowed nesting depth
        max_depth: usize,
        /// Actual depth that was reached
        actual_depth: usize,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Invalid nested structure (v0.3)
    InvalidNestedStructure {
        /// Description of the structural issue
        reason: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    /// Unclosed nested structure (v0.3)
    UnclosedNestedStructure {
        /// Type of structure ("record" or "array")
        structure_type: String,
        /// Line where structure was opened
        opened_at_line: usize,
        /// Column where structure was opened
        opened_at_column: usize,
        /// Line where error was detected
        line: usize,
        /// Column where error was detected
        column: usize,
    },
}

impl LnmpError {
    /// Adds error context with source snippet
    pub fn with_context(self, _context: ErrorContext) -> Self {
        // For now, we return self as-is since the error already contains line/column
        // In a future enhancement, we could store the context separately
        self
    }

    /// Formats the error with source context for rich error messages
    pub fn format_with_source(&self, source: &str) -> String {
        let (line, column) = self.position();
        
        let error_msg = format!("{}", self);
        let mut output = String::new();
        
        output.push_str(&format!("Error: {}\n", error_msg));
        output.push_str("  |\n");
        
        // Add source lines with line numbers
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(2);
        let end = (line + 1).min(lines.len());
        
        for line_num in start..end {
            let actual_line = line_num + 1; // 1-indexed for display
            let line_content = lines.get(line_num).unwrap_or(&"");
            output.push_str(&format!("{} | {}\n", actual_line, line_content));
            
            // Add error indicator on the error line
            if line_num + 1 == line {
                let spaces = " ".repeat(column.saturating_sub(1));
                output.push_str(&format!("  | {}^ here\n", spaces));
            }
        }
        
        output.push_str("  |\n");
        output
    }

    /// Returns the line and column position of the error
    pub fn position(&self) -> (usize, usize) {
        match self {
            LnmpError::InvalidCharacter { line, column, .. } => (*line, *column),
            LnmpError::UnterminatedString { line, column } => (*line, *column),
            LnmpError::UnexpectedToken { line, column, .. } => (*line, *column),
            LnmpError::InvalidFieldId { line, column, .. } => (*line, *column),
            LnmpError::InvalidValue { line, column, .. } => (*line, *column),
            LnmpError::UnexpectedEof { line, column } => (*line, *column),
            LnmpError::InvalidEscapeSequence { line, column, .. } => (*line, *column),
            LnmpError::StrictModeViolation { line, column, .. } => (*line, *column),
            LnmpError::TypeHintMismatch { line, column, .. } => (*line, *column),
            LnmpError::InvalidTypeHint { line, column, .. } => (*line, *column),
            LnmpError::ChecksumMismatch { line, column, .. } => (*line, *column),
            LnmpError::NestingTooDeep { line, column, .. } => (*line, *column),
            LnmpError::InvalidNestedStructure { line, column, .. } => (*line, *column),
            LnmpError::UnclosedNestedStructure { line, column, .. } => (*line, *column),
        }
    }
}

impl std::fmt::Display for LnmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LnmpError::InvalidCharacter { char, line, column } => write!(
                f,
                "Invalid character '{}' at line {}, column {}",
                char, line, column
            ),
            LnmpError::UnterminatedString { line, column } => write!(
                f,
                "Unterminated string at line {}, column {}",
                line, column
            ),
            LnmpError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => write!(
                f,
                "Unexpected token at line {}, column {}: expected {}, found {:?}",
                line, column, expected, found
            ),
            LnmpError::InvalidFieldId {
                value,
                line,
                column,
            } => write!(
                f,
                "Invalid field ID at line {}, column {}: '{}'",
                line, column, value
            ),
            LnmpError::InvalidValue {
                field_id,
                reason,
                line,
                column,
            } => write!(
                f,
                "Invalid value for field {} at line {}, column {}: {}",
                field_id, line, column, reason
            ),
            LnmpError::UnexpectedEof { line, column } => write!(
                f,
                "Unexpected end of file at line {}, column {}",
                line, column
            ),
            LnmpError::InvalidEscapeSequence {
                sequence,
                line,
                column,
            } => write!(
                f,
                "Invalid escape sequence '{}' at line {}, column {}",
                sequence, line, column
            ),
            LnmpError::StrictModeViolation {
                reason,
                line,
                column,
            } => write!(
                f,
                "Strict mode violation at line {}, column {}: {}",
                line, column, reason
            ),
            LnmpError::TypeHintMismatch {
                field_id,
                expected_type,
                actual_value,
                line,
                column,
            } => write!(
                f,
                "Type hint mismatch for field {} at line {}, column {}: expected type '{}', got {}",
                field_id, line, column, expected_type, actual_value
            ),
            LnmpError::InvalidTypeHint { hint, line, column } => write!(
                f,
                "Invalid type hint '{}' at line {}, column {}",
                hint, line, column
            ),
            LnmpError::ChecksumMismatch {
                field_id,
                expected,
                found,
                line,
                column,
            } => write!(
                f,
                "Checksum mismatch for field {} at line {}, column {}: expected {}, found {}",
                field_id, line, column, expected, found
            ),
            LnmpError::NestingTooDeep {
                max_depth,
                actual_depth,
                line,
                column,
            } => write!(
                f,
                "Nesting too deep at line {}, column {}: maximum depth is {}, but reached {}",
                line, column, max_depth, actual_depth
            ),
            LnmpError::InvalidNestedStructure {
                reason,
                line,
                column,
            } => write!(
                f,
                "Invalid nested structure at line {}, column {}: {}",
                line, column, reason
            ),
            LnmpError::UnclosedNestedStructure {
                structure_type,
                opened_at_line,
                opened_at_column,
                line,
                column,
            } => write!(
                f,
                "Unclosed {} at line {}, column {} (opened at line {}, column {})",
                structure_type, line, column, opened_at_line, opened_at_column
            ),
        }
    }
}

impl std::error::Error for LnmpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new(1, 5);
        assert_eq!(ctx.line, 1);
        assert_eq!(ctx.column, 5);
        assert!(ctx.source_snippet.is_empty());
        assert!(ctx.source_file.is_none());
    }

    #[test]
    fn test_error_context_with_snippet() {
        let source = "F7:b=1\nF12:i=14532#DEADBEEF\nF23:sa=[admin,dev]";
        let ctx = ErrorContext::with_snippet(2, 15, source);
        assert_eq!(ctx.line, 2);
        assert_eq!(ctx.column, 15);
        assert!(ctx.source_snippet.contains("F12:i=14532"));
    }

    #[test]
    fn test_error_context_with_file() {
        let source = "F7:b=1\nF12:i=14532";
        let ctx = ErrorContext::with_file(1, 5, source, "test.lnmp".to_string());
        assert_eq!(ctx.line, 1);
        assert_eq!(ctx.source_file, Some("test.lnmp".to_string()));
    }

    #[test]
    fn test_invalid_character_display() {
        let error = LnmpError::InvalidCharacter {
            char: '@',
            line: 1,
            column: 5,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 1"));
        assert!(msg.contains("column 5"));
        assert!(msg.contains("'@'"));
    }

    #[test]
    fn test_unterminated_string_display() {
        let error = LnmpError::UnterminatedString {
            line: 1,
            column: 10,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 1"));
        assert!(msg.contains("column 10"));
        assert!(msg.contains("Unterminated string"));
    }

    #[test]
    fn test_unexpected_token_display() {
        let error = LnmpError::UnexpectedToken {
            expected: "equals sign".to_string(),
            found: Token::Semicolon,
            line: 1,
            column: 5,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 1"));
        assert!(msg.contains("column 5"));
        assert!(msg.contains("expected equals sign"));
    }

    #[test]
    fn test_invalid_field_id_display() {
        let error = LnmpError::InvalidFieldId {
            value: "99999".to_string(),
            line: 2,
            column: 3,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 2"));
        assert!(msg.contains("column 3"));
        assert!(msg.contains("99999"));
    }

    #[test]
    fn test_invalid_value_display() {
        let error = LnmpError::InvalidValue {
            field_id: 12,
            reason: "not a valid integer".to_string(),
            line: 3,
            column: 10,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("field 12"));
        assert!(msg.contains("line 3"));
        assert!(msg.contains("column 10"));
        assert!(msg.contains("not a valid integer"));
    }

    #[test]
    fn test_unexpected_eof_display() {
        let error = LnmpError::UnexpectedEof { line: 5, column: 1 };
        let msg = format!("{}", error);
        assert!(msg.contains("line 5"));
        assert!(msg.contains("column 1"));
        assert!(msg.contains("end of file"));
    }

    #[test]
    fn test_invalid_escape_sequence_display() {
        let error = LnmpError::InvalidEscapeSequence {
            sequence: "\\x".to_string(),
            line: 1,
            column: 15,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 1"));
        assert!(msg.contains("column 15"));
        assert!(msg.contains("\\x"));
    }

    #[test]
    fn test_checksum_mismatch_display() {
        let error = LnmpError::ChecksumMismatch {
            field_id: 12,
            expected: "6A93B3F1".to_string(),
            found: "DEADBEEF".to_string(),
            line: 2,
            column: 15,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("field 12"));
        assert!(msg.contains("line 2"));
        assert!(msg.contains("column 15"));
        assert!(msg.contains("6A93B3F1"));
        assert!(msg.contains("DEADBEEF"));
    }

    #[test]
    fn test_nesting_too_deep_display() {
        let error = LnmpError::NestingTooDeep {
            max_depth: 10,
            actual_depth: 15,
            line: 5,
            column: 20,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 5"));
        assert!(msg.contains("column 20"));
        assert!(msg.contains("10"));
        assert!(msg.contains("15"));
        assert!(msg.contains("Nesting too deep"));
    }

    #[test]
    fn test_invalid_nested_structure_display() {
        let error = LnmpError::InvalidNestedStructure {
            reason: "mismatched braces".to_string(),
            line: 3,
            column: 12,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("line 3"));
        assert!(msg.contains("column 12"));
        assert!(msg.contains("mismatched braces"));
    }

    #[test]
    fn test_unclosed_nested_structure_display() {
        let error = LnmpError::UnclosedNestedStructure {
            structure_type: "record".to_string(),
            opened_at_line: 1,
            opened_at_column: 5,
            line: 3,
            column: 1,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("record"));
        assert!(msg.contains("line 3"));
        assert!(msg.contains("column 1"));
        assert!(msg.contains("line 1"));
        assert!(msg.contains("column 5"));
    }

    #[test]
    fn test_format_with_source() {
        let source = "F7:b=1\nF12:i=14532#DEADBEEF\nF23:sa=[admin,dev]";
        let error = LnmpError::ChecksumMismatch {
            field_id: 12,
            expected: "6A93B3F1".to_string(),
            found: "DEADBEEF".to_string(),
            line: 2,
            column: 15,
        };
        let formatted = error.format_with_source(source);
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("F12:i=14532#DEADBEEF"));
        assert!(formatted.contains("^ here"));
    }

    #[test]
    fn test_error_position() {
        let error = LnmpError::NestingTooDeep {
            max_depth: 10,
            actual_depth: 15,
            line: 5,
            column: 20,
        };
        let (line, column) = error.position();
        assert_eq!(line, 5);
        assert_eq!(column, 20);
    }

    #[test]
    fn test_error_equality() {
        let error1 = LnmpError::UnexpectedEof { line: 1, column: 1 };
        let error2 = LnmpError::UnexpectedEof { line: 1, column: 1 };
        let error3 = LnmpError::UnexpectedEof { line: 2, column: 1 };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_clone() {
        let error = LnmpError::InvalidFieldId {
            value: "test".to_string(),
            line: 1,
            column: 1,
        };
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_error_context_equality() {
        let ctx1 = ErrorContext::new(1, 5);
        let ctx2 = ErrorContext::new(1, 5);
        let ctx3 = ErrorContext::new(2, 5);

        assert_eq!(ctx1, ctx2);
        assert_ne!(ctx1, ctx3);
    }
}
