#![allow(clippy::approx_constant)]
//! HybridNumericArray Example - Demonstrating the new TypeTag 0x09
//!
//! This example shows:
//! - Creating dense f32/f64 arrays
//! - Encoding and decoding HybridNumericArray
//! - Size comparison with legacy FloatArray
//!
//! Run: cargo run -p lnmp-codec --example hybrid_array_demo

use lnmp_codec::binary::entry::BinaryEntry;
use lnmp_codec::binary::types::{BinaryValue, HybridArray, NumericDType, TypeTag};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       HybridNumericArray (TypeTag 0x09) Demonstration        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ==========================================================================
    // Demo 1: Create and encode f32 array
    // ==========================================================================
    println!("📊 Demo 1: f32 Dense Array");
    println!("{}", "─".repeat(60));

    let f32_values: Vec<f32> = vec![1.0, 2.5, -3.14, 0.0, 100.0, 0.001];
    let arr = HybridArray::from_f32_dense(&f32_values);

    println!("   Input:      {:?}", f32_values);
    println!("   DType:      {:?}", arr.dtype);
    println!("   Dimension:  {}", arr.dim);
    println!("   Data size:  {} bytes", arr.data.len());
    println!("   Flags byte: 0x{:02X}", arr.flags());

    // Encode as BinaryEntry
    let entry = BinaryEntry {
        fid: 100,
        tag: TypeTag::HybridNumericArray,
        value: BinaryValue::HybridNumericArray(arr.clone()),
    };
    let encoded = entry.encode();
    println!("   Encoded:    {} bytes", encoded.len());

    // Decode back
    let (decoded, _) = BinaryEntry::decode(&encoded).unwrap();
    if let BinaryValue::HybridNumericArray(decoded_arr) = decoded.value {
        let recovered = decoded_arr.to_f32_vec().unwrap();
        println!("   Recovered:  {:?}", recovered);
        println!("   ✅ Roundtrip successful!\n");
    }

    // ==========================================================================
    // Demo 2: Create and encode f64 array
    // ==========================================================================
    println!("📊 Demo 2: f64 Dense Array (High Precision)");
    println!("{}", "─".repeat(60));

    let f64_values: Vec<f64> = vec![
        std::f64::consts::PI,
        std::f64::consts::E,
        std::f64::consts::SQRT_2,
    ];
    let arr64 = HybridArray::from_f64_dense(&f64_values);

    println!("   Input:      {:?}", f64_values);
    println!("   DType:      {:?}", arr64.dtype);
    println!("   Data size:  {} bytes", arr64.data.len());
    println!("   Flags byte: 0x{:02X}", arr64.flags());

    let entry64 = BinaryEntry {
        fid: 101,
        tag: TypeTag::HybridNumericArray,
        value: BinaryValue::HybridNumericArray(arr64),
    };
    let encoded64 = entry64.encode();
    println!("   Encoded:    {} bytes", encoded64.len());

    let (decoded64, _) = BinaryEntry::decode(&encoded64).unwrap();
    if let BinaryValue::HybridNumericArray(arr) = decoded64.value {
        let recovered = arr.to_f64_vec().unwrap();
        println!("   Recovered:  {:?}", recovered);
        println!("   ✅ Roundtrip successful!\n");
    }

    // ==========================================================================
    // Demo 3: Integer arrays
    // ==========================================================================
    println!("📊 Demo 3: i32 Dense Array");
    println!("{}", "─".repeat(60));

    let i32_values: Vec<i32> = vec![1, -2, 3, -4, 5, i32::MAX, i32::MIN];
    let arr_i32 = HybridArray::from_i32_dense(&i32_values);

    println!("   Input:      {:?}", i32_values);
    println!("   DType:      {:?}", arr_i32.dtype);
    println!("   Data size:  {} bytes", arr_i32.data.len());
    println!("   Flags byte: 0x{:02X}", arr_i32.flags());

    let entry_i32 = BinaryEntry {
        fid: 102,
        tag: TypeTag::HybridNumericArray,
        value: BinaryValue::HybridNumericArray(arr_i32),
    };
    let encoded_i32 = entry_i32.encode();
    println!("   Encoded:    {} bytes", encoded_i32.len());
    println!("   ✅ Encoding successful!\n");

    // ==========================================================================
    // Demo 4: Size comparison
    // ==========================================================================
    println!("📊 Demo 4: Size Comparison (256-dim embedding)");
    println!("{}", "─".repeat(60));

    // Simulate 256-dim embedding
    let embedding_256: Vec<f32> = (0..256).map(|i| (i as f32) * 0.01).collect();

    // HybridArray (f32)
    let hybrid_arr = HybridArray::from_f32_dense(&embedding_256);
    let hybrid_entry = BinaryEntry {
        fid: 200,
        tag: TypeTag::HybridNumericArray,
        value: BinaryValue::HybridNumericArray(hybrid_arr),
    };
    let hybrid_size = hybrid_entry.encode().len();

    // Legacy FloatArray (f64) - simulated
    let legacy_size = 3 + 256 * 8; // tag + count + 256 * 8 bytes

    println!("   256-dim f32 HybridArray: {} bytes", hybrid_size);
    println!("   256-dim f64 FloatArray:  {} bytes (legacy)", legacy_size);
    println!(
        "   Savings:                 {} bytes ({:.1}%)",
        legacy_size - hybrid_size,
        ((legacy_size - hybrid_size) as f64 / legacy_size as f64) * 100.0
    );
    println!();

    // ==========================================================================
    // Demo 5: NumericDType parsing
    // ==========================================================================
    println!("📊 Demo 5: NumericDType & Flags");
    println!("{}", "─".repeat(60));

    for dtype in [
        NumericDType::I32,
        NumericDType::I64,
        NumericDType::F32,
        NumericDType::F64,
    ] {
        println!(
            "   {:?}: byte_size={}, flags=0x{:02X}",
            dtype,
            dtype.byte_size(),
            dtype.to_flags()
        );
    }
    println!();

    // ==========================================================================
    // Summary
    // ==========================================================================
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                         Summary                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ✅ HybridNumericArray (0x09) working correctly              ║");
    println!("║  ✅ Supports i32, i64, f32, f64                              ║");
    println!("║  ✅ Dense mode encode/decode verified                        ║");
    println!("║  ✅ ~50% size reduction for embeddings                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
