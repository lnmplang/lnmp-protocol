//! Embedding operations example
//!
//! This example demonstrates:
//! - Creating embedding vectors
//! - Vector delta computation
//! - Delta encoding/decoding

use lnmp::embedding::{Vector, VectorDelta};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Embedding Example ===\n");

    // 1. Create F32 vectors
    println!("--- F32 Vectors ---");
    let vec1 = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    println!("Vector 1: dim={}, bytes={}", vec1.dim, vec1.data.len());

    let vec2 = Vector::from_f32(vec![0.1, 0.25, 0.3, 0.45, 0.5]);
    println!("Vector 2: dim={}, bytes={}", vec2.dim, vec2.data.len());

    // 2. Compute delta between vectors
    println!("\n--- Vector Delta ---");
    let delta = VectorDelta::from_vectors(&vec1, &vec2, 0)?;
    println!("Delta base_id: {}", delta.base_id);
    println!("Delta changes: {} indices", delta.changes.len());

    for change in &delta.changes {
        println!("  Index {}: value changed", change.index);
    }

    // 3. Encode delta
    println!("\n--- Delta Encoding ---");
    let encoded = delta.encode()?;
    println!("Encoded delta size: {} bytes", encoded.len());

    // 4. Decode delta
    println!("\n--- Delta Decoding ---");
    let decoded = VectorDelta::decode(&encoded)?;
    println!(
        "Decoded delta: base_id={}, changes={}",
        decoded.base_id,
        decoded.changes.len()
    );

    // 5. Apply delta to vector
    println!("\n--- Applying Delta ---");
    let updated = delta.apply(&vec1)?;
    println!("Applied delta successfully");
    println!("Updated vector dim: {}", updated.dim);

    println!("\n✅ Embedding operations completed successfully!");

    Ok(())
}
