//! Binary decoder for converting LNMP binary format to text format.
//!
//! The BinaryDecoder converts LNMP records from binary format (v0.4) to text format (v0.3).
//! It validates the binary structure and ensures canonical form compliance.

use super::error::BinaryError;
use super::frame::BinaryFrame;
use crate::encoder::Encoder;
use lnmp_core::LnmpRecord;

/// Configuration for binary decoding
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Whether to validate field ordering (canonical form)
    pub validate_ordering: bool,
    /// Whether to check for trailing data
    pub strict_parsing: bool,

    // v0.5 fields
    /// Whether to allow streaming frame processing (v0.5)
    pub allow_streaming: bool,
    /// Whether to validate nesting depth and structure (v0.5)
    pub validate_nesting: bool,
    /// Whether to allow delta packet processing (v0.5)
    pub allow_delta: bool,
    /// Maximum nesting depth for nested structures (v0.5)
    pub max_depth: usize,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            validate_ordering: false,
            strict_parsing: false,
            // v0.5 defaults
            allow_streaming: false,
            validate_nesting: false,
            allow_delta: false,
            max_depth: 32,
        }
    }
}

impl DecoderConfig {
    /// Creates a new decoder configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to validate field ordering
    pub fn with_validate_ordering(mut self, validate: bool) -> Self {
        self.validate_ordering = validate;
        self
    }

    /// Sets whether to check for trailing data
    pub fn with_strict_parsing(mut self, strict: bool) -> Self {
        self.strict_parsing = strict;
        self
    }

    // v0.5 builder methods

    /// Enables streaming frame processing (v0.5)
    pub fn with_streaming(mut self, allow: bool) -> Self {
        self.allow_streaming = allow;
        self
    }

    /// Enables nesting validation (v0.5)
    pub fn with_validate_nesting(mut self, validate: bool) -> Self {
        self.validate_nesting = validate;
        self
    }

    /// Enables delta packet processing (v0.5)
    pub fn with_delta(mut self, allow: bool) -> Self {
        self.allow_delta = allow;
        self
    }

    /// Sets maximum nesting depth for nested structures (v0.5)
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

/// Binary decoder for LNMP v0.4
///
/// Converts LNMP records from binary format (v0.4) to text format (v0.3).
/// The decoder validates the binary structure and can optionally enforce
/// canonical form compliance.
///
/// # Examples
///
/// ```
/// use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
/// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
///
/// // Create and encode a record
/// let mut record = LnmpRecord::new();
/// record.add_field(LnmpField {
///     fid: 7,
///     value: LnmpValue::Bool(true),
/// });
///
/// let encoder = BinaryEncoder::new();
/// let binary = encoder.encode(&record).unwrap();
///
/// // Decode back to record
/// let decoder = BinaryDecoder::new();
/// let decoded_record = decoder.decode(&binary).unwrap();
/// ```
#[derive(Debug)]
pub struct BinaryDecoder {
    config: DecoderConfig,
}

impl BinaryDecoder {
    /// Creates a new binary decoder with default configuration
    ///
    /// Default configuration:
    /// - `validate_ordering`: false
    /// - `strict_parsing`: false
    pub fn new() -> Self {
        Self {
            config: DecoderConfig::default(),
        }
    }

    /// Creates a binary decoder with custom configuration
    pub fn with_config(config: DecoderConfig) -> Self {
        Self { config }
    }

    /// Decodes binary format to LnmpRecord
    ///
    /// The decoder will:
    /// 1. Validate the version byte (must be 0x04)
    /// 2. Decode the BinaryFrame from bytes
    /// 3. Validate field ordering (if validate_ordering is enabled)
    /// 4. Check for trailing data (if strict_parsing is enabled)
    /// 5. Convert the frame to an LnmpRecord
    ///
    /// # Arguments
    ///
    /// * `bytes` - Binary-encoded LNMP data
    ///
    /// # Returns
    ///
    /// An LnmpRecord representing the decoded data
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Version byte is not 0x04 (UnsupportedVersion)
    /// - Binary data is malformed (UnexpectedEof, InvalidVarInt, etc.)
    /// - Field ordering is invalid (CanonicalViolation, if validate_ordering is enabled)
    /// - Trailing data is present (TrailingData, if strict_parsing is enabled)
    pub fn decode(&self, bytes: &[u8]) -> Result<LnmpRecord, BinaryError> {
        // Decode the binary frame
        let frame = if self.config.validate_ordering {
            BinaryFrame::decode(bytes)?
        } else {
            BinaryFrame::decode_allow_unsorted(bytes)?
        };

        // Convert frame to record
        let record = frame.to_record();

        // Validate field ordering if enabled
        if self.config.validate_ordering {
            self.validate_field_ordering(&record)?;
        }

        // Check for trailing data if strict parsing is enabled
        if self.config.strict_parsing {
            // Calculate how many bytes were consumed
            let consumed = self.calculate_frame_size(bytes)?;
            if consumed < bytes.len() {
                return Err(BinaryError::TrailingData {
                    bytes_remaining: bytes.len() - consumed,
                });
            }
        }

        Ok(record)
    }

    /// Decodes binary format to text format
    ///
    /// This method:
    /// 1. Decodes the binary data to an LnmpRecord
    /// 2. Encodes the record to canonical text format using the v0.3 encoder
    ///
    /// # Arguments
    ///
    /// * `bytes` - Binary-encoded LNMP data
    ///
    /// # Returns
    ///
    /// A string containing the canonical text representation
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if decoding fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
    ///
    /// let text = "F7=1;F12=14532";
    /// let encoder = BinaryEncoder::new();
    /// let binary = encoder.encode_text(text).unwrap();
    ///
    /// let decoder = BinaryDecoder::new();
    /// let decoded_text = decoder.decode_to_text(&binary).unwrap();
    /// ```
    pub fn decode_to_text(&self, bytes: &[u8]) -> Result<String, BinaryError> {
        // Decode to LnmpRecord
        let record = self.decode(bytes)?;

        // Encode to canonical text format using v0.3 encoder
        let encoder = Encoder::new();
        Ok(encoder.encode(&record))
    }

    /// Validates that fields are in ascending FID order (canonical form)
    fn validate_field_ordering(&self, record: &LnmpRecord) -> Result<(), BinaryError> {
        let fields = record.fields();

        for i in 1..fields.len() {
            if fields[i].fid < fields[i - 1].fid {
                return Err(BinaryError::CanonicalViolation {
                    reason: format!(
                        "Fields not in ascending FID order: F{} appears after F{}",
                        fields[i].fid,
                        fields[i - 1].fid
                    ),
                });
            }
        }

        Ok(())
    }

    /// Calculates the size of the frame in bytes
    fn calculate_frame_size(&self, bytes: &[u8]) -> Result<usize, BinaryError> {
        // Decode the frame to determine its size
        // We need to re-decode to track the exact number of bytes consumed
        let mut offset = 0;

        // VERSION (1 byte)
        if bytes.is_empty() {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: bytes.len(),
            });
        }
        offset += 1;

        // FLAGS (1 byte)
        if bytes.len() < offset + 1 {
            return Err(BinaryError::UnexpectedEof {
                expected: offset + 1,
                found: bytes.len(),
            });
        }
        offset += 1;

        // ENTRY_COUNT (VarInt)
        let (entry_count, consumed) =
            super::varint::decode(&bytes[offset..]).map_err(|_| BinaryError::InvalidVarInt {
                reason: "Invalid entry count VarInt".to_string(),
            })?;
        offset += consumed;

        if entry_count < 0 {
            return Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0,
                reason: format!("Negative entry count: {}", entry_count),
            });
        }

        let entry_count = entry_count as usize;

        // Decode each entry to calculate size
        for _ in 0..entry_count {
            let (_, consumed) = super::entry::BinaryEntry::decode(&bytes[offset..])?;
            offset += consumed;
        }

        Ok(offset)
    }

    /// Detects the binary format version from the first byte
    ///
    /// This method examines the version byte to determine which version of the
    /// LNMP binary protocol is being used. This is useful for backward compatibility
    /// and version-specific handling.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Binary data to inspect
    ///
    /// # Returns
    ///
    /// The version byte (0x04 for v0.4, 0x05 for v0.5, etc.)
    ///
    /// # Errors
    ///
    /// Returns `BinaryError::UnexpectedEof` if the data is too short to contain a version byte
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::binary::BinaryDecoder;
    ///
    /// let decoder = BinaryDecoder::new();
    /// let v04_data = vec![0x04, 0x00, 0x00]; // v0.4 binary
    /// let version = decoder.detect_version(&v04_data).unwrap();
    /// assert_eq!(version, 0x04);
    /// ```
    pub fn detect_version(&self, bytes: &[u8]) -> Result<u8, BinaryError> {
        if bytes.is_empty() {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: 0,
            });
        }
        Ok(bytes[0])
    }

    /// Checks if the binary data contains nested structures (v0.5+ feature)
    ///
    /// This method scans the binary data to determine if it uses v0.5+ type tags
    /// (NestedRecord 0x06, NestedArray 0x07, or reserved types 0x08-0x0F).
    /// This is useful for determining compatibility with v0.4 decoders.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Binary data to inspect
    ///
    /// # Returns
    ///
    /// `true` if the data contains v0.5+ type tags, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::binary::BinaryDecoder;
    ///
    /// let decoder = BinaryDecoder::new();
    /// let v04_data = vec![0x04, 0x00, 0x01, 0x07, 0x00, 0x03, 0x01]; // v0.4 with Bool
    /// assert!(!decoder.supports_nested(&v04_data));
    /// ```
    pub fn supports_nested(&self, bytes: &[u8]) -> bool {
        // Try to parse the frame and check for v0.5 type tags
        // We need to scan through entries looking for type tags >= 0x06

        if bytes.len() < 3 {
            return false; // Too short to contain any entries
        }

        let mut offset = 0;

        // Skip VERSION (1 byte)
        offset += 1;

        // Skip FLAGS (1 byte)
        offset += 1;

        // Read ENTRY_COUNT (VarInt)
        let (entry_count, consumed) = match super::varint::decode(&bytes[offset..]) {
            Ok(result) => result,
            Err(_) => return false,
        };
        offset += consumed;

        if entry_count < 0 {
            return false;
        }

        let entry_count = entry_count as usize;

        // Scan each entry for v0.5 type tags
        for _ in 0..entry_count {
            if offset >= bytes.len() {
                return false;
            }

            // Try to decode the entry
            match super::entry::BinaryEntry::decode(&bytes[offset..]) {
                Ok((entry, consumed)) => {
                    // Check if the type tag is v0.5+
                    let type_tag = entry.type_tag();
                    if type_tag.is_v0_5_type() {
                        return true;
                    }
                    offset += consumed;
                }
                Err(_) => return false,
            }
        }

        false
    }
}

impl Default for BinaryDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::super::encoder::BinaryEncoder;
    use super::*;
    use lnmp_core::{LnmpField, LnmpValue};

    #[test]
    fn test_new_decoder() {
        let decoder = BinaryDecoder::new();
        assert!(!decoder.config.validate_ordering);
        assert!(!decoder.config.strict_parsing);
        // v0.5 defaults
        assert!(!decoder.config.allow_streaming);
        assert!(!decoder.config.validate_nesting);
        assert!(!decoder.config.allow_delta);
        assert_eq!(decoder.config.max_depth, 32);
    }

    #[test]
    fn test_decoder_with_config() {
        let config = DecoderConfig::new()
            .with_validate_ordering(true)
            .with_strict_parsing(true);

        let decoder = BinaryDecoder::with_config(config);
        assert!(decoder.config.validate_ordering);
        assert!(decoder.config.strict_parsing);
    }

    #[test]
    fn test_decode_empty_record() {
        let encoder = BinaryEncoder::new();
        let record = LnmpRecord::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(decoded.fields().len(), 0);
    }

    #[test]
    fn test_decode_single_field() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(decoded.fields().len(), 1);
        assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
    }

    #[test]
    fn test_decode_multiple_fields() {
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

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(decoded.fields().len(), 3);
        assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(decoded.get_field(12).unwrap().value, LnmpValue::Int(14532));
        assert_eq!(
            decoded.get_field(23).unwrap().value,
            LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_decode_all_types() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(-42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14159),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(false),
        });
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("hello".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(-42));
        assert_eq!(
            decoded.get_field(2).unwrap().value,
            LnmpValue::Float(3.14159)
        );
        assert_eq!(decoded.get_field(3).unwrap().value, LnmpValue::Bool(false));
        assert_eq!(
            decoded.get_field(4).unwrap().value,
            LnmpValue::String("hello".to_string())
        );
        assert_eq!(
            decoded.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_decode_unsupported_version() {
        let bytes = vec![0x99, 0x00, 0x00]; // Invalid version
        let decoder = BinaryDecoder::new();
        let result = decoder.decode(&bytes);

        assert!(matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x99, .. })
        ));
    }

    #[test]
    fn test_decode_version_0x04_accepted() {
        let bytes = vec![0x04, 0x00, 0x00]; // Valid version, empty record
        let decoder = BinaryDecoder::new();
        let result = decoder.decode(&bytes);

        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_to_text_simple() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let text = decoder.decode_to_text(&binary).unwrap();

        assert_eq!(text, "F7=1");
    }

    #[test]
    fn test_decode_to_text_multiple_fields() {
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

        let decoder = BinaryDecoder::new();
        let text = decoder.decode_to_text(&binary).unwrap();

        // Fields should be in canonical order (sorted by FID)
        assert_eq!(text, "F7=1\nF12=14532\nF23=[admin,dev]");
    }

    #[test]
    fn test_decode_to_text_canonical_format() {
        // Test that output is in canonical format (newline-separated, sorted)
        let mut record = LnmpRecord::new();
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

        let decoder = BinaryDecoder::new();
        let text = decoder.decode_to_text(&binary).unwrap();

        // Should be sorted by FID
        assert_eq!(text, "F7=1\nF12=14532\nF23=[admin]");
    }

    #[test]
    fn test_validate_ordering_accepts_sorted() {
        let mut record = LnmpRecord::new();
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

        let config = DecoderConfig::new().with_validate_ordering(true);
        let decoder = BinaryDecoder::with_config(config);
        let result = decoder.decode(&binary);

        assert!(result.is_ok());
    }

    #[test]
    fn test_trailing_data_detection() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let mut binary = encoder.encode(&record).unwrap();

        // Add trailing data
        binary.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let config = DecoderConfig::new().with_strict_parsing(true);
        let decoder = BinaryDecoder::with_config(config);
        let result = decoder.decode(&binary);

        assert!(matches!(
            result,
            Err(BinaryError::TrailingData { bytes_remaining: 4 })
        ));
    }

    #[test]
    fn test_trailing_data_ignored_without_strict() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let mut binary = encoder.encode(&record).unwrap();

        // Add trailing data
        binary.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let decoder = BinaryDecoder::new(); // strict_parsing = false
        let result = decoder.decode(&binary);

        // Should succeed without strict parsing
        assert!(result.is_ok());
    }

    #[test]
    fn test_roundtrip_binary_to_text_to_binary() {
        let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";

        // Text -> Binary
        let encoder = BinaryEncoder::new();
        let binary1 = encoder.encode_text(original_text).unwrap();

        // Binary -> Text
        let decoder = BinaryDecoder::new();
        let text = decoder.decode_to_text(&binary1).unwrap();

        // Text -> Binary again
        let binary2 = encoder.encode_text(&text).unwrap();

        // Binary representations should be identical
        assert_eq!(binary1, binary2);
    }

    #[test]
    fn test_roundtrip_text_to_binary_to_text() {
        let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";

        // Text -> Binary
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(original_text).unwrap();

        // Binary -> Text
        let decoder = BinaryDecoder::new();
        let decoded_text = decoder.decode_to_text(&binary).unwrap();

        // Should produce identical canonical text
        assert_eq!(decoded_text, original_text);
    }

    #[test]
    fn test_roundtrip_unsorted_text() {
        let unsorted_text = "F23=[admin]\nF7=1\nF12=14532";

        // Text -> Binary
        let encoder = BinaryEncoder::new();
        let binary = encoder.encode_text(unsorted_text).unwrap();

        // Binary -> Text
        let decoder = BinaryDecoder::new();
        let decoded_text = decoder.decode_to_text(&binary).unwrap();

        // Should be sorted in canonical form
        assert_eq!(decoded_text, "F7=1\nF12=14532\nF23=[admin]");
    }

    #[test]
    fn test_roundtrip_all_types() {
        let test_cases = vec![
            ("F1=-42", "F1=-42"),
            ("F2=3.14159", "F2=3.14159"),
            ("F3=0", "F3=0"),
            ("F4=1", "F4=1"),
            ("F5=\"hello\\nworld\"", "F5=\"hello\\nworld\""),
            ("F6=[\"a\",\"b\",\"c\"]", "F6=[a,b,c]"), // Simple strings don't need quotes
            ("F7=[]", ""), // Empty arrays are omitted during canonicalization (Requirement 9.3)
        ];

        for (input, expected) in test_cases {
            let encoder = BinaryEncoder::new();
            let binary = encoder.encode_text(input).unwrap();

            let decoder = BinaryDecoder::new();
            let decoded = decoder.decode_to_text(&binary).unwrap();

            assert_eq!(decoded, expected, "Failed for: {}", input);
        }
    }

    #[test]
    fn test_roundtrip_stability() {
        let text = "F7=1\nF12=14532";

        let encoder = BinaryEncoder::new();
        let decoder = BinaryDecoder::new();

        // Multiple round-trips should be stable
        let mut current = text.to_string();
        for _ in 0..5 {
            let binary = encoder.encode_text(&current).unwrap();
            current = decoder.decode_to_text(&binary).unwrap();
        }

        assert_eq!(current, text);
    }

    #[test]
    fn test_decode_insufficient_data() {
        let bytes = vec![0x04]; // Only version, no flags
        let decoder = BinaryDecoder::new();
        let result = decoder.decode(&bytes);

        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decode_invalid_entry_count() {
        let bytes = vec![0x04, 0x00, 0x80]; // Incomplete VarInt
        let decoder = BinaryDecoder::new();
        let result = decoder.decode(&bytes);

        assert!(matches!(result, Err(BinaryError::InvalidVarInt { .. })));
    }

    #[test]
    fn test_decode_empty_string() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("".to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::String("".to_string())
        );
    }

    #[test]
    fn test_decode_empty_array() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(vec![]),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::StringArray(vec![])
        );
    }

    #[test]
    fn test_decode_special_characters() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("hello\nworld".to_string()),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("path\\to\\file".to_string()),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::String("say \"hello\"".to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::String("hello\nworld".to_string())
        );
        assert_eq!(
            decoded.get_field(2).unwrap().value,
            LnmpValue::String("path\\to\\file".to_string())
        );
        assert_eq!(
            decoded.get_field(3).unwrap().value,
            LnmpValue::String("say \"hello\"".to_string())
        );
    }

    #[test]
    fn test_decode_unicode() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("emoji: 🎯".to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::String("emoji: 🎯".to_string())
        );
    }

    #[test]
    fn test_default_decoder() {
        let decoder = BinaryDecoder::default();
        assert!(!decoder.config.validate_ordering);
        assert!(!decoder.config.strict_parsing);
    }

    #[test]
    fn test_decoder_config_builder() {
        let config = DecoderConfig::new()
            .with_validate_ordering(true)
            .with_strict_parsing(true);

        assert!(config.validate_ordering);
        assert!(config.strict_parsing);
    }

    #[test]
    fn test_decoder_config_v05_fields() {
        let config = DecoderConfig::new()
            .with_streaming(true)
            .with_validate_nesting(true)
            .with_delta(true)
            .with_max_depth(64);

        assert!(config.allow_streaming);
        assert!(config.validate_nesting);
        assert!(config.allow_delta);
        assert_eq!(config.max_depth, 64);
    }

    #[test]
    fn test_decoder_config_v05_defaults() {
        let config = DecoderConfig::default();

        assert!(!config.allow_streaming);
        assert!(!config.validate_nesting);
        assert!(!config.allow_delta);
        assert_eq!(config.max_depth, 32);
    }

    #[test]
    fn test_decoder_config_backward_compatibility() {
        // v0.4 configurations should work without any changes
        let v04_config = DecoderConfig::new()
            .with_validate_ordering(true)
            .with_strict_parsing(true);

        // v0.4 fields should work as before
        assert!(v04_config.validate_ordering);
        assert!(v04_config.strict_parsing);

        // v0.5 fields should have safe defaults (disabled)
        assert!(!v04_config.allow_streaming);
        assert!(!v04_config.validate_nesting);
        assert!(!v04_config.allow_delta);
    }

    #[test]
    fn test_decoder_config_mixed_v04_v05() {
        // Test that v0.4 and v0.5 configurations can be mixed
        let config = DecoderConfig::new()
            .with_validate_ordering(true) // v0.4
            .with_streaming(true) // v0.5
            .with_strict_parsing(true) // v0.4
            .with_delta(true); // v0.5

        assert!(config.validate_ordering);
        assert!(config.strict_parsing);
        assert!(config.allow_streaming);
        assert!(config.allow_delta);
    }

    #[test]
    fn test_decoder_v04_mode_decoding() {
        // Test that decoder with v0.5 disabled behaves like v0.4
        let config = DecoderConfig::new()
            .with_streaming(false)
            .with_validate_nesting(false)
            .with_delta(false);

        let decoder = BinaryDecoder::with_config(config);

        // Should decode a v0.4 record just like before
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoded = decoder.decode(&binary).unwrap();

        // Should decode correctly
        assert_eq!(decoded.fields().len(), 1);
        assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
    }

    #[test]
    fn test_decoder_v04_compatibility_with_existing_binary() {
        // Test that v0.5 decoder can decode v0.4 binary format
        let v05_decoder = BinaryDecoder::new(); // All v0.5 features disabled by default

        // Create a v0.4 binary (no nested structures)
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let encoder = BinaryEncoder::new();
        let v04_binary = encoder.encode(&record).unwrap();

        // v0.5 decoder should decode v0.4 binary without issues
        let decoded = v05_decoder.decode(&v04_binary).unwrap();

        assert_eq!(decoded.fields().len(), 2);
        assert_eq!(decoded.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(decoded.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_decode_large_fid() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 65535,
            value: LnmpValue::Int(42),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(decoded.get_field(65535).unwrap().value, LnmpValue::Int(42));
    }

    #[test]
    fn test_decode_large_integer() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(i64::MAX),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(i64::MAX)
        );
    }

    #[test]
    fn test_decode_negative_integer() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(i64::MIN),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let decoder = BinaryDecoder::new();
        let decoded = decoder.decode(&binary).unwrap();

        assert_eq!(
            decoded.get_field(1).unwrap().value,
            LnmpValue::Int(i64::MIN)
        );
    }
}
