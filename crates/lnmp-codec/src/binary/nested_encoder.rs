//! Nested structure encoding for LNMP v0.5 binary format.
//!
//! This module provides encoding support for nested records and arrays in the binary format.
//! It implements recursive encoding with depth validation and size limits to prevent
//! stack overflow and memory exhaustion attacks.

use lnmp_core::{LnmpRecord, LnmpValue};
use super::error::BinaryError;
use super::types::TypeTag;
use super::varint;

/// Configuration for nested structure encoding (v0.5)
#[derive(Debug, Clone)]
pub struct NestedEncoderConfig {
    /// Maximum nesting depth allowed (default: 32)
    pub max_depth: usize,
    /// Maximum record size in bytes (None = unlimited)
    pub max_record_size: Option<usize>,
    /// Whether to validate canonical ordering
    pub validate_canonical: bool,
}

impl Default for NestedEncoderConfig {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_record_size: None,
            validate_canonical: false,
        }
    }
}

impl NestedEncoderConfig {
    /// Creates a new nested encoder configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum nesting depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the maximum record size in bytes
    pub fn with_max_record_size(mut self, max_size: Option<usize>) -> Self {
        self.max_record_size = max_size;
        self
    }

    /// Sets whether to validate canonical ordering
    pub fn with_validate_canonical(mut self, validate: bool) -> Self {
        self.validate_canonical = validate;
        self
    }
}

/// Binary nested structure encoder for LNMP v0.5
///
/// Encodes nested records and arrays with depth validation and size limits.
#[derive(Debug)]
pub struct BinaryNestedEncoder {
    config: NestedEncoderConfig,
}

impl BinaryNestedEncoder {
    /// Creates a new nested encoder with default configuration
    pub fn new() -> Self {
        Self {
            config: NestedEncoderConfig::default(),
        }
    }

    /// Creates a nested encoder with custom configuration
    pub fn with_config(config: NestedEncoderConfig) -> Self {
        Self { config }
    }

    /// Encodes a nested record to binary format
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
    /// * `record` - The nested record to encode
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the binary-encoded nested record
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Nesting depth exceeds configured maximum
    /// - Record size exceeds configured maximum
    /// - Field encoding fails
    pub fn encode_nested_record(&self, record: &LnmpRecord) -> Result<Vec<u8>, BinaryError> {
        self.encode_nested_record_with_depth(record, 0)
    }

    /// Encodes a nested record with depth tracking
    fn encode_nested_record_with_depth(
        &self,
        record: &LnmpRecord,
        current_depth: usize,
    ) -> Result<Vec<u8>, BinaryError> {
        // Validate depth
        if current_depth >= self.config.max_depth {
            return Err(BinaryError::NestingDepthExceeded {
                depth: current_depth,
                max: self.config.max_depth,
            });
        }

        let mut buffer = Vec::new();

        // Write TAG byte (0x06 for NestedRecord)
        buffer.push(TypeTag::NestedRecord.to_u8());

        // Get fields (sorted for canonical ordering)
        let fields = record.sorted_fields();

        // Encode FIELD_COUNT as VarInt
        let field_count = fields.len() as i64;
        buffer.extend_from_slice(&varint::encode(field_count));

        // Recursively encode each field
        for field in fields {
            // Encode FID as VarInt
            buffer.extend_from_slice(&varint::encode(field.fid as i64));

            // Encode VALUE recursively
            let value_bytes = self.encode_value_recursive(&field.value, current_depth + 1)?;
            buffer.extend_from_slice(&value_bytes);

            // Check size limit if configured
            if let Some(max_size) = self.config.max_record_size {
                if buffer.len() > max_size {
                    return Err(BinaryError::RecordSizeExceeded {
                        size: buffer.len(),
                        max: max_size,
                    });
                }
            }
        }

        Ok(buffer)
    }

    /// Encodes a nested array to binary format
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
    /// * `records` - The array of records to encode
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the binary-encoded nested array
    ///
    /// # Errors
    ///
    /// Returns `BinaryError` if:
    /// - Nesting depth exceeds configured maximum
    /// - Record size exceeds configured maximum
    /// - Record encoding fails
    pub fn encode_nested_array(&self, records: &[LnmpRecord]) -> Result<Vec<u8>, BinaryError> {
        self.encode_nested_array_with_depth(records, 0)
    }

    /// Encodes a nested array with depth tracking
    fn encode_nested_array_with_depth(
        &self,
        records: &[LnmpRecord],
        current_depth: usize,
    ) -> Result<Vec<u8>, BinaryError> {
        // Validate depth
        if current_depth >= self.config.max_depth {
            return Err(BinaryError::NestingDepthExceeded {
                depth: current_depth,
                max: self.config.max_depth,
            });
        }

        let mut buffer = Vec::new();

        // Write TAG byte (0x07 for NestedArray)
        buffer.push(TypeTag::NestedArray.to_u8());

        // Encode ELEMENT_COUNT as VarInt
        let element_count = records.len() as i64;
        buffer.extend_from_slice(&varint::encode(element_count));

        // Recursively encode each record
        for record in records {
            // For records within arrays, we need to encode them with canonical ordering
            let record_bytes = self.encode_nested_record_with_depth(record, current_depth + 1)?;
            buffer.extend_from_slice(&record_bytes);

            // Check size limit if configured
            if let Some(max_size) = self.config.max_record_size {
                if buffer.len() > max_size {
                    return Err(BinaryError::RecordSizeExceeded {
                        size: buffer.len(),
                        max: max_size,
                    });
                }
            }
        }

        Ok(buffer)
    }

    /// Encodes a value recursively, handling nested structures
    fn encode_value_recursive(
        &self,
        value: &LnmpValue,
        current_depth: usize,
    ) -> Result<Vec<u8>, BinaryError> {
        // Validate depth for nested structures
        if current_depth >= self.config.max_depth {
            if matches!(value, LnmpValue::NestedRecord(_) | LnmpValue::NestedArray(_)) {
                return Err(BinaryError::NestingDepthExceeded {
                    depth: current_depth,
                    max: self.config.max_depth,
                });
            }
        }

        match value {
            LnmpValue::Int(i) => {
                let mut buffer = Vec::new();
                buffer.push(TypeTag::Int.to_u8());
                buffer.extend_from_slice(&varint::encode(*i));
                Ok(buffer)
            }
            LnmpValue::Float(f) => {
                let mut buffer = Vec::new();
                buffer.push(TypeTag::Float.to_u8());
                buffer.extend_from_slice(&f.to_le_bytes());
                Ok(buffer)
            }
            LnmpValue::Bool(b) => {
                let mut buffer = Vec::new();
                buffer.push(TypeTag::Bool.to_u8());
                buffer.push(if *b { 0x01 } else { 0x00 });
                Ok(buffer)
            }
            LnmpValue::String(s) => {
                let mut buffer = Vec::new();
                buffer.push(TypeTag::String.to_u8());
                let bytes = s.as_bytes();
                buffer.extend_from_slice(&varint::encode(bytes.len() as i64));
                buffer.extend_from_slice(bytes);
                Ok(buffer)
            }
            LnmpValue::StringArray(arr) => {
                let mut buffer = Vec::new();
                buffer.push(TypeTag::StringArray.to_u8());
                buffer.extend_from_slice(&varint::encode(arr.len() as i64));
                for s in arr {
                    let bytes = s.as_bytes();
                    buffer.extend_from_slice(&varint::encode(bytes.len() as i64));
                    buffer.extend_from_slice(bytes);
                }
                Ok(buffer)
            }
            LnmpValue::NestedRecord(record) => {
                self.encode_nested_record_with_depth(record, current_depth)
            }
            LnmpValue::NestedArray(records) => {
                self.encode_nested_array_with_depth(records, current_depth)
            }
        }
    }
}

impl Default for BinaryNestedEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::LnmpField;

    #[test]
    fn test_nested_encoder_config_default() {
        let config = NestedEncoderConfig::default();
        assert_eq!(config.max_depth, 32);
        assert_eq!(config.max_record_size, None);
        assert!(!config.validate_canonical);
    }

    #[test]
    fn test_nested_encoder_config_builder() {
        let config = NestedEncoderConfig::new()
            .with_max_depth(16)
            .with_max_record_size(Some(1024))
            .with_validate_canonical(true);

        assert_eq!(config.max_depth, 16);
        assert_eq!(config.max_record_size, Some(1024));
        assert!(config.validate_canonical);
    }

    #[test]
    fn test_encode_empty_nested_record() {
        let encoder = BinaryNestedEncoder::new();
        let record = LnmpRecord::new();

        let result = encoder.encode_nested_record(&record).unwrap();

        // Should have TAG (0x06) + FIELD_COUNT (0)
        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x00); // Field count = 0
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_encode_single_level_nested_record() {
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

        let result = encoder.encode_nested_record(&record).unwrap();

        // Should start with TAG (0x06) + FIELD_COUNT (2)
        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x02); // Field count = 2
    }

    #[test]
    fn test_encode_nested_record_canonical_ordering() {
        let encoder = BinaryNestedEncoder::new();
        let mut record = LnmpRecord::new();
        // Add fields in non-sorted order
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(2),
        });

        let result = encoder.encode_nested_record(&record).unwrap();

        // Verify TAG and count
        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x03); // Field count = 3

        // Fields should be encoded in sorted order: 2, 5, 10
        // After TAG and COUNT, we should see FID=2 first
        assert_eq!(result[2], 0x02); // FID = 2 (VarInt)
    }

    #[test]
    fn test_encode_empty_nested_array() {
        let encoder = BinaryNestedEncoder::new();
        let records: Vec<LnmpRecord> = vec![];

        let result = encoder.encode_nested_array(&records).unwrap();

        // Should have TAG (0x07) + ELEMENT_COUNT (0)
        assert_eq!(result[0], 0x07); // NestedArray tag
        assert_eq!(result[1], 0x00); // Element count = 0
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_encode_nested_array_single_record() {
        let encoder = BinaryNestedEncoder::new();
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let result = encoder.encode_nested_array(&[record]).unwrap();

        // Should start with TAG (0x07) + ELEMENT_COUNT (1)
        assert_eq!(result[0], 0x07); // NestedArray tag
        assert_eq!(result[1], 0x01); // Element count = 1
        // Next should be the nested record (TAG 0x06)
        assert_eq!(result[2], 0x06); // NestedRecord tag
    }

    #[test]
    fn test_encode_nested_array_multiple_records() {
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

        let result = encoder.encode_nested_array(&[record1, record2]).unwrap();

        // Should start with TAG (0x07) + ELEMENT_COUNT (2)
        assert_eq!(result[0], 0x07); // NestedArray tag
        assert_eq!(result[1], 0x02); // Element count = 2
    }

    #[test]
    fn test_encode_depth_limit_exceeded() {
        let config = NestedEncoderConfig::new().with_max_depth(2);
        let encoder = BinaryNestedEncoder::with_config(config);

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

        let result = encoder.encode_nested_record(&level1);

        assert!(result.is_err());
        match result {
            Err(BinaryError::NestingDepthExceeded { depth, max }) => {
                assert_eq!(max, 2);
                assert!(depth >= 2);
            }
            _ => panic!("Expected NestingDepthExceeded error"),
        }
    }

    #[test]
    fn test_encode_size_limit_exceeded() {
        let config = NestedEncoderConfig::new().with_max_record_size(Some(10));
        let encoder = BinaryNestedEncoder::with_config(config);

        let mut record = LnmpRecord::new();
        // Add enough fields to exceed the size limit
        for i in 0..100 {
            record.add_field(LnmpField {
                fid: i,
                value: LnmpValue::String("test".to_string()),
            });
        }

        let result = encoder.encode_nested_record(&record);

        assert!(result.is_err());
        match result {
            Err(BinaryError::RecordSizeExceeded { size, max }) => {
                assert_eq!(max, 10);
                assert!(size > 10);
            }
            _ => panic!("Expected RecordSizeExceeded error"),
        }
    }

    #[test]
    fn test_encode_multi_level_nested_record() {
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

        let result = encoder.encode_nested_record(&outer_record).unwrap();

        // Should start with TAG (0x06) + FIELD_COUNT (1)
        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x01); // Field count = 1
        // Next should be FID=2
        assert_eq!(result[2], 0x02); // FID = 2
        // Next should be nested record TAG (0x06)
        assert_eq!(result[3], 0x06); // Inner NestedRecord tag
    }

    #[test]
    fn test_encode_nested_record_depth_2() {
        let encoder = BinaryNestedEncoder::new();

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

        let result = encoder.encode_nested_record(&level1).unwrap();
        assert!(result.len() > 0);
        assert_eq!(result[0], 0x06); // NestedRecord tag
    }

    #[test]
    fn test_encode_nested_record_depth_3() {
        let encoder = BinaryNestedEncoder::new();

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

        let result = encoder.encode_nested_record(&level1).unwrap();
        assert!(result.len() > 0);
        assert_eq!(result[0], 0x06); // NestedRecord tag
    }

    #[test]
    fn test_encode_nested_record_depth_4() {
        let encoder = BinaryNestedEncoder::new();

        // Create depth-4 nested structure
        let mut level4 = LnmpRecord::new();
        level4.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });

        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::NestedRecord(Box::new(level4)),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });

        let result = encoder.encode_nested_record(&level1).unwrap();
        assert!(result.len() > 0);
        assert_eq!(result[0], 0x06); // NestedRecord tag
    }

    #[test]
    fn test_encode_nested_record_depth_5() {
        let encoder = BinaryNestedEncoder::new();

        // Create depth-5 nested structure
        let mut level5 = LnmpRecord::new();
        level5.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });

        let mut level4 = LnmpRecord::new();
        level4.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::NestedRecord(Box::new(level5)),
        });

        let mut level3 = LnmpRecord::new();
        level3.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::NestedRecord(Box::new(level4)),
        });

        let mut level2 = LnmpRecord::new();
        level2.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::NestedRecord(Box::new(level3)),
        });

        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });

        let result = encoder.encode_nested_record(&level1).unwrap();
        assert!(result.len() > 0);
        assert_eq!(result[0], 0x06); // NestedRecord tag
    }

    #[test]
    fn test_encode_depth_limit_enforced_at_exact_limit() {
        let config = NestedEncoderConfig::new().with_max_depth(2);
        let encoder = BinaryNestedEncoder::with_config(config);

        // Create a structure at depth 1 (should succeed)
        let mut level1 = LnmpRecord::new();
        level1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        // This should succeed (depth 0 when encoding starts)
        let result = encoder.encode_nested_record(&level1);
        assert!(result.is_ok());

        // Create a structure at depth 2 (should succeed)
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

        // This should succeed (depth 1 when encoding the nested record)
        let result = encoder.encode_nested_record(&outer);
        assert!(result.is_ok());

        // Create a structure at depth 3 (should fail)
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

        let mut level1_deep = LnmpRecord::new();
        level1_deep.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::NestedRecord(Box::new(level2)),
        });

        // This should fail (depth 2 when encoding the deepest nested record)
        let result = encoder.encode_nested_record(&level1_deep);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_size_limit_enforced_incrementally() {
        let config = NestedEncoderConfig::new().with_max_record_size(Some(50));
        let encoder = BinaryNestedEncoder::with_config(config);

        let mut record = LnmpRecord::new();
        // Add fields until we exceed the limit
        for i in 0..20 {
            record.add_field(LnmpField {
                fid: i,
                value: LnmpValue::String("test".to_string()),
            });
        }

        let result = encoder.encode_nested_record(&record);
        assert!(result.is_err());
        match result {
            Err(BinaryError::RecordSizeExceeded { .. }) => {}
            _ => panic!("Expected RecordSizeExceeded error"),
        }
    }

    #[test]
    fn test_encode_canonical_ordering_at_all_levels() {
        let encoder = BinaryNestedEncoder::new();

        // Create nested structure with unsorted fields at each level
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(3),
        });
        inner.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(1),
        });
        inner.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(2),
        });

        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::String("last".to_string()),
        });
        outer.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });
        outer.add_field(LnmpField {
            fid: 15,
            value: LnmpValue::String("middle".to_string()),
        });

        let result = encoder.encode_nested_record(&outer).unwrap();

        // Verify outer level is sorted (FID 1 should come first)
        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x03); // Field count = 3
        assert_eq!(result[2], 0x01); // First FID = 1 (sorted)
    }

    #[test]
    fn test_encode_empty_nested_records_at_multiple_levels() {
        let encoder = BinaryNestedEncoder::new();

        let inner = LnmpRecord::new(); // Empty
        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let result = encoder.encode_nested_record(&outer).unwrap();

        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x01); // Field count = 1
        assert_eq!(result[2], 0x01); // FID = 1
        assert_eq!(result[3], 0x06); // Inner NestedRecord tag
        assert_eq!(result[4], 0x00); // Inner field count = 0
    }

    #[test]
    fn test_encode_empty_nested_arrays() {
        let encoder = BinaryNestedEncoder::new();

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::NestedArray(vec![]),
        });

        let result = encoder.encode_nested_record(&record).unwrap();

        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x01); // Field count = 1
        assert_eq!(result[2], 0x01); // FID = 1
        assert_eq!(result[3], 0x07); // NestedArray tag
        assert_eq!(result[4], 0x00); // Element count = 0
    }

    #[test]
    fn test_encode_nested_array_with_canonical_ordering() {
        let encoder = BinaryNestedEncoder::new();

        // Create records with unsorted fields
        let mut record1 = LnmpRecord::new();
        record1.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(2),
        });
        record1.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::Int(1),
        });

        let mut record2 = LnmpRecord::new();
        record2.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(4),
        });
        record2.add_field(LnmpField {
            fid: 15,
            value: LnmpValue::Int(3),
        });

        let result = encoder.encode_nested_array(&[record1, record2]).unwrap();

        // Verify array structure
        assert_eq!(result[0], 0x07); // NestedArray tag
        assert_eq!(result[1], 0x02); // Element count = 2

        // First record should have sorted fields
        assert_eq!(result[2], 0x06); // First record tag
        assert_eq!(result[3], 0x02); // First record field count = 2
        assert_eq!(result[4], 0x05); // First FID = 5 (sorted)
    }

    #[test]
    fn test_encode_mixed_primitive_and_nested_fields() {
        let encoder = BinaryNestedEncoder::new();

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

        let result = encoder.encode_nested_record(&outer).unwrap();

        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x03); // Field count = 3
    }

    #[test]
    fn test_encode_all_primitive_types_in_nested_record() {
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

        let result = encoder.encode_nested_record(&record).unwrap();

        assert_eq!(result[0], 0x06); // NestedRecord tag
        assert_eq!(result[1], 0x05); // Field count = 5
    }
}
