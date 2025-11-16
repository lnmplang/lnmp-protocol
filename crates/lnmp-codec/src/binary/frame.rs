//! Binary frame structure for LNMP v0.4 protocol.
//!
//! A BinaryFrame represents a complete LNMP record in binary format:
//! ```text
//! ┌─────────┬─────────┬─────────────┬──────────────────────┐
//! │ VERSION │  FLAGS  │ ENTRY_COUNT │      ENTRIES...      │
//! │ (1 byte)│(1 byte) │  (VarInt)   │     (variable)       │
//! └─────────┴─────────┴─────────────┴──────────────────────┘
//! ```

use lnmp_core::{LnmpRecord, LnmpField};
use super::error::BinaryError;
use super::entry::BinaryEntry;
use super::varint;

/// Protocol version for LNMP v0.4 binary format
const VERSION_0_4: u8 = 0x04;

/// Binary frame representing a complete LNMP record
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryFrame {
    /// Protocol version byte (0x04 for v0.4)
    version: u8,
    /// Flags byte (reserved for future use, 0x00 in v0.4)
    flags: u8,
    /// Entries in the frame
    entries: Vec<BinaryEntry>,
}

impl BinaryFrame {
    /// Creates a new frame with version 0x04 and flags 0x00
    ///
    /// # Arguments
    ///
    /// * `entries` - Vector of binary entries (should be sorted by FID for canonical form)
    pub fn new(entries: Vec<BinaryEntry>) -> Self {
        Self {
            version: VERSION_0_4,
            flags: 0x00,
            entries,
        }
    }

    /// Encodes the frame to bytes
    ///
    /// Binary layout:
    /// - VERSION (1 byte): 0x04
    /// - FLAGS (1 byte): 0x00
    /// - ENTRY_COUNT (VarInt): Number of entries
    /// - ENTRIES: Each entry encoded sequentially
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Write VERSION
        bytes.push(self.version);
        
        // Write FLAGS
        bytes.push(self.flags);
        
        // Write ENTRY_COUNT as VarInt
        bytes.extend_from_slice(&varint::encode(self.entries.len() as i64));
        
        // Write each entry
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.encode());
        }
        
        bytes
    }

    /// Decodes a frame from bytes
    ///
    /// # Errors
    ///
    /// Returns errors for:
    /// - `UnexpectedEof`: Insufficient data
    /// - `UnsupportedVersion`: Version byte is not 0x04
    /// - `InvalidVarInt`: Malformed entry count
    /// - Entry decoding errors
    pub fn decode(bytes: &[u8]) -> Result<Self, BinaryError> {
        let mut offset = 0;
        
        // Read VERSION (1 byte)
        if bytes.len() < 1 {
            return Err(BinaryError::UnexpectedEof {
                expected: 1,
                found: bytes.len(),
            });
        }
        let version = bytes[offset];
        offset += 1;
        
        // Validate version
        if version != VERSION_0_4 {
            return Err(BinaryError::UnsupportedVersion {
                found: version,
                supported: vec![VERSION_0_4],
            });
        }
        
        // Read FLAGS (1 byte)
        if bytes.len() < offset + 1 {
            return Err(BinaryError::UnexpectedEof {
                expected: offset + 1,
                found: bytes.len(),
            });
        }
        let flags = bytes[offset];
        offset += 1;
        
        // Decode ENTRY_COUNT (VarInt)
        let (entry_count, consumed) = varint::decode(&bytes[offset..])
            .map_err(|_| BinaryError::InvalidVarInt {
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
        let mut entries = Vec::with_capacity(entry_count);
        
        // Decode each entry
        for _ in 0..entry_count {
            let (entry, consumed) = BinaryEntry::decode(&bytes[offset..])?;
            offset += consumed;
            entries.push(entry);
        }
        
        Ok(Self {
            version,
            flags,
            entries,
        })
    }

    /// Converts to LnmpRecord
    pub fn to_record(&self) -> LnmpRecord {
        let fields: Vec<LnmpField> = self.entries
            .iter()
            .map(|entry| entry.to_field())
            .collect();
        
        LnmpRecord::from_sorted_fields(fields)
    }

    /// Creates from LnmpRecord, sorting fields by FID
    ///
    /// # Errors
    ///
    /// Returns `BinaryError::InvalidValue` if any field contains nested structures
    /// (not supported in v0.4 binary format)
    pub fn from_record(record: &LnmpRecord) -> Result<Self, BinaryError> {
        // Get fields sorted by FID for canonical form
        let sorted_fields = record.sorted_fields();
        
        // Convert each field to BinaryEntry
        let mut entries = Vec::with_capacity(sorted_fields.len());
        for field in sorted_fields {
            entries.push(BinaryEntry::from_field(&field)?);
        }
        
        Ok(Self::new(entries))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::LnmpValue;
    use super::super::types::{BinaryValue, TypeTag};

    #[test]
    fn test_new_frame() {
        let entries = vec![
            BinaryEntry {
                fid: 1,
                tag: TypeTag::Int,
                value: BinaryValue::Int(42),
            },
        ];
        
        let frame = BinaryFrame::new(entries.clone());
        assert_eq!(frame.version, VERSION_0_4);
        assert_eq!(frame.flags, 0x00);
        assert_eq!(frame.entries, entries);
    }

    #[test]
    fn test_encode_empty_frame() {
        let frame = BinaryFrame::new(vec![]);
        let bytes = frame.encode();
        
        // VERSION
        assert_eq!(bytes[0], 0x04);
        // FLAGS
        assert_eq!(bytes[1], 0x00);
        // ENTRY_COUNT (0 as VarInt)
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes.len(), 3);
    }

    #[test]
    fn test_encode_single_entry() {
        let entries = vec![
            BinaryEntry {
                fid: 7,
                tag: TypeTag::Bool,
                value: BinaryValue::Bool(true),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let bytes = frame.encode();
        
        // VERSION
        assert_eq!(bytes[0], 0x04);
        // FLAGS
        assert_eq!(bytes[1], 0x00);
        // ENTRY_COUNT (1 as VarInt)
        assert_eq!(bytes[2], 0x01);
        // Entry data follows
        assert!(bytes.len() > 3);
    }

    #[test]
    fn test_encode_multiple_entries() {
        let entries = vec![
            BinaryEntry {
                fid: 7,
                tag: TypeTag::Bool,
                value: BinaryValue::Bool(true),
            },
            BinaryEntry {
                fid: 12,
                tag: TypeTag::Int,
                value: BinaryValue::Int(14532),
            },
            BinaryEntry {
                fid: 23,
                tag: TypeTag::StringArray,
                value: BinaryValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let bytes = frame.encode();
        
        // VERSION
        assert_eq!(bytes[0], 0x04);
        // FLAGS
        assert_eq!(bytes[1], 0x00);
        // ENTRY_COUNT (3 as VarInt)
        assert_eq!(bytes[2], 0x03);
    }

    #[test]
    fn test_decode_empty_frame() {
        let frame = BinaryFrame::new(vec![]);
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        
        assert_eq!(decoded, frame);
    }

    #[test]
    fn test_decode_single_entry() {
        let entries = vec![
            BinaryEntry {
                fid: 1,
                tag: TypeTag::String,
                value: BinaryValue::String("hello".to_string()),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        
        assert_eq!(decoded, frame);
    }

    #[test]
    fn test_decode_multiple_entries() {
        let entries = vec![
            BinaryEntry {
                fid: 1,
                tag: TypeTag::Int,
                value: BinaryValue::Int(-42),
            },
            BinaryEntry {
                fid: 2,
                tag: TypeTag::Float,
                value: BinaryValue::Float(3.14),
            },
            BinaryEntry {
                fid: 3,
                tag: TypeTag::Bool,
                value: BinaryValue::Bool(false),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        
        assert_eq!(decoded, frame);
    }

    #[test]
    fn test_decode_unsupported_version() {
        let bytes = vec![0x99, 0x00, 0x00]; // Invalid version
        let result = BinaryFrame::decode(&bytes);
        
        assert!(matches!(
            result,
            Err(BinaryError::UnsupportedVersion { found: 0x99, .. })
        ));
    }

    #[test]
    fn test_decode_insufficient_data_version() {
        let bytes = vec![];
        let result = BinaryFrame::decode(&bytes);
        
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decode_insufficient_data_flags() {
        let bytes = vec![0x04]; // Only version, no flags
        let result = BinaryFrame::decode(&bytes);
        
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decode_invalid_entry_count_varint() {
        let bytes = vec![0x04, 0x00, 0x80]; // Incomplete VarInt
        let result = BinaryFrame::decode(&bytes);
        
        assert!(matches!(result, Err(BinaryError::InvalidVarInt { .. })));
    }

    #[test]
    fn test_decode_insufficient_entry_data() {
        let bytes = vec![0x04, 0x00, 0x01]; // Says 1 entry but no entry data
        let result = BinaryFrame::decode(&bytes);
        
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_to_record() {
        let entries = vec![
            BinaryEntry {
                fid: 7,
                tag: TypeTag::Bool,
                value: BinaryValue::Bool(true),
            },
            BinaryEntry {
                fid: 12,
                tag: TypeTag::Int,
                value: BinaryValue::Int(14532),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let record = frame.to_record();
        
        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_from_record() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        
        let frame = BinaryFrame::from_record(&record).unwrap();
        
        // Should be sorted by FID
        assert_eq!(frame.entries.len(), 2);
        assert_eq!(frame.entries[0].fid, 7);
        assert_eq!(frame.entries[1].fid, 12);
    }

    #[test]
    fn test_from_record_sorts_fields() {
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
        
        let frame = BinaryFrame::from_record(&record).unwrap();
        
        // Should be sorted by FID: 7, 12, 23
        assert_eq!(frame.entries.len(), 3);
        assert_eq!(frame.entries[0].fid, 7);
        assert_eq!(frame.entries[1].fid, 12);
        assert_eq!(frame.entries[2].fid, 23);
    }

    #[test]
    fn test_roundtrip_record_to_frame_to_record() {
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
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::String("hello".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        });
        
        let frame = BinaryFrame::from_record(&record).unwrap();
        let decoded_record = frame.to_record();
        
        // Fields should match (in sorted order)
        assert_eq!(decoded_record.fields().len(), 5);
        assert_eq!(decoded_record.get_field(1).unwrap().value, LnmpValue::Int(-42));
        assert_eq!(decoded_record.get_field(2).unwrap().value, LnmpValue::Float(3.14159));
        assert_eq!(decoded_record.get_field(3).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(decoded_record.get_field(4).unwrap().value, LnmpValue::String("hello".to_string()));
        assert_eq!(
            decoded_record.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_roundtrip_frame_encode_decode() {
        let entries = vec![
            BinaryEntry {
                fid: 1,
                tag: TypeTag::Int,
                value: BinaryValue::Int(100),
            },
            BinaryEntry {
                fid: 2,
                tag: TypeTag::String,
                value: BinaryValue::String("test".to_string()),
            },
        ];
        
        let frame = BinaryFrame::new(entries);
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        
        assert_eq!(decoded, frame);
    }

    #[test]
    fn test_flags_preserved() {
        let frame = BinaryFrame::new(vec![]);
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        
        assert_eq!(decoded.flags, 0x00);
    }

    #[test]
    fn test_empty_record() {
        let record = LnmpRecord::new();
        let frame = BinaryFrame::from_record(&record).unwrap();
        
        assert_eq!(frame.entries.len(), 0);
        
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        let decoded_record = decoded.to_record();
        
        assert_eq!(decoded_record.fields().len(), 0);
    }

    #[test]
    fn test_from_record_with_all_types() {
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
        
        let frame = BinaryFrame::from_record(&record).unwrap();
        assert_eq!(frame.entries.len(), 5);
        
        let bytes = frame.encode();
        let decoded = BinaryFrame::decode(&bytes).unwrap();
        let decoded_record = decoded.to_record();
        
        assert_eq!(decoded_record.fields().len(), 5);
    }

    #[test]
    fn test_version_validation() {
        // Test that only version 0x04 is accepted
        let valid_bytes = vec![0x04, 0x00, 0x00];
        assert!(BinaryFrame::decode(&valid_bytes).is_ok());
        
        let invalid_versions = vec![0x00, 0x01, 0x02, 0x03, 0x05, 0xFF];
        for version in invalid_versions {
            let bytes = vec![version, 0x00, 0x00];
            let result = BinaryFrame::decode(&bytes);
            assert!(matches!(result, Err(BinaryError::UnsupportedVersion { .. })));
        }
    }
}
