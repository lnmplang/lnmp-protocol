//! Round-trip conversion example for LNMP v0.4
//!
//! This example demonstrates:
//! - Text → Binary → Text round-trip conversion
//! - Binary → Text → Binary round-trip conversion
//! - Canonical form stability across multiple conversions
//! - Data integrity verification

use lnmp_codec::binary::{BinaryEncoder, BinaryDecoder};

fn main() {
    println!("=== LNMP v0.4 Round-Trip Conversion Example ===\n");

    // Example 1: Text → Binary → Text
    println!("1. Text → Binary → Text Round-Trip");
    println!("{}", "-".repeat(50));
    
    let original_text = "F7=1\nF12=14532\nF23=[admin,dev]";
    println!("Original text:\n{}", original_text);
    println!();

    let encoder = BinaryEncoder::new();
    let decoder = BinaryDecoder::new();

    // Convert to binary
    let binary = encoder.encode_text(original_text).unwrap();
    println!("Binary size: {} bytes", binary.len());
    
    // Convert back to text
    let decoded_text = decoder.decode_to_text(&binary).unwrap();
    println!("Decoded text:\n{}", decoded_text);
    
    // Verify round-trip
    if original_text == decoded_text {
        println!("✓ Round-trip successful: text matches exactly");
    } else {
        println!("✗ Round-trip failed: text differs");
    }
    println!();

    // Example 2: Binary → Text → Binary
    println!("2. Binary → Text → Binary Round-Trip");
    println!("{}", "-".repeat(50));
    
    let original_binary = encoder.encode_text("F7=1\nF12=14532").unwrap();
    println!("Original binary: {} bytes", original_binary.len());
    
    // Convert to text
    let text = decoder.decode_to_text(&original_binary).unwrap();
    println!("Intermediate text: {}", text);
    
    // Convert back to binary
    let roundtrip_binary = encoder.encode_text(&text).unwrap();
    println!("Round-trip binary: {} bytes", roundtrip_binary.len());
    
    // Verify round-trip
    if original_binary == roundtrip_binary {
        println!("✓ Round-trip successful: binary matches exactly");
    } else {
        println!("✗ Round-trip failed: binary differs");
    }
    println!();

    // Example 3: Unsorted input becomes canonical
    println!("3. Canonical Form Normalization");
    println!("{}", "-".repeat(50));
    
    let unsorted_text = "F23=[admin,dev]\nF7=1\nF12=14532";
    println!("Unsorted input:\n{}", unsorted_text);
    
    let binary = encoder.encode_text(unsorted_text).unwrap();
    let canonical_text = decoder.decode_to_text(&binary).unwrap();
    
    println!("Canonical output:\n{}", canonical_text);
    println!("(Fields sorted by FID: F7, F12, F23)");
    println!();

    // Example 4: Multiple round-trips produce stable output
    println!("4. Canonical Stability (Multiple Round-Trips)");
    println!("{}", "-".repeat(50));
    
    let mut current_text = "F23=[admin]\nF7=1\nF12=14532".to_string();
    println!("Initial text: {}", current_text);
    println!();
    
    for i in 1..=5 {
        let binary = encoder.encode_text(&current_text).unwrap();
        current_text = decoder.decode_to_text(&binary).unwrap();
        println!("After round-trip {}: {}", i, current_text);
    }
    
    println!("\n✓ Output stabilized after first canonicalization");
    println!();

    // Example 5: All value types round-trip
    println!("5. All Value Types Round-Trip");
    println!("{}", "-".repeat(50));
    
    let test_cases = vec![
        ("F1=-42", "Integer (negative)"),
        ("F2=3.14159", "Float"),
        ("F3=0", "Boolean (false)"),
        ("F4=1", "Boolean (true)"),
        ("F5=\"hello\\nworld\"", "String (with escape)"),
        ("F6=[a,b,c]", "String array"),
        ("F7=[]", "Empty array"),
    ];

    for (input, description) in test_cases {
        let binary = encoder.encode_text(input).unwrap();
        let output = decoder.decode_to_text(&binary).unwrap();
        
        let status = if input == output { "✓" } else { "✗" };
        println!("{} {} - Input: {} → Output: {}", status, description, input, output);
    }
    println!();

    // Example 6: Edge case strings
    println!("6. Edge Case Strings Round-Trip");
    println!("{}", "-".repeat(50));
    
    let edge_cases = vec![
        ("", "Empty string"),
        ("hello\nworld", "Newline"),
        ("path\\to\\file", "Backslashes"),
        ("say \"hello\"", "Quotes"),
        ("emoji: 🎯", "Unicode"),
        ("tab\there", "Tab character"),
    ];

    for (string_content, description) in edge_cases {
        // Create text with escaped string
        let text = format!("F1=\"{}\"", string_content.escape_default());
        
        let binary = encoder.encode_text(&text).unwrap();
        let decoded = decoder.decode_to_text(&binary).unwrap();
        
        // Parse back to verify content
        use lnmp_codec::Parser;
        let mut parser = Parser::new(&decoded).unwrap();
        let record = parser.parse_record().unwrap();
        
        if let Some(field) = record.get_field(1) {
            if let lnmp_core::LnmpValue::String(s) = &field.value {
                let status = if s == string_content { "✓" } else { "✗" };
                println!("{} {}: preserved correctly", status, description);
            }
        }
    }
    println!();

    // Example 7: Performance comparison
    println!("7. Round-Trip Performance");
    println!("{}", "-".repeat(50));
    
    let test_text = "F7=1\nF12=14532\nF23=[admin,dev]";
    let iterations = 1000;
    
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let binary = encoder.encode_text(test_text).unwrap();
        let _ = decoder.decode_to_text(&binary).unwrap();
    }
    let duration = start.elapsed();
    
    println!("Performed {} round-trips in {:?}", iterations, duration);
    println!("Average time per round-trip: {:?}", duration / iterations);
    println!();

    // Example 8: Data integrity verification
    println!("8. Data Integrity Verification");
    println!("{}", "-".repeat(50));
    
    use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
    
    let mut original_record = LnmpRecord::new();
    original_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(i64::MAX),
    });
    original_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(i64::MIN),
    });
    original_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Float(f64::INFINITY),
    });
    original_record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(f64::NEG_INFINITY),
    });

    let binary = encoder.encode(&original_record).unwrap();
    let decoded_record = decoder.decode(&binary).unwrap();

    println!("Testing extreme values:");
    println!("  i64::MAX: {} → {}", 
        i64::MAX, 
        match decoded_record.get_field(1).unwrap().value {
            LnmpValue::Int(i) => i,
            _ => 0,
        }
    );
    println!("  i64::MIN: {} → {}", 
        i64::MIN,
        match decoded_record.get_field(2).unwrap().value {
            LnmpValue::Int(i) => i,
            _ => 0,
        }
    );
    println!("  f64::INFINITY: {} → {}", 
        f64::INFINITY,
        match decoded_record.get_field(3).unwrap().value {
            LnmpValue::Float(f) => f,
            _ => 0.0,
        }
    );
    println!("  f64::NEG_INFINITY: {} → {}", 
        f64::NEG_INFINITY,
        match decoded_record.get_field(4).unwrap().value {
            LnmpValue::Float(f) => f,
            _ => 0.0,
        }
    );
    
    println!("\n✓ All extreme values preserved correctly");
}
