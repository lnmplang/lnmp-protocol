//! Builder pattern for constructing canonically-ordered records
//!
//! The `RecordBuilder` provides a fluent API for building `LnmpRecord` instances
//! with guaranteed canonical field ordering (sorted by FID).
//!
//! # Example
//!
//! ```
//! use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};
//!
//! let record = RecordBuilder::new()
//!     .add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) })
//!     .add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) })
//!     .add_field(LnmpField { fid: 12, value: LnmpValue::Int(2) })
//!     .build();
//!
//! // Fields are automatically sorted by FID
//! assert_eq!(record.fields()[0].fid, 7);
//! assert_eq!(record.fields()[1].fid, 12);
//! assert_eq!(record.fields()[2].fid, 23);
//! ```

use crate::{LnmpField, LnmpRecord};

/// Builder for creating LnmpRecords with guaranteed canonical field ordering
///
/// This builder accumulates fields and sorts them by FID before creating
/// the final record, ensuring canonical representation regardless of the
/// order in which fields are added.
#[derive(Debug, Clone, Default)]
pub struct RecordBuilder {
    fields: Vec<LnmpField>,
}

impl RecordBuilder {
    /// Creates a new empty builder
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::RecordBuilder;
    ///
    /// let builder = RecordBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field to the builder
    ///
    /// Fields will be automatically sorted by FID when `build()` is called.
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};
    ///
    /// let record = RecordBuilder::new()
    ///     .add_field(LnmpField { fid: 10, value: LnmpValue::Int(1) })
    ///     .build();
    /// ```
    pub fn add_field(mut self, field: LnmpField) -> Self {
        self.fields.push(field);
        self
    }

    /// Adds multiple fields at once
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};
    ///
    /// let fields = vec![
    ///     LnmpField { fid: 10, value: LnmpValue::Int(1) },
    ///     LnmpField { fid: 20, value: LnmpValue::Int(2) },
    /// ];
    ///
    /// let record = RecordBuilder::new()
    ///     .add_fields(fields)
    ///     .build();
    /// ```
    pub fn add_fields(mut self, fields: impl IntoIterator<Item = LnmpField>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Builds the record with fields sorted by FID
    ///
    /// This consumes the builder and returns an `LnmpRecord` with fields
    /// in canonical order (sorted by field ID).
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::{RecordBuilder, LnmpField, LnmpValue};
    ///
    /// let record = RecordBuilder::new()
    ///     .add_field(LnmpField { fid: 23, value: LnmpValue::Int(3) })
    ///     .add_field(LnmpField { fid: 7, value: LnmpValue::Int(1) })
    ///     .build();
    ///
    /// // Fields are sorted
    /// assert_eq!(record.fields()[0].fid, 7);
    /// assert_eq!(record.fields()[1].fid, 23);
    /// ```
    pub fn build(mut self) -> LnmpRecord {
        self.fields.sort_by_key(|f| f.fid);
        LnmpRecord::from_sorted_fields(self.fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LnmpValue;

    #[test]
    fn test_builder_empty() {
        let record = RecordBuilder::new().build();
        assert_eq!(record.fields().len(), 0);
    }

    #[test]
    fn test_builder_single_field() {
        let record = RecordBuilder::new()
            .add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Int(1),
            })
            .build();

        assert_eq!(record.fields().len(), 1);
        assert_eq!(record.fields()[0].fid, 10);
    }

    #[test]
    fn test_builder_sorted_automatically() {
        let record = RecordBuilder::new()
            .add_field(LnmpField {
                fid: 23,
                value: LnmpValue::Int(3),
            })
            .add_field(LnmpField {
                fid: 7,
                value: LnmpValue::Int(1),
            })
            .add_field(LnmpField {
                fid: 12,
                value: LnmpValue::Int(2),
            })
            .build();

        // Fields should be sorted by FID
        assert_eq!(record.fields()[0].fid, 7);
        assert_eq!(record.fields()[1].fid, 12);
        assert_eq!(record.fields()[2].fid, 23);
    }

    #[test]
    fn test_builder_add_fields_batch() {
        let fields = vec![
            LnmpField {
                fid: 20,
                value: LnmpValue::Int(2),
            },
            LnmpField {
                fid: 10,
                value: LnmpValue::Int(1),
            },
        ];

        let record = RecordBuilder::new().add_fields(fields).build();

        // Fields should be sorted
        assert_eq!(record.fields()[0].fid, 10);
        assert_eq!(record.fields()[1].fid, 20);
    }

    #[test]
    fn test_builder_chaining() {
        let record = RecordBuilder::new()
            .add_field(LnmpField {
                fid: 30,
                value: LnmpValue::Int(3),
            })
            .add_fields(vec![
                LnmpField {
                    fid: 10,
                    value: LnmpValue::Int(1),
                },
                LnmpField {
                    fid: 20,
                    value: LnmpValue::Int(2),
                },
            ])
            .build();

        // All fields should be sorted
        assert_eq!(record.fields().len(), 3);
        assert_eq!(record.fields()[0].fid, 10);
        assert_eq!(record.fields()[1].fid, 20);
        assert_eq!(record.fields()[2].fid, 30);
    }
}
