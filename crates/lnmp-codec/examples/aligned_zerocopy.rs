#![allow(clippy::approx_constant)]
//! Aligned Zero-Copy Example
//!
//! Demonstrates the aligned-zerocopy feature for true zero-copy typed slice access.
//!
//! Run with feature: cargo run -p lnmp-codec --example aligned_zerocopy --features aligned-zerocopy
//! Run without feature: cargo run -p lnmp-codec --example aligned_zerocopy

#[cfg(feature = "aligned-zerocopy")]
fn main() {
    use lnmp_codec::binary::types::HybridArray;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Aligned Zero-Copy Feature Demonstration              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Demo 1: f32 array
    println!("📊 Demo 1: f32 Array");
    println!("{}", "─".repeat(60));

    let f32_values = vec![1.0f32, 2.5, -3.14, 0.0, 100.0];
    let arr = HybridArray::from_f32_dense(&f32_values);

    println!("   Original: {:?}", f32_values);

    match arr.as_f32_slice() {
        Some(slice) => {
            println!("   ✅ ALIGNED! Zero-copy slice access");
            println!("   Slice: {:?}", slice);
            println!("   Length: {}", slice.len());
            println!("   No allocation! 🚀\n");
        }
        None => {
            println!("   ⚠️  Not aligned, using fallback");
            let vec = arr.to_f32_vec().unwrap();
            println!("   Vec: {:?}", vec);
            println!("   (Allocated {} bytes)\n", vec.len() * 4);
        }
    }

    // Demo 2: f64 array
    println!("📊 Demo 2: f64 Array (High Precision)");
    println!("{}", "─".repeat(60));

    let f64_values = vec![std::f64::consts::PI, std::f64::consts::E];
    let arr = HybridArray::from_f64_dense(&f64_values);

    println!("   Original: {:?}", f64_values);

    match arr.as_f64_slice() {
        Some(slice) => {
            println!("   ✅ ALIGNED! Zero-copy slice");
            println!("   Slice: {:?}", slice);
        }
        None => {
            println!("   ⚠️  Not aligned");
            let vec = arr.to_f64_vec().unwrap();
            println!("   Vec: {:?}", vec);
        }
    }
    println!();

    // Demo 3: Alignment statistics
    println!("📊 Demo 3: Alignment Statistics");
    println!("{}", "─".repeat(60));

    let mut aligned_count = 0;
    let total = 100;

    for _ in 0..total {
        let arr = HybridArray::from_f32_dense(&[1.0, 2.0, 3.0]);
        if arr.as_f32_slice().is_some() {
            aligned_count += 1;
        }
    }

    println!("   Total arrays created: {}", total);
    println!("   Aligned: {}", aligned_count);
    println!("   Not aligned: {}", total - aligned_count);
    println!(
        "   Alignment rate: {:.1}%\n",
        (aligned_count as f64 / total as f64) * 100.0
    );

    // Demo 4: Performance comparison
    println!("📊 Demo 4: Performance Impact");
    println!("{}", "─".repeat(60));

    let arr = HybridArray::from_f32_dense(&(0..256).map(|i| i as f32).collect::<Vec<_>>());

    use std::time::Instant;

    // Benchmark vec access
    let start = Instant::now();
    for _ in 0..10000 {
        let _vec = arr.to_f32_vec().unwrap();
    }
    let vec_time = start.elapsed();

    // Benchmark slice access
    let start = Instant::now();
    for _ in 0..10000 {
        match arr.as_f32_slice() {
            Some(_slice) => {}
            None => {
                let _ = arr.to_f32_vec();
            }
        }
    }
    let slice_time = start.elapsed();

    println!("   to_f32_vec():   {:?}", vec_time);
    println!("   as_f32_slice(): {:?}", slice_time);

    if slice_time < vec_time {
        let speedup = vec_time.as_nanos() as f64 / slice_time.as_nanos() as f64;
        println!("   Speedup: {:.1}x faster! 🚀\n", speedup);
    } else {
        println!("   (Platform may not guarantee alignment)\n");
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                         Summary                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ✅ aligned-zerocopy feature enabled                         ║");
    println!("║  ✅ Runtime alignment check (safe)                           ║");
    println!("║  ✅ Automatic fallback to to_f32_vec()                       ║");
    println!("║  ✅ True zero-copy when aligned                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[cfg(not(feature = "aligned-zerocopy"))]
fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Feature 'aligned-zerocopy' Disabled              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("This example requires the 'aligned-zerocopy' feature.");
    println!("\nTo run with the feature enabled:");
    println!("  cargo run -p lnmp-codec --example aligned_zerocopy --features aligned-zerocopy");
    println!("\nTo enable in your project:");
    println!("  [dependencies]");
    println!("  lnmp-codec = {{ version = \"0.5.16\", features = [\"aligned-zerocopy\"] }}");
}
