//! Helper utilities for event generation

use lnmp::prelude::*;

/// Helper to add a string field to a record
pub fn add_string_field(record: &mut LnmpRecord, fid: u16, value: impl Into<String>) {
    record.add_field(LnmpField {
        fid,
        value: LnmpValue::String(value.into()),
    });
}

/// Helper to add an integer field to a record
pub fn add_int_field(record: &mut LnmpRecord, fid: u16, value: i64) {
    record.add_field(LnmpField {
        fid,
        value: LnmpValue::Int(value),
    });
}

/// Helper to add a float field to a record
pub fn add_float_field(record: &mut LnmpRecord, fid: u16, value: f32) {
    record.add_field(LnmpField {
        fid,
        value: LnmpValue::Float(value as f64),
    });
}

/// Helper to add a boolean field to a record
pub fn add_bool_field(record: &mut LnmpRecord, fid: u16, value: bool) {
    record.add_field(LnmpField {
        fid,
        value: LnmpValue::Bool(value),
    });
}
