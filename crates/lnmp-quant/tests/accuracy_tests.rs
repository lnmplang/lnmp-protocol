use lnmp_embedding::{SimilarityMetric, Vector};
use lnmp_quant::{dequantize_embedding, quantize_embedding, QuantScheme};

#[test]
fn test_semantic_preservation() {
    // Simulate a realistic embedding vector (normalized)
    let values: Vec<f32> = vec![
        0.123, -0.456, 0.789, -0.234, 0.567, -0.890, 0.345, -0.678, 0.901, -0.123, 0.456, -0.789,
        0.234, -0.567, 0.890, -0.345, 0.678, -0.901, 0.012, -0.345, 0.678, -0.901, 0.234, -0.567,
        0.890, -0.123, 0.456, -0.789, 0.345, -0.678, 0.901, -0.234,
    ];

    let original = Vector::from_f32(values);
    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    // Test cosine similarity
    let cosine_sim = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();
    assert!(
        cosine_sim > 0.98,
        "Cosine similarity should be > 0.98, got {}",
        cosine_sim
    );

    // Test dot product preservation
    let dot_original = original
        .similarity(&original, SimilarityMetric::DotProduct)
        .unwrap();
    let dot_restored = restored
        .similarity(&restored, SimilarityMetric::DotProduct)
        .unwrap();
    let dot_diff = (dot_original - dot_restored).abs() / dot_original;
    assert!(
        dot_diff < 0.1,
        "Dot product should be preserved within 10%, got diff ratio {}",
        dot_diff
    );
}

#[test]
fn test_quantization_error_bounds() {
    let values: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0) - 0.5).collect();
    let original = Vector::from_f32(values.clone());

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let restored_values = restored.as_f32().unwrap();

    // Calculate mean absolute error
    let mut total_error = 0.0f32;
    for (i, &orig) in values.iter().enumerate() {
        let error = (orig - restored_values[i]).abs();
        total_error += error;
    }
    let mean_error = total_error / values.len() as f32;

    // Mean error should be small
    assert!(
        mean_error < 0.01,
        "Mean absolute error should be < 0.01, got {}",
        mean_error
    );
}

#[test]
fn test_scale_calculation_accuracy() {
    let original = Vector::from_f32(vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();

    // For range [-1.0, 1.0], scale should be approximately 2.0 / 255
    let expected_scale = 2.0 / 255.0;
    let scale_diff = (quantized.scale - expected_scale).abs();

    assert!(
        scale_diff < 0.001,
        "Scale calculation should be accurate, expected ~{}, got {}",
        expected_scale,
        quantized.scale
    );
}

#[test]
fn test_similarity_with_other_vectors() {
    // Test that quantization preserves relative similarity between vectors
    let vec1 = Vector::from_f32(vec![1.0, 0.0, 0.0, 0.0]);
    let vec2 = Vector::from_f32(vec![0.0, 1.0, 0.0, 0.0]);
    let vec3 = Vector::from_f32(vec![0.707, 0.707, 0.0, 0.0]); // Between vec1 and vec2

    let q1 = quantize_embedding(&vec1, QuantScheme::QInt8).unwrap();
    let q2 = quantize_embedding(&vec2, QuantScheme::QInt8).unwrap();
    let q3 = quantize_embedding(&vec3, QuantScheme::QInt8).unwrap();

    let r1 = dequantize_embedding(&q1).unwrap();
    let r2 = dequantize_embedding(&q2).unwrap();
    let r3 = dequantize_embedding(&q3).unwrap();

    // Original similarities
    let orig_sim_12 = vec1.similarity(&vec2, SimilarityMetric::Cosine).unwrap();
    let orig_sim_13 = vec1.similarity(&vec3, SimilarityMetric::Cosine).unwrap();
    let orig_sim_23 = vec2.similarity(&vec3, SimilarityMetric::Cosine).unwrap();

    // Restored similarities
    let rest_sim_12 = r1.similarity(&r2, SimilarityMetric::Cosine).unwrap();
    let rest_sim_13 = r1.similarity(&r3, SimilarityMetric::Cosine).unwrap();
    let rest_sim_23 = r2.similarity(&r3, SimilarityMetric::Cosine).unwrap();

    // Similarities should be preserved
    assert!(
        (orig_sim_12 - rest_sim_12).abs() < 0.1,
        "Similarity between orthogonal vectors should be preserved"
    );
    assert!(
        (orig_sim_13 - rest_sim_13).abs() < 0.1,
        "Similarity should be preserved"
    );
    assert!(
        (orig_sim_23 - rest_sim_23).abs() < 0.1,
        "Similarity should be preserved"
    );
}

#[test]
fn test_normalized_embedding_quantization() {
    // Test with L2-normalized embedding (common in real embeddings)
    let mut values: Vec<f32> = (0..128).map(|i| i as f32 / 128.0).collect();

    // Normalize
    let norm: f32 = values.iter().map(|x| x * x).sum::<f32>().sqrt();
    for val in &mut values {
        *val /= norm;
    }

    let original = Vector::from_f32(values);
    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.99,
        "Normalized embedding should preserve high accuracy: {}",
        similarity
    );
}
