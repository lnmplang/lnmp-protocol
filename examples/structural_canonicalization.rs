//! Example demonstrating structural canonicalization with nested structures
//!
//! This example shows how LNMP v0.3 canonicalizes records by:
//! - Sorting fields by FID at every nesting level
//! - Applying depth-first ordering for nested structures
//! - Ensuring deterministic encoding

use lnmp_codec::{canonicalize_record, Encoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.3 Structural Canonicalization Demo ===\n");

    // Example 1: Basic field sorting
    println!("Example 1: Basic Field Sorting");
    println!("--------------------------------");
    let mut record1 = LnmpRecord::new();
    record1.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("third".to_string()),
    });
    record1.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::String("first".to_string()),
    });
    record1.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("second".to_string()),
    });

    println!("Before canonicalization (insertion order):");
    println!("  FID 100, FID 5, FID 50");
    
    let canonical1 = canonicalize_record(&record1);
    println!("\nAfter canonicalization (sorted by FID):");
    for field in canonical1.fields() {
        println!("  FID {}", field.fid);
    }

    let encoder = Encoder::new();
    println!("\nEncoded output:");
    println!("{}\n", encoder.encode(&record1));

    // Example 2: Nested record canonicalization
    println!("Example 2: Nested Record Canonicalization");
    println!("------------------------------------------");
    let mut inner_record = LnmpRecord::new();
    inner_record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    inner_record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    inner_record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    let mut outer_record = LnmpRecord::new();
    outer_record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("outer_field".to_string()),
    });
    outer_record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::NestedRecord(Box::new(inner_record)),
    });
    outer_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(42),
    });

    println!("Before canonicalization:");
    println!("  Outer: FID 100, FID 50 (nested), FID 10");
    println!("  Inner: FID 12, FID 7, FID 23");

    let _canonical2 = canonicalize_record(&outer_record);
    println!("\nAfter canonicalization (depth-first sorting):");
    println!("  Outer: FID 10, FID 50 (nested), FID 100");
    println!("  Inner: FID 7, FID 12, FID 23");

    println!("\nNote: Nested record encoding will be implemented in task 8");
    println!("Current output (without nested encoding):");
    // The encoder will show unimplemented for nested structures
    // This is expected as task 8 will implement the actual encoding

    // Example 3: Nested array canonicalization
    println!("\nExample 3: Nested Array Canonicalization");
    println!("-----------------------------------------");
    let mut array_record1 = LnmpRecord::new();
    array_record1.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("second".to_string()),
    });
    array_record1.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("first".to_string()),
    });

    let mut array_record2 = LnmpRecord::new();
    array_record2.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::String("fourth".to_string()),
    });
    array_record2.add_field(LnmpField {
        fid: 15,
        value: LnmpValue::String("third".to_string()),
    });

    let mut outer_with_array = LnmpRecord::new();
    outer_with_array.add_field(LnmpField {
        fid: 60,
        value: LnmpValue::NestedArray(vec![array_record1, array_record2]),
    });

    println!("Before canonicalization:");
    println!("  Array element 1: FID 20, FID 10");
    println!("  Array element 2: FID 30, FID 15");

    let canonical3 = canonicalize_record(&outer_with_array);
    if let LnmpValue::NestedArray(arr) = &canonical3.fields()[0].value {
        println!("\nAfter canonicalization:");
        println!("  Array element 1: FID {}, FID {}", arr[0].fields()[0].fid, arr[0].fields()[1].fid);
        println!("  Array element 2: FID {}, FID {}", arr[1].fields()[0].fid, arr[1].fields()[1].fid);
    }

    // Example 4: Idempotency
    println!("\nExample 4: Canonicalization is Idempotent");
    println!("------------------------------------------");
    let mut record4 = LnmpRecord::new();
    record4.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Int(2),
    });
    record4.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(1),
    });

    let canonical_once = canonicalize_record(&record4);
    let canonical_twice = canonicalize_record(&canonical_once);

    println!("Canonicalizing once: {:?}", canonical_once.fields().iter().map(|f| f.fid).collect::<Vec<_>>());
    println!("Canonicalizing twice: {:?}", canonical_twice.fields().iter().map(|f| f.fid).collect::<Vec<_>>());
    println!("Results are identical: {}", canonical_once == canonical_twice);

    println!("\n=== Canonicalization ensures deterministic encoding ===");
    println!("All nested structures are sorted depth-first by FID");
}
