//! Example demonstrating LNMP v0.5 Delta Encoding & Partial Update Layer (DPL)
//!
//! This example shows:
//! - Computing deltas between records
//! - Delta operations: SET_FIELD, DELETE_FIELD, UPDATE_FIELD, MERGE_RECORD
//! - Encoding and decoding delta packets
//! - Applying deltas to base records
//! - Bandwidth savings measurement
//! - Incremental updates for nested structures

use lnmp_codec::binary::{
    BinaryEncoder, DeltaEncoder, DeltaDecoder, DeltaConfig, DeltaOperation,
};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.5 Delta Encoding & Partial Update Layer Example ===\n");

    // Example 1: Simple field update
    println!("1. Simple Field Update:");
    simple_field_update();
    println!();

    // Example 2: Multiple field changes
    println!("2. Multiple Field Changes:");
    multiple_field_changes();
    println!();

    // Example 3: Field deletion
    println!("3. Field Deletion:");
    field_deletion_example();
    println!();

    // Example 4: Nested record merge
    println!("4. Nested Record Merge:");
    nested_record_merge();
    println!();

    // Example 5: Bandwidth savings measurement
    println!("5. Bandwidth Savings:");
    bandwidth_savings_example();
    println!();

    // Example 6: Incremental updates
    println!("6. Incremental Updates:");
    incremental_updates_example();
    println!();
}

fn simple_field_update() {
    // Create base record
    let mut base = LnmpRecord::new();
    base.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    base.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    base.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::String("admin".to_string()),
    });

    println!("   Base record: F7=1, F12=14532, F23=admin");

    // Create updated record (only F12 changed)
    let mut updated = base.clone();
    if updated.get_field(12).is_some() {
        updated.remove_field(12);
        updated.add_field(LnmpField { fid: 12, value: LnmpValue::Int(99999) });
    }

    println!("   Updated record: F7=1, F12=99999, F23=admin");

    // Compute delta
    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_ops = delta_encoder.compute_delta(&base, &updated).unwrap();

    println!("   Delta operations: {} operation(s)", delta_ops.len());
    for op in &delta_ops {
        println!("     F{}: {:?}", op.target_fid, op.operation);
    }

    // Encode delta
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
    println!("   Delta binary size: {} bytes", delta_binary.len());

    // Compare with full encoding
    let encoder = BinaryEncoder::new();
    let full_binary = encoder.encode(&updated).unwrap();
    println!("   Full binary size: {} bytes", full_binary.len());
    println!("   Savings: {} bytes ({:.1}%)", 
             full_binary.len() - delta_binary.len(),
             (1.0 - delta_binary.len() as f64 / full_binary.len() as f64) * 100.0);

    // Apply delta
    let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoded_ops = delta_decoder.decode_delta(&delta_binary).unwrap();
    let mut result = base.clone();
    delta_decoder.apply_delta(&mut result, &decoded_ops).unwrap();

    println!("   ✓ Delta applied successfully");
    if let Some(field) = result.get_field(12) {
        if let LnmpValue::Int(val) = field.value {
            println!("   ✓ F12 updated to: {}", val);
        }
    }
}

fn multiple_field_changes() {
    // Create base record
    let mut base = LnmpRecord::new();
    for i in 1..=10 {
        base.add_field(LnmpField {
            fid: i,
            value: LnmpValue::Int(i as i64 * 100),
        });
    }

    println!("   Base record: 10 fields (F1-F10)");

    // Update 3 fields
    let mut updated = base.clone();
    if updated.get_field(2).is_some() {
        updated.remove_field(2);
        updated.add_field(LnmpField { fid: 2, value: LnmpValue::Int(999) });
    }
    if updated.get_field(5).is_some() {
        updated.remove_field(5);
        updated.add_field(LnmpField { fid: 5, value: LnmpValue::Int(888) });
    }
    if updated.get_field(8).is_some() {
        updated.remove_field(8);
        updated.add_field(LnmpField { fid: 8, value: LnmpValue::Int(777) });
    }

    println!("   Updated: F2, F5, F8 changed");

    // Compute and encode delta
    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_ops = delta_encoder.compute_delta(&base, &updated).unwrap();
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();

    println!("   Delta operations: {}", delta_ops.len());
    println!("   Delta size: {} bytes", delta_binary.len());

    // Apply delta
    let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoded_ops = delta_decoder.decode_delta(&delta_binary).unwrap();
    let mut result = base.clone();
    delta_decoder.apply_delta(&mut result, &decoded_ops).unwrap();

    println!("   ✓ All changes applied successfully");
}

fn field_deletion_example() {
    // Create base record
    let mut base = LnmpRecord::new();
    base.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    base.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    base.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::String("admin".to_string()),
    });

    println!("   Base record: F7, F12, F23");

    // Create updated record with F12 removed
    let mut updated = LnmpRecord::new();
    updated.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    updated.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::String("admin".to_string()),
    });

    println!("   Updated record: F7, F23 (F12 deleted)");

    // Compute delta
    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_ops = delta_encoder.compute_delta(&base, &updated).unwrap();

    println!("   Delta operations:");
    for op in &delta_ops {
        match op.operation {
            DeltaOperation::DeleteField => {
                println!("     DELETE F{}", op.target_fid);
            }
            _ => {}
        }
    }

    // Encode and apply delta
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
    let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoded_ops = delta_decoder.decode_delta(&delta_binary).unwrap();
    let mut result = base.clone();
    delta_decoder.apply_delta(&mut result, &decoded_ops).unwrap();

    println!("   ✓ Field deleted successfully");
    println!("   Result has {} fields", result.fields().len());
}

fn nested_record_merge() {
    // Create base record with nested structure
    let mut base_inner = LnmpRecord::new();
    base_inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("old_value".to_string()),
    });
    base_inner.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(100),
    });

    let mut base = LnmpRecord::new();
    base.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("outer".to_string()),
    });
    base.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::NestedRecord(Box::new(base_inner)),
    });

    println!("   Base: F10=outer, F20={{F1=old_value, F2=100}}");

    // Create updated record with nested changes
    let mut updated_inner = LnmpRecord::new();
    updated_inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("new_value".to_string()),
    });
    updated_inner.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(200),
    });
    updated_inner.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let mut updated = LnmpRecord::new();
    updated.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("outer".to_string()),
    });
    updated.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::NestedRecord(Box::new(updated_inner)),
    });

    println!("   Updated: F10=outer, F20={{F1=new_value, F2=200, F3=1}}");

    // Compute delta
    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_ops = delta_encoder.compute_delta(&base, &updated).unwrap();

    println!("   Delta operations:");
    for op in &delta_ops {
        match op.operation {
            DeltaOperation::MergeRecord => {
                println!("     MERGE F{} (nested record)", op.target_fid);
            }
            _ => {
                println!("     {:?} F{}", op.operation, op.target_fid);
            }
        }
    }

    // Apply delta
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
    let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));
    let decoded_ops = delta_decoder.decode_delta(&delta_binary).unwrap();
    let mut result = base.clone();
    delta_decoder.apply_delta(&mut result, &decoded_ops).unwrap();

    println!("   ✓ Nested record merged successfully");
}

fn bandwidth_savings_example() {
    // Create a large base record
    let mut base = LnmpRecord::new();
    for i in 1..=50 {
        base.add_field(LnmpField {
            fid: i,
            value: LnmpValue::String(format!("Field {} with substantial content", i)),
        });
    }

    // Update only 5 fields
    let mut updated = base.clone();
    for i in [5, 15, 25, 35, 45] {
        if updated.get_field(i).is_some() {
            updated.remove_field(i);
            updated.add_field(LnmpField { fid: i, value: LnmpValue::Int(1) });
        }
    }

    println!("   Base record: 50 fields");
    println!("   Updated: 5 fields changed (10%)");

    // Measure full encoding
    let encoder = BinaryEncoder::new();
    let full_binary = encoder.encode(&updated).unwrap();
    println!("   Full encoding: {} bytes", full_binary.len());

    // Measure delta encoding
    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_ops = delta_encoder.compute_delta(&base, &updated).unwrap();
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
    println!("   Delta encoding: {} bytes", delta_binary.len());

    let savings = full_binary.len() - delta_binary.len();
    let savings_pct = (1.0 - delta_binary.len() as f64 / full_binary.len() as f64) * 100.0;
    println!("   ✓ Bandwidth savings: {} bytes ({:.1}%)", savings, savings_pct);
}

fn incremental_updates_example() {
    // Simulate a series of incremental updates
    let mut current = LnmpRecord::new();
    current.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(0),
    });
    current.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("initial".to_string()),
    });

    println!("   Initial state: F1=0, F2=initial");

    let config = DeltaConfig::new().with_enable_delta(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    let delta_decoder = DeltaDecoder::with_config(DeltaConfig::new().with_enable_delta(true));

    // Update 1: Increment counter
    let mut update1 = current.clone();
    if update1.get_field(1).is_some() {
        update1.remove_field(1);
        update1.add_field(LnmpField { fid: 1, value: LnmpValue::Int(1) });
    }

    let delta1 = delta_encoder.compute_delta(&current, &update1).unwrap();
    let delta1_binary = delta_encoder.encode_delta(&delta1).unwrap();
    println!("   Update 1: F1=1 (delta: {} bytes)", delta1_binary.len());
    
    let decoded1 = delta_decoder.decode_delta(&delta1_binary).unwrap();
    delta_decoder.apply_delta(&mut current, &decoded1).unwrap();

    // Update 2: Increment counter again
    let mut update2 = current.clone();
    if update2.get_field(1).is_some() {
        update2.remove_field(1);
        update2.add_field(LnmpField { fid: 1, value: LnmpValue::Int(2) });
    }

    let delta2 = delta_encoder.compute_delta(&current, &update2).unwrap();
    let delta2_binary = delta_encoder.encode_delta(&delta2).unwrap();
    println!("   Update 2: F1=2 (delta: {} bytes)", delta2_binary.len());
    
    let decoded2 = delta_decoder.decode_delta(&delta2_binary).unwrap();
    delta_decoder.apply_delta(&mut current, &decoded2).unwrap();

    // Update 3: Change string
    let mut update3 = current.clone();
    if update3.get_field(2).is_some() {
        update3.remove_field(2);
        update3.add_field(LnmpField { fid: 2, value: LnmpValue::String("updated".to_string()) });
    }

    let delta3 = delta_encoder.compute_delta(&current, &update3).unwrap();
    let delta3_binary = delta_encoder.encode_delta(&delta3).unwrap();
    println!("   Update 3: F2=updated (delta: {} bytes)", delta3_binary.len());
    
    let decoded3 = delta_decoder.decode_delta(&delta3_binary).unwrap();
    delta_decoder.apply_delta(&mut current, &decoded3).unwrap();

    println!("   ✓ Final state after 3 incremental updates:");
    if let Some(field) = current.get_field(1) {
        if let LnmpValue::Int(val) = field.value {
            println!("     F1={}", val);
        }
    }
    if let Some(field) = current.get_field(2) {
        if let LnmpValue::String(val) = &field.value {
            println!("     F2={}", val);
        }
    }

    let total_delta_size = delta1_binary.len() + delta2_binary.len() + delta3_binary.len();
    let encoder = BinaryEncoder::new();
    let full_size = encoder.encode(&current).unwrap().len();
    println!("   Total delta size: {} bytes", total_delta_size);
    println!("   Full encoding: {} bytes", full_size);
    println!("   Efficiency: {:.1}% of full encoding", 
             total_delta_size as f64 / full_size as f64 * 100.0);
}
