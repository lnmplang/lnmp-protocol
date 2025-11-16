//! Core type definitions for LNMP values and field identifiers.

use crate::LnmpRecord;

/// Field identifier type (0-65535)
pub type FieldId = u16;

/// Supported value types in LNMP v0.3
#[derive(Debug, Clone, PartialEq)]
pub enum LnmpValue {
    /// Integer value (i64)
    Int(i64),
    /// Floating-point value (f64)
    Float(f64),
    /// Boolean value (true/false)
    Bool(bool),
    /// String value
    String(String),
    /// Array of strings
    StringArray(Vec<String>),
    /// Nested record (v0.3)
    NestedRecord(Box<LnmpRecord>),
    /// Array of nested records (v0.3)
    NestedArray(Vec<LnmpRecord>),
}

impl LnmpValue {
    /// Returns the depth of nesting (0 for primitive values)
    pub fn depth(&self) -> usize {
        match self {
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::String(_)
            | LnmpValue::StringArray(_) => 0,
            LnmpValue::NestedRecord(record) => {
                1 + record
                    .fields()
                    .iter()
                    .map(|field| field.value.depth())
                    .max()
                    .unwrap_or(0)
            }
            LnmpValue::NestedArray(records) => {
                1 + records
                    .iter()
                    .flat_map(|record| record.fields().iter().map(|field| field.value.depth()))
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// Validates the structural integrity of the value
    pub fn validate_structure(&self) -> Result<(), String> {
        match self {
            // Primitive types are always valid
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::String(_)
            | LnmpValue::StringArray(_) => Ok(()),
            
            // Validate nested record
            LnmpValue::NestedRecord(record) => {
                for field in record.fields() {
                    field.value.validate_structure()?;
                }
                Ok(())
            }
            
            // Validate nested array
            LnmpValue::NestedArray(records) => {
                for record in records {
                    for field in record.fields() {
                        field.value.validate_structure()?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// Type hint for field values (LNMP v0.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeHint {
    /// Integer type hint (:i)
    Int,
    /// Float type hint (:f)
    Float,
    /// Boolean type hint (:b)
    Bool,
    /// String type hint (:s)
    String,
    /// String array type hint (:sa)
    StringArray,
    /// Record type hint (:r) - v0.3
    Record,
    /// Record array type hint (:ra) - v0.3
    RecordArray,
}

impl TypeHint {
    /// Returns the string representation of the type hint
    pub fn as_str(&self) -> &'static str {
        match self {
            TypeHint::Int => "i",
            TypeHint::Float => "f",
            TypeHint::Bool => "b",
            TypeHint::String => "s",
            TypeHint::StringArray => "sa",
            TypeHint::Record => "r",
            TypeHint::RecordArray => "ra",
        }
    }

    /// Parses a type hint from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "i" => Some(TypeHint::Int),
            "f" => Some(TypeHint::Float),
            "b" => Some(TypeHint::Bool),
            "s" => Some(TypeHint::String),
            "sa" => Some(TypeHint::StringArray),
            "r" => Some(TypeHint::Record),
            "ra" => Some(TypeHint::RecordArray),
            _ => None,
        }
    }

    /// Validates that a value matches this type hint
    pub fn validates(&self, value: &LnmpValue) -> bool {
        match (self, value) {
            (TypeHint::Int, LnmpValue::Int(_)) => true,
            (TypeHint::Float, LnmpValue::Float(_)) => true,
            (TypeHint::Bool, LnmpValue::Bool(_)) => true,
            (TypeHint::String, LnmpValue::String(_)) => true,
            (TypeHint::StringArray, LnmpValue::StringArray(_)) => true,
            (TypeHint::Record, LnmpValue::NestedRecord(_)) => true,
            (TypeHint::RecordArray, LnmpValue::NestedArray(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_hint_as_str() {
        assert_eq!(TypeHint::Int.as_str(), "i");
        assert_eq!(TypeHint::Float.as_str(), "f");
        assert_eq!(TypeHint::Bool.as_str(), "b");
        assert_eq!(TypeHint::String.as_str(), "s");
        assert_eq!(TypeHint::StringArray.as_str(), "sa");
        assert_eq!(TypeHint::Record.as_str(), "r");
        assert_eq!(TypeHint::RecordArray.as_str(), "ra");
    }

    #[test]
    fn test_type_hint_from_str() {
        assert_eq!(TypeHint::from_str("i"), Some(TypeHint::Int));
        assert_eq!(TypeHint::from_str("f"), Some(TypeHint::Float));
        assert_eq!(TypeHint::from_str("b"), Some(TypeHint::Bool));
        assert_eq!(TypeHint::from_str("s"), Some(TypeHint::String));
        assert_eq!(TypeHint::from_str("sa"), Some(TypeHint::StringArray));
        assert_eq!(TypeHint::from_str("r"), Some(TypeHint::Record));
        assert_eq!(TypeHint::from_str("ra"), Some(TypeHint::RecordArray));
        assert_eq!(TypeHint::from_str("invalid"), None);
        assert_eq!(TypeHint::from_str(""), None);
    }

    #[test]
    fn test_type_hint_validates_int() {
        let hint = TypeHint::Int;
        assert!(hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::Float(3.14)));
        assert!(!hint.validates(&LnmpValue::Bool(true)));
        assert!(!hint.validates(&LnmpValue::String("test".to_string())));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
        
        let nested_record = LnmpValue::NestedRecord(Box::new(crate::LnmpRecord::new()));
        assert!(!hint.validates(&nested_record));
        
        let nested_array = LnmpValue::NestedArray(vec![]);
        assert!(!hint.validates(&nested_array));
    }

    #[test]
    fn test_type_hint_validates_float() {
        let hint = TypeHint::Float;
        assert!(hint.validates(&LnmpValue::Float(3.14)));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::Bool(true)));
        assert!(!hint.validates(&LnmpValue::String("test".to_string())));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
    }

    #[test]
    fn test_type_hint_validates_bool() {
        let hint = TypeHint::Bool;
        assert!(hint.validates(&LnmpValue::Bool(true)));
        assert!(hint.validates(&LnmpValue::Bool(false)));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::Float(3.14)));
        assert!(!hint.validates(&LnmpValue::String("test".to_string())));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
    }

    #[test]
    fn test_type_hint_validates_string() {
        let hint = TypeHint::String;
        assert!(hint.validates(&LnmpValue::String("test".to_string())));
        assert!(hint.validates(&LnmpValue::String("".to_string())));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::Float(3.14)));
        assert!(!hint.validates(&LnmpValue::Bool(true)));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
    }

    #[test]
    fn test_type_hint_validates_string_array() {
        let hint = TypeHint::StringArray;
        assert!(hint.validates(&LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()])));
        assert!(hint.validates(&LnmpValue::StringArray(vec![])));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::Float(3.14)));
        assert!(!hint.validates(&LnmpValue::Bool(true)));
        assert!(!hint.validates(&LnmpValue::String("test".to_string())));
    }

    #[test]
    fn test_type_hint_round_trip() {
        let hints = vec![
            TypeHint::Int,
            TypeHint::Float,
            TypeHint::Bool,
            TypeHint::String,
            TypeHint::StringArray,
            TypeHint::Record,
            TypeHint::RecordArray,
        ];

        for hint in hints {
            let str_repr = hint.as_str();
            let parsed = TypeHint::from_str(str_repr);
            assert_eq!(parsed, Some(hint));
        }
    }

    #[test]
    fn test_type_hint_validates_record() {
        use crate::{LnmpField, LnmpRecord};
        
        let hint = TypeHint::Record;
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        
        let nested_record = LnmpValue::NestedRecord(Box::new(record));
        assert!(hint.validates(&nested_record));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
    }

    #[test]
    fn test_type_hint_validates_record_array() {
        use crate::{LnmpField, LnmpRecord};
        
        let hint = TypeHint::RecordArray;
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(1),
        });
        
        let nested_array = LnmpValue::NestedArray(vec![record]);
        assert!(hint.validates(&nested_array));
        assert!(!hint.validates(&LnmpValue::Int(42)));
        assert!(!hint.validates(&LnmpValue::StringArray(vec![])));
    }

    #[test]
    fn test_depth_primitive_values() {
        assert_eq!(LnmpValue::Int(42).depth(), 0);
        assert_eq!(LnmpValue::Float(3.14).depth(), 0);
        assert_eq!(LnmpValue::Bool(true).depth(), 0);
        assert_eq!(LnmpValue::String("test".to_string()).depth(), 0);
        assert_eq!(LnmpValue::StringArray(vec!["a".to_string()]).depth(), 0);
    }

    #[test]
    fn test_depth_nested_record() {
        use crate::{LnmpField, LnmpRecord};
        
        let mut inner_record = LnmpRecord::new();
        inner_record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        
        let nested = LnmpValue::NestedRecord(Box::new(inner_record));
        assert_eq!(nested.depth(), 1);
    }

    #[test]
    fn test_depth_deeply_nested_record() {
        use crate::{LnmpField, LnmpRecord};
        
        // Create a 3-level nested structure
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
        
        let nested = LnmpValue::NestedRecord(Box::new(level1));
        assert_eq!(nested.depth(), 3);
    }

    #[test]
    fn test_depth_nested_array() {
        use crate::{LnmpField, LnmpRecord};
        
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
        
        let nested_array = LnmpValue::NestedArray(vec![record1, record2]);
        assert_eq!(nested_array.depth(), 1);
    }

    #[test]
    fn test_depth_empty_nested_structures() {
        use crate::LnmpRecord;
        
        let empty_record = LnmpValue::NestedRecord(Box::new(LnmpRecord::new()));
        assert_eq!(empty_record.depth(), 1);
        
        let empty_array = LnmpValue::NestedArray(vec![]);
        assert_eq!(empty_array.depth(), 1);
    }

    #[test]
    fn test_validate_structure_primitive_values() {
        assert!(LnmpValue::Int(42).validate_structure().is_ok());
        assert!(LnmpValue::Float(3.14).validate_structure().is_ok());
        assert!(LnmpValue::Bool(true).validate_structure().is_ok());
        assert!(LnmpValue::String("test".to_string()).validate_structure().is_ok());
        assert!(LnmpValue::StringArray(vec!["a".to_string()]).validate_structure().is_ok());
    }

    #[test]
    fn test_validate_structure_nested_record() {
        use crate::{LnmpField, LnmpRecord};
        
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("test".to_string()),
        });
        
        let nested = LnmpValue::NestedRecord(Box::new(record));
        assert!(nested.validate_structure().is_ok());
    }

    #[test]
    fn test_validate_structure_nested_array() {
        use crate::{LnmpField, LnmpRecord};
        
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
        
        let nested_array = LnmpValue::NestedArray(vec![record1, record2]);
        assert!(nested_array.validate_structure().is_ok());
    }

    #[test]
    fn test_validate_structure_deeply_nested() {
        use crate::{LnmpField, LnmpRecord};
        
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
        
        let nested = LnmpValue::NestedRecord(Box::new(outer));
        assert!(nested.validate_structure().is_ok());
    }
}
