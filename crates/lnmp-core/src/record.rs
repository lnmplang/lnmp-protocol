//! Record and field structures for LNMP data.
//!
//! ## Field Ordering
//!
//! `LnmpRecord` stores fields internally in a `Vec`, maintaining **insertion order**.
//! However, for deterministic behavior and canonical representation, use `sorted_fields()`:
//!
//! ```
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) });
//! record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });
//! record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) });
//!
//! // Insertion order (non-deterministic across different constructions)
//! assert_eq!(record.fields()[0].fid, 23);
//! assert_eq!(record.fields()[1].fid, 7);
//! assert_eq!(record.fields()[2].fid, 12);
//!
//! // Canonical order (deterministic, sorted by FID)
//! let sorted = record.sorted_fields();
//! assert_eq!(sorted[0].fid, 7);
//! assert_eq!(sorted[1].fid, 12);
//! assert_eq!(sorted[2].fid, 23);
//! ```
//!
//! ## When to Use Each
//!
//! - **`fields()`**: When insertion order is semantically important
//!   - Direct iteration over fields as added
//!   - Structural equality (order-sensitive)
//!
//! - **`sorted_fields()`**: For canonical representation
//!   - Encoding (text/binary)
//!   - Checksum computation
//!   - Semantic comparison (order-independent)
//!   - Deterministic output
//!
//! ## Deterministic Guarantees
//!
//! The following components **always use `sorted_fields()`** for determinism:
//! - `SemanticChecksum::serialize_value()` - Field-order-independent checksums
//! - All encoders in `lnmp-codec` - Canonical output
//! - Binary format - Sorted fields for stable round-trips
//!
//! This ensures that two records with the same fields but different insertion
//! orders will produce identical checksums and encodings.

use crate::{FieldId, LnmpValue};

/// A single field assignment (field ID + value pair)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LnmpField {
    /// Field identifier
    pub fid: FieldId,
    /// Field value
    pub value: LnmpValue,
}

/// A complete LNMP record (collection of fields)
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LnmpRecord {
    fields: Vec<LnmpField>,
}

impl LnmpRecord {
    /// Creates a new empty record
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field to the record
    pub fn add_field(&mut self, field: LnmpField) {
        self.fields.push(field);
    }

    /// Gets a field by field ID (returns the first match if duplicates exist)
    pub fn get_field(&self, fid: FieldId) -> Option<&LnmpField> {
        self.fields.iter().find(|f| f.fid == fid)
    }

    /// Removes all fields with the given field ID
    pub fn remove_field(&mut self, fid: FieldId) {
        self.fields.retain(|f| f.fid != fid);
    }

    /// Returns a slice of all fields in the record
    pub fn fields(&self) -> &[LnmpField] {
        &self.fields
    }

    /// Consumes the record and returns the fields vector
    pub fn into_fields(self) -> Vec<LnmpField> {
        self.fields
    }

    /// Returns fields sorted by field ID (stable sort preserves insertion order for duplicates)
    pub fn sorted_fields(&self) -> Vec<LnmpField> {
        let mut sorted = self.fields.clone();
        sorted.sort_by_key(|f| f.fid);
        sorted
    }

    /// Creates a record from a vector of fields (typically already sorted)
    pub fn from_sorted_fields(fields: Vec<LnmpField>) -> Self {
        Self { fields }
    }

    /// Creates a record from fields, automatically sorting by FID
    ///
    /// This ensures canonical field ordering regardless of input order.
    /// Use this constructor when building records from unsorted field collections.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    ///
    /// let fields = vec![
    ///     LnmpField { fid: 23, value: LnmpValue::Int(3) },
    ///     LnmpField { fid: 7, value: LnmpValue::Int(1) },
    ///     LnmpField { fid: 12, value: LnmpValue::Int(2) },
    /// ];
    ///
    /// let record = LnmpRecord::from_fields(fields);
    ///
    /// // Fields are automatically sorted by FID
    /// assert_eq!(record.fields()[0].fid, 7);
    /// assert_eq!(record.fields()[1].fid, 12);
    /// assert_eq!(record.fields()[2].fid, 23);
    /// ```
    pub fn from_fields(mut fields: Vec<LnmpField>) -> Self {
        fields.sort_by_key(|f| f.fid);
        Self::from_sorted_fields(fields)
    }

    /// Validates this record against structural limits (depth, field counts, lengths).
    pub fn validate_with_limits(
        &self,
        limits: &crate::limits::StructuralLimits,
    ) -> Result<(), crate::limits::StructuralError> {
        limits.validate_record(self)
    }

    /// Compares two records based on canonical form (field order independent).
    ///
    /// This method compares records semantically by comparing their sorted fields.
    /// Two records are canonically equal if they have the same fields (same FID and value),
    /// regardless of the order in which fields were added.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    ///
    /// let mut rec1 = LnmpRecord::new();
    /// rec1.add_field(LnmpField { fid: 12, value: LnmpValue::Int(1) });
    /// rec1.add_field(LnmpField { fid: 7, value: LnmpValue::Int(2) });
    ///
    /// let mut rec2 = LnmpRecord::new();
    /// rec2.add_field(LnmpField { fid: 7, value: LnmpValue::Int(2) });
    /// rec2.add_field(LnmpField { fid: 12, value: LnmpValue::Int(1) });
    ///
    /// // Structural equality: order matters
    /// assert_ne!(rec1, rec2);
    ///
    /// // Canonical equality: order doesn't matter
    /// assert!(rec1.canonical_eq(&rec2));
    /// ```
    pub fn canonical_eq(&self, other: &Self) -> bool {
        self.sorted_fields() == other.sorted_fields()
    }

    /// Computes a hash based on canonical field ordering.
    ///
    /// This method computes a hash using sorted fields, ensuring that
    /// the hash is independent of field insertion order. This is useful
    /// for using `LnmpRecord` in `HashMap` or `HashSet` with semantic equality.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    /// use std::hash::{Hash, Hasher};
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// let mut rec1 = LnmpRecord::new();
    /// rec1.add_field(LnmpField { fid: 12, value: LnmpValue::Int(1) });
    /// rec1.add_field(LnmpField { fid: 7, value: LnmpValue::Int(2) });
    ///
    /// let mut rec2 = LnmpRecord::new();
    /// rec2.add_field(LnmpField { fid: 7, value: LnmpValue::Int(2) });
    /// rec2.add_field(LnmpField { fid: 12, value: LnmpValue::Int(1) });
    ///
    /// let mut hasher1 = DefaultHasher::new();
    /// rec1.canonical_hash(&mut hasher1);
    /// let hash1 = hasher1.finish();
    ///
    /// let mut hasher2 = DefaultHasher::new();
    /// rec2.canonical_hash(&mut hasher2);
    /// let hash2 = hasher2.finish();
    ///
    /// // Same canonical hash despite different insertion order
    /// assert_eq!(hash1, hash2);
    /// ```
    pub fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;

        // Hash the sorted fields
        for field in self.sorted_fields() {
            field.fid.hash(state);

            // Hash the value based on its type
            match &field.value {
                LnmpValue::Int(i) => {
                    0u8.hash(state); // Discriminant
                    i.hash(state);
                }
                LnmpValue::Float(f) => {
                    1u8.hash(state); // Discriminant
                                     // Hash float bits for deterministic hashing
                    f.to_bits().hash(state);
                }
                LnmpValue::Bool(b) => {
                    2u8.hash(state); // Discriminant
                    b.hash(state);
                }
                LnmpValue::String(s) => {
                    3u8.hash(state); // Discriminant
                    s.hash(state);
                }
                LnmpValue::StringArray(arr) => {
                    4u8.hash(state); // Discriminant
                    arr.len().hash(state);
                    for s in arr {
                        s.hash(state);
                    }
                }
                LnmpValue::IntArray(arr) => {
                    10u8.hash(state); // Discriminant
                    arr.len().hash(state);
                    for &i in arr {
                        i.hash(state);
                    }
                }
                LnmpValue::FloatArray(arr) => {
                    11u8.hash(state); // Discriminant
                    arr.len().hash(state);
                    for &f in arr {
                        f.to_bits().hash(state);
                    }
                }
                LnmpValue::BoolArray(arr) => {
                    12u8.hash(state); // Discriminant
                    arr.len().hash(state);
                    for &b in arr {
                        b.hash(state);
                    }
                }
                LnmpValue::NestedRecord(record) => {
                    5u8.hash(state); // Discriminant
                                     // Recursively use canonical hash
                    record.canonical_hash(state);
                }
                LnmpValue::NestedArray(records) => {
                    6u8.hash(state); // Discriminant
                    records.len().hash(state);
                    for rec in records {
                        rec.canonical_hash(state);
                    }
                }
                LnmpValue::Embedding(vec) => {
                    7u8.hash(state); // Discriminant
                                     // Hash the embedding data using public fields
                    vec.dim.hash(state);
                    format!("{:?}", vec.dtype).hash(state); // Hash enum variant
                    vec.data.hash(state);
                }
                LnmpValue::EmbeddingDelta(delta) => {
                    8u8.hash(state); // Discriminant
                                     // Hash delta base_id and changes
                    delta.base_id.hash(state);
                    delta.changes.len().hash(state);
                    for change in &delta.changes {
                        change.index.hash(state);
                        change.delta.to_bits().hash(state);
                    }
                }
                #[cfg(feature = "quant")]
                LnmpValue::QuantizedEmbedding(qv) => {
                    9u8.hash(state); // Discriminant
                                     // Hash quantized data
                    format!("{:?}", qv.scheme).hash(state);
                    qv.scale.to_bits().hash(state);
                    qv.zero_point.hash(state);
                    qv.data.hash(state);
                }
            }
        }
    }

    /// Validates that fields are in canonical order (sorted by FID).
    ///
    /// Returns `Ok(())` if fields are sorted, or an error with details about
    /// the first out-of-order field.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    ///
    /// // Sorted record
    /// let record = LnmpRecord::from_fields(vec![
    ///     LnmpField { fid: 7, value: LnmpValue::Int(1) },
    ///     LnmpField { fid: 12, value: LnmpValue::Int(2) },
    /// ]);
    /// assert!(record.validate_field_ordering().is_ok());
    ///
    /// // Unsorted record
    /// let mut record = LnmpRecord::new();
    /// record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) });
    /// record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });
    /// assert!(record.validate_field_ordering().is_err());
    /// ```
    pub fn validate_field_ordering(&self) -> Result<(), FieldOrderingError> {
        let fields = self.fields();

        for i in 1..fields.len() {
            if fields[i].fid < fields[i - 1].fid {
                return Err(FieldOrderingError {
                    position: i,
                    current_fid: fields[i].fid,
                    previous_fid: fields[i - 1].fid,
                });
            }
        }

        Ok(())
    }

    /// Returns whether fields are in canonical order (sorted by FID).
    ///
    /// This is a convenience method that returns a boolean instead of a Result.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};
    ///
    /// let record = RecordBuilder::new()
    ///     .add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) })
    ///     .build();
    ///
    /// assert!(record.is_canonical_order());
    /// ```
    pub fn is_canonical_order(&self) -> bool {
        self.validate_field_ordering().is_ok()
    }

    /// Returns the number of out-of-order field pairs.
    ///
    /// A count of 0 means the record is in canonical order.
    /// Higher counts indicate more disorder.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    ///
    /// let mut record = LnmpRecord::new();
    /// record.add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) });
    /// record.add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) });
    /// record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) });
    ///
    /// // 23 > 7 (disorder), 7 < 12 (ok), so 1 disorder
    /// assert_eq!(record.count_ordering_violations(), 1);
    /// ```
    pub fn count_ordering_violations(&self) -> usize {
        let fields = self.fields();
        let mut count = 0;

        for i in 1..fields.len() {
            if fields[i].fid < fields[i - 1].fid {
                count += 1;
            }
        }

        count
    }
}

/// Error returned when field ordering validation fails
#[derive(Debug, Clone, PartialEq)]
pub struct FieldOrderingError {
    /// Position (index) where the ordering violation was found
    pub position: usize,
    /// FID of the field at the violation position
    pub current_fid: FieldId,
    /// FID of the previous field (which is greater than current_fid)
    pub previous_fid: FieldId,
}

impl std::fmt::Display for FieldOrderingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Field ordering violation at position {}: F{} appears after F{} (expected ascending FID order)",
            self.position, self.current_fid, self.previous_fid
        )
    }
}

impl std::error::Error for FieldOrderingError {}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_field_ordering_sorted() {
        let record = LnmpRecord::from_fields(vec![
            LnmpField {
                fid: 7,
                value: LnmpValue::Int(1),
            },
            LnmpField {
                fid: 12,
                value: LnmpValue::Int(2),
            },
            LnmpField {
                fid: 23,
                value: LnmpValue::Int(3),
            },
        ]);

        assert!(record.validate_field_ordering().is_ok());
    }

    #[test]
    fn test_validate_field_ordering_unsorted() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });

        let result = record.validate_field_ordering();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.position, 1);
        assert_eq!(err.current_fid, 7);
        assert_eq!(err.previous_fid, 12);
    }

    #[test]
    fn test_validate_field_ordering_empty() {
        let record = LnmpRecord::new();
        assert!(record.validate_field_ordering().is_ok());
    }

    #[test]
    fn test_is_canonical_order() {
        let sorted = LnmpRecord::from_fields(vec![
            LnmpField {
                fid: 1,
                value: LnmpValue::Int(1),
            },
            LnmpField {
                fid: 2,
                value: LnmpValue::Int(2),
            },
        ]);
        assert!(sorted.is_canonical_order());

        let mut unsorted = LnmpRecord::new();
        unsorted.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(2),
        });
        unsorted.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        assert!(!unsorted.is_canonical_order());
    }

    #[test]
    fn test_count_ordering_violations_none() {
        let record = LnmpRecord::from_fields(vec![
            LnmpField {
                fid: 1,
                value: LnmpValue::Int(1),
            },
            LnmpField {
                fid: 2,
                value: LnmpValue::Int(2),
            },
            LnmpField {
                fid: 3,
                value: LnmpValue::Int(3),
            },
        ]);

        assert_eq!(record.count_ordering_violations(), 0);
    }

    #[test]
    fn test_count_ordering_violations_one() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        // 23 > 7 is a violation, then 7 < 12 is ok
        assert_eq!(record.count_ordering_violations(), 1);
    }

    #[test]
    fn test_count_ordering_violations_multiple() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Int(0),
        });

        // 23 > 7 (violation), 7 > 3 (violation)
        assert_eq!(record.count_ordering_violations(), 2);
    }

    #[test]
    fn test_field_ordering_error_display() {
        let error = FieldOrderingError {
            position: 1,
            current_fid: 7,
            previous_fid: 12,
        };

        let msg = error.to_string();
        assert!(msg.contains("position 1"));
        assert!(msg.contains("F7"));
        assert!(msg.contains("F12"));
        assert!(msg.contains("ascending FID order"));
    }

    #[test]
    fn test_new_record_is_empty() {
        let record = LnmpRecord::new();
        assert_eq!(record.fields().len(), 0);
    }

    #[test]
    fn test_add_field() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.fields()[0].fid, 12);
    }

    #[test]
    fn test_get_field() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let field = record.get_field(12);
        assert!(field.is_some());
        assert_eq!(field.unwrap().value, LnmpValue::Int(14532));

        let missing = record.get_field(99);
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_field_with_duplicates() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::String("first".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::String("second".to_string()),
        });

        // Should return the first match
        let field = record.get_field(5);
        assert!(field.is_some());
        assert_eq!(field.unwrap().value, LnmpValue::String("first".to_string()));
    }

    #[test]
    fn test_fields_iteration() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Float(3.14),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Bool(false),
        });

        let fields = record.fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].fid, 1);
        assert_eq!(fields[1].fid, 2);
        assert_eq!(fields[2].fid, 3);
    }

    #[test]
    fn test_into_fields() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String("test".to_string()),
        });

        let fields = record.into_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].fid, 10);
    }

    #[test]
    fn test_lnmp_value_equality() {
        assert_eq!(LnmpValue::Int(42), LnmpValue::Int(42));
        assert_ne!(LnmpValue::Int(42), LnmpValue::Int(43));

        assert_eq!(LnmpValue::Float(3.14), LnmpValue::Float(3.14));

        assert_eq!(LnmpValue::Bool(true), LnmpValue::Bool(true));
        assert_ne!(LnmpValue::Bool(true), LnmpValue::Bool(false));

        assert_eq!(
            LnmpValue::String("hello".to_string()),
            LnmpValue::String("hello".to_string())
        );

        assert_eq!(
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
            LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_lnmp_value_clone() {
        let original = LnmpValue::StringArray(vec!["test".to_string()]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_empty_record() {
        let record = LnmpRecord::new();
        assert_eq!(record.fields().len(), 0);
        assert!(record.get_field(1).is_none());
    }

    #[test]
    fn test_record_with_all_value_types() {
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
            value: LnmpValue::String("hello world".to_string()),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        assert_eq!(record.fields().len(), 5);
        assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(-42));
        assert_eq!(
            record.get_field(2).unwrap().value,
            LnmpValue::Float(3.14159)
        );
        assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(
            record.get_field(4).unwrap().value,
            LnmpValue::String("hello world".to_string())
        );
        assert_eq!(
            record.get_field(5).unwrap().value,
            LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
        );
    }

    #[test]
    fn test_sorted_fields_basic() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let sorted = record.sorted_fields();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].fid, 7);
        assert_eq!(sorted[1].fid, 12);
        assert_eq!(sorted[2].fid, 23);
    }

    #[test]
    fn test_sorted_fields_preserves_duplicate_order() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::String("first".to_string()),
        });
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Int(100),
        });
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::String("second".to_string()),
        });

        let sorted = record.sorted_fields();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].fid, 5);
        assert_eq!(sorted[0].value, LnmpValue::String("first".to_string()));
        assert_eq!(sorted[1].fid, 5);
        assert_eq!(sorted[1].value, LnmpValue::String("second".to_string()));
        assert_eq!(sorted[2].fid, 10);
    }

    #[test]
    fn test_sorted_fields_already_sorted() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(2),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Int(3),
        });

        let sorted = record.sorted_fields();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].fid, 1);
        assert_eq!(sorted[1].fid, 2);
        assert_eq!(sorted[2].fid, 3);
    }

    #[test]
    fn test_sorted_fields_empty_record() {
        let record = LnmpRecord::new();
        let sorted = record.sorted_fields();
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_sorted_fields_does_not_modify_original() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::Int(3),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });

        let _sorted = record.sorted_fields();

        // Original record should remain unchanged
        assert_eq!(record.fields()[0].fid, 23);
        assert_eq!(record.fields()[1].fid, 7);
    }

    #[test]
    fn test_canonical_eq_same_order() {
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        assert_eq!(rec1, rec2); // Structural equality
        assert!(rec1.canonical_eq(&rec2)); // Canonical equality
    }

    #[test]
    fn test_canonical_eq_different_order() {
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        assert_ne!(rec1, rec2); // Structural inequality (different order)
        assert!(rec1.canonical_eq(&rec2)); // Canonical equality (same fields)
    }

    #[test]
    fn test_canonical_eq_different_values() {
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2), // Different value
        });

        assert!(!rec1.canonical_eq(&rec2));
    }

    #[test]
    fn test_canonical_eq_with_nested_records() {
        // Create two nested records with SAME field order
        // (Note: canonical_eq compares sorted fields at top level,
        //  but nested NestedRecord values use PartialEq which is order-sensitive)
        let mut inner1 = LnmpRecord::new();
        inner1.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });
        inner1.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(200),
        });

        let mut inner2 = LnmpRecord::new();
        inner2.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(100),
        });
        inner2.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(200),
        });

        // Create outer records in different field orders
        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner1)),
        });
        rec1.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(999),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(999),
        });
        rec2.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::NestedRecord(Box::new(inner2)),
        });

        // Different field order at top level
        assert_ne!(rec1, rec2);

        // But canonical_eq should work
        assert!(rec1.canonical_eq(&rec2));
    }

    #[test]
    fn test_canonical_hash_same_order() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(2),
        });

        let mut hasher1 = DefaultHasher::new();
        rec1.canonical_hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        rec2.canonical_hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_canonical_hash_different_order() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });
        rec2.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });

        let mut hasher1 = DefaultHasher::new();
        rec1.canonical_hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        rec2.canonical_hash(&mut hasher2);
        let hash2 = hasher2.finish();

        // Same hash despite different insertion order
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_canonical_hash_different_values() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut rec1 = LnmpRecord::new();
        rec1.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(1),
        });

        let mut rec2 = LnmpRecord::new();
        rec2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Int(2),
        });

        let mut hasher1 = DefaultHasher::new();
        rec1.canonical_hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        rec2.canonical_hash(&mut hasher2);
        let hash2 = hasher2.finish();

        // Different hash for different values
        assert_ne!(hash1, hash2);
    }
}
