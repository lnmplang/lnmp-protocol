#![allow(clippy::approx_constant)]
//! Alignment Rate Test - Direct Binary Frames
//!
//! Tests real-world alignment when HybridArray data comes from binary frames at varying offsets.
//!
//! Run: cargo run -p lnmp-codec --example alignment_test --features aligned-zerocopy --release

#[cfg(feature = "aligned-zerocopy")]
fn main() {
    use lnmp_codec::binary::types::{HybridArray, NumericDType};

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Alignment Test: Binary Frame Offsets                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test 1: Direct construction (baseline)
    println!("📊 Test 1: Direct Construction (Baseline)");
    println!("{}", "─".repeat(60));

    let mut aligned_count = 0;
    let total = 1000;

    for _ in 0..total {
        let values: Vec<f32> = (0..256).map(|x| x as f32 * 0.01).collect();
        let arr = HybridArray::from_f32_dense(&values);

        if arr.as_f32_slice().is_some() {
            aligned_count += 1;
        }
    }

    let rate = (aligned_count as f64 / total as f64) * 100.0;
    println!("   Aligned: {}/{} ({:.1}%)", aligned_count, total, rate);
    println!("   → Direct Vec<u8> construction\n");

    // Test 2: Simulate frame offset variations
    println!("📊 Test 2: Simulated Frame Offsets");
    println!("{}", "─".repeat(60));

    for offset in 0..16 {
        let mut aligned_count = 0;
        let total = 100;

        for _ in 0..total {
            // Create a buffer with given offset
            let mut buffer = vec![0u8; offset];

            // Append f32 data
            let values: Vec<f32> = (0..128).map(|x| x as f32).collect();
            for v in &values {
                buffer.extend_from_slice(&v.to_le_bytes());
            }

            // Extract data starting from offset
            let data = buffer[offset..].to_vec();

            let arr = HybridArray {
                dtype: NumericDType::F32,
                sparse: false,
                dim: values.len(),
                data,
            };

            if arr.as_f32_slice().is_some() {
                aligned_count += 1;
            }
        }

        let rate = (aligned_count as f64 / total as f64) * 100.0;
        let symbol = if rate > 90.0 {
            "✅"
        } else if rate > 50.0 {
            "⚠️"
        } else {
            "❌"
        };
        println!(
            "   Offset {:2}: {}/{} aligned ({:5.1}%) {}",
            offset, aligned_count, total, rate, symbol
        );
    }
    println!();

    // Test 3: Slice from larger buffer (realistic decode_view scenario)
    println!("📊 Test 3: Slice from Network Buffer");
    println!("{}", "─".repeat(60));

    let mut results = vec![];

    for buffer_start_offset in [0, 1, 2, 3, 4, 5, 6, 7, 8, 15, 31, 63] {
        let mut aligned_count = 0;
        let total = 100;

        for _ in 0..total {
            // Simulate network buffer with random start offset
            let mut buffer = vec![0xFF; buffer_start_offset];

            // Add some header bytes (version, flags, etc.)
            buffer.extend_from_slice(&[0x05, 0x00, 0x02]); // version, flags, field count

            // Add field: FID=50, TypeTag=0x09 (HybridArray)
            buffer.push(50); // FID
            buffer.push(0x09); // TypeTag
            buffer.push(0x02); // flags (f32, dense)
            buffer.push(128); // dim

            // Add f32 data
            let values: Vec<f32> = (0..128).map(|x| x as f32).collect();
            for v in &values {
                buffer.extend_from_slice(&v.to_le_bytes());
            }

            // Extract HybridArray data portion (after header)
            let data_start = buffer_start_offset + 8; // Skip to array data
            let data = buffer[data_start..].to_vec();

            let arr = HybridArray {
                dtype: NumericDType::F32,
                sparse: false,
                dim: 128,
                data,
            };

            if arr.as_f32_slice().is_some() {
                aligned_count += 1;
            }
        }

        let rate = (aligned_count as f64 / total as f64) * 100.0;
        results.push((buffer_start_offset, rate));
        println!(
            "   Buffer offset {:2}: {}/{} aligned ({:5.1}%)",
            buffer_start_offset, aligned_count, total, rate
        );
    }
    println!();

    // Calculate statistics
    let avg_rate = results.iter().map(|(_, r)| r).sum::<f64>() / results.len() as f64;
    let min_rate = results.iter().map(|(_, r)| *r).fold(100.0, f64::min);
    let max_rate = results.iter().map(|(_, r)| *r).fold(0.0, f64::max);

    // Summary
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Statistics                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Average alignment rate: {:<10.1}%                        ║",
        avg_rate
    );
    println!(
        "║  Minimum rate:           {:<10.1}%                        ║",
        min_rate
    );
    println!(
        "║  Maximum rate:           {:<10.1}%                        ║",
        max_rate
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    if avg_rate < 50.0 {
        println!("║  ❌ LOW alignment rate in real scenarios                    ║");
        println!("║  → as_f32_slice() mostly returns None                      ║");
        println!("║  → MUST use to_f32_vec() as primary path                   ║");
        println!("║  → aligned-zerocopy provides minimal benefit               ║");
    } else if avg_rate < 90.0 {
        println!("║  ⚠️  VARIABLE alignment rate                                ║");
        println!(
            "║  → as_f32_slice() works {:.0}% of the time                   ║",
            avg_rate
        );
        println!("║  → Fallback to to_f32_vec() essential                      ║");
        println!("║  → aligned-zerocopy helps when aligned                     ║");
    } else {
        println!("║  ✅ HIGH alignment rate!                                    ║");
        println!(
            "║  → as_f32_slice() works {:.0}% of the time                   ║",
            avg_rate
        );
        println!("║  → Zero-copy usually successful                            ║");
        println!("║  → aligned-zerocopy very useful!                           ║");
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[cfg(not(feature = "aligned-zerocopy"))]
fn main() {
    println!("Feature 'aligned-zerocopy' required!");
    println!("Run: cargo run -p lnmp-codec --example alignment_test --features aligned-zerocopy");
}
