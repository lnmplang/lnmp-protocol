//! Core type definitions for LNMP values and field identifiers.

use crate::LnmpRecord;
use std::str::FromStr;

/// Field identifier type (0-65535)
pub type FieldId = u16;

/// LNMP value types supporting all primitives, arrays, and nested structures.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Array of integers (v0.6)
    IntArray(Vec<i64>),
    /// Array of floats (v0.6)
    FloatArray(Vec<f64>),
    /// Array of booleans (v0.6)
    BoolArray(Vec<bool>),
    /// Nested record (v0.3)
    NestedRecord(Box<LnmpRecord>),
    /// Array of nested records (v0.3)
    NestedArray(Vec<LnmpRecord>),
    /// Vector Embedding (v0.5)
    Embedding(lnmp_embedding::Vector),
    /// Delta update for embedding vector
    EmbeddingDelta(lnmp_embedding::VectorDelta),
    /// Quantized embedding vector (v0.5.2)
    #[cfg(feature = "quant")]
    QuantizedEmbedding(lnmp_quant::QuantizedVector),
}

impl LnmpValue {
    /// Returns the depth of nesting (0 for primitive values)
    pub fn depth(&self) -> usize {
        match self {
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::String(_)
            | LnmpValue::StringArray(_)
            | LnmpValue::IntArray(_)
            | LnmpValue::FloatArray(_)
            | LnmpValue::BoolArray(_)
            | LnmpValue::Embedding(_)
            | LnmpValue::EmbeddingDelta(_) => 0,
            #[cfg(feature = "quant")]
            LnmpValue::QuantizedEmbedding(_) => 0,
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

    /// Validates the structural integrity of the value without imposing limits.
    ///
    /// This uses an iterative walk to avoid deep-recursion stack overflows.
    pub fn validate_structure(&self) -> Result<(), String> {
        self.validate_with_max_depth(usize::MAX)
    }

    /// Validates structure while enforcing a maximum nesting depth.
    ///
    /// Returns an error string if the structure contains invalid nested values
    /// or if `max_depth` is exceeded.
    pub fn validate_with_max_depth(&self, max_depth: usize) -> Result<(), String> {
        let mut stack: Vec<(usize, &LnmpValue)> = vec![(0, self)];

        while let Some((depth, value)) = stack.pop() {
            if depth > max_depth {
                return Err(format!(
                    "maximum nesting depth exceeded (max={}, saw={})",
                    max_depth, depth
                ));
            }

            match value {
                LnmpValue::Int(_)
                | LnmpValue::Float(_)
                | LnmpValue::Bool(_)
                | LnmpValue::String(_)
                | LnmpValue::StringArray(_)
                | LnmpValue::IntArray(_)
                | LnmpValue::FloatArray(_)
                | LnmpValue::BoolArray(_)
                | LnmpValue::Embedding(_)
                | LnmpValue::EmbeddingDelta(_) => {}
                #[cfg(feature = "quant")]
                LnmpValue::QuantizedEmbedding(_) => {}

                LnmpValue::NestedRecord(record) => {
                    for field in record.fields() {
                        stack.push((depth + 1, &field.value));
                    }
                }

                LnmpValue::NestedArray(records) => {
                    for record in records {
                        for field in record.fields() {
                            stack.push((depth + 1, &field.value));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates the structural integrity of the value (deprecated name).
    #[deprecated(note = "use validate_with_max_depth or validate_structure instead")]
    pub fn validate_structure_recursive(&self) -> Result<(), String> {
        match self {
            // Primitive types are always valid
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::String(_)
            | LnmpValue::StringArray(_)
            | LnmpValue::IntArray(_)
            | LnmpValue::FloatArray(_)
            | LnmpValue::BoolArray(_)
            | LnmpValue::Embedding(_)
            | LnmpValue::EmbeddingDelta(_) => Ok(()),
            #[cfg(feature = "quant")]
            LnmpValue::QuantizedEmbedding(_) => Ok(()),

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
    /// Int array type hint (:ia) - v0.6
    IntArray,
    /// Float array type hint (:fa) - v0.6
    FloatArray,
    /// Bool array type hint (:ba) - v0.6
    BoolArray,
    /// Record type hint (:r) - v0.3
    Record,
    /// Record array type hint (:ra) - v0.3
    RecordArray,
    /// Embedding type hint (:v) - v0.5
    Embedding,
    /// Quantized embedding type hint (:qv) - v0.5.2
    #[cfg(feature = "quant")]
    QuantizedEmbedding,
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
            TypeHint::IntArray => "ia",
            TypeHint::FloatArray => "fa",
            TypeHint::BoolArray => "ba",
            TypeHint::Record => "r",
            TypeHint::RecordArray => "ra",
            TypeHint::Embedding => "v",
            #[cfg(feature = "quant")]
            TypeHint::QuantizedEmbedding => "qv",
        }
    }

    /// Parses a type hint from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "i" => Some(TypeHint::Int),
            "f" => Some(TypeHint::Float),
            "b" => Some(TypeHint::Bool),
            "s" => Some(TypeHint::String),
            "sa" => Some(TypeHint::StringArray),
            "ia" => Some(TypeHint::IntArray),
            "fa" => Some(TypeHint::FloatArray),
            "ba" => Some(TypeHint::BoolArray),
            "r" => Some(TypeHint::Record),
            "ra" => Some(TypeHint::RecordArray),
            "v" => Some(TypeHint::Embedding),
            #[cfg(feature = "quant")]
            "qv" => Some(TypeHint::QuantizedEmbedding),
            _ => None,
        }
    }

    /// Deprecated wrapper for backwards compatibility with older API.
    ///
    /// Use `TypeHint::parse(s)` or `str::parse::<TypeHint>(s)` instead.
    #[allow(clippy::should_implement_trait)]
    #[deprecated(
        note = "TypeHint::from_str is deprecated; use TypeHint::parse() or str::parse::<TypeHint>()"
    )]
    pub fn from_str(s: &str) -> Option<Self> {
        Self::parse(s)
    }

    /// Validates that a value matches this type hint
    pub fn validates(&self, value: &LnmpValue) -> bool {
        #[cfg(feature = "quant")]
        {
            matches!(
                (self, value),
                (TypeHint::Int, LnmpValue::Int(_))
                    | (TypeHint::Float, LnmpValue::Float(_))
                    | (TypeHint::Bool, LnmpValue::Bool(_))
                    | (TypeHint::String, LnmpValue::String(_))
                    | (TypeHint::StringArray, LnmpValue::StringArray(_))
                    | (TypeHint::IntArray, LnmpValue::IntArray(_))
                    | (TypeHint::FloatArray, LnmpValue::FloatArray(_))
                    | (TypeHint::BoolArray, LnmpValue::BoolArray(_))
                    | (TypeHint::Record, LnmpValue::NestedRecord(_))
                    | (TypeHint::RecordArray, LnmpValue::NestedArray(_))
                    | (TypeHint::Embedding, LnmpValue::Embedding(_))
                    | (
                        TypeHint::QuantizedEmbedding,
                        LnmpValue::QuantizedEmbedding(_)
                    )
            )
        }
        #[cfg(not(feature = "quant"))]
        {
            matches!(
                (self, value),
                (TypeHint::Int, LnmpValue::Int(_))
                    | (TypeHint::Float, LnmpValue::Float(_))
                    | (TypeHint::Bool, LnmpValue::Bool(_))
                    | (TypeHint::String, LnmpValue::String(_))
                    | (TypeHint::StringArray, LnmpValue::StringArray(_))
                    | (TypeHint::IntArray, LnmpValue::IntArray(_))
                    | (TypeHint::FloatArray, LnmpValue::FloatArray(_))
                    | (TypeHint::BoolArray, LnmpValue::BoolArray(_))
                    | (TypeHint::Record, LnmpValue::NestedRecord(_))
                    | (TypeHint::RecordArray, LnmpValue::NestedArray(_))
                    | (TypeHint::Embedding, LnmpValue::Embedding(_))
            )
        }
    }
}

/// Implement std::str::FromStr for TypeHint so callers can use `str::parse::<TypeHint>()`.
impl FromStr for TypeHint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(TypeHint::Int),
            "f" => Ok(TypeHint::Float),
            "b" => Ok(TypeHint::Bool),
            "s" => Ok(TypeHint::String),
            "sa" => Ok(TypeHint::StringArray),
            "ia" => Ok(TypeHint::IntArray),
            "fa" => Ok(TypeHint::FloatArray),
            "ba" => Ok(TypeHint::BoolArray),
            "r" => Ok(TypeHint::Record),
            "ra" => Ok(TypeHint::RecordArray),
            "v" => Ok(TypeHint::Embedding),
            #[cfg(feature = "quant")]
            "qv" => Ok(TypeHint::QuantizedEmbedding),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

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
        assert_eq!(TypeHint::parse("i"), Some(TypeHint::Int));
        assert_eq!(TypeHint::parse("f"), Some(TypeHint::Float));
        assert_eq!(TypeHint::parse("b"), Some(TypeHint::Bool));
        assert_eq!(TypeHint::parse("s"), Some(TypeHint::String));
        assert_eq!(TypeHint::parse("sa"), Some(TypeHint::StringArray));
        assert_eq!(TypeHint::parse("r"), Some(TypeHint::Record));
        assert_eq!(TypeHint::parse("ra"), Some(TypeHint::RecordArray));
        assert_eq!(TypeHint::parse("invalid"), None);
        assert_eq!(TypeHint::parse(""), None);
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
        assert!(hint.validates(&LnmpValue::StringArray(vec![
            "a".to_string(),
            "b".to_string()
        ])));
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
            let parsed = TypeHint::parse(str_repr);
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
    fn validate_with_max_depth_rejects_excess() {
        use crate::{LnmpField, LnmpRecord};

        // Create a 2-level nesting: depth 2 should fail when max_depth=1.
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let value = LnmpValue::NestedRecord(Box::new(outer));
        let err = value.validate_with_max_depth(1).unwrap_err();
        assert!(err.contains("maximum nesting depth exceeded"));
    }

    #[test]
    fn test_validate_structure_primitive_values() {
        assert!(LnmpValue::Int(42).validate_structure().is_ok());
        assert!(LnmpValue::Float(3.14).validate_structure().is_ok());
        assert!(LnmpValue::Bool(true).validate_structure().is_ok());
        assert!(LnmpValue::String("test".to_string())
            .validate_structure()
            .is_ok());
        assert!(LnmpValue::StringArray(vec!["a".to_string()])
            .validate_structure()
            .is_ok());
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

    #[test]
    fn test_embedding_value() {
        use crate::types::LnmpValue;
        use lnmp_embedding::Vector;

        let vec = Vector::from_f32(vec![0.1, 0.2, 0.3]);
        let val = LnmpValue::Embedding(vec.clone());

        assert_eq!(val.depth(), 0);
        assert!(val.validate_structure().is_ok());

        let hint = TypeHint::Embedding;
        assert!(hint.validates(&val));
        assert_eq!(hint.as_str(), "v");
    }
}
