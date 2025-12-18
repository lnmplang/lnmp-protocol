//! Binary encoder for converting LNMP text format to binary format.
//!
//! The BinaryEncoder converts LNMP records from text format (v0.3) to binary format (v0.4).
//! It ensures canonical form by sorting fields by FID before encoding.

use super::delta::{DeltaConfig, DeltaEncoder};
use super::error::BinaryError;
use super::frame::BinaryFrame;
use crate::config::{ParserConfig, ParsingMode, TextInputMode};
use crate::parser::Parser;
use lnmp_core::{LnmpField, LnmpRecord};

/// Configuration for binary encoding
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Whether to validate canonical form before encoding
    pub validate_canonical: bool,
    /// Whether to sort fields by FID (ensures canonical binary)
    pub sort_fields: bool,
    /// How to preprocess incoming text before parsing
    pub text_input_mode: TextInputMode,
    /// Optional semantic dictionary for value normalization prior to encoding
    pub semantic_dictionary: Option<lnmp_sfe::SemanticDictionary>,

    // v0.5 fields
    /// Whether to enable nested binary structure encoding (v0.5)
    pub enable_nested_binary: bool,
    /// Maximum nesting depth for nested structures (v0.5)
    pub max_depth: usize,
    /// Whether to enable streaming mode for large payloads (v0.5)
    pub streaming_mode: bool,
    /// Whether to enable delta encoding mode (v0.5)
    pub delta_mode: bool,
    /// Chunk size for streaming mode in bytes (v0.5)
    pub chunk_size: usize,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            validate_canonical: false,
            sort_fields: true,
            text_input_mode: TextInputMode::Strict,
            semantic_dictionary: None,
            // v0.5 defaults
            enable_nested_binary: false,
            max_depth: 32,
            streaming_mode: false,
            delta_mode: false,
            chunk_size: 4096,
        }
    }
}

impl EncoderConfig {
    /// Creates a new encoder configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to validate canonical form before encoding
    pub fn with_validate_canonical(mut self, validate: bool) -> Self {
        self.validate_canonical = validate;
        self.sort_fields = true;
        self
    }

    /// Sets whether to sort fields by FID
    pub fn with_sort_fields(mut self, sort: bool) -> Self {
        self.sort_fields = sort;
        self
    }

    /// Sets how text input should be pre-processed before parsing.
    pub fn with_text_input_mode(mut self, mode: TextInputMode) -> Self {
        self.text_input_mode = mode;
        self
    }

    /// Attaches a semantic dictionary for normalization prior to encoding.
    pub fn with_semantic_dictionary(mut self, dict: lnmp_sfe::SemanticDictionary) -> Self {
        self.semantic_dictionary = Some(dict);
        self
    }

    // v0.5 builder methods

    /// Enables nested binary structure encoding (v0.5)
    pub fn with_nested_binary(mut self, enable: bool) -> Self {
        self.enable_nested_binary = enable;
        self
    }

    /// Sets maximum nesting depth for nested structures (v0.5)
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Enables streaming mode for large payloads (v0.5)
    pub fn with_streaming_mode(mut self, enable: bool) -> Self {
        self.streaming_mode = enable;
        self
    }

    /// Enables delta encoding mode (v0.5)
    pub fn with_delta_mode(mut self, enable: bool) -> Self {
        self.delta_mode = enable;
        self
    }

    /// Sets chunk size for streaming mode in bytes (v0.5)
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Configures the encoder for v0.4 compatibility mode
    ///
    /// This disables all v0.5 features (nested structures, streaming, delta encoding)
    /// to ensure the output is compatible with v0.4 decoders.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::binary::EncoderConfig;
    ///
    /// let config = EncoderConfig::new().with_v0_4_compatibility();
    /// assert!(!config.enable_nested_binary);
    /// assert!(!config.streaming_mode);
    /// assert!(!config.delta_mode);
    /// ```
    pub fn with_v0_4_compatibility(mut self) -> Self {
        self.enable_nested_binary = false;
        self.streaming_mode = false;
        self.delta_mode = false;
        self
    }
}

/// Binary encoder for LNMP v0.4
///
/// Converts LNMP records from text format (v0.3) to binary format (v0.4).
/// The encoder ensures canonical form by sorting fields by FID before encoding.
///
/// # Examples
///
/// ```
/// use lnmp_codec::binary::BinaryEncoder;
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
///
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField {
///     fid: 7,
///     value: LnmpValue::Bool(true),
/// });
/// record.add_field(LnmpField {
///     fid: 12,
///     value: LnmpValue::Int(14532),
/// });
///
/// let encoder = BinaryEncoder::new();
/// let binary = encoder.encode(&record).unwrap();
/// ```
#[derive(Debug)]
pub struct BinaryEncoder {
    config: EncoderConfig,
    normalizer: Option<crate::normalizer::ValueNormalizer>,
}

impl BinaryEncoder {
    /// Creates a new binary encoder with default configuration
    ///
    /// Default configuration:
    /// - `validate_canonical`: false
    /// - `sort_fields`: true
    pub fn new() -> Self {
        Self {
            config: EncoderConfig::default(),
            normalizer: None,
        }
    }

    /// Creates a binary encoder with custom configuration
    pub fn with_config(config: EncoderConfig) -> Self {
        let normalizer = config.semantic_dictionary.as_ref().map(|dict| {
            crate::normalizer::ValueNormalizer::new(crate::normalizer::NormalizationConfig {
                semantic_dictionary: Some(dict.clone()),
                ..crate::normalizer::NormalizationConfig::default()
            })
        });
        Self { config, normalizer }
    }

    /// Sets delta mode on the encoder instance in a fluent interface style.
    pub fn with_delta_mode(mut self, enable: bool) -> Self {
        self.config.delta_mode = enable;
        self
    }

    /// Encodes an LnmpRecord to binary format
    ///
    /// The encoder will:
    /// 1. Sort fields by FID (if sort_fields is enabled)
    /// 2. Convert the record to a BinaryFrame
    /// 3. Encode the frame to bytes
    ///
    /// # Arguments
    ///
    /// * `record` - The LNMP record to encode
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the binary-encoded record
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - The record contains nested structures when v0.4 compatibility is enabled
    /// - Field conversion fails
    pub fn encode(&self, record: &LnmpRecord) -> Result<Vec<u8>, BinaryError> {
        // Guardrails for unimplemented v0.5 features
        if self.config.streaming_mode {
            return Err(BinaryError::UnsupportedFeature {
                feature: "binary streaming mode".to_string(),
            });
        }
        if self.config.enable_nested_binary {
            return Err(BinaryError::UnsupportedFeature {
                feature: "nested binary encoding".to_string(),
            });
        }
        if self.config.chunk_size == 0 {
            return Err(BinaryError::UnsupportedFeature {
                feature: "chunk_size=0 is invalid".to_string(),
            });
        }

        // Check for v0.4 compatibility mode
        if !self.config.enable_nested_binary {
            // In v0.4 compatibility mode, validate that the record doesn't contain nested structures
            self.validate_v0_4_compatibility(record)?;
        }

        // Apply semantic normalization if configured
        let normalized_record = if let Some(norm) = &self.normalizer {
            let mut out = LnmpRecord::new();
            for field in record.fields() {
                let normalized_value = norm.normalize_with_fid(Some(field.fid), &field.value);
                out.add_field(LnmpField {
                    fid: field.fid,
                    value: normalized_value,
                });
            }
            out
        } else {
            record.clone()
        };

        // Convert record to BinaryFrame (this automatically sorts by FID)
        let frame = BinaryFrame::from_record(&normalized_record)?;

        // Encode frame to bytes
        Ok(frame.encode())
    }

    /// Validates that a record is compatible with v0.4 binary format
    ///
    /// This checks that the record doesn't contain any nested structures (NestedRecord or NestedArray),
    /// which are not supported in v0.4.
    ///
    /// # Errors
    ///
    /// Returns `BinaryError::InvalidValue` if the record contains nested structures
    fn validate_v0_4_compatibility(&self, record: &LnmpRecord) -> Result<(), BinaryError> {
        use lnmp_core::LnmpValue;

        for field in record.fields() {
            match &field.value {
                LnmpValue::NestedRecord(_) => {
                    return Err(BinaryError::InvalidValue {
                        field_id: field.fid,
                        type_tag: 0x04,
                        reason: "Nested records not supported in v0.4 binary format. Use v0.5 with enable_nested_binary=true or convert to flat structure.".to_string(),
                    });
                }
                LnmpValue::NestedArray(_) => {
                    return Err(BinaryError::InvalidValue {
                        field_id: field.fid,
                        type_tag: 0x07,
                        reason: "Nested arrays not supported in v0.4 binary format. Use v0.5 with enable_nested_binary=true or convert to flat structure.".to_string(),
                    });
                }
                _ => {} // Other types are fine
            }
        }

        Ok(())
    }

    /// Encodes text format directly to binary
    ///
    /// This method:
    /// 1. Parses the text using the v0.3 parser
    /// 2. Converts the parsed record to binary format
    /// 3. Ensures fields are sorted by FID
    ///
    /// # Arguments
    ///
    /// * `text` - LNMP text format string (v0.3)
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the binary-encoded record
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Text parsing fails
    /// - The record contains nested structures (not supported in v0.4)
    /// - Field conversion fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::binary::BinaryEncoder;
    ///
    /// let text = "F7=1;F12=14532;F23=[\"admin\",\"dev\"]";
    /// let encoder = BinaryEncoder::new();
    /// let binary = encoder.encode_text(text).unwrap();
    /// ```
    pub fn encode_text(&self, text: &str) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_with_mode(text, self.config.text_input_mode)
    }

    /// Encodes text using strict input rules (no sanitization).
    pub fn encode_text_strict(&self, text: &str) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_with_mode(text, TextInputMode::Strict)
    }

    /// Encodes text using lenient sanitization before parsing.
    pub fn encode_text_lenient(&self, text: &str) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_with_mode(text, TextInputMode::Lenient)
    }

    /// Convenience: enforce strict input + strict grammar.
    pub fn encode_text_strict_profile(&self, text: &str) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_with_profile(text, TextInputMode::Strict, ParsingMode::Strict)
    }

    /// Convenience: lenient input + loose grammar (LLM-facing).
    pub fn encode_text_llm_profile(&self, text: &str) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_with_profile(text, TextInputMode::Lenient, ParsingMode::Loose)
    }

    /// Encodes text with both text input mode and parsing mode specified.
    ///
    /// This is useful to enforce strict grammar (ParsingMode::Strict) together with strict input,
    /// or to provide a fully lenient LLM-facing path (ParsingMode::Loose + Lenient input).
    pub fn encode_text_with_profile(
        &self,
        text: &str,
        text_mode: TextInputMode,
        parsing_mode: ParsingMode,
    ) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_internal(text, text_mode, parsing_mode)
    }

    fn encode_text_with_mode(
        &self,
        text: &str,
        mode: TextInputMode,
    ) -> Result<Vec<u8>, BinaryError> {
        self.encode_text_internal(text, mode, ParserConfig::default().mode)
    }

    fn encode_text_internal(
        &self,
        text: &str,
        text_mode: TextInputMode,
        parsing_mode: ParsingMode,
    ) -> Result<Vec<u8>, BinaryError> {
        let parser_config = ParserConfig {
            mode: parsing_mode,
            text_input_mode: text_mode,
            ..ParserConfig::default()
        };

        let mut parser = Parser::with_config(text, parser_config)
            .map_err(|e| BinaryError::TextFormatError { source: e })?;
        let record = parser
            .parse_record()
            .map_err(|e| BinaryError::TextFormatError { source: e })?;
        self.encode(&record)
    }

    /// Encodes delta packet (binary) from a base record and updated record.
    ///
    /// If the encoder config has `delta_mode=true` or a non-None `delta_config` is provided,
    /// this method computes a delta using `DeltaEncoder` and returns the encoded delta bytes.
    /// Otherwise it returns an error.
    pub fn encode_delta_from(
        &self,
        base: &LnmpRecord,
        updated: &LnmpRecord,
        delta_config: Option<DeltaConfig>,
    ) -> Result<Vec<u8>, BinaryError> {
        // Determine config to use. Merge encoder config delta_mode into the provided delta_config
        let mut config = delta_config.unwrap_or_default();
        // If encoder's delta_mode is enabled, also enable delta in the delta config
        if self.config.delta_mode {
            config.enable_delta = true;
        }

        // Validate that delta is enabled in either encoder config or provided delta config
        if !self.config.delta_mode && !config.enable_delta {
            return Err(BinaryError::DeltaError {
                reason: "Delta mode not enabled in encoder or provided delta config".to_string(),
            });
        }

        // Check configs

        let delta_encoder = DeltaEncoder::with_config(config);
        let compute_result = delta_encoder.compute_delta(base, updated);
        match compute_result {
            Ok(ops) => match delta_encoder.encode_delta(&ops) {
                Ok(bytes) => Ok(bytes),
                Err(e) => Err(BinaryError::DeltaError {
                    reason: format!("encode_delta failed: {}", e),
                }),
            },
            Err(e) => Err(BinaryError::DeltaError {
                reason: format!("compute_delta failed: {}", e),
            }),
        }
    }
}

impl Default for BinaryEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::*;
    use crate::binary::BinaryDecoder;
    use lnmp_core::{LnmpField, LnmpValue};

    #[test]
    fn test_new_encoder() {
        let encoder = BinaryEncoder::new();
        assert!(encoder.config.sort_fields);
        assert!(!encoder.config.validate_canonical);
        // v0.5 defaults
        assert_eq!(encoder.config.text_input_mode, TextInputMode::Strict);
        assert!(!encoder.config.enable_nested_binary);
        assert_eq!(encoder.config.max_depth, 32);
        assert!(!encoder.config.streaming_mode);
        assert!(!encoder.config.delta_mode);
        assert_eq!(encoder.config.chunk_size, 4096);
    }

    #[test]
    fn test_encoder_with_config() {
        let config = EncoderConfig::new()
            .with_validate_canonical(true)
            .with_sort_fields(false);

        let encoder = BinaryEncoder::with_config(config);
        assert!(!encoder.config.sort_fields);
        assert!(encoder.config.validate_canonical);
    }

    #[test]
    fn test_encode_empty_record() {
        let record = LnmpRecord::new();
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        // Should have VERSION, FLAGS, and ENTRY_COUNT=0
        assert_eq!(binary.len(), 3);
        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x00); // ENTRY_COUNT=0
    }

    #[test]
    fn test_encode_single_field() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        // Should have VERSION, FLAGS, ENTRY_COUNT=1, and entry data
        assert!(binary.len() > 3);
        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x01); // ENTRY_COUNT=1
    }

    #[test]
    fn test_encode_multiple_fields() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x03); // ENTRY_COUNT=3
    }

    #[test]
    fn test_encode_sorts_fields() {
        let mut record = LnmpRecord::new();
        // Add fields in non-sorted order
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string()]),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        // Decode to verify field order
        use super::super::frame::BinaryFrame;
        let frame = BinaryFrame::decode(&binary).unwrap();
        let decoded_record = frame.to_record();

        // Fields should be in sorted order: 7, 12, 23
        let fields = decoded_record.fields();
        assert_eq!(fields[0].fid, 7);
        assert_eq!(fields[1].fid, 12);
        assert_eq!(fields[2].fid, 23);
    }

    #[test]
    fn test_encode_delta_integration() {
        use crate::binary::{DeltaConfig, DeltaDecoder};
        use lnmp_core::{LnmpField, LnmpValue};

        let mut base = LnmpRecord::new();
        base.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        base.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("v1".to_string()),
        });

        let mut updated = base.clone();
        updated.remove_field(1);
        updated.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(2),
        });

        // BinaryEncoder delta_mode disabled should return DeltaError
        let encoder = BinaryEncoder::new();
        let err = encoder
            .encode_delta_from(&base, &updated, None)
            .unwrap_err();
        assert!(matches!(err, BinaryError::DeltaError { .. }));

        // Provide delta config to enable delta
        let config = DeltaConfig::new().with_enable_delta(true);
        let bytes = encoder
            .encode_delta_from(&base, &updated, Some(config))
            .unwrap();
        assert_eq!(bytes[0], crate::binary::DELTA_TAG);

        // Use delta decoder to decode and apply
        let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));
        let ops = delta_decoder.decode_delta(&bytes).unwrap();
        let mut result = base.clone();
        delta_decoder.apply_delta(&mut result, &ops).unwrap();

        // After applying delta, result should equal updated
        assert_eq!(result.fields(), updated.fields());
    }

    #[test]
    fn test_encode_delta_from_encoder_config_enabled() {
        use crate::binary::EncoderConfig;
        use lnmp_core::{LnmpField, LnmpValue};

        let mut base = LnmpRecord::new();
        base.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        base.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("v1".to_string()),
        });

        let mut updated = base.clone();
        updated.remove_field(1);
        updated.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(2),
        });

        // BinaryEncoder with delta mode enabled should succeed when using encode_delta_from and None delta_config
        let config = EncoderConfig::new().with_delta_mode(true);
        let encoder = BinaryEncoder::with_config(config);
        let bytes = encoder.encode_delta_from(&base, &updated, None).unwrap();
        assert_eq!(bytes[0], crate::binary::DELTA_TAG);
    }

    #[test]
    fn test_encode_text_simple() {
        let text = "F7=1";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x01); // ENTRY_COUNT=1
    }

    #[test]
    fn test_encode_text_multiple_fields() {
        let text = "F7=1;F12=14532;F23=[\"admin\",\"dev\"]";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x03); // ENTRY_COUNT=3
    }

    #[test]
    fn test_encode_text_unsorted() {
        let text = "F23=[\"admin\"];F7=1;F12=14532";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        // Decode to verify fields are sorted
        use super::super::frame::BinaryFrame;
        let frame = BinaryFrame::decode(&binary).unwrap();
        let decoded_record = frame.to_record();

        let fields = decoded_record.fields();
        assert_eq!(fields[0].fid, 7);
        assert_eq!(fields[1].fid, 12);
        assert_eq!(fields[2].fid, 23);
    }

    #[test]
    fn test_encode_text_with_newlines() {
        let text = "F7=1\nF12=14532\nF23=[\"admin\",\"dev\"]";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x03); // ENTRY_COUNT=3
    }

    #[test]
    fn test_encode_text_lenient_repairs_quotes() {
        let text = "F7=\"hello";
        let encoder = BinaryEncoder::with_config(
            EncoderConfig::new().with_text_input_mode(TextInputMode::Lenient),
        );
        let binary = encoder.encode_text(text).unwrap();

        assert_eq!(binary[0], 0x04); // VERSION
    }

    #[test]
    fn test_encode_text_all_types() {
        let text = "F1=-42;F2=3.14;F3=0;F4=\"hello\";F5=[\"a\",\"b\"]";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        // Decode and verify all types
        use super::super::frame::BinaryFrame;
        let frame = BinaryFrame::decode(&binary).unwrap();
        let decoded_record = frame.to_record();

        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Int(-42)
        );
        assert_eq!(
            decoded_record.get_field(2).unwrap().value,
            LnmpValue::Float(3.14)
        );
        assert_eq!(
            decoded_record.get_field(3).unwrap().value,
            LnmpValue::Bool(false)
        );
        assert_eq!(
            decoded_record.get_field(4).unwrap().value,
            LnmpValue::String("hello".to_string())
        );
        assert_eq!(
            decoded_record.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_encode_text_strict_profile_rejects_loose_input() {
        let text = "F7=1;F12=14532; # comment should fail strict grammar";
        let encoder = BinaryEncoder::new();
        let err = encoder.encode_text_strict_profile(text).unwrap_err();
        match err {
            BinaryError::TextFormatError { .. } => {}
            other => panic!("expected TextFormatError, got {:?}", other),
        }
    }

    #[test]
    fn test_encode_text_llm_profile_lenient_succeeds() {
        let text = "F7=\"hello;world\";F8=unquoted token";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text_llm_profile(text).unwrap();
        assert_eq!(binary[0], 0x04);
    }

    #[test]
    fn test_encode_text_invalid() {
        let text = "INVALID";
        let encoder = BinaryEncoder::new();
        let result = encoder.encode_text(text);

        assert!(result.is_err());
    }

    #[test]
    fn test_encode_text_empty() {
        let text = "";
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(text).unwrap();

        // Empty text should produce empty record
        assert_eq!(binary.len(), 3);
        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x00); // ENTRY_COUNT=0
    }

    #[test]
    fn test_encode_all_value_types() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(2.718),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(false),
        });
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("world".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["x".to_string(), "y".to_string()]),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        // Decode and verify
        use super::super::frame::BinaryFrame;
        let frame = BinaryFrame::decode(&binary).unwrap();
        let decoded_record = frame.to_record();

        assert_eq!(decoded_record.fields().len(), 5);
        assert_eq!(
            decoded_record.get_field(1).unwrap().value,
            LnmpValue::Int(42)
        );
        assert_eq!(
            decoded_record.get_field(2).unwrap().value,
            LnmpValue::Float(2.718)
        );
        assert_eq!(
            decoded_record.get_field(3).unwrap().value,
            LnmpValue::Bool(false)
        );
        assert_eq!(
            decoded_record.get_field(4).unwrap().value,
            LnmpValue::String("world".to_string())
        );
        assert_eq!(
            decoded_record.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_default_encoder() {
        let encoder = BinaryEncoder::default();
        assert!(encoder.config.sort_fields);
        assert!(!encoder.config.validate_canonical);
    }

    #[test]
    fn test_encoder_config_builder() {
        let config = EncoderConfig::new()
            .with_validate_canonical(true)
            .with_sort_fields(true);

        assert!(config.validate_canonical);
        assert!(config.sort_fields);
    }

    #[test]
    fn test_encoder_config_v05_fields() {
        let config = EncoderConfig::new()
            .with_nested_binary(true)
            .with_max_depth(64)
            .with_streaming_mode(true)
            .with_delta_mode(true)
            .with_chunk_size(8192);

        assert!(config.enable_nested_binary);
        assert_eq!(config.max_depth, 64);
        assert!(config.streaming_mode);
        assert!(config.delta_mode);
        assert_eq!(config.chunk_size, 8192);
    }

    #[test]
    fn test_encoder_config_v05_defaults() {
        let config = EncoderConfig::default();

        assert!(!config.enable_nested_binary);
        assert_eq!(config.max_depth, 32);
        assert!(!config.streaming_mode);
        assert!(!config.delta_mode);
        assert_eq!(config.chunk_size, 4096);
    }

    #[test]
    fn test_encoder_config_backward_compatibility() {
        // v0.4 configurations should work without any changes
        let v04_config = EncoderConfig::new()
            .with_validate_canonical(true)
            .with_sort_fields(true);

        // v0.4 fields should work as before
        assert!(v04_config.validate_canonical);
        assert!(v04_config.sort_fields);

        // v0.5 fields should have safe defaults (disabled)
        assert!(!v04_config.enable_nested_binary);
        assert!(!v04_config.streaming_mode);
        assert!(!v04_config.delta_mode);
    }

    #[test]
    fn test_encoder_config_mixed_v04_v05() {
        // Test that v0.4 and v0.5 configurations can be mixed
        let config = EncoderConfig::new()
            .with_validate_canonical(true) // v0.4
            .with_nested_binary(true) // v0.5
            .with_sort_fields(true) // v0.4
            .with_streaming_mode(true); // v0.5

        assert!(config.validate_canonical);
        assert!(config.sort_fields);
        assert!(config.enable_nested_binary);
        assert!(config.streaming_mode);
    }

    #[test]
    fn test_encoder_applies_semantic_dictionary() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["Admin".to_string()]),
        });

        let mut dict = lnmp_sfe::SemanticDictionary::new();
        dict.add_equivalence(23, "Admin".to_string(), "admin".to_string());

        let config = EncoderConfig::new()
            .with_semantic_dictionary(dict)
            .with_validate_canonical(true);
        let encoder = BinaryEncoder::with_config(config);

        let binary = encoder.encode(&record).unwrap();
        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        match decoded.get_field(23).unwrap().value.clone() {
            LnmpValue::StringArray(vals) => assert_eq!(vals, vec!["admin".to_string()]),
            other => panic!("unexpected value {:?}", other),
        }
    }

    #[test]
    fn test_encoder_rejects_streaming_mode_until_implemented() {
        let config = EncoderConfig::new().with_streaming_mode(true);
        let encoder = BinaryEncoder::with_config(config);
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        let err = encoder.encode(&record).unwrap_err();
        assert!(matches!(err, BinaryError::UnsupportedFeature { .. }));
    }

    #[test]
    fn test_encoder_rejects_nested_binary_flag_until_implemented() {
        let config = EncoderConfig::new().with_nested_binary(true);
        let encoder = BinaryEncoder::with_config(config);
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        let err = encoder.encode(&record).unwrap_err();
        assert!(matches!(err, BinaryError::UnsupportedFeature { .. }));
    }

    #[test]
    fn test_encoder_rejects_zero_chunk_size() {
        let config = EncoderConfig::new().with_chunk_size(0);
        let encoder = BinaryEncoder::with_config(config);
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        let err = encoder.encode(&record).unwrap_err();
        assert!(matches!(err, BinaryError::UnsupportedFeature { .. }));
    }

    #[test]
    fn test_encoder_v04_mode_encoding() {
        // Test that encoder with v0.5 disabled behaves like v0.4
        let config = EncoderConfig::new()
            .with_nested_binary(false)
            .with_streaming_mode(false)
            .with_delta_mode(false);

        let encoder = BinaryEncoder::with_config(config);

        // Should encode a simple record just like v0.4
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let binary = encoder.encode(&record).unwrap();

        // Should produce v0.4 compatible output
        assert_eq!(binary[0], 0x04); // VERSION
        assert_eq!(binary[1], 0x00); // FLAGS
        assert_eq!(binary[2], 0x01); // ENTRY_COUNT=1
    }
}
