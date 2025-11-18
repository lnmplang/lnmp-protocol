//! Nested structure decoding for LNMP v0.5 binary format.
//!
//! This module provides decoding support for nested records and arrays in the binary format.
//! It implements recursive decoding with depth validation to prevent stack overflow attacks.

use super::error::BinaryError;
use super::types::TypeTag;
use super::varint;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

/// Configuration for nested structure decoding (v0.5)
#[derive(Debug, Clone)]
pub struct NestedDecoderConfig {
    /// Whether to allow nested structures (default: true)
    pub allow_nested: bool,
    /// Whether to validate nesting rules (default: false)
    pub validate_nesting: bool,
    /// Maximum nesting depth allowed (default: 32)
    pub max_depth: usize,
}

impl Default for NestedDecoderConfig {
    fn default() -> Self {
        Self {
            allow_nested: true,
            validate_nesting: false,
            max_depth: 32,
        }
    }
}

impl NestedDecoderConfig {
    /// Creates a new nested decoder configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to allow nested structures
    pub fn with_allow_nested(mut self, allow: bool) -> Self {
        self.allow_nested = allow;
        self
    }

    /// Sets whether to validate nesting rules
    pub fn with_validate_nesting(mut self, validate: bool) -> Self {
        self.validate_nesting = validate;
        self
    }

    /// Sets the maximum nesting depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
}

/// Binary nested structure decoder for LNMP v0.5
///
/// Decodes nested records and arrays with depth validation.
#[derive(Debug)]
pub struct BinaryNestedDecoder {
    config: NestedDecoderConfig,
}

impl BinaryNestedDecoder {
    /// Creates a new nested decoder with default configuration
    pub fn new() -> Self {
        Self {
            config: NestedDecoderConfig::default(),
        }
    }

    /// Creates a nested decoder with custom configuration
    pub fn with_config(config: NestedDecoderConfig) -> Self {
        Self { config }
    }

    /// Decodes a nested record from binary format
    ///
    /// Binary layout:
    /// ```text
    /// ┌──────────┬──────────────┬─────────────────────────────────┐
    /// │   TAG    │ FIELD_COUNT  │      FIELD ENTRIES...           │
    /// │ (1 byte) │  (VarInt)    │  { FID | VALUE }*               │
    /// └──────────┴──────────────┴─────────────────────────────────┘
    /// ```
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data to decode
    ///
    /// # Returns
    ///
    /// A tuple of (decoded_record, bytes_consumed)
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Nesting depth exceeds configured maximum
    /// - Binary data is malformed
    /// - TAG byte is not 0x06
    pub fn decode_nested_record(&self, bytes: &[u8]) -> Result<(LnmpRecord, usize), BinaryError> {
        self.decode_nested_record_with_depth(bytes, 0)
    }

    /// Decodes a nested record with depth tracking
    fn decode_nested_record_with_depth(
        &self,
        bytes: &[u8],
        current_depth: usize,
    ) -> Result<(LnmpRecord, usize), BinaryError> {
        // Validate depth
        if current_depth >= self.config.max_depth {
            return Err(BinaryError::NestingDepthExceeded {
                depth: current_depth,
                max: self.config.max_depth,
            });
        }

        let mut offset = 0;

        // Read and validate TAG byte (0x06)
        if bytes.is_empty() {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: bytes.len(),
            });
        }

        let tag = bytes[offset];
        offset += 1;

        if tag != TypeTag::NestedRecord.to_u8() {
            return Err(BinaryError::InvalidTypeTag { tag });
        }

        // Decode FIELD_COUNT from VarInt
        let (field_count, consumed) = varint::decode(&bytes[offset..])?;
        offset += consumed;

        if field_count < 0 {
            return Err(BinaryError::InvalidNestedStructure {
                reason: format!("Negative field count: {}", field_count),
            });
        }

        let field_count = field_count as usize;

        // Recursively decode each field entry
        let mut record = LnmpRecord::new();

        for _ in 0..field_count {
            // Decode FID as VarInt
            let (fid_i64, consumed) = varint::decode(&bytes[offset..])?;
            offset += consumed;

            if fid_i64 < 0 || fid_i64 > u16::MAX as i64 {
                return Err(BinaryError::InvalidFID {
                    fid: fid_i64 as u16,
                    reason: format!("FID out of range: {}", fid_i64),
                });
            }

            let fid = fid_i64 as u16;

            // Decode VALUE recursively (depth stays the same, will increment inside nested record/array)
            let (value, consumed) = self.decode_value_recursive(&bytes[offset..], current_depth)?;
            offset += consumed;

            // Add field to record
            record.add_field(LnmpField { fid, value });
        }

        Ok((record, offset))
    }

    /// Decodes a value recursively, handling nested structures
    fn decode_value_recursive(
        &self,
        bytes: &[u8],
        current_depth: usize,
    ) -> Result<(LnmpValue, usize), BinaryError> {
        // Validate depth for nested structures
        if current_depth >= self.config.max_depth {
            return Err(BinaryError::NestingDepthExceeded {
                depth: current_depth,
                max: self.config.max_depth,
            });
        }

        let mut offset = 0;

        // Read type tag
        if bytes.is_empty() {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: bytes.len(),
            });
        }

        let tag_byte = bytes[offset];
        offset += 1;

        let type_tag = TypeTag::from_u8(tag_byte)?;

        match type_tag {
            TypeTag::Int => {
                let (value, consumed) = varint::decode(&bytes[offset..])?;
                offset += consumed;
                Ok((LnmpValue::Int(value), offset))
            }
            TypeTag::Float => {
                if bytes.len() < offset + 8 {
                    return Err(BinaryError::UnexpectedEof {
                        expected: offset + 8,
                        found: bytes.len(),
                    });
                }
                let float_bytes: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
                let value = f64::from_le_bytes(float_bytes);
                offset += 8;
                Ok((LnmpValue::Float(value), offset))
            }
            TypeTag::Bool => {
                if bytes.len() < offset + 1 {
                    return Err(BinaryError::UnexpectedEof {
                        expected: offset + 1,
                        found: bytes.len(),
                    });
                }
                let value = bytes[offset] != 0x00;
                offset += 1;
                Ok((LnmpValue::Bool(value), offset))
            }
            TypeTag::String => {
                let (length, consumed) = varint::decode(&bytes[offset..])?;
                offset += consumed;

                if length < 0 {
                    return Err(BinaryError::InvalidValue {
                        field_id: 0,
                        type_tag: tag_byte,
                        reason: format!("Negative string length: {}", length),
                    });
                }

                let length = length as usize;

                if bytes.len() < offset + length {
                    return Err(BinaryError::UnexpectedEof {
                        expected: offset + length,
                        found: bytes.len(),
                    });
                }

                let string_bytes = &bytes[offset..offset + length];
                let value = String::from_utf8(string_bytes.to_vec())
                    .map_err(|_| BinaryError::InvalidUtf8 { field_id: 0 })?;
                offset += length;
                Ok((LnmpValue::String(value), offset))
            }
            TypeTag::StringArray => {
                let (count, consumed) = varint::decode(&bytes[offset..])?;
                offset += consumed;

                if count < 0 {
                    return Err(BinaryError::InvalidValue {
                        field_id: 0,
                        type_tag: tag_byte,
                        reason: format!("Negative array count: {}", count),
                    });
                }

                let count = count as usize;
                let mut array = Vec::with_capacity(count);

                for _ in 0..count {
                    let (length, consumed) = varint::decode(&bytes[offset..])?;
                    offset += consumed;

                    if length < 0 {
                        return Err(BinaryError::InvalidValue {
                            field_id: 0,
                            type_tag: tag_byte,
                            reason: format!("Negative string length in array: {}", length),
                        });
                    }

                    let length = length as usize;

                    if bytes.len() < offset + length {
                        return Err(BinaryError::UnexpectedEof {
                            expected: offset + length,
                            found: bytes.len(),
                        });
                    }

                    let string_bytes = &bytes[offset..offset + length];
                    let string = String::from_utf8(string_bytes.to_vec())
                        .map_err(|_| BinaryError::InvalidUtf8 { field_id: 0 })?;
                    offset += length;
                    array.push(string);
                }

                Ok((LnmpValue::StringArray(array), offset))
            }
            TypeTag::NestedRecord => {
                // Check if nested structures are allowed
                if !self.config.allow_nested {
                    return Err(BinaryError::NestedStructureNotSupported);
                }

                // Recursively decode nested record (offset already advanced past tag)
                // Increment depth when entering a nested record
                let (record, consumed) =
                    self.decode_nested_record_with_depth(&bytes[offset - 1..], current_depth + 1)?;
                // Subtract 1 because we already consumed the tag byte
                offset += consumed - 1;
                Ok((LnmpValue::NestedRecord(Box::new(record)), offset))
            }
            TypeTag::NestedArray => {
                // Check if nested structures are allowed
                if !self.config.allow_nested {
                    return Err(BinaryError::NestedStructureNotSupported);
                }

                // Recursively decode nested array (offset already advanced past tag)
                // Increment depth when entering a nested array
                let (records, consumed) =
                    self.decode_nested_array_with_depth(&bytes[offset - 1..], current_depth + 1)?;
                // Subtract 1 because we already consumed the tag byte
                offset += consumed - 1;
                Ok((LnmpValue::NestedArray(records), offset))
            }
            _ => Err(BinaryError::InvalidTypeTag { tag: tag_byte }),
        }
    }

    /// Decodes a nested array from binary format
    ///
    /// Binary layout:
    /// ```text
    /// ┌──────────┬──────────────┬─────────────────────────────────┐
    /// │   TAG    │ ELEM_COUNT   │      RECORD ENTRIES...          │
    /// │ (1 byte) │  (VarInt)    │  [ RECORD(...) ]*               │
    /// └──────────┴──────────────┴─────────────────────────────────┘
    /// ```
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data to decode
    ///
    /// # Returns
    ///
    /// A tuple of (decoded_records, bytes_consumed)
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Nesting depth exceeds configured maximum
    /// - Binary data is malformed
    /// - TAG byte is not 0x07
    pub fn decode_nested_array(
        &self,
        bytes: &[u8],
    ) -> Result<(Vec<LnmpRecord>, usize), BinaryError> {
        self.decode_nested_array_with_depth(bytes, 0)
    }

    /// Decodes a nested array with depth tracking
    fn decode_nested_array_with_depth(
        &self,
        bytes: &[u8],
        current_depth: usize,
    ) -> Result<(Vec<LnmpRecord>, usize), BinaryError> {
        // Validate depth
        if current_depth >= self.config.max_depth {
            return Err(BinaryError::NestingDepthExceeded {
                depth: current_depth,
                max: self.config.max_depth,
            });
        }

        let mut offset = 0;

        // Read and validate TAG byte (0x07)
        if bytes.is_empty() {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: bytes.len(),
            });
        }

        let tag = bytes[offset];
        offset += 1;

        if tag != TypeTag::NestedArray.to_u8() {
            return Err(BinaryError::InvalidTypeTag { tag });
        }

        // Decode ELEMENT_COUNT from VarInt
        let (element_count, consumed) = varint::decode(&bytes[offset..])?;
        offset += consumed;

        if element_count < 0 {
            return Err(BinaryError::InvalidNestedStructure {
                reason: format!("Negative element count: {}", element_count),
            });
        }

        let element_count = element_count as usize;

        // Recursively decode each record
        let mut records = Vec::with_capacity(element_count);

        for _ in 0..element_count {
            // Each record in the array is encoded as a nested record
            // Depth stays the same since we're already inside the array
            let (record, consumed) =
                self.decode_nested_record_with_depth(&bytes[offset..], current_depth)?;
            offset += consumed;
            records.push(record);
        }

        Ok((records, offset))
    }
}

impl Default for BinaryNestedDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_decoder_config_default() {
        let config = NestedDecoderConfig::default();
        assert!(config.allow_nested);
        assert!(!config.validate_nesting);
        assert_eq!(config.max_depth, 32);
    }

    #[test]
    fn test_nested_decoder_config_builder() {
        let config = NestedDecoderConfig::new()
            .with_allow_nested(false)
            .with_validate_nesting(true)
            .with_max_depth(16);

        assert!(!config.allow_nested);
        assert!(config.validate_nesting);
        assert_eq!(config.max_depth, 16);
    }

    #[test]
    fn test_nested_decoder_new() {
        let decoder = BinaryNestedDecoder::new();
        assert!(decoder.config.allow_nested);
        assert!(!decoder.config.validate_nesting);
        assert_eq!(decoder.config.max_depth, 32);
    }

    #[test]
    fn test_nested_decoder_with_config() {
        let config = NestedDecoderConfig::new()
            .with_max_depth(8)
            .with_validate_nesting(true);

        let decoder = BinaryNestedDecoder::with_config(config);
        assert_eq!(decoder.config.max_depth, 8);
        assert!(decoder.config.validate_nesting);
    }

    #[test]
    fn test_nested_decoder_default() {
        let decoder = BinaryNestedDecoder::default();
        assert!(decoder.config.allow_nested);
        assert_eq!(decoder.config.max_depth, 32);
    }
}

#[test]
fn test_decode_empty_nested_record() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let record = LnmpRecord::new();
    let binary = encoder.encode_nested_record(&record).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.fields().len(), 0);
    assert_eq!(consumed, binary.len());
}

#[test]
fn test_decode_single_level_nested_record() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    let binary = encoder.encode_nested_record(&record).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.fields().len(), 2);
    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        decoded.get_field(2).unwrap().value,
        LnmpValue::String("test".to_string())
    );
    assert_eq!(consumed, binary.len());
}

#[test]
fn test_decode_nested_record_invalid_tag() {
    let decoder = BinaryNestedDecoder::new();
    let bytes = vec![0x01, 0x00]; // Wrong tag (Int instead of NestedRecord)

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(
        result,
        Err(BinaryError::InvalidTypeTag { tag: 0x01 })
    ));
}

#[test]
fn test_decode_nested_record_insufficient_data() {
    let decoder = BinaryNestedDecoder::new();
    let bytes = vec![]; // Empty

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
}

#[test]
fn test_decode_nested_record_negative_field_count() {
    let decoder = BinaryNestedDecoder::new();
    // TAG (0x06) + negative field count
    let bytes = vec![0x06, 0x7F]; // -1 in VarInt

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(
        result,
        Err(BinaryError::InvalidNestedStructure { .. })
    ));
}

#[test]
fn test_decode_multi_level_nested_record() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();

    // Create a 2-level nested structure
    let mut inner_record = LnmpRecord::new();
    inner_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut outer_record = LnmpRecord::new();
    outer_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedRecord(Box::new(inner_record)),
    });

    let binary = encoder.encode_nested_record(&outer_record).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.fields().len(), 1);
    assert_eq!(consumed, binary.len());

    // Verify nested structure
    match &decoded.get_field(2).unwrap().value {
        LnmpValue::NestedRecord(inner) => {
            assert_eq!(inner.fields().len(), 1);
            assert_eq!(inner.get_field(1).unwrap().value, LnmpValue::Int(42));
        }
        _ => panic!("Expected NestedRecord"),
    }
}

#[test]
fn test_decode_nested_record_depth_limit() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let config = NestedDecoderConfig::new().with_max_depth(2);
    let decoder = BinaryNestedDecoder::with_config(config);

    let encoder = BinaryNestedEncoder::new();

    // Create a deeply nested structure (depth 3)
    let mut level3 = LnmpRecord::new();
    level3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
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

    let binary = encoder.encode_nested_record(&level1).unwrap();

    let result = decoder.decode_nested_record(&binary);
    assert!(matches!(
        result,
        Err(BinaryError::NestingDepthExceeded { .. })
    ));
}

#[test]
fn test_decode_nested_record_all_primitive_types() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(-42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(false),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::String("test".to_string()),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
    });

    let binary = encoder.encode_nested_record(&record).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(-42));
    assert_eq!(decoded.get_field(2).unwrap().value, LnmpValue::Float(3.14));
    assert_eq!(decoded.get_field(3).unwrap().value, LnmpValue::Bool(false));
    assert_eq!(
        decoded.get_field(4).unwrap().value,
        LnmpValue::String("test".to_string())
    );
    assert_eq!(
        decoded.get_field(5).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
    );
}

#[test]
fn test_decode_nested_record_roundtrip() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let mut original = LnmpRecord::new();
    original.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });
    original.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("hello".to_string()),
    });

    let binary = encoder.encode_nested_record(&original).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    // Compare sorted fields for equality
    assert_eq!(original.sorted_fields(), decoded.sorted_fields());
}

#[test]
fn test_decode_empty_nested_array() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let records: Vec<LnmpRecord> = vec![];
    let binary = encoder.encode_nested_array(&records).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_array(&binary).unwrap();

    assert_eq!(decoded.len(), 0);
    assert_eq!(consumed, binary.len());
}

#[test]
fn test_decode_nested_array_single_record() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let binary = encoder.encode_nested_array(&[record.clone()]).unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_array(&binary).unwrap();

    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].sorted_fields(), record.sorted_fields());
    assert_eq!(consumed, binary.len());
}

#[test]
fn test_decode_nested_array_multiple_records() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();

    let mut record1 = LnmpRecord::new();
    record1.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });

    let mut record2 = LnmpRecord::new();
    record2.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(2),
    });

    let binary = encoder
        .encode_nested_array(&[record1.clone(), record2.clone()])
        .unwrap();

    let decoder = BinaryNestedDecoder::new();
    let (decoded, consumed) = decoder.decode_nested_array(&binary).unwrap();

    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].sorted_fields(), record1.sorted_fields());
    assert_eq!(decoded[1].sorted_fields(), record2.sorted_fields());
    assert_eq!(consumed, binary.len());
}

#[test]
fn test_decode_nested_array_invalid_tag() {
    let decoder = BinaryNestedDecoder::new();
    let bytes = vec![0x06, 0x00]; // Wrong tag (NestedRecord instead of NestedArray)

    let result = decoder.decode_nested_array(&bytes);
    assert!(matches!(
        result,
        Err(BinaryError::InvalidTypeTag { tag: 0x06 })
    ));
}

#[test]
fn test_decode_nested_array_negative_element_count() {
    let decoder = BinaryNestedDecoder::new();
    // TAG (0x07) + negative element count
    let bytes = vec![0x07, 0x7F]; // -1 in VarInt

    let result = decoder.decode_nested_array(&bytes);
    assert!(matches!(
        result,
        Err(BinaryError::InvalidNestedStructure { .. })
    ));
}

#[test]
fn test_decode_nested_array_depth_limit() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let config = NestedDecoderConfig::new().with_max_depth(2);
    let decoder = BinaryNestedDecoder::with_config(config);

    let encoder = BinaryNestedEncoder::new();

    // Create a deeply nested structure (depth 3)
    let mut level3 = LnmpRecord::new();
    level3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut level2 = LnmpRecord::new();
    level2.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedRecord(Box::new(level3)),
    });

    let mut level1 = LnmpRecord::new();
    level1.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::NestedArray(vec![level2]),
    });

    let binary = encoder.encode_nested_record(&level1).unwrap();

    let result = decoder.decode_nested_record(&binary);
    assert!(matches!(
        result,
        Err(BinaryError::NestingDepthExceeded { .. })
    ));
}

#[test]
fn test_decode_nested_array_roundtrip() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let mut record1 = LnmpRecord::new();
    record1.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("first".to_string()),
    });

    let mut record2 = LnmpRecord::new();
    record2.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("second".to_string()),
    });

    let original = vec![record1.clone(), record2.clone()];

    let binary = encoder.encode_nested_array(&original).unwrap();
    let (decoded, _) = decoder.decode_nested_array(&binary).unwrap();

    assert_eq!(decoded.len(), original.len());
    for (i, record) in decoded.iter().enumerate() {
        assert_eq!(record.sorted_fields(), original[i].sorted_fields());
    }
}

#[test]
fn test_decode_deeply_nested_record_depth_2() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    // Create depth-2 nested structure
    let mut level2 = LnmpRecord::new();
    level2.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });

    let mut level1 = LnmpRecord::new();
    level1.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedRecord(Box::new(level2)),
    });

    let binary = encoder.encode_nested_record(&level1).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    // Verify structure
    match &decoded.get_field(2).unwrap().value {
        LnmpValue::NestedRecord(inner) => {
            assert_eq!(inner.get_field(1).unwrap().value, LnmpValue::Int(100));
        }
        _ => panic!("Expected NestedRecord"),
    }
}

#[test]
fn test_decode_deeply_nested_record_depth_3() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    // Create depth-3 nested structure
    let mut level3 = LnmpRecord::new();
    level3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
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

    let binary = encoder.encode_nested_record(&level1).unwrap();
    let result = decoder.decode_nested_record(&binary);
    assert!(result.is_ok());
}

#[test]
fn test_decode_malformed_nested_structure_incomplete_field() {
    let decoder = BinaryNestedDecoder::new();
    // TAG (0x06) + FIELD_COUNT (1) + FID (1) but no VALUE
    let bytes = vec![0x06, 0x01, 0x01];

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
}

#[test]
fn test_decode_malformed_nested_structure_invalid_fid() {
    let decoder = BinaryNestedDecoder::new();
    // TAG (0x06) + FIELD_COUNT (1) + negative FID
    let bytes = vec![0x06, 0x01, 0x7F]; // -1 as FID

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(result, Err(BinaryError::InvalidFID { .. })));
}

#[test]
fn test_decode_nested_structure_not_allowed() {
    let config = NestedDecoderConfig::new().with_allow_nested(false);
    let decoder = BinaryNestedDecoder::with_config(config);

    // Try to decode a nested record value
    // TAG (0x06) + FIELD_COUNT (0)
    let bytes = vec![0x06, 0x00];

    let result = decoder.decode_value_recursive(&bytes, 0);
    assert!(matches!(
        result,
        Err(BinaryError::NestedStructureNotSupported)
    ));
}

#[test]
fn test_decode_nested_array_not_allowed() {
    let config = NestedDecoderConfig::new().with_allow_nested(false);
    let decoder = BinaryNestedDecoder::with_config(config);

    // Try to decode a nested array value
    // TAG (0x07) + ELEMENT_COUNT (0)
    let bytes = vec![0x07, 0x00];

    let result = decoder.decode_value_recursive(&bytes, 0);
    assert!(matches!(
        result,
        Err(BinaryError::NestedStructureNotSupported)
    ));
}

#[test]
fn test_decode_roundtrip_complex_nested_structure() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    // Create a complex nested structure with mixed types
    let mut inner1 = LnmpRecord::new();
    inner1.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("inner1".to_string()),
    });

    let mut inner2 = LnmpRecord::new();
    inner2.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(42),
    });

    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::NestedRecord(Box::new(inner1)),
    });
    outer.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedArray(vec![inner2]),
    });
    outer.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let binary = encoder.encode_nested_record(&outer).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.sorted_fields().len(), outer.sorted_fields().len());
}

#[test]
fn test_decode_empty_nested_records_at_multiple_levels() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let inner = LnmpRecord::new(); // Empty
    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });

    let binary = encoder.encode_nested_record(&outer).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    match &decoded.get_field(1).unwrap().value {
        LnmpValue::NestedRecord(inner) => {
            assert_eq!(inner.fields().len(), 0);
        }
        _ => panic!("Expected NestedRecord"),
    }
}

#[test]
fn test_decode_empty_nested_arrays() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::NestedArray(vec![]),
    });

    let binary = encoder.encode_nested_record(&record).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    match &decoded.get_field(1).unwrap().value {
        LnmpValue::NestedArray(arr) => {
            assert_eq!(arr.len(), 0);
        }
        _ => panic!("Expected NestedArray"),
    }
}

#[test]
fn test_decode_mixed_primitive_and_nested_fields() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("inner".to_string()),
    });

    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    outer.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });
    outer.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let binary = encoder.encode_nested_record(&outer).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(decoded.fields().len(), 3);
    assert_eq!(decoded.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(decoded.get_field(3).unwrap().value, LnmpValue::Bool(true));
}

#[test]
fn test_decode_depth_limit_enforced_at_exact_limit() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    // With max_depth=2, we can decode structures where the deepest call to
    // decode_nested_record_with_depth has current_depth < 2
    let config = NestedDecoderConfig::new().with_max_depth(2);
    let decoder = BinaryNestedDecoder::with_config(config);
    let encoder = BinaryNestedEncoder::new();

    // Flat record (no nesting) - should succeed
    let mut flat = LnmpRecord::new();
    flat.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let binary = encoder.encode_nested_record(&flat).unwrap();
    let result = decoder.decode_nested_record(&binary);
    assert!(result.is_ok(), "Flat record should succeed");

    // One level of nesting - should succeed
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });

    let binary = encoder.encode_nested_record(&outer).unwrap();
    let result = decoder.decode_nested_record(&binary);
    assert!(result.is_ok(), "One level of nesting should succeed");

    // Two levels of nesting - should fail with max_depth=2
    let mut level3 = LnmpRecord::new();
    level3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
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

    let binary = encoder.encode_nested_record(&level1).unwrap();
    let result = decoder.decode_nested_record(&binary);
    assert!(
        matches!(result, Err(BinaryError::NestingDepthExceeded { .. })),
        "Two levels of nesting should fail with max_depth=2"
    );
}

#[test]
fn test_decode_invalid_utf8_in_string() {
    let decoder = BinaryNestedDecoder::new();
    // TAG (0x06) + FIELD_COUNT (1) + FID (1) + String TAG (0x04) + length (3) + invalid UTF-8
    let bytes = vec![0x06, 0x01, 0x01, 0x04, 0x03, 0xFF, 0xFE, 0xFD];

    let result = decoder.decode_nested_record(&bytes);
    assert!(matches!(result, Err(BinaryError::InvalidUtf8 { .. })));
}

#[test]
fn test_decode_string_array_with_empty_strings() {
    use crate::binary::nested_encoder::BinaryNestedEncoder;

    let encoder = BinaryNestedEncoder::new();
    let decoder = BinaryNestedDecoder::new();

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::StringArray(vec!["".to_string(), "test".to_string(), "".to_string()]),
    });

    let binary = encoder.encode_nested_record(&record).unwrap();
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();

    assert_eq!(
        decoded.get_field(1).unwrap().value,
        LnmpValue::StringArray(vec!["".to_string(), "test".to_string(), "".to_string()])
    );
}
