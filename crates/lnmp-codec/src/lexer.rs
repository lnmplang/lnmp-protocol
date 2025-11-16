//! Lexer for tokenizing LNMP text format.

/// Token types in LNMP format
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Field prefix 'F'
    FieldPrefix,
    /// Numeric value (field ID or number)
    Number(String),
    /// Equals sign '='
    Equals,
    /// Semicolon ';'
    Semicolon,
    /// Colon ':' (for type hints)
    Colon,
    /// Type hint identifier (i, f, b, s, sa, r, ra)
    TypeHint(String),
    /// Left bracket '['
    LeftBracket,
    /// Right bracket ']'
    RightBracket,
    /// Left brace '{' (for nested records)
    LeftBrace,
    /// Right brace '}' (for nested records)
    RightBrace,
    /// Comma ','
    Comma,
    /// Quoted string "..."
    QuotedString(String),
    /// Unquoted string (identifier)
    UnquotedString(String),
    /// Hash '#' (for checksums or comments)
    Hash,
    /// Newline character
    Newline,
    /// End of file
    Eof,
}

/// Lexer for LNMP text format
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the current position (line, column)
    pub fn position(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    /// Peeks at the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    /// Peeks at the character at offset from current position
    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.input[self.position..].chars().nth(offset)
    }

    /// Advances to the next character and returns it
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    /// Skips whitespace (spaces and tabs, but not newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }
}

use crate::error::LnmpError;

impl<'a> Lexer<'a> {
    /// Reads the next token from the input
    pub fn next_token(&mut self) -> Result<Token, LnmpError> {
        self.skip_whitespace();

        match self.peek() {
            Some('#') => {
                self.advance();
                Ok(Token::Hash)
            }
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some(':') => {
                self.advance();
                // After colon, try to read a type hint
                self.read_type_hint()
            }
            Some('[') => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RightBracket)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('F') => {
                // Check if it's followed by a digit (field prefix)
                if let Some(next) = self.peek_ahead(1) {
                    if next.is_ascii_digit() {
                        self.advance(); // consume 'F'
                        return Ok(Token::FieldPrefix);
                    }
                }
                // Otherwise it's an unquoted string
                self.read_unquoted_string()
            }
            Some('"') => self.read_quoted_string(),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => self.read_number(),
            Some(ch) if is_unquoted_char(ch) => self.read_unquoted_string(),
            Some(ch) => {
                let (line, column) = self.position();
                Err(LnmpError::UnexpectedToken {
                    expected: "valid token".to_string(),
                    found: Token::UnquotedString(ch.to_string()),
                    line,
                    column,
                })
            }
        }
    }

    /// Reads a number (integer or float)
    fn read_number(&mut self) -> Result<Token, LnmpError> {
        let mut number = String::new();

        // Handle optional negative sign
        if self.peek() == Some('-') {
            number.push('-');
            self.advance();
        }

        // Read digits
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::Number(number))
    }

    /// Reads a quoted string with escape sequences
    fn read_quoted_string(&mut self) -> Result<Token, LnmpError> {
        let (start_line, start_column) = self.position();
        self.advance(); // consume opening quote

        let mut result = String::new();

        loop {
            match self.peek() {
                None => {
                    return Err(LnmpError::UnexpectedEof {
                        line: self.line,
                        column: self.column,
                    });
                }
                Some('"') => {
                    self.advance(); // consume closing quote
                    return Ok(Token::QuotedString(result));
                }
                Some('\\') => {
                    self.advance(); // consume backslash
                    match self.peek() {
                        Some('"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some('n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some(ch) => {
                            return Err(LnmpError::InvalidEscapeSequence {
                                sequence: format!("\\{}", ch),
                                line: start_line,
                                column: start_column,
                            });
                        }
                        None => {
                            return Err(LnmpError::UnexpectedEof {
                                line: self.line,
                                column: self.column,
                            });
                        }
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
            }
        }
    }

    /// Reads an unquoted string (identifier)
    fn read_unquoted_string(&mut self) -> Result<Token, LnmpError> {
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if is_unquoted_char(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::UnquotedString(result))
    }

    /// Reads a type hint identifier after a colon
    fn read_type_hint(&mut self) -> Result<Token, LnmpError> {
        let mut hint = String::new();

        // Read lowercase letters for type hint (i, f, b, s, sa)
        while let Some(ch) = self.peek() {
            if ch.is_ascii_lowercase() {
                hint.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if hint.is_empty() {
            // If no type hint follows colon, return just the colon token
            Ok(Token::Colon)
        } else {
            Ok(Token::TypeHint(hint))
        }
    }
}

/// Checks if a character is valid in an unquoted string
fn is_unquoted_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character_tokens() {
        let mut lexer = Lexer::new("=;[],");
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_field_prefix() {
        let mut lexer = Lexer::new("F12");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("123 -456 3.14 -2.5");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("123".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("-456".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("3.14".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("-2.5".to_string())
        );
    }

    #[test]
    fn test_quoted_string() {
        let mut lexer = Lexer::new(r#""hello world""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("hello world".to_string())
        );
    }

    #[test]
    fn test_quoted_string_with_escapes() {
        let mut lexer = Lexer::new(r#""hello \"world\"""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("hello \"world\"".to_string())
        );

        let mut lexer = Lexer::new(r#""line1\nline2""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("line1\nline2".to_string())
        );

        let mut lexer = Lexer::new(r#""tab\there""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("tab\there".to_string())
        );

        let mut lexer = Lexer::new(r#""back\\slash""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("back\\slash".to_string())
        );
    }

    #[test]
    fn test_unquoted_string() {
        let mut lexer = Lexer::new("hello_world test-123 file.txt");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::UnquotedString("hello_world".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::UnquotedString("test-123".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::UnquotedString("file.txt".to_string())
        );
    }

    #[test]
    fn test_newline() {
        let mut lexer = Lexer::new("F1=2\nF3=4");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("2".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Newline);
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("F1  =  2");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("2".to_string()));
    }

    #[test]
    fn test_comment_skipping() {
        // Comments are now represented as Hash token followed by content
        let mut lexer = Lexer::new("# This is a comment\nF1=2");
        assert_eq!(lexer.next_token().unwrap(), Token::Hash);
        // The rest of the comment line is tokenized as unquoted strings/numbers
        // Parser will handle skipping until newline
    }

    #[test]
    fn test_comment_at_end() {
        let mut lexer = Lexer::new("F1=2\n# Comment at end");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("2".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Newline);
        assert_eq!(lexer.next_token().unwrap(), Token::Hash);
        // Parser will handle skipping the rest of the comment
    }

    #[test]
    fn test_string_array() {
        let mut lexer = Lexer::new(r#"["admin","dev"]"#);
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBracket);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("admin".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("dev".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::RightBracket);
    }

    #[test]
    fn test_complete_field_assignment() {
        let mut lexer = Lexer::new("F12=14532");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("14532".to_string())
        );
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("F1=2\nF3=4");
        assert_eq!(lexer.position(), (1, 1));
        lexer.next_token().unwrap(); // F
        assert_eq!(lexer.position(), (1, 2));
        lexer.next_token().unwrap(); // 1
        lexer.next_token().unwrap(); // =
        lexer.next_token().unwrap(); // 2
        lexer.next_token().unwrap(); // \n
        assert_eq!(lexer.position(), (2, 1));
    }

    #[test]
    fn test_unterminated_string_error() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        let result = lexer.next_token();
        assert!(result.is_err());
        match result {
            Err(LnmpError::UnexpectedEof { .. }) => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let mut lexer = Lexer::new(r#""\x""#);
        let result = lexer.next_token();
        assert!(result.is_err());
        match result {
            Err(LnmpError::InvalidEscapeSequence { sequence, .. }) => {
                assert_eq!(sequence, "\\x");
            }
            _ => panic!("Expected InvalidEscapeSequence error"),
        }
    }

    #[test]
    fn test_f_as_unquoted_string() {
        let mut lexer = Lexer::new("False");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::UnquotedString("False".to_string())
        );
    }

    #[test]
    fn test_multiline_record() {
        let input = "F12=14532\nF7=1\nF23=[\"admin\",\"dev\"]";
        let mut lexer = Lexer::new(input);

        // F12=14532
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("14532".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Newline);

        // F7=1
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("7".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Newline);
    }

    #[test]
    fn test_inline_record() {
        let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("14532".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
    }

    #[test]
    fn test_type_hint_tokenization() {
        // Test F12:i=14532 format
        let mut lexer = Lexer::new("F12:i=14532");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("i".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("14532".to_string())
        );
    }

    #[test]
    fn test_all_type_hint_codes() {
        // Test integer type hint
        let mut lexer = Lexer::new(":i");
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("i".to_string()));

        // Test float type hint
        let mut lexer = Lexer::new(":f");
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("f".to_string()));

        // Test boolean type hint
        let mut lexer = Lexer::new(":b");
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("b".to_string()));

        // Test string type hint
        let mut lexer = Lexer::new(":s");
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("s".to_string()));

        // Test string array type hint
        let mut lexer = Lexer::new(":sa");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::TypeHint("sa".to_string())
        );
    }

    #[test]
    fn test_invalid_type_hint_codes() {
        // Invalid type hint should be tokenized as TypeHint with invalid value
        let mut lexer = Lexer::new(":xyz");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::TypeHint("xyz".to_string())
        );

        // Colon followed by non-letter should return just Colon
        let mut lexer = Lexer::new(":=");
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);

        // Colon followed by number should return just Colon
        let mut lexer = Lexer::new(":123");
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("123".to_string())
        );
    }

    #[test]
    fn test_type_hint_in_complete_field() {
        // Test complete field with type hint: F7:b=1
        let mut lexer = Lexer::new("F7:b=1");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("7".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("b".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));

        // Test float with type hint: F5:f=3.14
        let mut lexer = Lexer::new("F5:f=3.14");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("5".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("f".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("3.14".to_string())
        );

        // Test string with type hint: F10:s="test"
        let mut lexer = Lexer::new(r#"F10:s="test""#);
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("10".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("s".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("test".to_string())
        );

        // Test string array with type hint: F23:sa=["admin","dev"]
        let mut lexer = Lexer::new(r#"F23:sa=["admin","dev"]"#);
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("23".to_string()));
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::TypeHint("sa".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBracket);
    }

    #[test]
    fn test_type_hint_with_whitespace() {
        // Type hints should work with whitespace around them
        let mut lexer = Lexer::new("F12 :i =14532");
        assert_eq!(lexer.next_token().unwrap(), Token::FieldPrefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Number("12".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::TypeHint("i".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Equals);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Number("14532".to_string())
        );
    }
}
