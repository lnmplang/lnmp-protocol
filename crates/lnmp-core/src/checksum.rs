//! Semantic checksum (SC32) system for LNMP v0.3.
//!
//! This module provides the SC32 (Semantic Checksum 32-bit) system for preventing
//! LLM input drift and ensuring consistency. Checksums are computed from the combination
//! of Field ID, type hint, and normalized value.
//!
//! ## Algorithm
//!
//! 1. Normalize the value using ValueNormalizer
//! 2. Serialize as: `{fid}:{type_hint}:{normalized_value}`
//! 3. Compute CRC32 hash
//! 4. Return 32-bit checksum
//!
//! ## Example
//!
//! ```
//! use lnmp_core::{LnmpValue, TypeHint};
//! use lnmp_core::checksum::SemanticChecksum;
//!
//! let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &LnmpValue::Int(14532));
//! let formatted = SemanticChecksum::format(checksum);
//! println!("F12:i=14532#{}", formatted);
//! ```

use crate::{FieldId, LnmpValue, TypeHint};
use crc::Crc;
use crc::CRC_32_ISO_HDLC;

/// Semantic checksum computation and validation
pub struct SemanticChecksum;

impl SemanticChecksum {
    /// Computes SC32 checksum for a field
    ///
    /// The checksum is computed from the combination of:
    /// - Field ID (FID)
    /// - Type hint (optional)
    /// - Normalized value
    ///
    /// # Arguments
    ///
    /// * `fid` - The field identifier
    /// * `type_hint` - Optional type hint for the field
    /// * `value` - The field value
    ///
    /// # Returns
    ///
    /// A 32-bit checksum value
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpValue, TypeHint};
    /// use lnmp_core::checksum::SemanticChecksum;
    ///
    /// let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &LnmpValue::Int(14532));
    /// assert!(checksum > 0);
    /// ```
    pub fn compute(fid: FieldId, type_hint: Option<TypeHint>, value: &LnmpValue) -> u32 {
        // Serialize the field components into a canonical string
        let serialized = Self::serialize_for_checksum(fid, type_hint, value);
        
        // Compute CRC32/ISO-HDLC (zlib/IEEE) — canonical CRC32 variant
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(serialized.as_bytes());
        digest.finalize()
    }

    /// Validates checksum against field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field identifier
    /// * `type_hint` - Optional type hint for the field
    /// * `value` - The field value
    /// * `checksum` - The checksum to validate against
    ///
    /// # Returns
    ///
    /// `true` if the checksum matches, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpValue, TypeHint};
    /// use lnmp_core::checksum::SemanticChecksum;
    ///
    /// let value = LnmpValue::Int(14532);
    /// let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
    /// assert!(SemanticChecksum::validate(12, Some(TypeHint::Int), &value, checksum));
    /// ```
    pub fn validate(fid: FieldId, type_hint: Option<TypeHint>, value: &LnmpValue, checksum: u32) -> bool {
        let computed = Self::compute(fid, type_hint, value);
        computed == checksum
    }

    /// Formats checksum as 8-character hexadecimal string
    ///
    /// # Arguments
    ///
    /// * `checksum` - The checksum value to format
    ///
    /// # Returns
    ///
    /// An 8-character uppercase hexadecimal string
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::checksum::SemanticChecksum;
    ///
    /// let formatted = SemanticChecksum::format(0x36AAE667);
    /// assert_eq!(formatted, "36AAE667");
    /// ```
    pub fn format(checksum: u32) -> String {
        format!("{:08X}", checksum)
    }

    /// Parses a hexadecimal checksum string
    ///
    /// # Arguments
    ///
    /// * `s` - The hexadecimal string to parse (with or without 0x prefix)
    ///
    /// # Returns
    ///
    /// `Some(checksum)` if parsing succeeds, `None` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::checksum::SemanticChecksum;
    ///
    /// assert_eq!(SemanticChecksum::parse("36AAE667"), Some(0x36AAE667));
    /// assert_eq!(SemanticChecksum::parse("0x36AAE667"), Some(0x36AAE667));
    /// assert_eq!(SemanticChecksum::parse("invalid"), None);
    /// ```
    pub fn parse(s: &str) -> Option<u32> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        // Require exactly 8 hex digits for SC32 checksum format
        if s.len() != 8 {
            return None;
        }
        u32::from_str_radix(s, 16).ok()
    }

    /// Serializes field components for checksum computation
    ///
    /// Format: `{fid}:{type_hint}:{normalized_value}`
    ///
    /// If no type hint is provided, it's inferred from the value type.
    fn serialize_for_checksum(fid: FieldId, type_hint: Option<TypeHint>, value: &LnmpValue) -> String {
        let hint = type_hint.unwrap_or_else(|| Self::infer_type_hint(value));
        let value_str = Self::serialize_value(value);
        format!("{}:{}:{}", fid, hint.as_str(), value_str)
    }

    /// Infers type hint from value
    fn infer_type_hint(value: &LnmpValue) -> TypeHint {
        match value {
            LnmpValue::Int(_) => TypeHint::Int,
            LnmpValue::Float(_) => TypeHint::Float,
            LnmpValue::Bool(_) => TypeHint::Bool,
            LnmpValue::String(_) => TypeHint::String,
            LnmpValue::StringArray(_) => TypeHint::StringArray,
            LnmpValue::NestedRecord(_) => TypeHint::Record,
            LnmpValue::NestedArray(_) => TypeHint::RecordArray,
        }
    }

    /// Serializes a value to its canonical string representation
    ///
    /// This applies normalization rules:
    /// - Booleans: 1 or 0
    /// - Floats: -0.0 → 0.0, trailing zeros removed
    /// - Strings: as-is (case normalization handled by ValueNormalizer if needed)
    /// - Arrays: comma-separated values
    /// - Nested structures: recursive serialization
    fn serialize_value(value: &LnmpValue) -> String {
        match value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Float(f) => {
                // Normalize -0.0 to 0.0
                let normalized = if *f == 0.0 { 0.0 } else { *f };
                // Format and remove trailing zeros
                Self::format_float(normalized)
            }
            LnmpValue::Bool(b) => {
                // Canonical boolean representation: 1 or 0
                if *b { "1" } else { "0" }.to_string()
            }
            LnmpValue::String(s) => s.clone(),
            LnmpValue::StringArray(arr) => {
                // Serialize as comma-separated list
                arr.join(",")
            }
            LnmpValue::NestedRecord(record) => {
                // Serialize nested record fields in sorted order
                let mut parts = Vec::new();
                for field in record.sorted_fields() {
                    let field_str = format!(
                        "{}:{}:{}",
                        field.fid,
                        Self::infer_type_hint(&field.value).as_str(),
                        Self::serialize_value(&field.value)
                    );
                    parts.push(field_str);
                }
                format!("{{{}}}", parts.join(";"))
            }
            LnmpValue::NestedArray(records) => {
                // Serialize nested array elements
                let mut parts = Vec::new();
                for record in records {
                    let mut field_parts = Vec::new();
                    for field in record.sorted_fields() {
                        let field_str = format!(
                            "{}:{}:{}",
                            field.fid,
                            Self::infer_type_hint(&field.value).as_str(),
                            Self::serialize_value(&field.value)
                        );
                        field_parts.push(field_str);
                    }
                    parts.push(format!("{{{}}}", field_parts.join(";")));
                }
                format!("[{}]", parts.join(","))
            }
        }
    }

    /// Formats a float with trailing zeros removed
    fn format_float(f: f64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LnmpField, LnmpRecord};

    #[test]
    fn test_compute_int() {
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        
        // Checksum should be deterministic
        let checksum2 = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        assert_eq!(checksum, checksum2);
        
        // Checksum should be non-zero
        assert_ne!(checksum, 0);
    }

    #[test]
    fn test_compute_different_fid() {
        let value = LnmpValue::Int(14532);
        let checksum1 = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        let checksum2 = SemanticChecksum::compute(13, Some(TypeHint::Int), &value);
        
        // Different FIDs should produce different checksums
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_compute_different_type_hint() {
        let value = LnmpValue::Int(14532);
        let checksum1 = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        let checksum2 = SemanticChecksum::compute(12, Some(TypeHint::Float), &value);
        
        // Different type hints should produce different checksums
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_compute_different_value() {
        let value1 = LnmpValue::Int(14532);
        let value2 = LnmpValue::Int(14533);
        let checksum1 = SemanticChecksum::compute(12, Some(TypeHint::Int), &value1);
        let checksum2 = SemanticChecksum::compute(12, Some(TypeHint::Int), &value2);
        
        // Different values should produce different checksums
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_validate_correct_checksum() {
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        
        assert!(SemanticChecksum::validate(12, Some(TypeHint::Int), &value, checksum));
    }

    #[test]
    fn test_validate_incorrect_checksum() {
        let value = LnmpValue::Int(14532);
        let checksum = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        
        // Wrong checksum should fail validation
        assert!(!SemanticChecksum::validate(12, Some(TypeHint::Int), &value, checksum + 1));
    }

    #[test]
    fn test_format_checksum() {
        let checksum = 0x36AAE667;
        let formatted = SemanticChecksum::format(checksum);
        assert_eq!(formatted, "36AAE667");
        assert_eq!(formatted.len(), 8);
    }

    #[test]
    fn test_format_checksum_with_leading_zeros() {
        let checksum = 0x00000001;
        let formatted = SemanticChecksum::format(checksum);
        assert_eq!(formatted, "00000001");
        assert_eq!(formatted.len(), 8);
    }

    #[test]
    fn test_parse_checksum() {
        assert_eq!(SemanticChecksum::parse("36AAE667"), Some(0x36AAE667));
        assert_eq!(SemanticChecksum::parse("0x36AAE667"), Some(0x36AAE667));
        assert_eq!(SemanticChecksum::parse("00000001"), Some(0x00000001));
        assert_eq!(SemanticChecksum::parse("FFFFFFFF"), Some(0xFFFFFFFF));
    }

    #[test]
    fn test_parse_invalid_checksum() {
        assert_eq!(SemanticChecksum::parse("invalid"), None);
        assert_eq!(SemanticChecksum::parse(""), None);
        assert_eq!(SemanticChecksum::parse("GGGGGGGG"), None);
        assert_eq!(SemanticChecksum::parse("123"), Some(0x123)); // Valid but short
    }

    #[test]
    fn test_parse_format_round_trip() {
        let original = 0x36AAE667;
        let formatted = SemanticChecksum::format(original);
        let parsed = SemanticChecksum::parse(&formatted);
        assert_eq!(parsed, Some(original));
    }

    #[test]
    fn test_serialize_bool_canonical() {
        let value_true = LnmpValue::Bool(true);
        let value_false = LnmpValue::Bool(false);
        
        assert_eq!(SemanticChecksum::serialize_value(&value_true), "1");
        assert_eq!(SemanticChecksum::serialize_value(&value_false), "0");
    }

    #[test]
    fn test_serialize_float_negative_zero() {
        let value = LnmpValue::Float(-0.0);
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "0");
    }

    #[test]
    fn test_serialize_float_trailing_zeros() {
        let value = LnmpValue::Float(3.140);
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "3.14");
    }

    #[test]
    fn test_serialize_float_no_trailing_zeros() {
        let value = LnmpValue::Float(3.14);
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "3.14");
    }

    #[test]
    fn test_serialize_string() {
        let value = LnmpValue::String("test".to_string());
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "test");
    }

    #[test]
    fn test_serialize_string_array() {
        let value = LnmpValue::StringArray(vec![
            "admin".to_string(),
            "developer".to_string(),
            "user".to_string(),
        ]);
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "admin,developer,user");
    }

    #[test]
    fn test_serialize_empty_string_array() {
        let value = LnmpValue::StringArray(vec![]);
        let serialized = SemanticChecksum::serialize_value(&value);
        assert_eq!(serialized, "");
    }

    #[test]
    fn test_compute_with_inferred_type_hint() {
        let value = LnmpValue::Int(14532);
        let checksum_explicit = SemanticChecksum::compute(12, Some(TypeHint::Int), &value);
        let checksum_inferred = SemanticChecksum::compute(12, None, &value);
        
        // Should produce the same checksum
        assert_eq!(checksum_explicit, checksum_inferred);
    }

    #[test]
    fn test_infer_type_hint() {
        assert_eq!(SemanticChecksum::infer_type_hint(&LnmpValue::Int(42)), TypeHint::Int);
        assert_eq!(SemanticChecksum::infer_type_hint(&LnmpValue::Float(3.14)), TypeHint::Float);
        assert_eq!(SemanticChecksum::infer_type_hint(&LnmpValue::Bool(true)), TypeHint::Bool);
        assert_eq!(SemanticChecksum::infer_type_hint(&LnmpValue::String("test".to_string())), TypeHint::String);
        assert_eq!(SemanticChecksum::infer_type_hint(&LnmpValue::StringArray(vec![])), TypeHint::StringArray);
    }

    #[test]
    fn test_serialize_nested_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        
        let value = LnmpValue::NestedRecord(Box::new(record));
        let serialized = SemanticChecksum::serialize_value(&value);
        
        // Fields should be sorted by FID (7 before 12)
        assert_eq!(serialized, "{7:b:1;12:i:1}");
    }

    #[test]
    fn test_serialize_nested_array() {
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
        
        let value = LnmpValue::NestedArray(vec![record1, record2]);
        let serialized = SemanticChecksum::serialize_value(&value);
        
        assert_eq!(serialized, "[{12:i:1},{12:i:2}]");
    }

    #[test]
    fn test_compute_nested_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        
        let value = LnmpValue::NestedRecord(Box::new(record));
        let checksum = SemanticChecksum::compute(50, Some(TypeHint::Record), &value);
        
        // Should be deterministic
        let checksum2 = SemanticChecksum::compute(50, Some(TypeHint::Record), &value);
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_compute_nested_array() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        
        let value = LnmpValue::NestedArray(vec![record]);
        let checksum = SemanticChecksum::compute(60, Some(TypeHint::RecordArray), &value);
        
        // Should be deterministic
        let checksum2 = SemanticChecksum::compute(60, Some(TypeHint::RecordArray), &value);
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_format_float_integer() {
        assert_eq!(SemanticChecksum::format_float(3.0), "3");
        assert_eq!(SemanticChecksum::format_float(0.0), "0");
        assert_eq!(SemanticChecksum::format_float(-5.0), "-5");
    }

    #[test]
    fn test_format_float_decimal() {
        assert_eq!(SemanticChecksum::format_float(3.14), "3.14");
        assert_eq!(SemanticChecksum::format_float(3.1), "3.1");
        assert_eq!(SemanticChecksum::format_float(0.5), "0.5");
    }

    #[test]
    fn test_serialize_for_checksum() {
        let value = LnmpValue::Int(14532);
        let serialized = SemanticChecksum::serialize_for_checksum(12, Some(TypeHint::Int), &value);
        assert_eq!(serialized, "12:i:14532");
    }

    #[test]
    fn test_serialize_for_checksum_no_type_hint() {
        let value = LnmpValue::Int(14532);
        let serialized = SemanticChecksum::serialize_for_checksum(12, None, &value);
        assert_eq!(serialized, "12:i:14532");
    }

    #[test]
    fn test_checksum_consistency_across_calls() {
        // Ensure the same input always produces the same checksum
        let value = LnmpValue::String("test".to_string());
        let checksums: Vec<u32> = (0..100)
            .map(|_| SemanticChecksum::compute(5, Some(TypeHint::String), &value))
            .collect();
        
        let first = checksums[0];
        assert!(checksums.iter().all(|&c| c == first));
    }
}
