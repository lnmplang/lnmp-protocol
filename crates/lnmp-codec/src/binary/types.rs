//! Binary type system for LNMP v0.5 protocol.

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
    /// Embedding type (v0.5) - TAG + ENCODED_VECTOR
    Embedding = 0x08,
    /// Hybrid numeric array type (v0.5.16) - TAG + FLAGS + COUNT + DATA
    /// Supports i32/i64/f32/f64 in dense or sparse mode
    HybridNumericArray = 0x09,
    /// Quantized embedding type (v0.5.2) - TAG + QUANTIZED_VECTOR
    QuantizedEmbedding = 0x0A,
    /// Integer array type (v0.6) - TAG + COUNT + INT entries
    IntArray = 0x0B,
    /// Float array type (v0.6) - TAG + COUNT + FLOAT entries
    FloatArray = 0x0C,
    /// Boolean array type (v0.6) - TAG + COUNT + BOOL entries
    BoolArray = 0x0D,
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
            0x08 => Ok(TypeTag::Embedding),
            0x09 => Ok(TypeTag::HybridNumericArray),
            0x0A => Ok(TypeTag::QuantizedEmbedding),
            0x0B => Ok(TypeTag::IntArray),
            0x0C => Ok(TypeTag::FloatArray),
            0x0D => Ok(TypeTag::BoolArray),
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
                | TypeTag::Embedding
                | TypeTag::QuantizedEmbedding
                | TypeTag::HybridNumericArray
                | TypeTag::IntArray
                | TypeTag::FloatArray
                | TypeTag::BoolArray
                | TypeTag::Reserved0E
                | TypeTag::Reserved0F
        )
    }

    /// Returns true if this is a reserved type tag
    pub fn is_reserved(&self) -> bool {
        matches!(self, TypeTag::Reserved0E | TypeTag::Reserved0F)
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
    /// Array of integers (v0.6)
    IntArray(Vec<i64>),
    /// Array of floats (v0.6)
    FloatArray(Vec<f64>),
    /// Array of booleans (v0.6)
    BoolArray(Vec<bool>),
    /// Nested record (v0.5)
    NestedRecord(Box<lnmp_core::LnmpRecord>),
    /// Array of nested records (v0.5)
    NestedArray(Vec<lnmp_core::LnmpRecord>),
    /// Embedding (v0.5)
    Embedding(lnmp_embedding::Vector),
    /// Quantized embedding (v0.5.2)
    QuantizedEmbedding(lnmp_quant::QuantizedVector),
    /// Hybrid numeric array (v0.5.16) - supports i32/i64/f32/f64, dense or sparse
    HybridNumericArray(HybridArray),
}

/// Hybrid numeric array supporting multiple data types and encoding modes
#[derive(Debug, Clone, PartialEq)]
pub struct HybridArray {
    /// Data type of elements
    pub dtype: NumericDType,
    /// Whether this is sparse encoded
    pub sparse: bool,
    /// Total dimension (for sparse arrays, this is the full dimension)
    pub dim: usize,
    /// Raw data (for dense: all values; for sparse: indices then values)
    pub data: Vec<u8>,
}

/// Numeric data type for HybridNumericArray
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericDType {
    /// 32-bit signed integer
    I32 = 0,
    /// 64-bit signed integer
    I64 = 1,
    /// 32-bit float
    F32 = 2,
    /// 64-bit float
    F64 = 3,
}

impl NumericDType {
    /// Element size in bytes
    pub fn byte_size(&self) -> usize {
        match self {
            NumericDType::I32 | NumericDType::F32 => 4,
            NumericDType::I64 | NumericDType::F64 => 8,
        }
    }

    /// Parse from flags byte (bits 0-1)
    pub fn from_flags(flags: u8) -> Self {
        match flags & 0x03 {
            0 => NumericDType::I32,
            1 => NumericDType::I64,
            2 => NumericDType::F32,
            _ => NumericDType::F64,
        }
    }

    /// Convert to flags bits
    pub fn to_flags(&self) -> u8 {
        *self as u8
    }
}

impl HybridArray {
    /// Create a new dense f32 array
    pub fn from_f32_dense(values: &[f32]) -> Self {
        let mut data = Vec::with_capacity(values.len() * 4);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        Self {
            dtype: NumericDType::F32,
            sparse: false,
            dim: values.len(),
            data,
        }
    }

    /// Create a new dense f64 array
    pub fn from_f64_dense(values: &[f64]) -> Self {
        let mut data = Vec::with_capacity(values.len() * 8);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        Self {
            dtype: NumericDType::F64,
            sparse: false,
            dim: values.len(),
            data,
        }
    }

    /// Create a new dense i32 array
    pub fn from_i32_dense(values: &[i32]) -> Self {
        let mut data = Vec::with_capacity(values.len() * 4);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        Self {
            dtype: NumericDType::I32,
            sparse: false,
            dim: values.len(),
            data,
        }
    }

    /// Create a new dense i64 array
    pub fn from_i64_dense(values: &[i64]) -> Self {
        let mut data = Vec::with_capacity(values.len() * 8);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes());
        }
        Self {
            dtype: NumericDType::I64,
            sparse: false,
            dim: values.len(),
            data,
        }
    }

    /// Get f32 values (dense mode only)
    pub fn as_f32_vec(&self) -> Option<Vec<f32>> {
        if self.dtype != NumericDType::F32 || self.sparse {
            return None;
        }
        let mut result = Vec::with_capacity(self.dim);
        for chunk in self.data.chunks_exact(4) {
            result.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Some(result)
    }

    /// Get f64 values (dense mode only)
    pub fn as_f64_vec(&self) -> Option<Vec<f64>> {
        if self.dtype != NumericDType::F64 || self.sparse {
            return None;
        }
        let mut result = Vec::with_capacity(self.dim);
        for chunk in self.data.chunks_exact(8) {
            result.push(f64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]));
        }
        Some(result)
    }

    /// Encode flags byte
    pub fn flags(&self) -> u8 {
        let mut flags = self.dtype.to_flags();
        if self.sparse {
            flags |= 0x04; // bit 2
        }
        flags
    }
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
            LnmpValue::IntArray(arr) => Ok(BinaryValue::IntArray(arr.clone())),
            LnmpValue::FloatArray(arr) => Ok(BinaryValue::FloatArray(arr.clone())),
            LnmpValue::BoolArray(arr) => Ok(BinaryValue::BoolArray(arr.clone())),
            LnmpValue::NestedRecord(rec) => Ok(BinaryValue::NestedRecord(rec.clone())),
            LnmpValue::NestedArray(arr) => Ok(BinaryValue::NestedArray(arr.clone())),
            LnmpValue::Embedding(vec) => Ok(BinaryValue::Embedding(vec.clone())),
            LnmpValue::EmbeddingDelta(_) => Err(BinaryError::InvalidValue {
                reason: "EmbeddingDelta cannot be encoded as BinaryValue, use full embedding"
                    .into(),
                field_id: 0,
                type_tag: 0x08,
            }),
            LnmpValue::QuantizedEmbedding(qv) => Ok(BinaryValue::QuantizedEmbedding(qv.clone())),
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
            LnmpValue::IntArray(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x0B,
                reason: "IntArray not supported in v0.4 binary format".to_string(),
            }),
            LnmpValue::FloatArray(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x0C,
                reason: "FloatArray not supported in v0.4 binary format".to_string(),
            }),
            LnmpValue::BoolArray(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x0D,
                reason: "BoolArray not supported in v0.4 binary format".to_string(),
            }),
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
            LnmpValue::Embedding(_) => Err(BinaryError::InvalidValue {
                field_id: 0,
                type_tag: 0x08,
                reason: "Embeddings not supported in v0.4 binary format".to_string(),
            }),
            LnmpValue::EmbeddingDelta(_) => Err(BinaryError::InvalidValue {
                reason: "EmbeddingDelta not supported in v0.4".to_string(),
                field_id: 0,
                type_tag: 0x08,
            }),
            LnmpValue::QuantizedEmbedding(_) => Err(BinaryError::InvalidValue {
                reason: "QuantizedEmbedding not supported in v0.4".to_string(),
                field_id: 0,
                type_tag: 0x0A,
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
            BinaryValue::IntArray(arr) => LnmpValue::IntArray(arr.clone()),
            BinaryValue::FloatArray(arr) => LnmpValue::FloatArray(arr.clone()),
            BinaryValue::BoolArray(arr) => LnmpValue::BoolArray(arr.clone()),
            BinaryValue::NestedRecord(rec) => LnmpValue::NestedRecord(rec.clone()),
            BinaryValue::NestedArray(arr) => LnmpValue::NestedArray(arr.clone()),
            BinaryValue::Embedding(vec) => LnmpValue::Embedding(vec.clone()),
            BinaryValue::QuantizedEmbedding(qv) => LnmpValue::QuantizedEmbedding(qv.clone()),
            BinaryValue::HybridNumericArray(arr) => {
                // Convert to appropriate LnmpValue based on dtype
                match arr.dtype {
                    NumericDType::I32 | NumericDType::I64 => {
                        // Convert to IntArray
                        if let Some(vals) = arr.as_f64_vec() {
                            LnmpValue::IntArray(vals.iter().map(|v| *v as i64).collect())
                        } else {
                            LnmpValue::IntArray(vec![])
                        }
                    }
                    NumericDType::F32 => {
                        if let Some(vals) = arr.as_f32_vec() {
                            LnmpValue::FloatArray(vals.iter().map(|v| *v as f64).collect())
                        } else {
                            LnmpValue::FloatArray(vec![])
                        }
                    }
                    NumericDType::F64 => {
                        if let Some(vals) = arr.as_f64_vec() {
                            LnmpValue::FloatArray(vals)
                        } else {
                            LnmpValue::FloatArray(vec![])
                        }
                    }
                }
            }
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
            BinaryValue::IntArray(_) => TypeTag::IntArray,
            BinaryValue::FloatArray(_) => TypeTag::FloatArray,
            BinaryValue::BoolArray(_) => TypeTag::BoolArray,
            BinaryValue::NestedRecord(_) => TypeTag::NestedRecord,
            BinaryValue::NestedArray(_) => TypeTag::NestedArray,
            BinaryValue::Embedding(_) => TypeTag::Embedding,
            BinaryValue::QuantizedEmbedding(_) => TypeTag::QuantizedEmbedding,
            BinaryValue::HybridNumericArray(_) => TypeTag::HybridNumericArray,
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
        assert_eq!(TypeTag::from_u8(0x08).unwrap(), TypeTag::Embedding);
        assert_eq!(TypeTag::from_u8(0x09).unwrap(), TypeTag::HybridNumericArray);
        assert_eq!(TypeTag::from_u8(0x0A).unwrap(), TypeTag::QuantizedEmbedding);
        assert_eq!(TypeTag::from_u8(0x0B).unwrap(), TypeTag::IntArray);
        assert_eq!(TypeTag::from_u8(0x0C).unwrap(), TypeTag::FloatArray);
        assert_eq!(TypeTag::from_u8(0x0D).unwrap(), TypeTag::BoolArray);
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
            TypeTag::Embedding,
            TypeTag::HybridNumericArray,
            TypeTag::QuantizedEmbedding,
            TypeTag::IntArray,
            TypeTag::FloatArray,
            TypeTag::BoolArray,
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
        assert!(TypeTag::Embedding.is_v0_5_type());
        assert!(TypeTag::HybridNumericArray.is_v0_5_type());
        assert!(TypeTag::QuantizedEmbedding.is_v0_5_type());
        assert!(TypeTag::IntArray.is_v0_5_type());
        assert!(TypeTag::FloatArray.is_v0_5_type());
        assert!(TypeTag::BoolArray.is_v0_5_type());
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
        assert!(!TypeTag::Embedding.is_reserved());
        assert!(!TypeTag::HybridNumericArray.is_reserved());
        assert!(!TypeTag::QuantizedEmbedding.is_reserved());
        assert!(!TypeTag::IntArray.is_reserved());
        assert!(!TypeTag::FloatArray.is_reserved());
        assert!(!TypeTag::BoolArray.is_reserved());
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

    #[test]
    fn test_hybrid_array_f32_dense() {
        let values: Vec<f32> = vec![1.0, 2.5, -3.14, 0.0, 100.0];
        let arr = HybridArray::from_f32_dense(&values);

        assert_eq!(arr.dtype, NumericDType::F32);
        assert!(!arr.sparse);
        assert_eq!(arr.dim, 5);
        assert_eq!(arr.data.len(), 20); // 5 * 4 bytes

        // Verify we can get values back
        let recovered = arr.as_f32_vec().unwrap();
        assert_eq!(recovered.len(), 5);
        assert!((recovered[0] - 1.0).abs() < 0.0001);
        assert!((recovered[1] - 2.5).abs() < 0.0001);
        assert!((recovered[2] - (-3.14)).abs() < 0.0001);
    }

    #[test]
    fn test_hybrid_array_flags() {
        let arr_i32 = HybridArray::from_i32_dense(&[1, 2, 3]);
        assert_eq!(arr_i32.flags(), 0x00); // I32, dense

        let arr_f32 = HybridArray::from_f32_dense(&[1.0, 2.0]);
        assert_eq!(arr_f32.flags(), 0x02); // F32, dense

        let arr_f64 = HybridArray::from_f64_dense(&[1.0, 2.0]);
        assert_eq!(arr_f64.flags(), 0x03); // F64, dense
    }

    #[test]
    fn test_numeric_dtype_byte_size() {
        assert_eq!(NumericDType::I32.byte_size(), 4);
        assert_eq!(NumericDType::I64.byte_size(), 8);
        assert_eq!(NumericDType::F32.byte_size(), 4);
        assert_eq!(NumericDType::F64.byte_size(), 8);
    }
}
