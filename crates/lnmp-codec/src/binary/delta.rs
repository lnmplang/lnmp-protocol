//! Delta Encoding & Partial Update Layer (DPL) for LNMP v0.5.
//!
//! This module provides efficient delta encoding for transmitting only changed fields
//! in record updates, minimizing bandwidth usage for incremental changes.

use super::error::BinaryError;
use lnmp_core::{FieldId, LnmpRecord, LnmpValue};

/// Delta operation types for partial updates
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaOperation {
    /// Set field value (0x01)
    SetField = 0x01,
    /// Delete field (0x02)
    DeleteField = 0x02,
    /// Update existing field value (0x03)
    UpdateField = 0x03,
    /// Merge nested record (0x04)
    MergeRecord = 0x04,
}

impl DeltaOperation {
    /// Converts a byte to a DeltaOperation
    pub fn from_u8(byte: u8) -> Result<Self, DeltaError> {
        match byte {
            0x01 => Ok(DeltaOperation::SetField),
            0x02 => Ok(DeltaOperation::DeleteField),
            0x03 => Ok(DeltaOperation::UpdateField),
            0x04 => Ok(DeltaOperation::MergeRecord),
            _ => Err(DeltaError::InvalidOperation { op_code: byte }),
        }
    }

    /// Converts the DeltaOperation to a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Delta packet tag (0xB0)
pub const DELTA_TAG: u8 = 0xB0;

/// Represents a single delta operation
#[derive(Debug, Clone, PartialEq)]
pub struct DeltaOp {
    /// Target field identifier
    pub target_fid: FieldId,
    /// Operation type
    pub operation: DeltaOperation,
    /// Payload data (encoded value or nested operations)
    pub payload: Vec<u8>,
}

impl DeltaOp {
    /// Creates a new delta operation
    pub fn new(target_fid: FieldId, operation: DeltaOperation, payload: Vec<u8>) -> Self {
        Self {
            target_fid,
            operation,
            payload,
        }
    }
}

/// Configuration for delta encoding
#[derive(Debug, Clone)]
pub struct DeltaConfig {
    /// Enable delta encoding mode
    pub enable_delta: bool,
    /// Track changes for delta computation
    pub track_changes: bool,
}

impl DeltaConfig {
    /// Creates a new DeltaConfig with default settings
    pub fn new() -> Self {
        Self {
            enable_delta: false,
            track_changes: false,
        }
    }

    /// Enables delta encoding
    pub fn with_enable_delta(mut self, enable: bool) -> Self {
        self.enable_delta = enable;
        self
    }

    /// Enables change tracking
    pub fn with_track_changes(mut self, track: bool) -> Self {
        self.track_changes = track;
        self
    }
}

impl Default for DeltaConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for delta encoding operations
#[derive(Debug, Clone, PartialEq)]
pub enum DeltaError {
    /// Invalid target FID
    InvalidTargetFid {
        /// The invalid FID
        fid: FieldId,
    },
    /// Invalid operation code
    InvalidOperation {
        /// The invalid operation code
        op_code: u8,
    },
    /// Merge conflict
    MergeConflict {
        /// Field ID where conflict occurred
        fid: FieldId,
        /// Reason for the conflict
        reason: String,
    },
    /// Delta application failed
    DeltaApplicationFailed {
        /// Reason for the failure
        reason: String,
    },
    /// Binary encoding error
    BinaryError {
        /// The underlying binary error
        source: BinaryError,
    },
}

impl std::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaError::InvalidTargetFid { fid } => {
                write!(f, "Invalid target FID: {}", fid)
            }
            DeltaError::InvalidOperation { op_code } => {
                write!(f, "Invalid operation code: 0x{:02X}", op_code)
            }
            DeltaError::MergeConflict { fid, reason } => {
                write!(f, "Merge conflict at FID {}: {}", fid, reason)
            }
            DeltaError::DeltaApplicationFailed { reason } => {
                write!(f, "Delta application failed: {}", reason)
            }
            DeltaError::BinaryError { source } => {
                write!(f, "Binary error: {}", source)
            }
        }
    }
}

impl std::error::Error for DeltaError {}

impl From<BinaryError> for DeltaError {
    fn from(err: BinaryError) -> Self {
        DeltaError::BinaryError { source: err }
    }
}

/// Delta encoder for computing and encoding delta operations
pub struct DeltaEncoder {
    config: DeltaConfig,
}

impl DeltaEncoder {
    /// Creates a new delta encoder with default configuration
    pub fn new() -> Self {
        Self {
            config: DeltaConfig::default(),
        }
    }

    /// Creates a delta encoder with custom configuration
    pub fn with_config(config: DeltaConfig) -> Self {
        Self { config }
    }

    /// Computes delta operations between two records
    ///
    /// Identifies changed, added, and deleted fields between old and new records.
    ///
    /// # Arguments
    ///
    /// * `old` - The original record
    /// * `new` - The updated record
    ///
    /// # Returns
    ///
    /// A vector of delta operations representing the changes
    pub fn compute_delta(
        &self,
        old: &LnmpRecord,
        new: &LnmpRecord,
    ) -> Result<Vec<DeltaOp>, DeltaError> {
        // If delta is not enabled in the config, we return an error.

        if !self.config.enable_delta {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: "Delta is disabled in configuration".to_string(),
            });
        }

        self.diff_records(old, new)
    }

    /// Identifies changed, added, and deleted fields between two records
    fn diff_records(&self, old: &LnmpRecord, new: &LnmpRecord) -> Result<Vec<DeltaOp>, DeltaError> {
        use std::collections::HashSet;

        let mut ops = Vec::new();

        // Get all FIDs from both records
        let old_fids: HashSet<FieldId> = old.fields().iter().map(|f| f.fid).collect();
        let new_fids: HashSet<FieldId> = new.fields().iter().map(|f| f.fid).collect();

        // Process fields in sorted order for deterministic output
        let mut all_fids: Vec<FieldId> = old_fids.union(&new_fids).copied().collect();
        all_fids.sort_unstable();

        for fid in all_fids {
            let old_field = old.get_field(fid);
            let new_field = new.get_field(fid);

            match (old_field, new_field) {
                (None, Some(new_f)) => {
                    // Field added - use SET_FIELD
                    let payload = self.encode_value(&new_f.value)?;
                    ops.push(DeltaOp::new(fid, DeltaOperation::SetField, payload));
                }
                (Some(_), None) => {
                    // Field deleted - use DELETE_FIELD
                    ops.push(DeltaOp::new(fid, DeltaOperation::DeleteField, vec![]));
                }
                (Some(old_f), Some(new_f)) => {
                    // Field exists in both - check if changed
                    if old_f.value != new_f.value {
                        // Check if both are nested records - use MERGE_RECORD
                        match (&old_f.value, &new_f.value) {
                            (
                                LnmpValue::NestedRecord(old_rec),
                                LnmpValue::NestedRecord(new_rec),
                            ) => {
                                // Recursively compute delta for nested record
                                let nested_ops = self.diff_records(old_rec, new_rec)?;
                                let payload = self.encode_nested_ops(&nested_ops)?;
                                ops.push(DeltaOp::new(fid, DeltaOperation::MergeRecord, payload));
                            }
                            _ => {
                                // Value changed - use UPDATE_FIELD
                                let payload = self.encode_value(&new_f.value)?;
                                ops.push(DeltaOp::new(fid, DeltaOperation::UpdateField, payload));
                            }
                        }
                    }
                    // If values are equal, no operation needed
                }
                (None, None) => {
                    // Should never happen since we iterate over union of FIDs
                    unreachable!()
                }
            }
        }

        Ok(ops)
    }

    /// Encodes a single value to bytes
    fn encode_value(&self, value: &LnmpValue) -> Result<Vec<u8>, DeltaError> {
        use super::entry::BinaryEntry;
        use super::types::BinaryValue;

        // Convert LnmpValue to BinaryValue
        let binary_value = BinaryValue::from_lnmp_value(value)?;

        // Create a temporary entry to encode the value
        let entry = BinaryEntry::new(0, binary_value);

        // Encode just the value part (skip FID)
        let full_encoding = entry.encode();

        // Skip the FID bytes (2 bytes) and return just the type tag + value
        if full_encoding.len() >= 2 {
            Ok(full_encoding[2..].to_vec())
        } else {
            Err(DeltaError::DeltaApplicationFailed {
                reason: "Invalid value encoding".to_string(),
            })
        }
    }

    /// Encodes nested delta operations
    fn encode_nested_ops(&self, ops: &[DeltaOp]) -> Result<Vec<u8>, DeltaError> {
        use super::varint;

        let mut result = Vec::new();

        // Encode operation count
        let count_bytes = varint::encode(ops.len() as i64);
        result.extend_from_slice(&count_bytes);

        // Encode each operation
        for op in ops {
            // Encode FID
            let fid_bytes = varint::encode(op.target_fid as i64);
            result.extend_from_slice(&fid_bytes);

            // Encode operation code
            result.push(op.operation.to_u8());

            // Encode payload length
            let payload_len_bytes = varint::encode(op.payload.len() as i64);
            result.extend_from_slice(&payload_len_bytes);

            // Encode payload
            result.extend_from_slice(&op.payload);
        }

        Ok(result)
    }

    /// Encodes delta operations to binary format
    ///
    /// # Arguments
    ///
    /// * `ops` - The delta operations to encode
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the encoded delta packet
    pub fn encode_delta(&self, ops: &[DeltaOp]) -> Result<Vec<u8>, DeltaError> {
        use super::varint;

        let mut result = Vec::new();

        // Write DELTA_TAG
        result.push(DELTA_TAG);

        // Encode operation count
        let count_bytes = varint::encode(ops.len() as i64);
        result.extend_from_slice(&count_bytes);

        // Encode each operation
        for op in ops {
            // Encode TARGET_FID
            let fid_bytes = varint::encode(op.target_fid as i64);
            result.extend_from_slice(&fid_bytes);

            // Encode OP_CODE
            result.push(op.operation.to_u8());

            // Encode payload length
            let payload_len_bytes = varint::encode(op.payload.len() as i64);
            result.extend_from_slice(&payload_len_bytes);

            // Encode payload
            result.extend_from_slice(&op.payload);
        }

        Ok(result)
    }
}

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta decoder for parsing and applying delta operations
pub struct DeltaDecoder {
    config: DeltaConfig,
}

impl DeltaDecoder {
    /// Creates a new delta decoder with default configuration
    pub fn new() -> Self {
        Self {
            config: DeltaConfig::default(),
        }
    }

    /// Creates a delta decoder with custom configuration
    pub fn with_config(config: DeltaConfig) -> Self {
        Self { config }
    }

    /// Decodes delta operations from binary format
    ///
    /// # Arguments
    ///
    /// * `bytes` - The encoded delta packet
    ///
    /// # Returns
    ///
    /// A vector of delta operations
    ///
    /// # Errors
    ///
    /// Returns `DeltaError` if the packet is malformed
    pub fn decode_delta(&self, bytes: &[u8]) -> Result<Vec<DeltaOp>, DeltaError> {
        if !self.config.enable_delta {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: "Delta is disabled in configuration".to_string(),
            });
        }
        use super::varint;

        if bytes.is_empty() {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: "Empty delta packet".to_string(),
            });
        }

        let mut offset = 0;

        // Read DELTA_TAG
        if bytes[offset] != DELTA_TAG {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: format!(
                    "Invalid delta tag: expected 0xB0, found 0x{:02X}",
                    bytes[offset]
                ),
            });
        }
        offset += 1;

        // Read operation count
        let (count, consumed) = varint::decode(&bytes[offset..])?;
        offset += consumed;

        if count < 0 {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: format!("Negative operation count: {}", count),
            });
        }

        let count = count as usize;
        let mut ops = Vec::with_capacity(count);

        // Decode each operation
        for _ in 0..count {
            // Read TARGET_FID
            let (fid, consumed) = varint::decode(&bytes[offset..])?;
            offset += consumed;

            if fid < 0 || fid > u16::MAX as i64 {
                return Err(DeltaError::InvalidTargetFid { fid: fid as u16 });
            }
            let target_fid = fid as u16;

            // Read OP_CODE
            if offset >= bytes.len() {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: "Unexpected end of delta packet".to_string(),
                })?;
            }
            let operation = DeltaOperation::from_u8(bytes[offset])?;
            offset += 1;

            // Read payload length
            let (payload_len, consumed) = varint::decode(&bytes[offset..])?;
            offset += consumed;

            if payload_len < 0 {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: format!("Negative payload length: {}", payload_len),
                });
            }

            let payload_len = payload_len as usize;
            if offset + payload_len > bytes.len() {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: "Payload length exceeds available data".to_string(),
                })?;
            }

            // Read payload
            let payload = bytes[offset..offset + payload_len].to_vec();
            offset += payload_len;

            ops.push(DeltaOp::new(target_fid, operation, payload));
        }

        Ok(ops)
    }

    /// Applies delta operations to a base record
    ///
    /// # Arguments
    ///
    /// * `base` - The base record to apply operations to (modified in place)
    /// * `ops` - The delta operations to apply
    ///
    /// # Errors
    ///
    /// Returns `DeltaError` if:
    /// - A target FID is invalid
    /// - An operation cannot be applied
    /// - A merge conflict occurs
    pub fn apply_delta(&self, base: &mut LnmpRecord, ops: &[DeltaOp]) -> Result<(), DeltaError> {
        if !self.config.enable_delta {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: "Delta is disabled in configuration".to_string(),
            });
        }
        use lnmp_core::LnmpField;

        for op in ops {
            // Validate target FID exists for operations that require it
            match op.operation {
                DeltaOperation::UpdateField | DeltaOperation::MergeRecord => {
                    if base.get_field(op.target_fid).is_none() {
                        return Err(DeltaError::InvalidTargetFid { fid: op.target_fid });
                    }
                }
                _ => {}
            }

            match op.operation {
                DeltaOperation::SetField => {
                    // Decode value from payload and set field
                    let value = self.decode_value(&op.payload)?;
                    // Remove existing field if present, then add new one
                    base.remove_field(op.target_fid);
                    base.add_field(LnmpField {
                        fid: op.target_fid,
                        value,
                    });
                }
                DeltaOperation::DeleteField => {
                    // Remove field from record
                    base.remove_field(op.target_fid);
                }
                DeltaOperation::UpdateField => {
                    // Decode value from payload and update field
                    let value = self.decode_value(&op.payload)?;
                    // Remove existing field, then add updated one
                    base.remove_field(op.target_fid);
                    base.add_field(LnmpField {
                        fid: op.target_fid,
                        value,
                    });
                }
                DeltaOperation::MergeRecord => {
                    // Get existing nested record
                    let existing_field = base
                        .get_field(op.target_fid)
                        .ok_or(DeltaError::InvalidTargetFid { fid: op.target_fid })?;

                    match &existing_field.value {
                        LnmpValue::NestedRecord(existing_rec) => {
                            // Decode nested operations
                            let nested_ops = self.decode_nested_ops(&op.payload)?;

                            // Apply nested operations to a mutable copy
                            let mut updated_rec = (**existing_rec).clone();
                            self.apply_delta(&mut updated_rec, &nested_ops)?;

                            // Remove old field and add updated one
                            base.remove_field(op.target_fid);
                            base.add_field(LnmpField {
                                fid: op.target_fid,
                                value: LnmpValue::NestedRecord(Box::new(updated_rec)),
                            });
                        }
                        _ => {
                            return Err(DeltaError::MergeConflict {
                                fid: op.target_fid,
                                reason: "Target field is not a nested record".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Decodes a value from payload bytes
    fn decode_value(&self, payload: &[u8]) -> Result<LnmpValue, DeltaError> {
        use super::entry::BinaryEntry;

        if payload.is_empty() {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: "Empty value payload".to_string(),
            });
        }

        // Payload format: TYPE_TAG + VALUE_DATA
        // We need to reconstruct a full entry to decode it
        // Prepend a dummy FID (0x00, 0x00) to make it a valid entry
        let mut entry_bytes = vec![0x00, 0x00]; // Dummy FID
        entry_bytes.extend_from_slice(payload);

        let (entry, _) = BinaryEntry::decode(&entry_bytes)?;
        Ok(entry.value.to_lnmp_value())
    }

    /// Decodes nested delta operations from payload
    fn decode_nested_ops(&self, payload: &[u8]) -> Result<Vec<DeltaOp>, DeltaError> {
        use super::varint;

        let mut offset = 0;

        // Read operation count
        let (count, consumed) = varint::decode(&payload[offset..])?;
        offset += consumed;

        if count < 0 {
            return Err(DeltaError::DeltaApplicationFailed {
                reason: format!("Negative nested operation count: {}", count),
            });
        }

        let count = count as usize;
        let mut ops = Vec::with_capacity(count);

        // Decode each operation
        for _ in 0..count {
            // Read FID
            let (fid, consumed) = varint::decode(&payload[offset..])?;
            offset += consumed;

            if fid < 0 || fid > u16::MAX as i64 {
                return Err(DeltaError::InvalidTargetFid { fid: fid as u16 });
            }
            let target_fid = fid as u16;

            // Read operation code
            if offset >= payload.len() {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: "Unexpected end of nested operations".to_string(),
                })?;
            }
            let operation = DeltaOperation::from_u8(payload[offset])?;
            offset += 1;

            // Read payload length
            let (payload_len, consumed) = varint::decode(&payload[offset..])?;
            offset += consumed;

            if payload_len < 0 {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: format!("Negative nested payload length: {}", payload_len),
                });
            }

            let payload_len = payload_len as usize;
            if offset + payload_len > payload.len() {
                return Err(DeltaError::DeltaApplicationFailed {
                    reason: "Nested payload length exceeds available data".to_string(),
                })?;
            }

            // Read payload
            let op_payload = payload[offset..offset + payload_len].to_vec();
            offset += payload_len;

            ops.push(DeltaOp::new(target_fid, operation, op_payload));
        }

        Ok(ops)
    }
}

impl Default for DeltaDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_operation_from_u8() {
        assert_eq!(
            DeltaOperation::from_u8(0x01).unwrap(),
            DeltaOperation::SetField
        );
        assert_eq!(
            DeltaOperation::from_u8(0x02).unwrap(),
            DeltaOperation::DeleteField
        );
        assert_eq!(
            DeltaOperation::from_u8(0x03).unwrap(),
            DeltaOperation::UpdateField
        );
        assert_eq!(
            DeltaOperation::from_u8(0x04).unwrap(),
            DeltaOperation::MergeRecord
        );
    }

    #[test]
    fn test_delta_operation_from_u8_invalid() {
        assert!(DeltaOperation::from_u8(0x00).is_err());
        assert!(DeltaOperation::from_u8(0x05).is_err());
        assert!(DeltaOperation::from_u8(0xFF).is_err());
    }

    #[test]
    fn test_delta_operation_to_u8() {
        assert_eq!(DeltaOperation::SetField.to_u8(), 0x01);
        assert_eq!(DeltaOperation::DeleteField.to_u8(), 0x02);
        assert_eq!(DeltaOperation::UpdateField.to_u8(), 0x03);
        assert_eq!(DeltaOperation::MergeRecord.to_u8(), 0x04);
    }

    #[test]
    fn test_delta_operation_round_trip() {
        let ops = vec![
            DeltaOperation::SetField,
            DeltaOperation::DeleteField,
            DeltaOperation::UpdateField,
            DeltaOperation::MergeRecord,
        ];

        for op in ops {
            let byte = op.to_u8();
            let parsed = DeltaOperation::from_u8(byte).unwrap();
            assert_eq!(parsed, op);
        }
    }

    #[test]
    fn test_delta_tag_constant() {
        assert_eq!(DELTA_TAG, 0xB0);
    }

    #[test]
    fn test_delta_op_new() {
        let op = DeltaOp::new(12, DeltaOperation::SetField, vec![0x01, 0x02, 0x03]);
        assert_eq!(op.target_fid, 12);
        assert_eq!(op.operation, DeltaOperation::SetField);
        assert_eq!(op.payload, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_delta_config_default() {
        let config = DeltaConfig::new();
        assert!(!config.enable_delta);
        assert!(!config.track_changes);
    }

    #[test]
    fn test_delta_config_with_enable_delta() {
        let config = DeltaConfig::new().with_enable_delta(true);
        assert!(config.enable_delta);
        assert!(!config.track_changes);
    }

    #[test]
    fn test_delta_config_with_track_changes() {
        let config = DeltaConfig::new().with_track_changes(true);
        assert!(!config.enable_delta);
        assert!(config.track_changes);
    }

    #[test]
    fn test_delta_config_builder() {
        let config = DeltaConfig::new()
            .with_enable_delta(true)
            .with_track_changes(true);
        assert!(config.enable_delta);
        assert!(config.track_changes);
    }

    #[test]
    fn test_compute_delta_with_enable_flag() {
        use lnmp_core::{LnmpField, LnmpValue};

        let mut base = LnmpRecord::new();
        base.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        base.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("a".to_string()),
        });

        let mut updated = base.clone();
        updated.remove_field(1);
        updated.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(2),
        });

        let config = DeltaConfig::new().with_enable_delta(true);
        let encoder = DeltaEncoder::with_config(config);
        let ops = encoder.compute_delta(&base, &updated).unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].target_fid, 1);
        assert_eq!(ops[0].operation, DeltaOperation::UpdateField);
    }

    #[test]
    fn test_delta_error_display_invalid_target_fid() {
        let err = DeltaError::InvalidTargetFid { fid: 999 };
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid target FID"));
        assert!(msg.contains("999"));
    }

    #[test]
    fn test_delta_error_display_invalid_operation() {
        let err = DeltaError::InvalidOperation { op_code: 0xFF };
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid operation code"));
        assert!(msg.contains("0xFF"));
    }

    #[test]
    fn test_delta_error_display_merge_conflict() {
        let err = DeltaError::MergeConflict {
            fid: 42,
            reason: "Type mismatch".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Merge conflict"));
        assert!(msg.contains("42"));
        assert!(msg.contains("Type mismatch"));
    }

    #[test]
    fn test_delta_error_display_application_failed() {
        let err = DeltaError::DeltaApplicationFailed {
            reason: "Field not found".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Delta application failed"));
        assert!(msg.contains("Field not found"));
    }
}
