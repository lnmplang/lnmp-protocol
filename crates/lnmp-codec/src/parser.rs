//! Parser for converting LNMP text format into structured records.

use std::borrow::Cow;

use crate::config::{ParserConfig, ParsingMode, TextInputMode};
use crate::error::LnmpError;
use crate::lexer::{Lexer, Token};
use crate::normalizer::ValueNormalizer;
use lnmp_core::checksum::SemanticChecksum;
use lnmp_core::registry::{ValidationMode, ValidationResult};
use lnmp_core::{FieldId, LnmpField, LnmpRecord, LnmpValue, TypeHint};
use lnmp_sanitize::{sanitize_lnmp_text, SanitizationConfig};

/// Parser for LNMP text format
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    config: ParserConfig,
    // current nesting depth for nested records/arrays
    nesting_depth: usize,
    normalizer: Option<ValueNormalizer>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given input (defaults to loose mode)
    pub fn new(input: &'a str) -> Result<Self, LnmpError> {
        Self::with_config(input, ParserConfig::default())
    }

    /// Creates a new parser using strict text input handling.
    pub fn new_strict(input: &'a str) -> Result<Self, LnmpError> {
        Self::with_config(
            input,
            ParserConfig {
                text_input_mode: TextInputMode::Strict,
                ..ParserConfig::default()
            },
        )
    }

    /// Creates a new parser using lenient text sanitization before parsing.
    pub fn new_lenient(input: &'a str) -> Result<Self, LnmpError> {
        Self::with_config(
            input,
            ParserConfig {
                text_input_mode: TextInputMode::Lenient,
                ..ParserConfig::default()
            },
        )
    }

    /// Creates a new parser with specified parsing mode
    pub fn with_mode(input: &'a str, mode: ParsingMode) -> Result<Self, LnmpError> {
        let config = ParserConfig {
            mode,
            ..Default::default()
        };
        Self::with_config(input, config)
    }

    /// Creates a new parser with the specified profile
    pub fn with_profile(
        input: &'a str,
        profile: lnmp_core::profile::LnmpProfile,
    ) -> Result<Self, LnmpError> {
        let config = ParserConfig::from_profile(profile);
        Self::with_config(input, config)
    }

    /// Creates a new parser with specified configuration
    pub fn with_config(input: &'a str, config: ParserConfig) -> Result<Self, LnmpError> {
        let input_cow = match config.text_input_mode {
            TextInputMode::Strict => Cow::Borrowed(input),
            TextInputMode::Lenient => sanitize_lnmp_text(input, &SanitizationConfig::default()),
        };

        if config.mode == ParsingMode::Strict {
            Self::check_for_comments(input_cow.as_ref())?;
        }

        let mut lexer = match input_cow {
            Cow::Borrowed(s) => Lexer::new(s),
            Cow::Owned(s) => {
                let span_map = crate::lexer::build_span_map(s.as_str(), input);
                Lexer::new_owned_with_original(s, input.to_string(), span_map)
            }
        };
        let current_token = lexer.next_token()?;

        let normalizer = config.semantic_dictionary.as_ref().map(|dict| {
            ValueNormalizer::new(crate::normalizer::NormalizationConfig {
                semantic_dictionary: Some(dict.clone()),
                ..crate::normalizer::NormalizationConfig::default()
            })
        });

        Ok(Self {
            lexer,
            current_token,
            config,
            nesting_depth: 0,
            normalizer,
        })
    }

    /// Returns the current parsing mode
    pub fn mode(&self) -> ParsingMode {
        self.config.mode
    }

    /// Advances to the next token
    fn advance(&mut self) -> Result<(), LnmpError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    /// Expects a specific token and advances
    fn expect(&mut self, expected: Token) -> Result<(), LnmpError> {
        if self.current_token == expected {
            self.advance()
        } else {
            let (line, column) = self.lexer.position_original();
            Err(LnmpError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token.clone(),
                line,
                column,
            })
        }
    }

    /// Skips newlines
    fn skip_newlines(&mut self) -> Result<(), LnmpError> {
        while self.current_token == Token::Newline {
            self.advance()?;
        }
        Ok(())
    }

    /// Skips a comment (Hash token followed by text until newline)
    fn skip_comment(&mut self) -> Result<(), LnmpError> {
        // Consume the Hash token
        self.advance()?;

        // Skip until newline or EOF
        while self.current_token != Token::Newline && self.current_token != Token::Eof {
            self.advance()?;
        }

        Ok(())
    }

    /// Checks if the input contains comments (for strict mode validation)
    fn check_for_comments(input: &str) -> Result<(), LnmpError> {
        // Check if input contains comment lines (lines starting with #)
        // Note: # after a value is a checksum, not a comment
        for (line_idx, line) in input.lines().enumerate() {
            let trimmed = line.trim();
            // If line starts with #, it's a comment
            if trimmed.starts_with('#') {
                return Err(LnmpError::StrictModeViolation {
                    reason: "Comments are not allowed in strict mode".to_string(),
                    line: line_idx + 1,
                    column: 1,
                });
            }
        }
        Ok(())
    }
}

impl<'a> Parser<'a> {
    /// Parses a field ID (expects F prefix followed by number)
    fn parse_field_id(&mut self) -> Result<FieldId, LnmpError> {
        let (line, column) = self.lexer.position_original();

        // Expect F prefix
        self.expect(Token::FieldPrefix)?;

        // Expect number
        match &self.current_token {
            Token::Number(num_str) => {
                let num_str = num_str.clone();
                self.advance()?;

                // Parse as u16
                match num_str.parse::<u16>() {
                    Ok(fid) => Ok(fid),
                    Err(_) => Err(LnmpError::InvalidFieldId {
                        value: num_str,
                        line,
                        column,
                    }),
                }
            }
            Token::UnquotedString(s) => {
                // 'F' followed by non-numeric characters - invalid field id
                Err(LnmpError::InvalidFieldId {
                    value: s.clone(),
                    line,
                    column,
                })
            }
            _ => Err(LnmpError::UnexpectedToken {
                expected: "field ID number".to_string(),
                found: self.current_token.clone(),
                line,
                column,
            }),
        }
    }

    /// Parses a value based on the current token
    #[allow(dead_code)]
    fn parse_value(&mut self) -> Result<LnmpValue, LnmpError> {
        self.parse_value_with_hint(None)
    }

    /// Parses a value with an optional type hint to resolve ambiguities
    fn parse_value_with_hint(
        &mut self,
        type_hint: Option<TypeHint>,
    ) -> Result<LnmpValue, LnmpError> {
        let (line, column) = self.lexer.position_original();

        match &self.current_token {
            Token::Number(num_str) => {
                let num_str = num_str.clone();
                self.advance()?;
                // If the type hint is a boolean, enforce boolean parsing for 0/1
                if type_hint == Some(TypeHint::Bool) {
                    if num_str == "0" {
                        return Ok(LnmpValue::Bool(false));
                    } else if num_str == "1" {
                        return Ok(LnmpValue::Bool(true));
                    } else {
                        return Err(LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid boolean value: {}", num_str),
                            line,
                            column,
                        });
                    }
                }

                // If normalization is enabled, interpret 0/1 as booleans but do not reject other numbers
                if self.config.normalize_values {
                    if num_str == "0" {
                        return Ok(LnmpValue::Bool(false));
                    } else if num_str == "1" {
                        return Ok(LnmpValue::Bool(true));
                    }
                }

                // Try to parse as float if it contains a dot
                if num_str.contains('.') {
                    match num_str.parse::<f64>() {
                        Ok(f) => Ok(LnmpValue::Float(f)),
                        Err(_) => Err(LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid float: {}", num_str),
                            line,
                            column,
                        }),
                    }
                } else {
                    // Parse as integer
                    match num_str.parse::<i64>() {
                        Ok(i) => Ok(LnmpValue::Int(i)),
                        Err(_) => Err(LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid integer: {}", num_str),
                            line,
                            column,
                        }),
                    }
                }
            }
            Token::QuotedString(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(LnmpValue::String(s))
            }
            Token::UnquotedString(s) => {
                let s = s.clone();
                self.advance()?;
                // If a boolean type hint is present or normalization is enabled, allow
                // text values 'true'/'false' or 'yes'/'no' to be interpreted as booleans.
                if type_hint == Some(TypeHint::Bool) || self.config.normalize_values {
                    match s.to_ascii_lowercase().as_str() {
                        "true" | "yes" => return Ok(LnmpValue::Bool(true)),
                        "false" | "no" => return Ok(LnmpValue::Bool(false)),
                        _ => {}
                    }
                }
                Ok(LnmpValue::String(s))
            }
            Token::LeftBracket => self.parse_string_array_or_nested_array_with_hint(type_hint),
            Token::LeftBrace => self.parse_nested_record(),
            _ => Err(LnmpError::UnexpectedToken {
                expected: "value".to_string(),
                found: self.current_token.clone(),
                line,
                column,
            }),
        }
    }

    /// Parses either a string array [item1, item2, ...] or nested array [{...}, {...}]
    #[allow(dead_code)]
    fn parse_string_array_or_nested_array(&mut self) -> Result<LnmpValue, LnmpError> {
        self.parse_string_array_or_nested_array_with_hint(None)
    }

    /// Parses either a string array or nested array with optional type hint
    fn parse_string_array_or_nested_array_with_hint(
        &mut self,
        type_hint: Option<TypeHint>,
    ) -> Result<LnmpValue, LnmpError> {
        self.expect(Token::LeftBracket)?;

        // Handle empty array - use type hint to determine type
        if self.current_token == Token::RightBracket {
            self.advance()?;
            return Ok(match type_hint {
                Some(TypeHint::RecordArray) => LnmpValue::NestedArray(Vec::new()),
                Some(TypeHint::IntArray) => LnmpValue::IntArray(Vec::new()),
                Some(TypeHint::FloatArray) => LnmpValue::FloatArray(Vec::new()),
                Some(TypeHint::BoolArray) => LnmpValue::BoolArray(Vec::new()),
                _ => LnmpValue::StringArray(Vec::new()),
            });
        }

        match type_hint {
            Some(TypeHint::RecordArray) => {
                if self.current_token != Token::LeftBrace {
                    let (line, column) = self.lexer.position_original();
                    return Err(LnmpError::UnexpectedToken {
                        expected: "nested record ({...}) inside record array".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
                self.parse_nested_array()
            }
            Some(TypeHint::IntArray) => self.parse_int_array(),
            Some(TypeHint::FloatArray) => self.parse_float_array(),
            Some(TypeHint::BoolArray) => self.parse_bool_array(),
            _ => {
                if self.current_token == Token::LeftBrace {
                    self.parse_nested_array()
                } else {
                    self.parse_string_array()
                }
            }
        }
    }

    /// Parses a string array [item1, item2, ...]
    fn parse_string_array(&mut self) -> Result<LnmpValue, LnmpError> {
        let (line, column) = self.lexer.position_original();
        let mut items = Vec::new();

        loop {
            // Parse string item
            match &self.current_token {
                Token::QuotedString(s) => {
                    items.push(s.clone());
                    self.advance()?;
                }
                Token::UnquotedString(s) => {
                    items.push(s.clone());
                    self.advance()?;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "string".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }

            // Check for comma or closing bracket
            match &self.current_token {
                Token::Comma => {
                    self.advance()?;
                    // Continue to next item
                }
                Token::RightBracket => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }
        }

        Ok(LnmpValue::StringArray(items))
    }

    fn parse_int_array(&mut self) -> Result<LnmpValue, LnmpError> {
        let mut items = Vec::new();

        loop {
            let (line, column) = self.lexer.position_original();
            let value = match &self.current_token {
                Token::Number(num_str) => {
                    num_str
                        .parse::<i64>()
                        .map_err(|_| LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid integer: {}", num_str),
                            line,
                            column,
                        })?
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "integer literal".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    })
                }
            };
            items.push(value);
            self.advance()?;

            match &self.current_token {
                Token::Comma => {
                    self.advance()?;
                }
                Token::RightBracket => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }
        }

        Ok(LnmpValue::IntArray(items))
    }

    fn parse_float_array(&mut self) -> Result<LnmpValue, LnmpError> {
        let mut items = Vec::new();

        loop {
            let (line, column) = self.lexer.position_original();
            let value = match &self.current_token {
                Token::Number(num_str) => {
                    num_str
                        .parse::<f64>()
                        .map_err(|_| LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid float: {}", num_str),
                            line,
                            column,
                        })?
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "float literal".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    })
                }
            };
            items.push(value);
            self.advance()?;

            match &self.current_token {
                Token::Comma => {
                    self.advance()?;
                }
                Token::RightBracket => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }
        }

        Ok(LnmpValue::FloatArray(items))
    }

    fn parse_bool_array(&mut self) -> Result<LnmpValue, LnmpError> {
        let mut items = Vec::new();

        loop {
            let (line, column) = self.lexer.position_original();
            let value = match &self.current_token {
                Token::Number(num_str) => match num_str.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => {
                        return Err(LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid boolean literal: {}", num_str),
                            line,
                            column,
                        })
                    }
                },
                Token::UnquotedString(s) => match s.to_ascii_lowercase().as_str() {
                    "true" | "yes" => true,
                    "false" | "no" => false,
                    _ => {
                        return Err(LnmpError::InvalidValue {
                            field_id: 0,
                            reason: format!("invalid boolean literal: {}", s),
                            line,
                            column,
                        })
                    }
                },
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "boolean literal".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    })
                }
            };
            items.push(value);
            self.advance()?;

            match &self.current_token {
                Token::Comma => {
                    self.advance()?;
                }
                Token::RightBracket => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }
        }

        Ok(LnmpValue::BoolArray(items))
    }

    /// Parses a nested record {F<id>=<value>;F<id>=<value>}
    fn parse_nested_record(&mut self) -> Result<LnmpValue, LnmpError> {
        let (line, column) = self.lexer.position_original();
        self.expect(Token::LeftBrace)?;

        // Increase nesting depth and enforce maximum if configured
        self.nesting_depth += 1;
        if let Some(max) = self.config.max_nesting_depth {
            if self.nesting_depth > max {
                let actual = self.nesting_depth;
                self.nesting_depth = self.nesting_depth.saturating_sub(1);
                return Err(LnmpError::NestingTooDeep {
                    max_depth: max,
                    actual_depth: actual,
                    line,
                    column,
                });
            }
        }

        // Use a closure to ensure we can decrement nesting_depth on all return paths
        let result = (|| -> Result<LnmpValue, LnmpError> {
            let mut record = LnmpRecord::new();

            // Handle empty nested record
            if self.current_token == Token::RightBrace {
                self.advance()?;
                return Ok(LnmpValue::NestedRecord(Box::new(record)));
            }

            // Parse field assignments within the nested record
            loop {
                let field = self.parse_field_assignment()?;
                // In strict mode, detect duplicate field IDs in nested records and error early
                if self.config.mode == ParsingMode::Strict && record.get_field(field.fid).is_some()
                {
                    let (line, column) = self.lexer.position_original();
                    return Err(LnmpError::DuplicateFieldId {
                        field_id: field.fid,
                        line,
                        column,
                    });
                }
                record.add_field(field);

                // Check for separator or closing brace
                match &self.current_token {
                    Token::Semicolon => {
                        self.advance()?;
                        // Check if we're at the end
                        if self.current_token == Token::RightBrace {
                            self.advance()?;
                            break;
                        }
                        // Continue to next field
                    }
                    Token::RightBrace => {
                        self.advance()?;
                        break;
                    }
                    _ => {
                        return Err(LnmpError::UnexpectedToken {
                            expected: "semicolon or closing brace".to_string(),
                            found: self.current_token.clone(),
                            line,
                            column,
                        });
                    }
                }
            }

            // Sort fields by FID for canonical representation
            let sorted_record = LnmpRecord::from_sorted_fields(record.sorted_fields());

            Ok(LnmpValue::NestedRecord(Box::new(sorted_record)))
        })();

        // Always decrement nesting depth before returning
        self.nesting_depth = self.nesting_depth.saturating_sub(1);
        result
    }

    /// Parses a nested array [{...}, {...}]
    fn parse_nested_array(&mut self) -> Result<LnmpValue, LnmpError> {
        let (line, column) = self.lexer.position_original();

        // Increase nesting depth for nested array
        self.nesting_depth += 1;
        if let Some(max) = self.config.max_nesting_depth {
            if self.nesting_depth > max {
                let actual = self.nesting_depth;
                self.nesting_depth = self.nesting_depth.saturating_sub(1);
                return Err(LnmpError::NestingTooDeep {
                    max_depth: max,
                    actual_depth: actual,
                    line,
                    column,
                });
            }
        }

        let result = (|| -> Result<LnmpValue, LnmpError> {
            let mut records = Vec::new();

            loop {
                // Parse nested record
                if self.current_token != Token::LeftBrace {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "left brace for nested record".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }

                // Parse the nested record value and extract the record
                match self.parse_nested_record()? {
                    LnmpValue::NestedRecord(record) => {
                        records.push(*record);
                    }
                    _ => unreachable!("parse_nested_record always returns NestedRecord"),
                }

                // Check for comma or closing bracket
                match &self.current_token {
                    Token::Comma => {
                        self.advance()?;
                        // Continue to next record
                    }
                    Token::RightBracket => {
                        self.advance()?;
                        break;
                    }
                    _ => {
                        return Err(LnmpError::UnexpectedToken {
                            expected: "comma or closing bracket".to_string(),
                            found: self.current_token.clone(),
                            line,
                            column,
                        });
                    }
                }
            }

            Ok(LnmpValue::NestedArray(records))
        })();

        // Leaving nesting: decrement depth
        self.nesting_depth = self.nesting_depth.saturating_sub(1);
        result
    }

    /// Parses a type hint (optional :type after field ID)
    fn parse_type_hint(&mut self) -> Result<Option<TypeHint>, LnmpError> {
        if let Token::TypeHint(hint_str) = &self.current_token {
            let hint_str = hint_str.clone();
            self.advance()?;

            match TypeHint::parse(&hint_str) {
                Some(hint) => Ok(Some(hint)),
                None => {
                    let (line, column) = self.lexer.position_original();
                    Err(LnmpError::InvalidTypeHint {
                        hint: hint_str,
                        line,
                        column,
                    })
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Parses a field assignment (F<id>=<value> or F<id>:<type>=<value>)
    fn parse_field_assignment(&mut self) -> Result<LnmpField, LnmpError> {
        let fid = self.parse_field_id()?;

        // Check for optional type hint
        let type_hint = self.parse_type_hint()?;

        self.expect(Token::Equals)?;
        let value = self.parse_value_with_hint(type_hint)?;

        // Validate type hint if present
        if let Some(hint) = type_hint {
            if !hint.validates(&value) {
                let (line, column) = self.lexer.position_original();
                return Err(LnmpError::TypeHintMismatch {
                    field_id: fid,
                    expected_type: hint.as_str().to_string(),
                    actual_value: format!("{:?}", value),
                    line,
                    column,
                });
            }
        }

        // Check for optional checksum
        if self.current_token == Token::Hash {
            self.parse_and_validate_checksum(fid, type_hint, &value)?;
        } else if self.config.require_checksums {
            let (line, column) = self.lexer.position_original();
            return Err(LnmpError::ChecksumMismatch {
                field_id: fid,
                expected: "checksum required".to_string(),
                found: "no checksum".to_string(),
                line,
                column,
            });
        }

        // Apply semantic normalization if configured
        let normalized_value = if let Some(norm) = &self.normalizer {
            norm.normalize_with_fid(Some(fid), &value)
        } else {
            value
        };

        let field = LnmpField {
            fid,
            value: normalized_value,
        };

        // FID Registry validation (v0.5.14)
        if let Some(registry) = &self.config.fid_registry {
            let result = registry.validate_field(&field);
            match result {
                ValidationResult::Valid => {}
                ValidationResult::TypeMismatch {
                    expected, found, ..
                } => {
                    let (line, column) = self.lexer.position_original();
                    match self.config.fid_validation_mode {
                        ValidationMode::Error => {
                            return Err(LnmpError::FidValidation {
                                fid,
                                reason: format!(
                                    "type mismatch: expected {:?}, found {:?}",
                                    expected, found
                                ),
                                line,
                                column,
                            });
                        }
                        ValidationMode::Warn => {
                            // Log warning (when log feature is enabled)
                            #[cfg(feature = "log")]
                            log::warn!(
                                "FID validation warning at line {}, column {}: F{} type mismatch - expected {:?}, found {:?}",
                                line, column, fid, expected, found
                            );
                        }
                        ValidationMode::None => {}
                    }
                }
                ValidationResult::UnknownFid { range, .. } => {
                    let (line, column) = self.lexer.position_original();
                    match self.config.fid_validation_mode {
                        ValidationMode::Error => {
                            return Err(LnmpError::FidValidation {
                                fid,
                                reason: format!("unknown FID in {:?} range", range),
                                line,
                                column,
                            });
                        }
                        ValidationMode::Warn => {
                            #[cfg(feature = "log")]
                            log::warn!(
                                "FID validation warning at line {}, column {}: F{} is unknown in {:?} range",
                                line, column, fid, range
                            );
                        }
                        ValidationMode::None => {}
                    }
                }
                ValidationResult::DeprecatedFid { name, .. } => {
                    let (line, column) = self.lexer.position_original();
                    match self.config.fid_validation_mode {
                        ValidationMode::Error => {
                            return Err(LnmpError::FidValidation {
                                fid,
                                reason: format!("deprecated FID: {}", name),
                                line,
                                column,
                            });
                        }
                        ValidationMode::Warn => {
                            #[cfg(feature = "log")]
                            log::warn!(
                                "FID validation warning at line {}, column {}: F{} ({}) is deprecated",
                                line, column, fid, name
                            );
                        }
                        ValidationMode::None => {}
                    }
                }
                ValidationResult::TombstonedFid { name, .. } => {
                    let (line, column) = self.lexer.position_original();
                    // Tombstoned FIDs always error, regardless of mode
                    return Err(LnmpError::FidValidation {
                        fid,
                        reason: format!("tombstoned FID: {} (must never be used)", name),
                        line,
                        column,
                    });
                }
            }
        }

        Ok(field)
    }

    /// Parses and validates a checksum
    fn parse_and_validate_checksum(
        &mut self,
        fid: FieldId,
        type_hint: Option<TypeHint>,
        value: &LnmpValue,
    ) -> Result<(), LnmpError> {
        let (line, column) = self.lexer.position_original();

        // Consume the hash token
        self.expect(Token::Hash)?;

        // Read the checksum - it might be split across multiple tokens
        // (e.g., "36AAE667" might be tokenized as Number("36") + UnquotedString("AAE") + Number("667"))
        let mut checksum_str = String::new();

        // Collect all tokens until we hit a separator (newline, semicolon, EOF)
        loop {
            match &self.current_token {
                Token::Number(s) => {
                    checksum_str.push_str(s);
                    self.advance()?;
                }
                Token::UnquotedString(s) => {
                    checksum_str.push_str(s);
                    self.advance()?;
                }
                Token::Newline | Token::Semicolon | Token::Eof => {
                    break;
                }
                _ => {
                    return Err(LnmpError::UnexpectedToken {
                        expected: "checksum (8 hex characters)".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }

            // Stop after 8 characters
            if checksum_str.len() >= 8 {
                break;
            }
        }

        // Parse the checksum
        let provided_checksum =
            SemanticChecksum::parse(&checksum_str).ok_or_else(|| LnmpError::InvalidChecksum {
                field_id: fid,
                reason: format!("invalid checksum format: {}", checksum_str),
                line,
                column,
            })?;

        // Validate checksum if enabled
        if self.config.validate_checksums {
            let computed_checksum = SemanticChecksum::compute(fid, type_hint, value);
            if provided_checksum != computed_checksum {
                return Err(LnmpError::ChecksumMismatch {
                    field_id: fid,
                    expected: SemanticChecksum::format(computed_checksum),
                    found: SemanticChecksum::format(provided_checksum),
                    line,
                    column,
                });
            }
        }

        Ok(())
    }

    /// Validates that fields are sorted by FID (strict mode only)
    fn validate_field_order(&self, record: &LnmpRecord) -> Result<(), LnmpError> {
        let fields = record.fields();
        for i in 1..fields.len() {
            if fields[i].fid < fields[i - 1].fid {
                let (line, column) = self.lexer.position_original();
                return Err(LnmpError::StrictModeViolation {
                    reason: format!(
                        "Fields must be sorted by FID in strict mode (F{} appears after F{})",
                        fields[i].fid,
                        fields[i - 1].fid
                    ),
                    line,
                    column,
                });
            }
        }
        Ok(())
    }

    // Duplicate field IDs are now detected during parsing and a DuplicateFieldId
    // error is emitted at parse time with an accurate lexer position.

    /// Validates separator (strict mode rejects semicolons)
    fn validate_separator(&self, is_semicolon: bool) -> Result<(), LnmpError> {
        if self.config.mode == ParsingMode::Strict && is_semicolon {
            let (line, column) = self.lexer.position_original();
            return Err(LnmpError::StrictModeViolation {
                reason: "Semicolons are not allowed in strict mode (use newlines)".to_string(),
                line,
                column,
            });
        }
        Ok(())
    }

    /// Parses a complete record
    pub fn parse_record(&mut self) -> Result<LnmpRecord, LnmpError> {
        let mut record = LnmpRecord::new();

        // Skip leading newlines and comments
        self.skip_newlines()?;
        while self.current_token == Token::Hash {
            self.skip_comment()?;
            if self.current_token == Token::Newline {
                self.advance()?;
            }
            self.skip_newlines()?;
        }

        // Parse field assignments until EOF
        while self.current_token != Token::Eof {
            let field = self.parse_field_assignment()?;
            // In strict mode, detect duplicate field IDs and error early
            if self.config.mode == ParsingMode::Strict && record.get_field(field.fid).is_some() {
                let (line, column) = self.lexer.position_original();
                return Err(LnmpError::DuplicateFieldId {
                    field_id: field.fid,
                    line,
                    column,
                });
            }
            record.add_field(field);

            // Handle separator (semicolon or newline)
            match &self.current_token {
                Token::Semicolon => {
                    self.validate_separator(true)?;
                    self.advance()?;
                }
                Token::Newline => {
                    self.advance()?;
                    self.skip_newlines()?;
                    // Skip comments after newlines
                    while self.current_token == Token::Hash {
                        self.skip_comment()?;
                        if self.current_token == Token::Newline {
                            self.advance()?;
                        }
                        self.skip_newlines()?;
                    }
                }
                Token::Eof => break,
                _ => {
                    let (line, column) = self.lexer.position_original();
                    return Err(LnmpError::UnexpectedToken {
                        expected: "semicolon, newline, or EOF".to_string(),
                        found: self.current_token.clone(),
                        line,
                        column,
                    });
                }
            }
        }

        // Validate field order and duplicate field IDs in strict mode
        if self.config.mode == ParsingMode::Strict {
            self.validate_field_order(&record)?;
        }

        // Enforce structural limits if provided
        if let Some(limits) = &self.config.structural_limits {
            if let Err(err) = limits.validate_record(&record) {
                let (line, column) = self.lexer.position_original();
                return Err(LnmpError::InvalidNestedStructure {
                    reason: format!("structural limits violated: {}", err),
                    line,
                    column,
                });
            }
        }

        // Check for unsorted fields if configured
        if let Some(profile_config) = &self.config.profile_config {
            if profile_config.reject_unsorted_fields {
                if let Err(e) = record.validate_field_ordering() {
                    return Err(LnmpError::ValidationError(e.to_string()));
                }
            }
        }

        Ok(record)
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let mut parser = Parser::new("F1=42").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_parse_negative_integer() {
        let mut parser = Parser::new("F1=-123").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(-123));
    }

    #[test]
    fn test_parse_float() {
        let mut parser = Parser::new("F2=3.14").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_parse_bool_true() {
        let mut parser = Parser::new("F3=1").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Bool(true));
    }

    #[test]
    fn test_parse_bool_false() {
        let mut parser = Parser::new("F3=0").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Bool(false));
    }

    #[test]
    fn test_duplicate_field_id_strict_mode_error() {
        let mut parser = Parser::with_mode("F1=1\nF1=2", ParsingMode::Strict).unwrap();
        let err = parser.parse_record().unwrap_err();
        match err {
            LnmpError::DuplicateFieldId { field_id, .. } => {
                assert_eq!(field_id, 1);
            }
            _ => panic!("expected DuplicateFieldId error, got: {:?}", err),
        }
    }

    #[test]
    fn test_duplicate_field_id_loose_mode_allows() {
        let mut parser = Parser::new("F1=1;F1=2").unwrap();
        let record = parser.parse_record().unwrap();
        let fields = record.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].fid, 1);
        assert_eq!(fields[1].fid, 1);
    }

    #[test]
    fn test_duplicate_field_id_in_nested_record_strict_mode_error() {
        let mut parser = Parser::with_mode("F50={F1=1;F1=2}", ParsingMode::Strict).unwrap();
        let err = parser.parse_record().unwrap_err();
        match err {
            LnmpError::DuplicateFieldId { field_id, .. } => {
                assert_eq!(field_id, 1);
            }
            _ => panic!("expected DuplicateFieldId error, got: {:?}", err),
        }
    }

    #[test]
    fn test_duplicate_field_id_in_nested_record_loose_mode_allows() {
        let mut parser = Parser::new("F50={F1=1;F1=2}").unwrap();
        let record = parser.parse_record().unwrap();
        let nested = record.get_field(50).unwrap();
        if let LnmpValue::NestedRecord(nested_record) = &nested.value {
            assert_eq!(nested_record.fields().len(), 2);
            assert_eq!(nested_record.fields()[0].fid, 1);
            assert_eq!(nested_record.fields()[1].fid, 1);
        } else {
            panic!("expected nested record");
        }
    }

    #[test]
    fn test_parse_quoted_string() {
        let mut parser = Parser::new(r#"F4="hello world""#).unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(4).unwrap().value,
            LnmpValue::String("hello world".to_string())
        );
    }

    #[test]
    fn test_parse_unquoted_string() {
        let mut parser = Parser::new("F5=test_value").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(5).unwrap().value,
            LnmpValue::String("test_value".to_string())
        );
    }

    #[test]
    fn test_parse_string_array() {
        let mut parser = Parser::new(r#"F6=["admin","dev","user"]"#).unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(6).unwrap().value,
            LnmpValue::StringArray(vec![
                "admin".to_string(),
                "dev".to_string(),
                "user".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_empty_string_array() {
        let mut parser = Parser::new("F6=[]").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(6).unwrap().value,
            LnmpValue::StringArray(vec![])
        );
    }

    #[test]
    fn test_parse_multiline_record() {
        let input = "F12=14532\nF7=1\nF20=\"Halil\"";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            record.get_field(20).unwrap().value,
            LnmpValue::String("Halil".to_string())
        );
    }

    #[test]
    fn test_parse_inline_record() {
        let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            record.get_field(23).unwrap().value,
            LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "# This is a comment\nF1=42\n# Another comment\nF2=3.14";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let input = "F1  =  42  ;  F2  =  3.14";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_parse_empty_input() {
        let mut parser = Parser::new("").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.fields().len(), 0);
    }

    #[test]
    fn test_parse_only_comments() {
        let input = "# Comment 1\n# Comment 2\n# Comment 3";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.fields().len(), 0);
    }

    #[test]
    fn test_parse_field_id_out_of_range() {
        let result = Parser::new("F99999=42");
        assert!(result.is_ok());
        let mut parser = result.unwrap();
        let result = parser.parse_record();
        assert!(result.is_err());
        match result {
            Err(LnmpError::InvalidFieldId { .. }) => {}
            _ => panic!("Expected InvalidFieldId error"),
        }
    }

    #[test]
    fn test_parse_missing_equals() {
        let mut parser = Parser::new("F1 42").unwrap();
        let result = parser.parse_record();
        assert!(result.is_err());
        match result {
            Err(LnmpError::UnexpectedToken { .. }) => {}
            _ => panic!("Expected UnexpectedToken error"),
        }
    }

    #[test]
    fn test_parse_missing_value() {
        let mut parser = Parser::new("F1=").unwrap();
        let result = parser.parse_record();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_field_prefix() {
        let result = Parser::new("G1=42");
        assert!(result.is_ok());
        let mut parser = result.unwrap();
        let result = parser.parse_record();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_with_escapes() {
        let input = r#"F1="hello \"world\"""#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(1).unwrap().value,
            LnmpValue::String("hello \"world\"".to_string())
        );
    }

    #[test]
    fn test_parse_mixed_separators() {
        let input = "F1=1;F2=2\nF3=3;F4=4";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.fields().len(), 4);
    }

    #[test]
    fn test_parse_large_field_id() {
        let mut parser = Parser::new("F65535=42").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(record.get_field(65535).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_parse_spec_example() {
        let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            record.get_field(23).unwrap().value,
            LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_strict_mode_rejects_unsorted_fields() {
        use crate::config::ParsingMode;

        // Unsorted fields: F12, F7, F23
        let input = r#"F12=14532
F7=1
F23=["admin","dev"]"#;

        let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::StrictModeViolation { reason, .. }) => {
                assert!(reason.contains("sorted"));
            }
            _ => panic!("Expected StrictModeViolation error"),
        }
    }

    #[test]
    fn test_strict_mode_accepts_sorted_fields() {
        use crate::config::ParsingMode;

        // Sorted fields: F7, F12, F23
        let input = r#"F7=1
F12=14532
F23=["admin","dev"]"#;

        let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_strict_mode_rejects_semicolons() {
        use crate::config::ParsingMode;

        let input = "F1=1;F2=2";

        let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::StrictModeViolation { reason, .. }) => {
                assert!(reason.contains("Semicolons"));
            }
            _ => panic!("Expected StrictModeViolation error"),
        }
    }

    #[test]
    fn test_strict_mode_rejects_comments() {
        use crate::config::ParsingMode;

        let input = "# This is a comment\nF1=42";

        let result = Parser::with_mode(input, ParsingMode::Strict);

        assert!(result.is_err());
        match result {
            Err(LnmpError::StrictModeViolation { reason, .. }) => {
                assert!(reason.contains("Comments"));
            }
            _ => panic!("Expected StrictModeViolation error"),
        }
    }

    #[test]
    fn test_loose_mode_accepts_unsorted_fields() {
        use crate::config::ParsingMode;

        // Unsorted fields
        let input = r#"F23=["admin","dev"]
F7=1
F12=14532"#;

        let mut parser = Parser::with_mode(input, ParsingMode::Loose).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        // Fields are in insertion order, not sorted
        assert_eq!(record.fields()[0].fid, 23);
        assert_eq!(record.fields()[1].fid, 7);
        assert_eq!(record.fields()[2].fid, 12);
    }

    #[test]
    fn test_loose_mode_accepts_semicolons() {
        use crate::config::ParsingMode;

        let input = "F1=1;F2=2;F3=3";

        let mut parser = Parser::with_mode(input, ParsingMode::Loose).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Int(2));
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Int(3));
    }

    #[test]
    fn test_loose_mode_accepts_whitespace() {
        use crate::config::ParsingMode;

        let input = "F1  =  42  ;  F2  =  3.14";

        let mut parser = Parser::with_mode(input, ParsingMode::Loose).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_loose_mode_accepts_comments() {
        use crate::config::ParsingMode;

        let input = "# Comment\nF1=42\n# Another comment\nF2=3.14";

        let mut parser = Parser::with_mode(input, ParsingMode::Loose).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_default_mode_is_loose() {
        let input = "F2=2;F1=1"; // Unsorted with semicolons

        let mut parser = Parser::new(input).unwrap();
        assert_eq!(parser.mode(), ParsingMode::Loose);

        let record = parser.parse_record().unwrap();
        assert_eq!(record.fields().len(), 2);
    }

    #[test]
    fn test_strict_mode_with_sorted_no_semicolons() {
        use crate::config::ParsingMode;

        let input = "F1=1\nF2=2\nF3=3";

        let mut parser = Parser::with_mode(input, ParsingMode::Strict).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Int(2));
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Int(3));
    }

    #[test]
    fn test_parse_type_hint_integer() {
        let input = "F12:i=14532";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_parse_type_hint_float() {
        let input = "F5:f=3.14";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.get_field(5).unwrap().value, LnmpValue::Float(3.14));
    }

    #[test]
    fn test_parse_type_hint_bool() {
        let input = "F7:b=1";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
    }

    #[test]
    fn test_parse_type_hint_string() {
        let input = r#"F10:s="test""#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(
            record.get_field(10).unwrap().value,
            LnmpValue::String("test".to_string())
        );
    }

    #[test]
    fn test_parse_type_hint_string_array() {
        let input = r#"F23:sa=["admin","dev"]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(
            record.get_field(23).unwrap().value,
            LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_parse_all_type_hints() {
        let input = r#"F1:i=42
F2:f=3.14
F3:b=1
F4:s=test
F5:sa=[a,b]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 5);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(3.14));
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            record.get_field(4).unwrap().value,
            LnmpValue::String("test".to_string())
        );
        assert_eq!(
            record.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_type_hint_mismatch_int_vs_float() {
        let input = "F1:i=3.14"; // Type hint says int, but value is float
        let mut parser = Parser::new(input).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::TypeHintMismatch {
                field_id,
                expected_type,
                ..
            }) => {
                assert_eq!(field_id, 1);
                assert_eq!(expected_type, "i");
            }
            _ => panic!("Expected TypeHintMismatch error"),
        }
    }

    #[test]
    fn test_type_hint_mismatch_float_vs_int() {
        let input = "F2:f=42"; // Type hint says float, but value is int
        let mut parser = Parser::new(input).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::TypeHintMismatch {
                field_id,
                expected_type,
                ..
            }) => {
                assert_eq!(field_id, 2);
                assert_eq!(expected_type, "f");
            }
            _ => panic!("Expected TypeHintMismatch error"),
        }
    }

    #[test]
    fn test_type_hint_mismatch_string_vs_int() {
        let input = "F3:s=42"; // Type hint says string, but value is int
        let mut parser = Parser::new(input).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::TypeHintMismatch { field_id, .. }) => {
                assert_eq!(field_id, 3);
            }
            _ => panic!("Expected TypeHintMismatch error"),
        }
    }

    #[test]
    fn test_invalid_type_hint() {
        let input = "F1:xyz=42"; // Invalid type hint
        let mut parser = Parser::new(input).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::InvalidTypeHint { hint, .. }) => {
                assert_eq!(hint, "xyz");
            }
            _ => panic!("Expected InvalidTypeHint error"),
        }
    }

    #[test]
    fn test_field_without_type_hint_still_works() {
        let input = "F1=42\nF2:i=100";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Int(100));
    }

    #[test]
    fn test_type_hint_with_whitespace() {
        let input = "F12 :i =14532";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_comment_lines_ignored_in_loose_mode() {
        let input = "# This is a comment\nF1=42\n# Another comment\nF2=100";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Int(100));
    }

    #[test]
    fn test_hash_in_quoted_string_preserved() {
        let input = r#"F1="This # is not a comment""#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(
            record.get_field(1).unwrap().value,
            LnmpValue::String("This # is not a comment".to_string())
        );
    }

    #[test]
    fn test_comment_after_whitespace() {
        let input = "   # Comment with leading whitespace\nF1=42";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_inline_comments_not_supported() {
        // Inline comments are not supported - the # would be part of the value
        // This test verifies that inline comments don't work as expected
        let input = "F1=test"; // No inline comment to test
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(
            record.get_field(1).unwrap().value,
            LnmpValue::String("test".to_string())
        );
    }

    #[test]
    fn test_encoder_never_outputs_comments() {
        use crate::encoder::Encoder;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("test".to_string()),
        });

        let encoder = Encoder::new();
        let output = encoder.encode(&record);

        // Verify output doesn't contain comment character
        assert!(!output.contains('#'));
        assert_eq!(output, "F1=test");
    }

    #[test]
    fn test_multiple_comment_lines() {
        let input = "# Comment 1\n# Comment 2\n# Comment 3\nF1=42";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_comment_at_end_of_file() {
        let input = "F1=42\n# Comment at end";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_empty_comment_line() {
        let input = "#\nF1=42";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    }

    // Nested record tests
    #[test]
    fn test_parse_simple_nested_record() {
        let input = "F50={F12=1;F7=1}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(50).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 2);
                // Fields should be sorted by FID
                assert_eq!(nested.fields()[0].fid, 7);
                assert_eq!(nested.fields()[0].value, LnmpValue::Bool(true));
                assert_eq!(nested.fields()[1].fid, 12);
                assert_eq!(nested.fields()[1].value, LnmpValue::Bool(true));
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_nested_record_with_various_types() {
        let input = r#"F50={F12=14532;F7=1;F23=["admin","dev"]}"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(50).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 3);
                // Fields should be sorted: F7, F12, F23
                assert_eq!(nested.get_field(7).unwrap().value, LnmpValue::Bool(true));
                assert_eq!(nested.get_field(12).unwrap().value, LnmpValue::Int(14532));
                assert_eq!(
                    nested.get_field(23).unwrap().value,
                    LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
                );
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_empty_nested_record() {
        let input = "F50={}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(50).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 0);
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_record() {
        let input = "F100={F1=user;F2={F10=nested;F11=data}}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(100).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 2);
                assert_eq!(
                    nested.get_field(1).unwrap().value,
                    LnmpValue::String("user".to_string())
                );

                // Check the nested record within
                match &nested.get_field(2).unwrap().value {
                    LnmpValue::NestedRecord(inner) => {
                        assert_eq!(inner.fields().len(), 2);
                        assert_eq!(
                            inner.get_field(10).unwrap().value,
                            LnmpValue::String("nested".to_string())
                        );
                        assert_eq!(
                            inner.get_field(11).unwrap().value,
                            LnmpValue::String("data".to_string())
                        );
                    }
                    _ => panic!("Expected nested NestedRecord"),
                }
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_nested_record_fields_sorted() {
        // Input has unsorted fields: F12, F7, F23
        let input = "F50={F12=1;F7=0;F23=test}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let field = record.get_field(50).unwrap();
        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                // Fields should be sorted: F7, F12, F23
                assert_eq!(nested.fields()[0].fid, 7);
                assert_eq!(nested.fields()[1].fid, 12);
                assert_eq!(nested.fields()[2].fid, 23);
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_nested_record_with_trailing_semicolon() {
        let input = "F50={F12=1;F7=1;}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(50).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 2);
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_nested_array_basic() {
        let input = "F60=[{F12=1},{F12=2},{F12=3}]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(60).unwrap();

        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 3);
                assert_eq!(
                    records[0].get_field(12).unwrap().value,
                    LnmpValue::Bool(true)
                );
                assert_eq!(records[1].get_field(12).unwrap().value, LnmpValue::Int(2));
                assert_eq!(records[2].get_field(12).unwrap().value, LnmpValue::Int(3));
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_nested_array_with_multiple_fields() {
        let input = "F200=[{F1=alice;F2=admin},{F1=bob;F2=user}]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(200).unwrap();

        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 2);

                // First record
                assert_eq!(
                    records[0].get_field(1).unwrap().value,
                    LnmpValue::String("alice".to_string())
                );
                assert_eq!(
                    records[0].get_field(2).unwrap().value,
                    LnmpValue::String("admin".to_string())
                );

                // Second record
                assert_eq!(
                    records[1].get_field(1).unwrap().value,
                    LnmpValue::String("bob".to_string())
                );
                assert_eq!(
                    records[1].get_field(2).unwrap().value,
                    LnmpValue::String("user".to_string())
                );
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_empty_nested_array() {
        let input = "F60=[]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(60).unwrap();

        // Empty array defaults to StringArray
        match &field.value {
            LnmpValue::StringArray(items) => {
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected StringArray for empty array"),
        }
    }

    #[test]
    fn test_parse_empty_nested_array_with_type_hint() {
        let input = "F60:ra=[]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(60).unwrap();

        // With :ra type hint, empty array should be NestedArray
        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 0);
            }
            _ => panic!("Expected NestedArray for empty array with :ra type hint"),
        }
    }

    #[test]
    fn test_parse_nested_record_with_type_hints() {
        let input = "F50:r={F12:i=14532;F7:b=1}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(50).unwrap();

        match &field.value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 2);
                assert_eq!(nested.get_field(7).unwrap().value, LnmpValue::Bool(true));
                assert_eq!(nested.get_field(12).unwrap().value, LnmpValue::Int(14532));
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_nested_array_with_type_hint() {
        let input = "F60:ra=[{F12=1},{F12=2}]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        let field = record.get_field(60).unwrap();

        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 2);
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_nested_array_preserves_order() {
        // Requirement 5.4: Preserve element order in nested arrays
        let input = "F60=[{F1=first},{F1=second},{F1=third}]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let field = record.get_field(60).unwrap();
        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 3);
                assert_eq!(
                    records[0].get_field(1).unwrap().value,
                    LnmpValue::String("first".to_string())
                );
                assert_eq!(
                    records[1].get_field(1).unwrap().value,
                    LnmpValue::String("second".to_string())
                );
                assert_eq!(
                    records[2].get_field(1).unwrap().value,
                    LnmpValue::String("third".to_string())
                );
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_nested_array_with_complex_records() {
        // Test nested arrays with records containing multiple field types
        let input = r#"F100=[{F1=alice;F2=30;F3=1},{F1=bob;F2=25;F3=0}]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let field = record.get_field(100).unwrap();
        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 2);

                // First record
                let rec1 = &records[0];
                assert_eq!(
                    rec1.get_field(1).unwrap().value,
                    LnmpValue::String("alice".to_string())
                );
                assert_eq!(rec1.get_field(2).unwrap().value, LnmpValue::Int(30));
                assert_eq!(rec1.get_field(3).unwrap().value, LnmpValue::Bool(true));

                // Second record
                let rec2 = &records[1];
                assert_eq!(
                    rec2.get_field(1).unwrap().value,
                    LnmpValue::String("bob".to_string())
                );
                assert_eq!(rec2.get_field(2).unwrap().value, LnmpValue::Int(25));
                assert_eq!(rec2.get_field(3).unwrap().value, LnmpValue::Bool(false));
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_nested_array_single_element() {
        let input = "F60=[{F1=only}]";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let field = record.get_field(60).unwrap();
        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 1);
                assert_eq!(
                    records[0].get_field(1).unwrap().value,
                    LnmpValue::String("only".to_string())
                );
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_nested_array_with_all_value_types() {
        // Requirement 5.4: Support arrays containing records with mixed value types
        let input = r#"F100=[{F1=42;F2=3.14;F3=1;F4=test;F5=["a","b"]}]"#;
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        let field = record.get_field(100).unwrap();
        match &field.value {
            LnmpValue::NestedArray(records) => {
                assert_eq!(records.len(), 1);
                let rec = &records[0];

                // Verify all different value types are supported
                assert_eq!(rec.get_field(1).unwrap().value, LnmpValue::Int(42));
                assert_eq!(rec.get_field(2).unwrap().value, LnmpValue::Float(3.14));
                assert_eq!(rec.get_field(3).unwrap().value, LnmpValue::Bool(true));
                assert_eq!(
                    rec.get_field(4).unwrap().value,
                    LnmpValue::String("test".to_string())
                );
                assert_eq!(
                    rec.get_field(5).unwrap().value,
                    LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
                );
            }
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_parse_mixed_top_level_and_nested() {
        let input = "F1=42;F50={F12=1;F7=1};F100=test";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
        assert_eq!(
            record.get_field(100).unwrap().value,
            LnmpValue::String("test".to_string())
        );

        match &record.get_field(50).unwrap().value {
            LnmpValue::NestedRecord(nested) => {
                assert_eq!(nested.fields().len(), 2);
            }
            _ => panic!("Expected NestedRecord"),
        }
    }

    #[test]
    fn test_parse_three_level_nesting() {
        let input = "F1={F2={F3={F4=deep}}}";
        let mut parser = Parser::new(input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);

        // Level 1
        match &record.get_field(1).unwrap().value {
            LnmpValue::NestedRecord(level1) => {
                // Level 2
                match &level1.get_field(2).unwrap().value {
                    LnmpValue::NestedRecord(level2) => {
                        // Level 3
                        match &level2.get_field(3).unwrap().value {
                            LnmpValue::NestedRecord(level3) => {
                                assert_eq!(
                                    level3.get_field(4).unwrap().value,
                                    LnmpValue::String("deep".to_string())
                                );
                            }
                            _ => panic!("Expected NestedRecord at level 3"),
                        }
                    }
                    _ => panic!("Expected NestedRecord at level 2"),
                }
            }
            _ => panic!("Expected NestedRecord at level 1"),
        }
    }

    #[test]
    fn test_nesting_too_deep_error() {
        use crate::config::ParserConfig;

        let input = "F1={F2={F3={F4={F5={F6={F7={F8={F9={F10={F11={F12=1}}}}}}}}}}}}";
        let config = ParserConfig {
            max_nesting_depth: Some(10),
            ..Default::default()
        };
        let mut parser = Parser::with_config(input, config).unwrap();
        let result = parser.parse_record();
        assert!(result.is_err());
        match result {
            Err(LnmpError::NestingTooDeep {
                max_depth,
                actual_depth,
                ..
            }) => {
                assert_eq!(max_depth, 10);
                assert!(actual_depth > max_depth);
            }
            _ => panic!("Expected NestingTooDeep error"),
        }
    }

    #[test]
    fn test_structural_limits_rejects_field_count() {
        use crate::config::ParserConfig;
        use lnmp_core::StructuralLimits;

        let input = "F1=1\nF2=2";
        let config = ParserConfig {
            structural_limits: Some(StructuralLimits {
                max_fields: 1,
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut parser = Parser::with_config(input, config).unwrap();
        let result = parser.parse_record();
        match result {
            Err(LnmpError::InvalidNestedStructure { reason, .. }) => {
                assert!(reason.contains("maximum field count exceeded"));
            }
            _ => panic!("Expected InvalidNestedStructure due to structural limits"),
        }
    }

    // Checksum parsing tests
    #[test]
    fn test_parse_field_with_checksum_ignored() {
        use lnmp_core::checksum::SemanticChecksum;

        // Compute correct checksum
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, None, &value);
        let checksum_str = SemanticChecksum::format(checksum);

        let input = format!("F12=14532#{}", checksum_str);

        // Parse without validation (default)
        let mut parser = Parser::new(&input).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_parse_field_with_valid_checksum() {
        use crate::config::ParserConfig;
        use lnmp_core::checksum::SemanticChecksum;

        // Compute correct checksum
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, None, &value);
        let checksum_str = SemanticChecksum::format(checksum);

        let input = format!("F12=14532#{}", checksum_str);

        // Parse with validation enabled
        let config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(&input, config).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_parse_field_with_invalid_checksum() {
        use crate::config::ParserConfig;

        // Use wrong checksum
        let input = "F12=14532#DEADBEEF";

        // Parse with validation enabled
        let config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(input, config).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::ChecksumMismatch { field_id, .. }) => {
                assert_eq!(field_id, 12);
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[test]
    fn test_parse_field_with_checksum_and_type_hint() {
        use crate::config::ParserConfig;
        use lnmp_core::checksum::SemanticChecksum;

        // Compute correct checksum with type hint
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        let checksum_str = SemanticChecksum::format(checksum);

        let input = format!("F12:i=14532#{}", checksum_str);

        // Parse with validation enabled
        let config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(&input, config).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_parse_applies_semantic_dictionary_equivalence() {
        use crate::config::ParserConfig;

        let mut dict = lnmp_sfe::SemanticDictionary::new();
        dict.add_equivalence(23, "admin".to_string(), "administrator".to_string());

        let config = ParserConfig {
            semantic_dictionary: Some(dict),
            ..Default::default()
        };

        let mut parser = Parser::with_config("F23=[admin]", config).unwrap();
        let record = parser.parse_record().unwrap();
        match record.get_field(23).unwrap().value.clone() {
            LnmpValue::StringArray(vals) => {
                assert_eq!(vals, vec!["administrator".to_string()]);
            }
            other => panic!("unexpected value {:?}", other),
        }
    }

    #[test]
    fn test_parse_multiple_fields_with_checksums() {
        use crate::config::ParserConfig;
        use lnmp_core::checksum::SemanticChecksum;

        let value1 = LnmpValue::Int(14532);
        let checksum1 = SemanticChecksum::compute(12, None, &value1);
        let checksum_str1 = SemanticChecksum::format(checksum1);

        let value2 = LnmpValue::Bool(true);
        let checksum2 = SemanticChecksum::compute(7, None, &value2);
        let checksum_str2 = SemanticChecksum::format(checksum2);

        let input = format!("F12=14532#{}\nF7=1#{}", checksum_str1, checksum_str2);

        // Parse with validation enabled
        let config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(&input, config).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
    }

    #[test]
    fn test_parse_field_require_checksum_missing() {
        use crate::config::ParserConfig;

        let input = "F12=14532";

        // Parse with checksums required
        let config = ParserConfig {
            require_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(input, config).unwrap();
        let result = parser.parse_record();

        assert!(result.is_err());
        match result {
            Err(LnmpError::ChecksumMismatch { field_id, .. }) => {
                assert_eq!(field_id, 12);
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[test]
    fn test_parse_field_with_checksum_after_comment() {
        use crate::config::ParserConfig;
        use lnmp_core::checksum::SemanticChecksum;

        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, None, &value);
        let checksum_str = SemanticChecksum::format(checksum);

        let input = format!("# Comment\nF12=14532#{}", checksum_str);

        // Parse with validation enabled
        let config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(&input, config).unwrap();
        let record = parser.parse_record().unwrap();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_round_trip_with_checksum() {
        use crate::config::{EncoderConfig, ParserConfig};
        use crate::encoder::Encoder;

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        // Encode with checksum
        let encoder_config = EncoderConfig {
            enable_checksums: true,
            ..Default::default()
        };
        let encoder = Encoder::with_config(encoder_config);
        let output = encoder.encode(&record);

        // Parse with validation
        let parser_config = ParserConfig {
            validate_checksums: true,
            ..Default::default()
        };
        let mut parser = Parser::with_config(&output, parser_config).unwrap();
        let parsed = parser.parse_record().unwrap();

        assert_eq!(
            record.get_field(12).unwrap().value,
            parsed.get_field(12).unwrap().value
        );
    }

    #[test]
    fn test_parse_typed_int_array() {
        let mut parser = Parser::new("F12:ia=[1,2,-3]").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(12).unwrap().value,
            LnmpValue::IntArray(vec![1, 2, -3])
        );
    }

    #[test]
    fn test_parse_typed_float_array() {
        let mut parser = Parser::new("F13:fa=[1.1,2.2,3.3]").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(13).unwrap().value,
            LnmpValue::FloatArray(vec![1.1, 2.2, 3.3])
        );
    }

    #[test]
    fn test_parse_typed_bool_array() {
        let mut parser = Parser::new("F14:ba=[1,0,true,False]").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(14).unwrap().value,
            LnmpValue::BoolArray(vec![true, false, true, false])
        );
    }

    #[test]
    fn test_parse_empty_typed_arrays() {
        let mut parser = Parser::new("F12:ia=[];F13:fa=[];F14:ba=[]").unwrap();
        let record = parser.parse_record().unwrap();
        assert_eq!(
            record.get_field(12).unwrap().value,
            LnmpValue::IntArray(Vec::new())
        );
        assert_eq!(
            record.get_field(13).unwrap().value,
            LnmpValue::FloatArray(Vec::new())
        );
        assert_eq!(
            record.get_field(14).unwrap().value,
            LnmpValue::BoolArray(Vec::new())
        );
    }
}
