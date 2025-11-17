//! Example demonstrating LNMP v0.5 binary nested structures
//!
//! This example shows:
//! - Binary encoding of nested records (TypeTag 0x06)
//! - Binary encoding of nested arrays (TypeTag 0x07)
//! - Multi-level nesting with depth validation
//! - Canonical ordering at all nesting levels
//! - Round-trip binary encoding/decoding
//! - Size and depth limit enforcement

use lnmp_codec::binary::{
    BinaryNestedEncoder, BinaryNestedDecoder,
    NestedEncoderConfig, NestedDecoderConfig,
};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.5 Binary Nested Structures Example ===\n");

    // Example 1: Simple nested record in binary format
    println!("1. Simple Nested Record (Binary):");
    simple_nested_binary();
    println!();

    // Example 2: Nested array of records in binary format
    println!("2. Nested Array of Records (Binary):");
    nested_array_binary();
    println!();

    // Example 3: Multi-level nesting (depth 3)
    println!("3. Multi-Level Nested Structure:");
    multi_level_nesting();
    println!();

    // Example 4: Depth limit enforcement
    println!("4. Depth Limit Enforcement:");
    depth_limit_example();
    println!();

    // Example 5: Size limit enforcement
    println!("5. Size Limit Enforcement:");
    size_limit_example();
    println!();

    // Example 6: Canonical ordering at all levels
    println!("6. Canonical Ordering:");
    canonical_ordering_example();
    println!();
}

fn simple_nested_binary() {
    // Create a nested record: user with embedded address
    let mut address = LnmpRecord::new();
    address.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("123 Main St".to_string()),
    });
    address.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("Springfield".to_string()),
    });
    address.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::String("12345".to_string()),
    });

    let mut user = LnmpRecord::new();
    user.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("Alice".to_string()),
    });
    user.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    user.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::NestedRecord(Box::new(address)),
    });

    // Encode to binary with nested support
    let config = NestedEncoderConfig::new();
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&user).unwrap();
    
    println!("   Original record: user with nested address");
    println!("   Binary size: {} bytes", binary.len());
    println!("   Binary (hex): {}", hex_preview(&binary, 32));

    // Decode back
    let decoder_config = NestedDecoderConfig::new();
    let decoder = BinaryNestedDecoder::with_config(decoder_config);
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();
    
    println!("   Decoded successfully: {} fields", decoded.fields().len());
    if let Some(field) = decoded.get_field(20) {
        if let LnmpValue::NestedRecord(nested) = &field.value {
            println!("   Nested address has {} fields", nested.fields().len());
        }
    }
}

fn nested_array_binary() {
    // Create an array of user records
    let mut user1 = LnmpRecord::new();
    user1.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Alice".to_string()),
    });
    user1.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("admin".to_string()),
    });

    let mut user2 = LnmpRecord::new();
    user2.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Bob".to_string()),
    });
    user2.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("user".to_string()),
    });

    let mut user3 = LnmpRecord::new();
    user3.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Charlie".to_string()),
    });
    user3.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("guest".to_string()),
    });

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::NestedArray(vec![user1, user2, user3]),
    });

    // Encode to binary
    let config = NestedEncoderConfig::new();
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&record).unwrap();
    
    println!("   Array of 3 users");
    println!("   Binary size: {} bytes", binary.len());
    println!("   Binary (hex): {}", hex_preview(&binary, 32));

    // Decode back
    let decoder_config = NestedDecoderConfig::new();
    let decoder = BinaryNestedDecoder::with_config(decoder_config);
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();
    
    if let Some(field) = decoded.get_field(100) {
        if let LnmpValue::NestedArray(users) = &field.value {
            println!("   Decoded {} users successfully", users.len());
            for (i, user) in users.iter().enumerate() {
                println!("     User {}: {} fields", i + 1, user.fields().len());
            }
        }
    }
}

fn multi_level_nesting() {
    // Create a 3-level nested structure: company -> department -> employee
    let mut employee = LnmpRecord::new();
    employee.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("Alice".to_string()),
    });
    employee.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(14532),
    });

    let mut department = LnmpRecord::new();
    department.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("Engineering".to_string()),
    });
    department.add_field(LnmpField {
        fid: 11,
        value: LnmpValue::NestedRecord(Box::new(employee)),
    });

    let mut company = LnmpRecord::new();
    company.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("TechCorp".to_string()),
    });
    company.add_field(LnmpField {
        fid: 200,
        value: LnmpValue::NestedRecord(Box::new(department)),
    });

    // Encode with nested support
    let config = NestedEncoderConfig::new()
        .with_max_depth(32);
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&company).unwrap();
    
    println!("   3-level structure: company -> department -> employee");
    println!("   Binary size: {} bytes", binary.len());
    println!("   Nesting depth: {}", get_max_depth(&company));

    // Decode back
    let decoder_config = NestedDecoderConfig::new()
        .with_max_depth(32);
    let decoder = BinaryNestedDecoder::with_config(decoder_config);
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();
    
    println!("   Decoded successfully");
    println!("   Round-trip depth: {}", get_max_depth(&decoded));
}

fn depth_limit_example() {
    // Create a deeply nested structure
    let mut current = LnmpRecord::new();
    current.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("level 5".to_string()),
    });

    // Build up 5 levels of nesting
    for level in (1..=4).rev() {
        let mut parent = LnmpRecord::new();
        parent.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String(format!("level {}", level)),
        });
        parent.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::NestedRecord(Box::new(current)),
        });
        current = parent;
    }

    println!("   Created structure with depth: {}", get_max_depth(&current));

    // Try encoding with low depth limit
    let config = NestedEncoderConfig::new()
        .with_max_depth(3);
    let encoder = BinaryNestedEncoder::with_config(config);
    
    match encoder.encode_nested_record(&current) {
        Ok(_) => println!("   ✗ Unexpected success with depth limit 3"),
        Err(e) => println!("   ✓ Correctly rejected: {}", e),
    }

    // Try with sufficient depth limit
    let config = NestedEncoderConfig::new()
        .with_max_depth(10);
    let encoder = BinaryNestedEncoder::with_config(config);
    
    match encoder.encode_nested_record(&current) {
        Ok(binary) => println!("   ✓ Accepted with depth limit 10 ({} bytes)", binary.len()),
        Err(e) => println!("   ✗ Unexpected error: {}", e),
    }
}

fn size_limit_example() {
    // Create a record with many fields to exceed size limit
    let mut large_record = LnmpRecord::new();
    for i in 1..=100 {
        large_record.add_field(LnmpField {
            fid: i,
            value: LnmpValue::String(format!("Field {} with some data", i)),
        });
    }

    println!("   Created record with {} fields", large_record.fields().len());

    // Try encoding with small size limit
    let config = NestedEncoderConfig::new()
        .with_max_record_size(Some(500));
    let encoder = BinaryNestedEncoder::with_config(config);
    
    match encoder.encode_nested_record(&large_record) {
        Ok(_) => println!("   ✗ Unexpected success with 500 byte limit"),
        Err(e) => println!("   ✓ Correctly rejected: {}", e),
    }

    // Try with no size limit
    let config = NestedEncoderConfig::new()
        .with_max_record_size(None);
    let encoder = BinaryNestedEncoder::with_config(config);
    
    match encoder.encode_nested_record(&large_record) {
        Ok(binary) => println!("   ✓ Accepted with no size limit ({} bytes)", binary.len()),
        Err(e) => println!("   ✗ Unexpected error: {}", e),
    }
}

fn canonical_ordering_example() {
    // Create a nested record with unsorted fields
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 30,
        value: LnmpValue::Int(3),
    });
    inner.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Int(1),
    });
    inner.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::Int(2),
    });

    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 200,
        value: LnmpValue::String("last".to_string()),
    });
    outer.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });
    outer.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::String("middle".to_string()),
    });

    println!("   Original field order:");
    println!("     Outer: F200, F50, F100 (unsorted)");
    println!("     Inner: F30, F10, F20 (unsorted)");

    // Encode to binary (automatically canonicalizes)
    let config = NestedEncoderConfig::new()
        .with_validate_canonical(true);
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode_nested_record(&outer).unwrap();
    
    // Decode back
    let decoder_config = NestedDecoderConfig::new();
    let decoder = BinaryNestedDecoder::with_config(decoder_config);
    let (decoded, _) = decoder.decode_nested_record(&binary).unwrap();
    
    println!("   Canonical field order after round-trip:");
    print!("     Outer: ");
    for field in decoded.fields() {
        print!("F{} ", field.fid);
    }
    println!();
    
    if let Some(field) = decoded.get_field(50) {
        if let LnmpValue::NestedRecord(nested) = &field.value {
            print!("     Inner: ");
            for field in nested.fields() {
                print!("F{} ", field.fid);
            }
            println!();
        }
    }
    
    println!("   ✓ Fields automatically sorted at all nesting levels");
}

// Helper functions

fn hex_preview(bytes: &[u8], max_len: usize) -> String {
    let preview_len = bytes.len().min(max_len);
    let hex: String = bytes[..preview_len]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    
    if bytes.len() > max_len {
        format!("{}...", hex)
    } else {
        hex
    }
}

fn get_max_depth(record: &LnmpRecord) -> usize {
    record
        .fields()
        .iter()
        .map(|field| field.value.depth())
        .max()
        .unwrap_or(0)
}
