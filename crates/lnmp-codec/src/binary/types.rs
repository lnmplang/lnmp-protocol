//! Binary type system for LNMP v0.4 protocol.

use super::error::BinaryError;
use lnmp_core::LnmpValue;

/// Type tag for binary values (LNMP v0.4/v0.5)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    /// Integer type (VarInt encoded)
    Int = 0x01,
    /// Float type (8-byte IEEE754 LE)
    Float = 0x02,
    /// Boolean type (1 byte: 0x00 or 0x01)
    Bool = 0x03,
    /// String type (length VarInt + UTF-8 bytes)
    String = 0x04,
    /// String array type (count VarInt + repeated length+UTF-8)
    StringArray = 0x05,
    /// Nested record type (v0.5) - TAG + FIELD_COUNT + FID/VALUE pairs
    NestedRecord = 0x06,
    /// Nested array type (v0.5) - TAG + ELEMENT_COUNT + RECORD entries
    NestedArray = 0x07,
    /// Reserved for future use (v0.5+)
    Reserved08 = 0x08,
    /// Reserved for future use (v0.5+)
    Reserved09 = 0x09,
    /// Reserved for future use (v0.5+)
    Reserved0A = 0x0A,
    /// Reserved for future use (v0.5+)
    Reserved0B = 0x0B,
    /// Reserved for future use (v0.5+)
    Reserved0C = 0x0C,
    /// Reserved for future use (v0.5+)
    Reserved0D = 0x0D,
    /// Reserved for future use (v0.5+)
    Reserved0E = 0x0E,
    /// Reserved for future use (v0.5+)
    Reserved0F = 0x0F,
}

impl TypeTag {
    /// Converts a byte to a TypeTag
    pub fn from_u8(byte: u8) -> Result<Self, BinaryError> {
        match byte {
            0x01 => Ok(TypeTag::Int),
            0x02 => Ok(TypeTag::Float),
            0x03 => Ok(TypeTag::Bool),
            0x04 => Ok(TypeTag::String),
            0x05 => Ok(TypeTag::StringArray),
            0x06 => Ok(TypeTag::NestedRecord),
            0x07 => Ok(TypeTag::NestedArray),
            0x08 => Ok(TypeTag::Reserved08),
            0x09 => Ok(TypeTag::Reserved09),
            0x0A => Ok(TypeTag::Reserved0A),
            0x0B => Ok(TypeTag::Reserved0B),
            0x0C => Ok(TypeTag::Reserved0C),
            0x0D => Ok(TypeTag::Reserved0D),
            0x0E => Ok(TypeTag::Reserved0E),
            0x0F => Ok(TypeTag::Reserved0F),
            _ => Err(BinaryError::InvalidTypeTag { tag: byte }),
        }
    }

    /// Converts the TypeTag to a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns true if this is a v0.5+ type tag (nested or reserved)
    pub fn is_v0_5_type(&self) -> bool {
        matches!(
            self,
            TypeTag::NestedRecord
                | TypeTag::NestedArray
                | TypeTag::Reserved08
                | TypeTag::Reserved09
                | TypeTag::Reserved0A
                | TypeTag::Reserved0B
                | TypeTag::Reserved0C
                | TypeTag::Reserved0D
                | TypeTag::Reserved0E
                | TypeTag::Reserved0F
        )
    }

    /// Returns true if this is a reserved type tag
    pub fn is_reserved(&self) -> bool {
        matches!(
            self,
            TypeTag::Reserved08
                | TypeTag::Reserved09
                | TypeTag::Reserved0A
                | TypeTag::Reserved0B
                | TypeTag::Reserved0C
                | TypeTag::Reserved0D
                | TypeTag::Reserved0E
                | TypeTag::Reserved0F
        )
    }
}

/// Binary value representation for LNMP v0.4/v0.5
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryValue {
    /// Integer value (i64)
    Int(i64),
    /// Floating-point value (f64)
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// Array of strings
    StringArray(Vec<String>),
    /// Nested record (v0.5)
    NestedRecord(Box<lnmp_core::LnmpRecord>),
    /// Array of nested records (v0.5)
    NestedArray(Vec<lnmp_core::LnmpRecord>),
}

impl BinaryValue {
    /// Converts from LnmpValue to BinaryValue
    ///
    /// In v0.5, nested structures are supported. Use `from_lnmp_value_v0_4` for v0.4 compatibility.
    pub fn from_lnmp_value(value: &LnmpValue) -> Result<Self, BinaryError> {
        match value {
            LnmpValue::Int(i) => Ok(BinaryValue::Int(*i)),
            LnmpValue::Float(f) => Ok(BinaryValue::Float(*f)),
            LnmpValue::Bool(b) => Ok(BinaryValue::Bool(*b)),
            LnmpValue::String(s) => Ok(BinaryValue::String(s.clone())),
            LnmpValue::StringArray(arr) => Ok(BinaryValue::StringArray(arr.clone())),
            LnmpValue::NestedRecord(rec) => Ok(BinaryValue::NestedRecord(rec.clone())),
            LnmpValue::NestedArray(arr) => Ok(BinaryValue::NestedArray(arr.clone())),
        }
    }

    /// Converts from LnmpValue to BinaryValue (v0.4 compatibility mode)
    ///
    /// Returns an error if the value contains nested structures (not supported in v0.4)
    pub fn from_lnmp_value_v0_4(value: &LnmpValue) -> Result<Self, BinaryError> {
        match value {
            LnmpValue::Int(i) => Ok(BinaryValue::Int(*i)),
            LnmpValue::Float(f) => Ok(BinaryValue::Float(*f)),
            LnmpValue::Bool(b) => Ok(BinaryValue::Bool(*b)),
            LnmpValue::String(s) => Ok(BinaryValue::String(s.clone())),
            LnmpValue::StringArray(arr) => Ok(BinaryValue::StringArray(arr.clone())),
            LnmpValue::NestedRecord(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x06,
                reason: "Nested records not supported in v0.4 binary format".to_string(),
            }),
            LnmpValue::NestedArray(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x07,
                reason: "Nested arrays not supported in v0.4 binary format".to_string(),
            }),
        }
    }

    /// Converts to LnmpValue
    pub fn to_lnmp_value(&self) -> LnmpValue {
        match self {
            BinaryValue::Int(i) => LnmpValue::Int(*i),
            BinaryValue::Float(f) => LnmpValue::Float(*f),
            BinaryValue::Bool(b) => LnmpValue::Bool(*b),
            BinaryValue::String(s) => LnmpValue::String(s.clone()),
            BinaryValue::StringArray(arr) => LnmpValue::StringArray(arr.clone()),
            BinaryValue::NestedRecord(rec) => LnmpValue::NestedRecord(rec.clone()),
            BinaryValue::NestedArray(arr) => LnmpValue::NestedArray(arr.clone()),
        }
    }

    /// Returns the type tag for this value
    pub fn type_tag(&self) -> TypeTag {
        match self {
            BinaryValue::Int(_) => TypeTag::Int,
            BinaryValue::Float(_) => TypeTag::Float,
            BinaryValue::Bool(_) => TypeTag::Bool,
            BinaryValue::String(_) => TypeTag::String,
            BinaryValue::StringArray(_) => TypeTag::StringArray,
            BinaryValue::NestedRecord(_) => TypeTag::NestedRecord,
            BinaryValue::NestedArray(_) => TypeTag::NestedArray,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::*;
    use lnmp_core::LnmpRecord;

    #[test]
    fn test_type_tag_from_u8() {
        assert_eq!(TypeTag::from_u8(0x01).unwrap(), TypeTag::Int);
        assert_eq!(TypeTag::from_u8(0x02).unwrap(), TypeTag::Float);
        assert_eq!(TypeTag::from_u8(0x03).unwrap(), TypeTag::Bool);
        assert_eq!(TypeTag::from_u8(0x04).unwrap(), TypeTag::String);
        assert_eq!(TypeTag::from_u8(0x05).unwrap(), TypeTag::StringArray);
    }

    #[test]
    fn test_type_tag_from_u8_invalid() {
        assert!(TypeTag::from_u8(0x00).is_err());
        assert!(TypeTag::from_u8(0xFF).is_err());
    }

    #[test]
    fn test_type_tag_from_u8_v0_5_types() {
        // v0.5 types should now be valid
        assert_eq!(TypeTag::from_u8(0x06).unwrap(), TypeTag::NestedRecord);
        assert_eq!(TypeTag::from_u8(0x07).unwrap(), TypeTag::NestedArray);
    }

    #[test]
    fn test_type_tag_from_u8_reserved() {
        // Reserved types should be valid but marked as reserved
        assert_eq!(TypeTag::from_u8(0x08).unwrap(), TypeTag::Reserved08);
        assert_eq!(TypeTag::from_u8(0x09).unwrap(), TypeTag::Reserved09);
        assert_eq!(TypeTag::from_u8(0x0A).unwrap(), TypeTag::Reserved0A);
        assert_eq!(TypeTag::from_u8(0x0B).unwrap(), TypeTag::Reserved0B);
        assert_eq!(TypeTag::from_u8(0x0C).unwrap(), TypeTag::Reserved0C);
        assert_eq!(TypeTag::from_u8(0x0D).unwrap(), TypeTag::Reserved0D);
        assert_eq!(TypeTag::from_u8(0x0E).unwrap(), TypeTag::Reserved0E);
        assert_eq!(TypeTag::from_u8(0x0F).unwrap(), TypeTag::Reserved0F);
    }

    #[test]
    fn test_type_tag_to_u8() {
        assert_eq!(TypeTag::Int.to_u8(), 0x01);
        assert_eq!(TypeTag::Float.to_u8(), 0x02);
        assert_eq!(TypeTag::Bool.to_u8(), 0x03);
        assert_eq!(TypeTag::String.to_u8(), 0x04);
        assert_eq!(TypeTag::StringArray.to_u8(), 0x05);
    }

    #[test]
    fn test_type_tag_round_trip() {
        let tags = vec![
            TypeTag::Int,
            TypeTag::Float,
            TypeTag::Bool,
            TypeTag::String,
            TypeTag::StringArray,
            TypeTag::NestedRecord,
            TypeTag::NestedArray,
            TypeTag::Reserved08,
            TypeTag::Reserved09,
            TypeTag::Reserved0A,
            TypeTag::Reserved0B,
            TypeTag::Reserved0C,
            TypeTag::Reserved0D,
            TypeTag::Reserved0E,
            TypeTag::Reserved0F,
        ];

        for tag in tags {
            let byte = tag.to_u8();
            let parsed = TypeTag::from_u8(byte).unwrap();
            assert_eq!(parsed, tag);
        }
    }

    #[test]
    fn test_type_tag_is_v0_5_type() {
        // v0.4 types should return false
        assert!(!TypeTag::Int.is_v0_5_type());
        assert!(!TypeTag::Float.is_v0_5_type());
        assert!(!TypeTag::Bool.is_v0_5_type());
        assert!(!TypeTag::String.is_v0_5_type());
        assert!(!TypeTag::StringArray.is_v0_5_type());

        // v0.5 types should return true
        assert!(TypeTag::NestedRecord.is_v0_5_type());
        assert!(TypeTag::NestedArray.is_v0_5_type());
        assert!(TypeTag::Reserved08.is_v0_5_type());
        assert!(TypeTag::Reserved09.is_v0_5_type());
        assert!(TypeTag::Reserved0A.is_v0_5_type());
        assert!(TypeTag::Reserved0B.is_v0_5_type());
        assert!(TypeTag::Reserved0C.is_v0_5_type());
        assert!(TypeTag::Reserved0D.is_v0_5_type());
        assert!(TypeTag::Reserved0E.is_v0_5_type());
        assert!(TypeTag::Reserved0F.is_v0_5_type());
    }

    #[test]
    fn test_type_tag_is_reserved() {
        // Non-reserved types should return false
        assert!(!TypeTag::Int.is_reserved());
        assert!(!TypeTag::Float.is_reserved());
        assert!(!TypeTag::Bool.is_reserved());
        assert!(!TypeTag::String.is_reserved());
        assert!(!TypeTag::StringArray.is_reserved());
        assert!(!TypeTag::NestedRecord.is_reserved());
        assert!(!TypeTag::NestedArray.is_reserved());

        // Reserved types should return true
        assert!(TypeTag::Reserved08.is_reserved());
        assert!(TypeTag::Reserved09.is_reserved());
        assert!(TypeTag::Reserved0A.is_reserved());
        assert!(TypeTag::Reserved0B.is_reserved());
        assert!(TypeTag::Reserved0C.is_reserved());
        assert!(TypeTag::Reserved0D.is_reserved());
        assert!(TypeTag::Reserved0E.is_reserved());
        assert!(TypeTag::Reserved0F.is_reserved());
    }

    #[test]
    fn test_binary_value_from_lnmp_int() {
        let lnmp_val = LnmpValue::Int(42);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::Int(42));
    }

    #[test]
    fn test_binary_value_from_lnmp_float() {
        let lnmp_val = LnmpValue::Float(3.14);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::Float(3.14));
    }

    #[test]
    fn test_binary_value_from_lnmp_bool() {
        let lnmp_val = LnmpValue::Bool(true);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::Bool(true));
    }

    #[test]
    fn test_binary_value_from_lnmp_string() {
        let lnmp_val = LnmpValue::String("hello".to_string());
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::String("hello".to_string()));
    }

    #[test]
    fn test_binary_value_from_lnmp_string_array() {
        let lnmp_val = LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(
            binary_val,
            BinaryValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_binary_value_from_lnmp_nested_record() {
        // v0.5 now supports nested records
        let nested = LnmpValue::NestedRecord(Box::new(LnmpRecord::new()));
        let result = BinaryValue::from_lnmp_value(&nested);
        assert!(result.is_ok());
        match result.unwrap() {
            BinaryValue::NestedRecord(_) => {}
            _ => panic!("Expected NestedRecord variant"),
        }
    }

    #[test]
    fn test_binary_value_from_lnmp_nested_array() {
        // v0.5 now supports nested arrays
        let nested = LnmpValue::NestedArray(vec![]);
        let result = BinaryValue::from_lnmp_value(&nested);
        assert!(result.is_ok());
        match result.unwrap() {
            BinaryValue::NestedArray(_) => {}
            _ => panic!("Expected NestedArray variant"),
        }
    }

    #[test]
    fn test_binary_value_from_lnmp_nested_record_error_v0_4() {
        // v0.4 compatibility mode should still reject nested records
        let nested = LnmpValue::NestedRecord(Box::new(LnmpRecord::new()));
        let result = BinaryValue::from_lnmp_value_v0_4(&nested);
        assert!(result.is_err());
        match result {
            Err(BinaryError::InvalidValue { reason, .. }) => {
                assert!(reason.contains("not supported in v0.4"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_binary_value_from_lnmp_nested_array_error_v0_4() {
        // v0.4 compatibility mode should still reject nested arrays
        let nested = LnmpValue::NestedArray(vec![]);
        let result = BinaryValue::from_lnmp_value_v0_4(&nested);
        assert!(result.is_err());
        match result {
            Err(BinaryError::InvalidValue { reason, .. }) => {
                assert!(reason.contains("not supported in v0.4"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_binary_value_to_lnmp_int() {
        let binary_val = BinaryValue::Int(-42);
        let lnmp_val = binary_val.to_lnmp_value();
        assert_eq!(lnmp_val, LnmpValue::Int(-42));
    }

    #[test]
    fn test_binary_value_to_lnmp_float() {
        let binary_val = BinaryValue::Float(2.718);
        let lnmp_val = binary_val.to_lnmp_value();
        assert_eq!(lnmp_val, LnmpValue::Float(2.718));
    }

    #[test]
    fn test_binary_value_to_lnmp_bool() {
        let binary_val = BinaryValue::Bool(false);
        let lnmp_val = binary_val.to_lnmp_value();
        assert_eq!(lnmp_val, LnmpValue::Bool(false));
    }

    #[test]
    fn test_binary_value_to_lnmp_string() {
        let binary_val = BinaryValue::String("world".to_string());
        let lnmp_val = binary_val.to_lnmp_value();
        assert_eq!(lnmp_val, LnmpValue::String("world".to_string()));
    }

    #[test]
    fn test_binary_value_to_lnmp_string_array() {
        let binary_val = BinaryValue::StringArray(vec!["x".to_string(), "y".to_string()]);
        let lnmp_val = binary_val.to_lnmp_value();
        assert_eq!(
            lnmp_val,
            LnmpValue::StringArray(vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_binary_value_round_trip_int() {
        let original = LnmpValue::Int(12345);
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_round_trip_float() {
        let original = LnmpValue::Float(1.414);
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_round_trip_bool() {
        let original = LnmpValue::Bool(true);
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_round_trip_string() {
        let original = LnmpValue::String("test".to_string());
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_round_trip_string_array() {
        let original = LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]);
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_type_tag_int() {
        let val = BinaryValue::Int(100);
        assert_eq!(val.type_tag(), TypeTag::Int);
    }

    #[test]
    fn test_binary_value_type_tag_float() {
        let val = BinaryValue::Float(3.14);
        assert_eq!(val.type_tag(), TypeTag::Float);
    }

    #[test]
    fn test_binary_value_type_tag_bool() {
        let val = BinaryValue::Bool(true);
        assert_eq!(val.type_tag(), TypeTag::Bool);
    }

    #[test]
    fn test_binary_value_type_tag_string() {
        let val = BinaryValue::String("test".to_string());
        assert_eq!(val.type_tag(), TypeTag::String);
    }

    #[test]
    fn test_binary_value_type_tag_string_array() {
        let val = BinaryValue::StringArray(vec!["a".to_string()]);
        assert_eq!(val.type_tag(), TypeTag::StringArray);
    }

    #[test]
    fn test_binary_value_type_tag_nested_record() {
        let val = BinaryValue::NestedRecord(Box::new(LnmpRecord::new()));
        assert_eq!(val.type_tag(), TypeTag::NestedRecord);
    }

    #[test]
    fn test_binary_value_type_tag_nested_array() {
        let val = BinaryValue::NestedArray(vec![]);
        assert_eq!(val.type_tag(), TypeTag::NestedArray);
    }

    #[test]
    fn test_binary_value_round_trip_nested_record() {
        let mut record = LnmpRecord::new();
        record.add_field(lnmp_core::LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        let original = LnmpValue::NestedRecord(Box::new(record));
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_round_trip_nested_array() {
        let mut record = LnmpRecord::new();
        record.add_field(lnmp_core::LnmpField {
            fid: 1,
            value: LnmpValue::String("test".to_string()),
        });
        let original = LnmpValue::NestedArray(vec![record]);
        let binary = BinaryValue::from_lnmp_value(&original).unwrap();
        let back = binary.to_lnmp_value();
        assert_eq!(original, back);
    }

    #[test]
    fn test_binary_value_empty_string() {
        let lnmp_val = LnmpValue::String("".to_string());
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::String("".to_string()));
        let back = binary_val.to_lnmp_value();
        assert_eq!(back, lnmp_val);
    }

    #[test]
    fn test_binary_value_empty_string_array() {
        let lnmp_val = LnmpValue::StringArray(vec![]);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::StringArray(vec![]));
        let back = binary_val.to_lnmp_value();
        assert_eq!(back, lnmp_val);
    }

    #[test]
    fn test_binary_value_negative_int() {
        let lnmp_val = LnmpValue::Int(-9999);
        let binary_val = BinaryValue::from_lnmp_value(&lnmp_val).unwrap();
        assert_eq!(binary_val, BinaryValue::Int(-9999));
        let back = binary_val.to_lnmp_value();
        assert_eq!(back, lnmp_val);
    }

    #[test]
    fn test_binary_value_special_floats() {
        // Test NaN
        let nan_val = LnmpValue::Float(f64::NAN);
        let binary_nan = BinaryValue::from_lnmp_value(&nan_val).unwrap();
        match binary_nan {
            BinaryValue::Float(f) => assert!(f.is_nan()),
            _ => panic!("Expected Float variant"),
        }

        // Test Infinity
        let inf_val = LnmpValue::Float(f64::INFINITY);
        let binary_inf = BinaryValue::from_lnmp_value(&inf_val).unwrap();
        assert_eq!(binary_inf, BinaryValue::Float(f64::INFINITY));

        // Test Negative Infinity
        let neg_inf_val = LnmpValue::Float(f64::NEG_INFINITY);
        let binary_neg_inf = BinaryValue::from_lnmp_value(&neg_inf_val).unwrap();
        assert_eq!(binary_neg_inf, BinaryValue::Float(f64::NEG_INFINITY));
    }
}
