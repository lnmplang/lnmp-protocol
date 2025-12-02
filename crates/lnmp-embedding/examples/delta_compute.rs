//! Example showing how to compute and apply embedding deltas.
//!
//! Run with: `cargo run --example delta_compute -p lnmp-embedding`

use lnmp_embedding::{Vector, VectorDelta};

fn main() {
    println!("Computing embedding delta...\n");

    // 1. Base vector (e.g., previous state)
    let base_vec = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    println!("Base vector: {:?}", base_vec.as_f32().unwrap());

    // 2. Target vector (e.g., new state)
    // Only index 2 and 4 changed significantly
    let target_vec = Vector::from_f32(vec![0.1, 0.2, 0.8, 0.4, 0.9]);
    println!("Target vector: {:?}", target_vec.as_f32().unwrap());

    // 3. Compute delta
    // Base ID is 0 for this example
    let delta = VectorDelta::from_vectors(&base_vec, &target_vec, 0).unwrap();

    println!("\nComputed Delta:");
    println!("Number of changes: {}", delta.changes.len());
    println!("Change ratio: {:.2}%", delta.change_ratio(5) * 100.0);

    for change in &delta.changes {
        println!("  Index {}: delta = {:+.2}", change.index, change.delta);
    }

    // 4. Apply delta to base
    let current = delta.apply(&base_vec).unwrap();

    println!("\nAfter applying delta: {:?}", current.as_f32().unwrap());
    assert_eq!(current, target_vec);
    println!("\n✅ Delta applied successfully!");

    // 5. Show encoding efficiency
    let encoded = delta.encode().unwrap();
    println!("\nEncoding Efficiency:");
    println!("  Delta size: {} bytes", encoded.len());
    println!("  Full vector size: {} bytes", base_vec.data.len());
    println!(
        "  Compression: {:.1}%",
        (1.0 - (encoded.len() as f32 / base_vec.data.len() as f32)) * 100.0
    );
}
