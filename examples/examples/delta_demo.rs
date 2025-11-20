#![allow(clippy::useless_vec)]
//! LNMP Embedding Delta Demo
//!
//! This example demonstrates the delta encoding feature for embeddings:
//! - Computing deltas between embedding updates
//! - Encoding deltas to binary format
//! - Comparing delta vs full encoding sizes
//! - Applying deltas to reconstruct embeddings
//! - Adaptive strategy for delta vs full encoding

use lnmp_embedding::{UpdateStrategy, Vector, VectorDelta};

fn main() {
    println!("=== LNMP Embedding Delta Demo ===\n");

    // Demo 1: Basic Delta Operations
    demo_basic_delta();

    // Demo 2: Size Comparison
    demo_size_comparison();

    // Demo 3: Streaming Updates
    demo_streaming_updates();

    // Demo 4: Adaptive Strategy
    demo_adaptive_strategy();

    println!("\n=== All delta demos completed successfully! ===");
}

fn demo_basic_delta() {
    println!("--- Demo 1: Basic Delta Operations ---");

    // Create two embeddings with small differences
    let old_embedding = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    let new_embedding = Vector::from_f32(vec![0.1, 0.25, 0.3, 0.35, 0.5]);

    println!("Original: {:?}", old_embedding.as_f32().unwrap());
    println!("Updated:  {:?}", new_embedding.as_f32().unwrap());

    // Compute delta
    let delta = VectorDelta::from_vectors(&old_embedding, &new_embedding, 1001).unwrap();
    println!("Delta has {} changes", delta.changes.len());
    for change in &delta.changes {
        println!("  Index {}: delta = {:.4}", change.index, change.delta);
    }

    // Apply delta
    let reconstructed = delta.apply(&old_embedding).unwrap();
    assert_eq!(new_embedding, reconstructed);
    println!("✓ Delta application verified\n");
}

fn demo_size_comparison() {
    println!("--- Demo 2: Size Comparison ---");

    let dimensions = [128, 256, 512, 768, 1536];
    let change_percentages = [1, 5, 10, 30];

    println!("Dimension | Change% | Full Size | Delta Size | Savings");
    println!("----------|---------|-----------|------------|--------");

    for &dim in &dimensions {
        let old = Vector::from_f32(vec![0.1; dim]);

        for &change_pct in &change_percentages {
            let num_changes = (dim * change_pct) / 100;
            let mut new_data = vec![0.1; dim];

            // Change specific indices using iterator
            new_data
                .iter_mut()
                .take(num_changes)
                .for_each(|v| *v += 0.01);
            let new = Vector::from_f32(new_data);

            let delta = VectorDelta::from_vectors(&old, &new, 1).unwrap();
            let full_size = 4 + (dim * 4); // Header + F32 data
            let delta_size = delta.encoded_size();
            let savings = ((full_size - delta_size) as f32 / full_size as f32) * 100.0;

            println!(
                "{:9} | {:6}% | {:9} | {:10} | {:.1}%",
                dim, change_pct, full_size, delta_size, savings
            );
        }
    }
    println!();
}

fn demo_streaming_updates() {
    println!("--- Demo 3: Streaming Agent Updates ---");

    // Simulate an agent's embedding evolving over time
    let mut current_state = Vector::from_f32(vec![0.5; 512]);
    let base_id = 2001;

    println!("Agent reasoning with incremental updates...");

    for step in 1..=5 {
        // Simulate small context changes
        let mut new_data = current_state.as_f32().unwrap();
        // Change 10 values each step (~2%)
        new_data
            .iter_mut()
            .skip((step - 1) * 10)
            .take(10)
            .for_each(|v| *v += 0.01);
        let new_state = Vector::from_f32(new_data);

        let delta = VectorDelta::from_vectors(&current_state, &new_state, base_id).unwrap();
        let encoded = delta.encode().unwrap();

        println!(
            "Step {}: {} changes, {} bytes (vs 2052 bytes full)",
            step,
            delta.changes.len(),
            encoded.len()
        );

        // Apply delta
        current_state = delta.apply(&current_state).unwrap();
        assert_eq!(current_state, new_state);
    }

    println!("✓ All streaming updates verified\n");
}

fn demo_adaptive_strategy() {
    println!("--- Demo 4: Adaptive Strategy ---");

    let strategy = UpdateStrategy::default(); // 30% threshold
    let embedding = Vector::from_f32(vec![0.1; 1536]);

    // Test with different change ratios
    let test_cases = [
        (15, "Small update"),
        (154, "Medium update"),
        (461, "Large update"),
        (1536, "Complete change"),
    ];

    println!("Testing adaptive strategy (30% threshold):");
    for (num_changes, description) in test_cases {
        let mut new_data = vec![0.1; 1536];
        new_data
            .iter_mut()
            .take(num_changes.min(1536))
            .for_each(|v| *v += 0.01);
        let new_embedding = Vector::from_f32(new_data);

        let delta = VectorDelta::from_vectors(&embedding, &new_embedding, 1).unwrap();
        let should_use_delta = strategy.should_use_delta(&delta, 1536);
        let change_ratio = delta.change_ratio(1536) * 100.0;

        let recommendation = if should_use_delta {
            "Use DELTA"
        } else {
            "Use FULL"
        };

        println!(
            "{}: {:.1}% changed → {}",
            description, change_ratio, recommendation
        );
    }

    println!("\n✓ Adaptive strategy working correctly\n");
}
