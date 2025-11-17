//! Tests for Delta Encoding & Partial Update Layer (DPL)

use lnmp_codec::binary::{DeltaConfig, DeltaDecoder, DeltaEncoder, DeltaError, DeltaOperation};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

#[test]
fn test_compute_delta_simple_field_change() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    old_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("old".to_string()),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    new_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("new".to_string()),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Should have one UPDATE_FIELD operation for field 2
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].target_fid, 2);
    assert_eq!(ops[0].operation, DeltaOperation::UpdateField);
}

#[test]
fn test_compute_delta_field_added() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    new_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("added".to_string()),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Should have one SET_FIELD operation for field 2
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].target_fid, 2);
    assert_eq!(ops[0].operation, DeltaOperation::SetField);
}

#[test]
fn test_compute_delta_field_deleted() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    old_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("deleted".to_string()),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Should have one DELETE_FIELD operation for field 2
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].target_fid, 2);
    assert_eq!(ops[0].operation, DeltaOperation::DeleteField);
}

#[test]
fn test_compute_delta_nested_changes() {
    let mut inner_old = LnmpRecord::new();
    inner_old.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(100),
    });

    let mut inner_new = LnmpRecord::new();
    inner_new.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(200),
    });

    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::NestedRecord(Box::new(inner_old)),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::NestedRecord(Box::new(inner_new)),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Should have one MERGE_RECORD operation for field 5
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].target_fid, 5);
    assert_eq!(ops[0].operation, DeltaOperation::MergeRecord);
}

#[test]
fn test_apply_set_field_operation() {
    let mut base_record = LnmpRecord::new();
    base_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Create a new record with an additional field
    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    new_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("new".to_string()),
    });

    // Compute delta
    let ops = encoder.compute_delta(&base_record, &new_record).unwrap();

    // Apply delta
    decoder.apply_delta(&mut base_record, &ops).unwrap();

    // Verify the field was added
    assert_eq!(base_record.fields().len(), 2);
    assert_eq!(
        base_record.get_field(2).unwrap().value,
        LnmpValue::String("new".to_string())
    );
}

#[test]
fn test_apply_delete_field_operation() {
    let mut base_record = LnmpRecord::new();
    base_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    base_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("delete_me".to_string()),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Create a new record without field 2
    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    // Compute delta
    let ops = encoder.compute_delta(&base_record, &new_record).unwrap();

    // Apply delta
    decoder.apply_delta(&mut base_record, &ops).unwrap();

    // Verify the field was deleted
    assert_eq!(base_record.fields().len(), 1);
    assert!(base_record.get_field(2).is_none());
}

#[test]
fn test_apply_update_field_operation() {
    let mut base_record = LnmpRecord::new();
    base_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("old".to_string()),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Create a new record with updated field
    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("new".to_string()),
    });

    // Compute delta
    let ops = encoder.compute_delta(&base_record, &new_record).unwrap();

    // Apply delta
    decoder.apply_delta(&mut base_record, &ops).unwrap();

    // Verify the field was updated
    assert_eq!(
        base_record.get_field(1).unwrap().value,
        LnmpValue::String("new".to_string())
    );
}

#[test]
fn test_apply_merge_record_operation() {
    let mut inner_old = LnmpRecord::new();
    inner_old.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(100),
    });
    inner_old.add_field(LnmpField {
        fid: 11,
        value: LnmpValue::String("keep".to_string()),
    });

    let mut base_record = LnmpRecord::new();
    base_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::NestedRecord(Box::new(inner_old)),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Create a new record with updated nested field
    let mut inner_new = LnmpRecord::new();
    inner_new.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(200),
    });
    inner_new.add_field(LnmpField {
        fid: 11,
        value: LnmpValue::String("keep".to_string()),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::NestedRecord(Box::new(inner_new)),
    });

    // Compute delta
    let ops = encoder.compute_delta(&base_record, &new_record).unwrap();

    // Apply delta
    decoder.apply_delta(&mut base_record, &ops).unwrap();

    // Verify the nested field was updated
    match &base_record.get_field(5).unwrap().value {
        LnmpValue::NestedRecord(rec) => {
            assert_eq!(rec.get_field(10).unwrap().value, LnmpValue::Int(200));
            assert_eq!(
                rec.get_field(11).unwrap().value,
                LnmpValue::String("keep".to_string())
            );
        }
        _ => panic!("Expected nested record"),
    }
}

#[test]
fn test_delta_round_trip_stability() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    old_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("old".to_string()),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    new_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("new".to_string()),
    });
    new_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Compute delta
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Apply delta to old record
    let mut result_record = old_record.clone();
    decoder.apply_delta(&mut result_record, &ops).unwrap();

    // Result should match new record
    assert_eq!(result_record.fields().len(), new_record.fields().len());
    for field in new_record.fields() {
        assert_eq!(
            result_record.get_field(field.fid).unwrap().value,
            field.value
        );
    }
}

#[test]
fn test_delta_semantic_equivalence() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(200),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Compute and encode delta
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();
    let encoded = encoder.encode_delta(&ops).unwrap();

    // Decode delta
    let decoded_ops = decoder.decode_delta(&encoded).unwrap();

    // Apply decoded delta
    decoder.apply_delta(&mut old_record, &decoded_ops).unwrap();

    // Result should be semantically equivalent to new record
    assert_eq!(
        old_record.get_field(1).unwrap().value,
        new_record.get_field(1).unwrap().value
    );
}

#[test]
fn test_encode_decode_delta_packet() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    new_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("added".to_string()),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Compute delta
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Encode delta packet
    let encoded = encoder.encode_delta(&ops).unwrap();

    // Verify delta tag
    assert_eq!(encoded[0], 0xB0);

    // Decode delta packet
    let decoded_ops = decoder.decode_delta(&encoded).unwrap();

    // Verify operations match
    assert_eq!(decoded_ops.len(), ops.len());
    assert_eq!(decoded_ops[0].target_fid, ops[0].target_fid);
    assert_eq!(decoded_ops[0].operation, ops[0].operation);
}

#[test]
fn test_delta_config() {
    let config = DeltaConfig::new()
        .with_enable_delta(true)
        .with_track_changes(true);

    assert!(config.enable_delta);
    assert!(config.track_changes);

    let encoder = DeltaEncoder::with_config(config.clone());
    let _decoder = DeltaDecoder::with_config(config);

    // Just verify they can be created with config
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(2),
    });

    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();
    assert!(!ops.is_empty());
}

#[test]
fn test_delta_no_changes() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&record, &record).unwrap();

    // No changes should result in no operations
    assert_eq!(ops.len(), 0);
}

#[test]
fn test_compute_delta_disabled_errors() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });

    let mut new_record = old_record.clone();
    new_record.remove_field(1);

    // Default DeltaEncoder has delta disabled; compute_delta should return DeltaApplicationFailed
    let encoder = DeltaEncoder::new();
    let err = encoder.compute_delta(&old_record, &new_record).unwrap_err();
    assert!(matches!(err, DeltaError::DeltaApplicationFailed { .. }));
}

#[test]
fn test_decode_delta_disabled_errors() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });

    let mut new_record = old_record.clone();
    new_record.add_field(LnmpField { fid: 2, value: LnmpValue::Int(2) });

    // Use an enabled encoder to compute bytes
    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();
    let encoded = encoder.encode_delta(&ops).unwrap();

    // Default DeltaDecoder has delta disabled; decode_delta should fail
    let decoder = DeltaDecoder::new();
    let err = decoder.decode_delta(&encoded).unwrap_err();
    assert!(matches!(err, DeltaError::DeltaApplicationFailed { .. }));
}

#[test]
fn test_apply_delta_disabled_errors() {
    let mut base_record = LnmpRecord::new();
    base_record.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });

    let mut new_record = base_record.clone();
    new_record.add_field(LnmpField { fid: 2, value: LnmpValue::Int(2) });

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let ops = encoder.compute_delta(&base_record, &new_record).unwrap();

    // Default DeltaDecoder has delta disabled; apply_delta should fail
    let decoder = DeltaDecoder::new();
    let err = decoder.apply_delta(&mut base_record, &ops).unwrap_err();
    assert!(matches!(err, DeltaError::DeltaApplicationFailed { .. }));
}

#[test]
fn test_delta_multiple_changes() {
    let mut old_record = LnmpRecord::new();
    old_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });
    old_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("old".to_string()),
    });
    old_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(false),
    });

    let mut new_record = LnmpRecord::new();
    new_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(2),
    }); // Updated
    new_record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(3.14),
    }); // Added
                                            // Field 2 deleted
                                            // Field 3 deleted

    let encoder = DeltaEncoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    let ops = encoder.compute_delta(&old_record, &new_record).unwrap();

    // Should have operations for: update field 1, delete field 2, delete field 3, add field 4
    assert_eq!(ops.len(), 4);

    // Apply delta
    decoder.apply_delta(&mut old_record, &ops).unwrap();

    // Verify result matches new record
    assert_eq!(old_record.fields().len(), 2);
    assert_eq!(old_record.get_field(1).unwrap().value, LnmpValue::Int(2));
    assert_eq!(
        old_record.get_field(4).unwrap().value,
        LnmpValue::Float(3.14)
    );
    assert!(old_record.get_field(2).is_none());
    assert!(old_record.get_field(3).is_none());
}
