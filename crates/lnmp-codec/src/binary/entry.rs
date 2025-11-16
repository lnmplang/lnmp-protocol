//! Binary entry structure for LNMP v0.4 protocol.
//!
//! A BinaryEntry represents a single field in binary format, consisting of:
//! - FID (Field Identifier): 2 bytes, little-endian
//! - TAG (Type Tag): 1 byte
//! - VALUE: Variable length, encoding depends on type

use lnmp_core::{LnmpField, FieldId};
use super::error::BinaryError;
use super::types::{BinaryValue, TypeTag};
use super::varint;

/// A single field encoded in binary format
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryEntry {
    /// Field identifier (16-bit unsigned integer)
    pub fid: FieldId,
    /// Type tag indicating value type
    pub tag: TypeTag,
    /// The encoded value
    pub value: BinaryValue,
}

impl BinaryEntry {
    /// Creates a new binary entry
    pub fn new(fid: FieldId, value: BinaryValue) -> Self {
        Self {
            fid,
            tag: value.type_tag(),
            value,
        }
    }

    /// Creates a new binary entry from an LnmpField
    ///
    /// # Errors
    ///
    /// Returns `BinaryError::InvalidValue` if the field contains nested structures
    /// (not supported in v0.4 binary format)
    pub fn from_field(field: &LnmpField) -> Result<Self, BinaryError> {
        let value = BinaryValue::from_lnmp_value(&field.value)
            .map_err(|e| match e {
                BinaryError::InvalidValue { type_tag, reason, .. } => {
                    BinaryError::InvalidValue {
                        field_id: field.fid,
                        type_tag,
                        reason,
                    }
                }
                other => other,
            })?;
        
        Ok(Self {
            fid: field.fid,
            tag: value.type_tag(),
            value,
        })
    }

    /// Converts to an LnmpField
    pub fn to_field(&self) -> LnmpField {
        LnmpField {
            fid: self.fid,
            value: self.value.to_lnmp_value(),
        }
    }

    /// Returns the type tag of this entry
    pub fn type_tag(&self) -> TypeTag {
        self.tag
    }

    /// Encodes the entry to bytes
    ///
    /// Binary layout:
    /// ```text
    /// ┌──────────┬──────────┬──────────────────┐
    /// │   FID    │  THTAG   │      VALUE       │
    /// │ (2 bytes)│ (1 byte) │   (variable)     │
    /// └──────────┴──────────┴──────────────────┘
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Write FID (2 bytes, little-endian)
        bytes.extend_from_slice(&self.fid.to_le_bytes());
        
        // Write TAG (1 byte)
        bytes.push(self.tag.to_u8());
        
        // Write VALUE (encoding depends on type)
        match &self.value {
            BinaryValue::Int(i) => {
                // VarInt encoding
                bytes.extend_from_slice(&varint::encode(*i));
            }
            BinaryValue::Float(f) => {
                // 8 bytes IEEE754 little-endian
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            BinaryValue::Bool(b) => {
                // 1 byte: 0x00 for false, 0x01 for true
                bytes.push(if *b { 0x01 } else { 0x00 });
            }
            BinaryValue::String(s) => {
                // Length (VarInt) + UTF-8 bytes
                let utf8_bytes = s.as_bytes();
                bytes.extend_from_slice(&varint::encode(utf8_bytes.len() as i64));
                bytes.extend_from_slice(utf8_bytes);
            }
            BinaryValue::StringArray(arr) => {
                // Count (VarInt) + repeated (length + UTF-8 bytes)
                bytes.extend_from_slice(&varint::encode(arr.len() as i64));
                for s in arr {
                    let utf8_bytes = s.as_bytes();
                    bytes.extend_from_slice(&varint::encode(utf8_bytes.len() as i64));
                    bytes.extend_from_slice(utf8_bytes);
                }
            }
            BinaryValue::NestedRecord(_) | BinaryValue::NestedArray(_) => {
                // Nested structure encoding will be implemented in task 2
                // For now, this is a placeholder that should not be reached
                // as nested encoding requires special handling
                panic!("Nested structure encoding not yet implemented - use BinaryNestedEncoder");
            }
        }
        
        bytes
    }

    /// Decodes an entry from bytes
    ///
    /// Returns a tuple of (BinaryEntry, bytes_consumed)
    ///
    /// # Errors
    ///
    /// Returns errors for:
    /// - `UnexpectedEof`: Insufficient data
    /// - `InvalidTypeTag`: Unknown type tag
    /// - `InvalidVarInt`: Malformed VarInt
    /// - `InvalidUtf8`: Invalid UTF-8 in string
    /// - `InvalidValue`: Other value decoding errors
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), BinaryError> {
        let mut offset = 0;
        
        // Read FID (2 bytes, little-endian)
        if bytes.len() < 2 {
            return Err(BinaryError::UnexpectedEof {
                expected: 2,
                found: bytes.len(),
            });
        }
        let fid = u16::from_le_bytes([bytes[0], bytes[1]]);
        offset += 2;
        
        // Read TAG (1 byte)
        if bytes.len() < offset + 1 {
            return Err(BinaryError::UnexpectedEof {
                expected: offset + 1,
                found: bytes.len(),
            });
        }
        let tag = TypeTag::from_u8(bytes[offset])?;
        offset += 1;
        
        // Read VALUE (depends on type)
        let value = match tag {
            TypeTag::NestedRecord | TypeTag::NestedArray => {
                // Nested structure decoding will be implemented in task 3
                return Err(BinaryError::InvalidValue {
                    field_id: fid,
                    type_tag: tag.to_u8(),
                    reason: "Nested structure decoding not yet implemented - use BinaryNestedDecoder".to_string(),
                });
            }
            TypeTag::Reserved08 | TypeTag::Reserved09 | TypeTag::Reserved0A 
            | TypeTag::Reserved0B | TypeTag::Reserved0C | TypeTag::Reserved0D 
            | TypeTag::Reserved0E | TypeTag::Reserved0F => {
                return Err(BinaryError::InvalidValue {
                    field_id: fid,
                    type_tag: tag.to_u8(),
                    reason: format!("Reserved type tag 0x{:02X} cannot be used", tag.to_u8()),
                });
            }
            TypeTag::Int => {
                let (int_val, consumed) = varint::decode(&bytes[offset..])
                    .map_err(|_| BinaryError::InvalidValue {
                        field_id: fid,
                        type_tag: tag.to_u8(),
                        reason: "Invalid VarInt encoding".to_string(),
                    })?;
                offset += consumed;
                BinaryValue::Int(int_val)
            }
            TypeTag::Float => {
                if bytes.len() < offset + 8 {
                    return Err(BinaryError::UnexpectedEof {
                        expected: offset + 8,
                        found: bytes.len(),
                    });
                }
                let float_bytes: [u8; 8] = bytes[offset..offset + 8]
                    .try_into()
                    .expect("slice length checked");
                let float_val = f64::from_le_bytes(float_bytes);
                offset += 8;
                BinaryValue::Float(float_val)
            }
            TypeTag::Bool => {
                if bytes.len() < offset + 1 {
                    return Err(BinaryError::UnexpectedEof {
                        expected: offset + 1,
                        found: bytes.len(),
                    });
                }
                let bool_val = match bytes[offset] {
                    0x00 => false,
                    0x01 => true,
                    other => {
                        return Err(BinaryError::InvalidValue {
                            field_id: fid,
                            type_tag: tag.to_u8(),
                            reason: format!("Invalid boolean value: 0x{:02X} (expected 0x00 or 0x01)", other),
                        });
                    }
                };
                offset += 1;
                BinaryValue::Bool(bool_val)
            }
            TypeTag::String => {
                let (length, consumed) = varint::decode(&bytes[offset..])
                    .map_err(|_| BinaryError::InvalidValue {
                        field_id: fid,
                        type_tag: tag.to_u8(),
                        reason: "Invalid string length VarInt".to_string(),
                    })?;
                offset += consumed;
                
                if length < 0 {
                    return Err(BinaryError::InvalidValue {
                        field_id: fid,
                        type_tag: tag.to_u8(),
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
                
                let string_val = std::str::from_utf8(&bytes[offset..offset + length])
                    .map_err(|_| BinaryError::InvalidUtf8 { field_id: fid })?
                    .to_string();
                offset += length;
                BinaryValue::String(string_val)
            }
            TypeTag::StringArray => {
                let (count, consumed) = varint::decode(&bytes[offset..])
                    .map_err(|_| BinaryError::InvalidValue {
                        field_id: fid,
                        type_tag: tag.to_u8(),
                        reason: "Invalid array count VarInt".to_string(),
                    })?;
                offset += consumed;
                
                if count < 0 {
                    return Err(BinaryError::InvalidValue {
                        field_id: fid,
                        type_tag: tag.to_u8(),
                        reason: format!("Negative array count: {}", count),
                    });
                }
                
                let count = count as usize;
                let mut strings = Vec::with_capacity(count);
                
                for _ in 0..count {
                    let (length, consumed) = varint::decode(&bytes[offset..])
                        .map_err(|_| BinaryError::InvalidValue {
                            field_id: fid,
                            type_tag: tag.to_u8(),
                            reason: "Invalid string length in array".to_string(),
                        })?;
                    offset += consumed;
                    
                    if length < 0 {
                        return Err(BinaryError::InvalidValue {
                            field_id: fid,
                            type_tag: tag.to_u8(),
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
                    
                    let string_val = std::str::from_utf8(&bytes[offset..offset + length])
                        .map_err(|_| BinaryError::InvalidUtf8 { field_id: fid })?
                        .to_string();
                    offset += length;
                    strings.push(string_val);
                }
                
                BinaryValue::StringArray(strings)
            }
        };
        
        Ok((
            Self {
                fid,
                tag,
                value,
            },
            offset,
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::LnmpValue;

    #[test]
    fn test_from_field_int() {
        let field = LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        };
        
        let entry = BinaryEntry::from_field(&field).unwrap();
        assert_eq!(entry.fid, 12);
        assert_eq!(entry.tag, TypeTag::Int);
        assert_eq!(entry.value, BinaryValue::Int(14532));
    }

    #[test]
    fn test_from_field_float() {
        let field = LnmpField {
            fid: 5,
            value: LnmpValue::Float(3.14),
        };
        
        let entry = BinaryEntry::from_field(&field).unwrap();
        assert_eq!(entry.fid, 5);
        assert_eq!(entry.tag, TypeTag::Float);
        assert_eq!(entry.value, BinaryValue::Float(3.14));
    }

    #[test]
    fn test_from_field_bool() {
        let field = LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        };
        
        let entry = BinaryEntry::from_field(&field).unwrap();
        assert_eq!(entry.fid, 7);
        assert_eq!(entry.tag, TypeTag::Bool);
        assert_eq!(entry.value, BinaryValue::Bool(true));
    }

    #[test]
    fn test_from_field_string() {
        let field = LnmpField {
            fid: 1,
            value: LnmpValue::String("hello".to_string()),
        };
        
        let entry = BinaryEntry::from_field(&field).unwrap();
        assert_eq!(entry.fid, 1);
        assert_eq!(entry.tag, TypeTag::String);
        assert_eq!(entry.value, BinaryValue::String("hello".to_string()));
    }

    #[test]
    fn test_from_field_string_array() {
        let field = LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        };
        
        let entry = BinaryEntry::from_field(&field).unwrap();
        assert_eq!(entry.fid, 23);
        assert_eq!(entry.tag, TypeTag::StringArray);
        assert_eq!(
            entry.value,
            BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_to_field_int() {
        let entry = BinaryEntry {
            fid: 12,
            tag: TypeTag::Int,
            value: BinaryValue::Int(42),
        };
        
        let field = entry.to_field();
        assert_eq!(field.fid, 12);
        assert_eq!(field.value, LnmpValue::Int(42));
    }

    #[test]
    fn test_to_field_float() {
        let entry = BinaryEntry {
            fid: 5,
            tag: TypeTag::Float,
            value: BinaryValue::Float(2.718),
        };
        
        let field = entry.to_field();
        assert_eq!(field.fid, 5);
        assert_eq!(field.value, LnmpValue::Float(2.718));
    }

    #[test]
    fn test_to_field_bool() {
        let entry = BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(false),
        };
        
        let field = entry.to_field();
        assert_eq!(field.fid, 7);
        assert_eq!(field.value, LnmpValue::Bool(false));
    }

    #[test]
    fn test_to_field_string() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::String,
            value: BinaryValue::String("world".to_string()),
        };
        
        let field = entry.to_field();
        assert_eq!(field.fid, 1);
        assert_eq!(field.value, LnmpValue::String("world".to_string()));
    }

    #[test]
    fn test_to_field_string_array() {
        let entry = BinaryEntry {
            fid: 23,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec!["x".to_string(), "y".to_string()]),
        };
        
        let field = entry.to_field();
        assert_eq!(field.fid, 23);
        assert_eq!(
            field.value,
            LnmpValue::StringArray(vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_encode_int() {
        let entry = BinaryEntry {
            fid: 12,
            tag: TypeTag::Int,
            value: BinaryValue::Int(14532),
        };
        
        let bytes = entry.encode();
        
        // FID (12 in little-endian)
        assert_eq!(bytes[0], 0x0C);
        assert_eq!(bytes[1], 0x00);
        // TAG (Int = 0x01)
        assert_eq!(bytes[2], 0x01);
        // VALUE (14532 as VarInt)
        let varint_bytes = varint::encode(14532);
        assert_eq!(&bytes[3..], &varint_bytes[..]);
    }

    #[test]
    fn test_encode_float() {
        let entry = BinaryEntry {
            fid: 5,
            tag: TypeTag::Float,
            value: BinaryValue::Float(3.14),
        };
        
        let bytes = entry.encode();
        
        // FID (5 in little-endian)
        assert_eq!(bytes[0], 0x05);
        assert_eq!(bytes[1], 0x00);
        // TAG (Float = 0x02)
        assert_eq!(bytes[2], 0x02);
        // VALUE (3.14 as IEEE754 LE)
        let float_bytes = 3.14f64.to_le_bytes();
        assert_eq!(&bytes[3..11], &float_bytes[..]);
    }

    #[test]
    fn test_encode_bool_true() {
        let entry = BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        };
        
        let bytes = entry.encode();
        
        // FID (7 in little-endian)
        assert_eq!(bytes[0], 0x07);
        assert_eq!(bytes[1], 0x00);
        // TAG (Bool = 0x03)
        assert_eq!(bytes[2], 0x03);
        // VALUE (true = 0x01)
        assert_eq!(bytes[3], 0x01);
    }

    #[test]
    fn test_encode_bool_false() {
        let entry = BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(false),
        };
        
        let bytes = entry.encode();
        
        // FID (7 in little-endian)
        assert_eq!(bytes[0], 0x07);
        assert_eq!(bytes[1], 0x00);
        // TAG (Bool = 0x03)
        assert_eq!(bytes[2], 0x03);
        // VALUE (false = 0x00)
        assert_eq!(bytes[3], 0x00);
    }

    #[test]
    fn test_encode_string() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::String,
            value: BinaryValue::String("hello".to_string()),
        };
        
        let bytes = entry.encode();
        
        // FID (1 in little-endian)
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x00);
        // TAG (String = 0x04)
        assert_eq!(bytes[2], 0x04);
        // VALUE (length VarInt + UTF-8)
        let length_varint = varint::encode(5);
        assert_eq!(&bytes[3..3 + length_varint.len()], &length_varint[..]);
        let offset = 3 + length_varint.len();
        assert_eq!(&bytes[offset..], b"hello");
    }

    #[test]
    fn test_encode_string_array() {
        let entry = BinaryEntry {
            fid: 23,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        };
        
        let bytes = entry.encode();
        
        // FID (23 in little-endian)
        assert_eq!(bytes[0], 0x17);
        assert_eq!(bytes[1], 0x00);
        // TAG (StringArray = 0x05)
        assert_eq!(bytes[2], 0x05);
        
        let mut offset = 3;
        // Count VarInt (2)
        let count_varint = varint::encode(2);
        assert_eq!(&bytes[offset..offset + count_varint.len()], &count_varint[..]);
        offset += count_varint.len();
        
        // First string "admin"
        let len1_varint = varint::encode(5);
        assert_eq!(&bytes[offset..offset + len1_varint.len()], &len1_varint[..]);
        offset += len1_varint.len();
        assert_eq!(&bytes[offset..offset + 5], b"admin");
        offset += 5;
        
        // Second string "dev"
        let len2_varint = varint::encode(3);
        assert_eq!(&bytes[offset..offset + len2_varint.len()], &len2_varint[..]);
        offset += len2_varint.len();
        assert_eq!(&bytes[offset..offset + 3], b"dev");
    }

    #[test]
    fn test_decode_int() {
        let entry = BinaryEntry {
            fid: 12,
            tag: TypeTag::Int,
            value: BinaryValue::Int(14532),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn test_decode_float() {
        let entry = BinaryEntry {
            fid: 5,
            tag: TypeTag::Float,
            value: BinaryValue::Float(3.14),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn test_decode_bool() {
        let entry = BinaryEntry {
            fid: 7,
            tag: TypeTag::Bool,
            value: BinaryValue::Bool(true),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn test_decode_string() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::String,
            value: BinaryValue::String("hello".to_string()),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn test_decode_string_array() {
        let entry = BinaryEntry {
            fid: 23,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        };
        
        let bytes = entry.encode();
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn test_decode_with_trailing_data() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::Int,
            value: BinaryValue::Int(42),
        };
        
        let mut bytes = entry.encode();
        bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Extra bytes
        
        let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();
        
        assert_eq!(decoded, entry);
        assert_eq!(consumed, bytes.len() - 3); // Should not consume trailing bytes
    }

    #[test]
    fn test_decode_insufficient_data_fid() {
        let bytes = vec![0x01]; // Only 1 byte, need 2 for FID
        let result = BinaryEntry::decode(&bytes);
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decode_insufficient_data_tag() {
        let bytes = vec![0x01, 0x00]; // FID but no TAG
        let result = BinaryEntry::decode(&bytes);
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decode_invalid_type_tag() {
        let bytes = vec![0x01, 0x00, 0xFF]; // Invalid TAG
        let result = BinaryEntry::decode(&bytes);
        assert!(matches!(result, Err(BinaryError::InvalidTypeTag { .. })));
    }

    #[test]
    fn test_decode_invalid_bool_value() {
        let bytes = vec![0x01, 0x00, 0x03, 0x02]; // Bool with value 0x02
        let result = BinaryEntry::decode(&bytes);
        assert!(matches!(result, Err(BinaryError::InvalidValue { .. })));
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let mut bytes = vec![0x01, 0x00, 0x04]; // FID=1, TAG=String
        bytes.extend_from_slice(&varint::encode(3)); // Length=3
        bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
        
        let result = BinaryEntry::decode(&bytes);
        assert!(matches!(result, Err(BinaryError::InvalidUtf8 { .. })));
    }

    #[test]
    fn test_roundtrip_all_types() {
        let test_cases = vec![
            BinaryEntry {
                fid: 1,
                tag: TypeTag::Int,
                value: BinaryValue::Int(-42),
            },
            BinaryEntry {
                fid: 2,
                tag: TypeTag::Float,
                value: BinaryValue::Float(3.14159),
            },
            BinaryEntry {
                fid: 3,
                tag: TypeTag::Bool,
                value: BinaryValue::Bool(true),
            },
            BinaryEntry {
                fid: 4,
                tag: TypeTag::String,
                value: BinaryValue::String("test".to_string()),
            },
            BinaryEntry {
                fid: 5,
                tag: TypeTag::StringArray,
                value: BinaryValue::StringArray(vec!["a".to_string(), "b".to_string()]),
            },
        ];

        for entry in test_cases {
            let bytes = entry.encode();
            let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
            assert_eq!(decoded, entry);
        }
    }

    #[test]
    fn test_fid_boundary_values() {
        // Test FID = 0
        let entry0 = BinaryEntry {
            fid: 0,
            tag: TypeTag::Int,
            value: BinaryValue::Int(1),
        };
        let bytes0 = entry0.encode();
        let (decoded0, _) = BinaryEntry::decode(&bytes0).unwrap();
        assert_eq!(decoded0.fid, 0);

        // Test FID = 65535 (max u16)
        let entry_max = BinaryEntry {
            fid: 65535,
            tag: TypeTag::Int,
            value: BinaryValue::Int(1),
        };
        let bytes_max = entry_max.encode();
        let (decoded_max, _) = BinaryEntry::decode(&bytes_max).unwrap();
        assert_eq!(decoded_max.fid, 65535);
    }

    #[test]
    fn test_empty_string() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::String,
            value: BinaryValue::String("".to_string()),
        };
        
        let bytes = entry.encode();
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_empty_string_array() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::StringArray,
            value: BinaryValue::StringArray(vec![]),
        };
        
        let bytes = entry.encode();
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_string_with_unicode() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::String,
            value: BinaryValue::String("Hello 🎯 World".to_string()),
        };
        
        let bytes = entry.encode();
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_negative_int() {
        let entry = BinaryEntry {
            fid: 1,
            tag: TypeTag::Int,
            value: BinaryValue::Int(-9999),
        };
        
        let bytes = entry.encode();
        let (decoded, _) = BinaryEntry::decode(&bytes).unwrap();
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_special_floats() {
        // Test NaN
        let entry_nan = BinaryEntry {
            fid: 1,
            tag: TypeTag::Float,
            value: BinaryValue::Float(f64::NAN),
        };
        let bytes_nan = entry_nan.encode();
        let (decoded_nan, _) = BinaryEntry::decode(&bytes_nan).unwrap();
        match decoded_nan.value {
            BinaryValue::Float(f) => assert!(f.is_nan()),
            _ => panic!("Expected Float variant"),
        }

        // Test Infinity
        let entry_inf = BinaryEntry {
            fid: 2,
            tag: TypeTag::Float,
            value: BinaryValue::Float(f64::INFINITY),
        };
        let bytes_inf = entry_inf.encode();
        let (decoded_inf, _) = BinaryEntry::decode(&bytes_inf).unwrap();
        assert_eq!(decoded_inf, entry_inf);

        // Test Negative Infinity
        let entry_neg_inf = BinaryEntry {
            fid: 3,
            tag: TypeTag::Float,
            value: BinaryValue::Float(f64::NEG_INFINITY),
        };
        let bytes_neg_inf = entry_neg_inf.encode();
        let (decoded_neg_inf, _) = BinaryEntry::decode(&bytes_neg_inf).unwrap();
        assert_eq!(decoded_neg_inf, entry_neg_inf);
    }
}
