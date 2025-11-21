use lnmp_embedding::{SimilarityMetric, Vector};
use lnmp_quant::{dequantize_embedding, quantize_embedding, QuantScheme};

fn main() {
    let original = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
    let restored = dequantize_embedding(&quantized).unwrap();

    let orig_vals = original.as_f32().unwrap();
    let rest_vals = restored.as_f32().unwrap();

    println!("Original: {:?}", orig_vals);
    println!("Restored: {:?}", rest_vals);

    let similarity = original
        .similarity(&restored, SimilarityMetric::Cosine)
        .unwrap();

    println!("Cosine similarity: {}", similarity);
    println!("Scale: {}", quantized.scale);
    println!("Zero point: {}", quantized.zero_point);

    // Also try with more diverse values
    let original2 = Vector::from_f32(vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    let quantized2 = quantize_embedding(&original2, QuantScheme::QInt8).unwrap();
    let restored2 = dequantize_embedding(&quantized2).unwrap();

    let similarity2 = original2
        .similarity(&restored2, SimilarityMetric::Cosine)
        .unwrap();

    println!("\nWith negative values:");
    println!("Cosine similarity: {}", similarity2);
}
