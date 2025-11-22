//! Structural limit helpers for validating LNMP records and values.
//!
//! These limits are intended to provide a single place to constrain untrusted
//! inputs before they are handed off to parser/encoder layers. All counts and
//! lengths are measured in bytes and are inclusive.

use crate::{LnmpField, LnmpRecord, LnmpValue};

/// Errors returned when structural limits are exceeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralError {
    /// A record exceeded the configured maximum depth.
    MaxDepthExceeded {
        /// Maximum depth configured.
        max_depth: usize,
        /// Actual depth encountered.
        seen_depth: usize,
    },
    /// A record exceeded the configured total field count.
    MaxFieldsExceeded {
        /// Maximum fields configured.
        max_fields: usize,
        /// Actual field count encountered.
        seen_fields: usize,
    },
    /// A string value exceeded the configured maximum length.
    MaxStringLengthExceeded {
        /// Maximum string length configured.
        max_len: usize,
        /// Actual string length encountered.
        seen_len: usize,
    },
    /// An array contained more items than allowed.
    MaxArrayLengthExceeded {
        /// Maximum array length configured.
        max_len: usize,
        /// Actual array length encountered.
        seen_len: usize,
    },
}

impl std::fmt::Display for StructuralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructuralError::MaxDepthExceeded {
                max_depth,
                seen_depth,
            } => {
                write!(
                    f,
                    "maximum nesting depth exceeded (max={}, saw={})",
                    max_depth, seen_depth
                )
            }
            StructuralError::MaxFieldsExceeded {
                max_fields,
                seen_fields,
            } => {
                write!(
                    f,
                    "maximum field count exceeded (max={}, saw={})",
                    max_fields, seen_fields
                )
            }
            StructuralError::MaxStringLengthExceeded { max_len, seen_len } => {
                write!(
                    f,
                    "maximum string length exceeded (max={}, saw={})",
                    max_len, seen_len
                )
            }
            StructuralError::MaxArrayLengthExceeded { max_len, seen_len } => {
                write!(
                    f,
                    "maximum array length exceeded (max={}, saw={})",
                    max_len, seen_len
                )
            }
        }
    }
}

impl std::error::Error for StructuralError {}

/// Configurable structural limits checked against `LnmpRecord`/`LnmpValue`.
#[derive(Debug, Clone)]
pub struct StructuralLimits {
    /// Maximum allowed nesting depth (root = 0).
    pub max_depth: usize,
    /// Maximum total number of fields across the entire record (including nested).
    pub max_fields: usize,
    /// Maximum string length (bytes) for `String` values.
    pub max_string_len: usize,
    /// Maximum item count for arrays (string or nested record arrays).
    pub max_array_items: usize,
}

impl Default for StructuralLimits {
    fn default() -> Self {
        Self {
            // Depth 0 = top-level primitives, 1 = single layer nested record/array.
            max_depth: 32,
            // Generous default; callers should tune per use-case (e.g., LLM prompt budgets).
            max_fields: 4096,
            // Strings are expected to be short labels/values; cap to avoid unbounded text blobs.
            max_string_len: 16 * 1024,
            // Reasonable default to prevent pathological arrays.
            max_array_items: 1024,
        }
    }
}

impl StructuralLimits {
    /// Validates a record against the configured limits.
    pub fn validate_record(&self, record: &LnmpRecord) -> Result<(), StructuralError> {
        let mut field_count = 0;
        self.validate_fields(record.fields(), 0, &mut field_count)
    }

    fn validate_fields(
        &self,
        fields: &[LnmpField],
        depth: usize,
        field_count: &mut usize,
    ) -> Result<(), StructuralError> {
        if depth > self.max_depth {
            return Err(StructuralError::MaxDepthExceeded {
                max_depth: self.max_depth,
                seen_depth: depth,
            });
        }

        for field in fields {
            *field_count += 1;
            if *field_count > self.max_fields {
                return Err(StructuralError::MaxFieldsExceeded {
                    max_fields: self.max_fields,
                    seen_fields: *field_count,
                });
            }
            self.validate_value(&field.value, depth + 1, field_count)?;
        }

        Ok(())
    }

    fn validate_value(
        &self,
        value: &LnmpValue,
        depth: usize,
        field_count: &mut usize,
    ) -> Result<(), StructuralError> {
        match value {
            LnmpValue::String(s) => {
                if s.len() > self.max_string_len {
                    return Err(StructuralError::MaxStringLengthExceeded {
                        max_len: self.max_string_len,
                        seen_len: s.len(),
                    });
                }
                Ok(())
            }
            LnmpValue::StringArray(arr) => {
                if arr.len() > self.max_array_items {
                    return Err(StructuralError::MaxArrayLengthExceeded {
                        max_len: self.max_array_items,
                        seen_len: arr.len(),
                    });
                }
                for s in arr {
                    if s.len() > self.max_string_len {
                        return Err(StructuralError::MaxStringLengthExceeded {
                            max_len: self.max_string_len,
                            seen_len: s.len(),
                        });
                    }
                }
                Ok(())
            }
            LnmpValue::IntArray(ints) => {
                if ints.len() > self.max_array_items {
                    return Err(StructuralError::MaxArrayLengthExceeded {
                        max_len: self.max_array_items,
                        seen_len: ints.len(),
                    });
                }
                Ok(())
            }
            LnmpValue::FloatArray(floats) => {
                if floats.len() > self.max_array_items {
                    return Err(StructuralError::MaxArrayLengthExceeded {
                        max_len: self.max_array_items,
                        seen_len: floats.len(),
                    });
                }
                Ok(())
            }
            LnmpValue::BoolArray(bools) => {
                if bools.len() > self.max_array_items {
                    return Err(StructuralError::MaxArrayLengthExceeded {
                        max_len: self.max_array_items,
                        seen_len: bools.len(),
                    });
                }
                Ok(())
            }
            LnmpValue::NestedRecord(record) => {
                self.validate_fields(record.fields(), depth, field_count)
            }
            LnmpValue::NestedArray(records) => {
                if records.len() > self.max_array_items {
                    return Err(StructuralError::MaxArrayLengthExceeded {
                        max_len: self.max_array_items,
                        seen_len: records.len(),
                    });
                }
                for record in records {
                    self.validate_fields(record.fields(), depth, field_count)?;
                }
                Ok(())
            }
            // Primitive numeric/bool types do not need extra checks.
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::Embedding(_)
            | LnmpValue::EmbeddingDelta(_)
            | LnmpValue::QuantizedEmbedding(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_record(string_len: usize) -> LnmpRecord {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("a".repeat(string_len)),
        });
        record
    }

    #[test]
    fn validates_within_limits() {
        let limits = StructuralLimits::default();
        let record = basic_record(4);
        assert!(limits.validate_record(&record).is_ok());
    }

    #[test]
    fn rejects_oversized_string() {
        let limits = StructuralLimits {
            max_string_len: 2,
            ..StructuralLimits::default()
        };
        let record = basic_record(3);
        let err = limits.validate_record(&record).unwrap_err();
        assert!(matches!(
            err,
            StructuralError::MaxStringLengthExceeded { .. }
        ));
    }

    #[test]
    fn rejects_excessive_depth() {
        let mut inner = LnmpRecord::new();
        inner.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(1),
        });
        let mut outer = LnmpRecord::new();
        outer.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::NestedRecord(Box::new(inner)),
        });

        let limits = StructuralLimits {
            max_depth: 0,
            ..StructuralLimits::default()
        };
        let err = limits.validate_record(&outer).unwrap_err();
        assert!(matches!(err, StructuralError::MaxDepthExceeded { .. }));
    }

    #[test]
    fn rejects_field_count_overflow() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(1),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::Int(2),
        });

        let limits = StructuralLimits {
            max_fields: 1,
            ..StructuralLimits::default()
        };
        let err = limits.validate_record(&record).unwrap_err();
        assert!(matches!(err, StructuralError::MaxFieldsExceeded { .. }));
    }

    #[test]
    fn rejects_array_length_overflow() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
        });
        let limits = StructuralLimits {
            max_array_items: 1,
            ..StructuralLimits::default()
        };
        let err = limits.validate_record(&record).unwrap_err();
        assert!(matches!(
            err,
            StructuralError::MaxArrayLengthExceeded { .. }
        ));
    }
}
