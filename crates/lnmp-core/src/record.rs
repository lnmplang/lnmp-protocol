//! Record and field structures for LNMP data.

use crate::{FieldId, LnmpValue};

/// A single field assignment (field ID + value pair)
#[derive(Debug, Clone, PartialEq)]
pub struct LnmpField {
    /// Field identifier
    pub fid: FieldId,
    /// Field value
    pub value: LnmpValue,
}

/// A complete LNMP record (collection of fields)
#[derive(Debug, Clone, PartialEq, Default)]
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

    /// Validates this record against structural limits (depth, field counts, lengths).
    pub fn validate_with_limits(
        &self,
        limits: &crate::limits::StructuralLimits,
    ) -> Result<(), crate::limits::StructuralError> {
        limits.validate_record(self)
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

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
}
