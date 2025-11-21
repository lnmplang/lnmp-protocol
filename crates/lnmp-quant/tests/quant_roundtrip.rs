use lnmp_embedding::{SimilarityMetric, Vector};
use lnmp_quant::{dequantize_embedding, quantize_embedding, QuantScheme};

#[test]
fn test_qint8_roundtrip_128dim() {
    let values: Vec<f32> = (0..128).map(|i| (i as f32 / 128.0) - 0.5).collect();
    let original = Vector::from_f32(values);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.98,
        "Cosine similarity too low for 128-dim: {}",
        similarity
    );
}

#[test]
fn test_qint8_roundtrip_512dim() {
    let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
    let original = Vector::from_f32(values);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.98,
        "Cosine similarity too low for 512-dim: {}",
        similarity
    );
}

#[test]
fn test_qint8_roundtrip_1536dim() {
    let values: Vec<f32> = (0..1536).map(|i| (i as f32 / 1536.0) - 0.5).collect();
    let original = Vector::from_f32(values);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.98,
        "Cosine similarity too low for 1536-dim: {}",
        similarity
    );

    // Verify compression ratio
    assert_eq!(quantized.compression_ratio(), 4.0);
    assert_eq!(quantized.data_size(), 1536);
}

#[test]
fn test_zero_vector() {
    let original = Vector::from_f32(vec![0.0; 100]);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let restored_values = restored.as_f32().unwrap();
    for &val in &restored_values {
        assert!(
            val.abs() < 0.01,
            "Zero vector should remain close to zero after roundtrip"
        );
    }
}

#[test]
fn test_uniform_values() {
    let original = Vector::from_f32(vec![0.5; 256]);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let restored_values = restored.as_f32().unwrap();
    let first_val = restored_values[0];

    // All values should be approximately the same
    for &val in &restored_values {
        assert!(
            (val - first_val).abs() < 0.01,
            "Uniform values should remain uniform"
        );
    }
}

#[test]
fn test_extreme_range() {
    let original = Vector::from_f32(vec![-100.0, -50.0, 0.0, 50.0, 100.0]);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.98,
        "Extreme range should still preserve similarity: {}",
        similarity
    );
}

#[test]
fn test_mixed_positive_negative() {
    let values: Vec<f32> = (0..100)
        .map(|i| if i % 2 == 0 { i as f32 } else { -(i as f32) })
        .collect();
    let original = Vector::from_f32(values);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.99,
        "Mixed positive/negative should preserve similarity: {}",
        similarity
    );
}

#[test]
fn test_small_values() {
    let original = Vector::from_f32(vec![0.001, 0.002, 0.003, 0.004, 0.005]);

    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    assert!(
        similarity > 0.95,
        "Small values should still preserve similarity: {}",
        similarity
    );
}
