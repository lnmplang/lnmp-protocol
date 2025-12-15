//! Example showing batch operations on embedding vectors.
//!
//! Run with: `cargo run --example batch_ops -p lnmp-embedding`

use lnmp_embedding::{SimilarityMetric, Vector};

fn main() {
    println!("Performing batch vector operations...\n");

    let vec_a = Vector::from_f32(vec![1.0, 0.0, 1.0, 0.0]);
    let vec_b = Vector::from_f32(vec![0.0, 1.0, 0.0, 1.0]);
    let vec_c = Vector::from_f32(vec![1.0, 1.0, 1.0, 1.0]);

    // 1. Cosine Similarity
    let sim_ab = vec_a.similarity(&vec_b, SimilarityMetric::Cosine).unwrap();
    println!("Cosine(A, B): {:.4} (Orthogonal)", sim_ab);

    let sim_ac = vec_a.similarity(&vec_c, SimilarityMetric::Cosine).unwrap();
    println!("Cosine(A, C): {:.4}", sim_ac);

    // 2. Normalization (if supported by Vector struct, otherwise skip or implement manually)
    let vec_c_normalized = vec_c.normalize().unwrap();
    println!("Normalized Vector C:");
    if let Ok(data) = vec_c_normalized.as_f32() {
        println!("{:?}", data);
    }

    // Verify normalized length is 1.0
    let norm = vec_c_normalized.similarity(&vec_c_normalized, SimilarityMetric::DotProduct).unwrap().sqrt();
    println!("Norm of Normalized C: {:.4}", norm);


    // 3. Batch processing (e.g., finding closest vector)
    let query = Vector::from_f32(vec![0.9, 0.1, 0.9, 0.1]);
    let candidates = [&vec_a, &vec_b, &vec_c];

    println!("\nFinding closest vector to Query: {:?}", query);

    let mut best_score = -1.0;
    let mut best_idx = 0;

    for (i, candidate) in candidates.iter().enumerate() {
        let score = query
            .similarity(candidate, SimilarityMetric::Cosine)
            .unwrap();
        println!("Score with candidate {}: {:.4}", i, score);

        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    println!(
        "Best match: Candidate {} (Score: {:.4})",
        best_idx, best_score
    );
}
