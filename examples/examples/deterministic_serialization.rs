//! Example demonstrating deterministic serialization in LNMP v0.2

use lnmp_codec::{Encoder, Parser};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.2 Deterministic Serialization ===\n");

    // Example 1: Fields are always sorted by FID
    println!("1. Automatic field sorting:");
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Int(2),
    });

    println!("Insertion order: F100, F5, F50");

    let encoder = Encoder::new();
    let output = encoder.encode(&record);
    println!("Encoded output: {:?}", output);
    println!("(Fields are sorted: F5, F50, F100)\n");

    // Example 2: Multiple encodes produce identical output
    println!("2. Consistent output:");
    let output1 = encoder.encode(&record);
    let output2 = encoder.encode(&record);
    let output3 = encoder.encode(&record);

    println!("Encode 1: {:?}", output1);
    println!("Encode 2: {:?}", output2);
    println!("Encode 3: {:?}", output3);
    println!("All identical: {}\n", output1 == output2 && output2 == output3);

    // Example 3: Loose input becomes canonical
    println!("3. Normalization:");
    let loose_input = "F23=[a,b];F7=1;F12=100"; // Unsorted, semicolons
    println!("Loose input:  {:?}", loose_input);

    let mut parser = Parser::new(loose_input).unwrap();
    let parsed = parser.parse_record().unwrap();

    let canonical = encoder.encode(&parsed);
    println!("Canonical:    {:?}", canonical);
    println!("(Sorted, newlines, no spaces)\n");

    // Example 4: Round-trip stability
    println!("4. Round-trip stability:");
    let input = "F3=test;F1=42;F2=3.14"; // Unsorted
    println!("Original:  {:?}", input);

    let mut parser = Parser::new(input).unwrap();
    let record1 = parser.parse_record().unwrap();
    let encoded1 = encoder.encode(&record1);
    println!("Encode 1:  {:?}", encoded1);

    let mut parser2 = Parser::new(&encoded1).unwrap();
    let record2 = parser2.parse_record().unwrap();
    let encoded2 = encoder.encode(&record2);
    println!("Encode 2:  {:?}", encoded2);

    let mut parser3 = Parser::new(&encoded2).unwrap();
    let record3 = parser3.parse_record().unwrap();
    let encoded3 = encoder.encode(&record3);
    println!("Encode 3:  {:?}", encoded3);

    println!("Stable: {}\n", encoded1 == encoded2 && encoded2 == encoded3);

    // Example 5: Canonical format
    println!("5. Canonical format rules:");
    println!("✓ Fields sorted by FID");
    println!("✓ Newline-separated (no semicolons)");
    println!("✓ No whitespace around equals");
    println!("✓ No spaces after commas in arrays");
    println!("✓ No comments");

    let mut demo = LnmpRecord::new();
    demo.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::StringArray(vec!["a".to_string(), "b".to_string()]),
    });
    demo.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    println!("\nExample canonical output:");
    println!("{}", encoder.encode(&demo));
}
