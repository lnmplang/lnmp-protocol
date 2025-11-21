//! FP16: Half-precision floating-point quantization
//!
//! This module implements FP16 (half-precision float) quantization which provides
//! 2x compression with minimal accuracy loss (~99.9%). It's a near-lossless option
//! that's faster than full precision but maintains very high accuracy.

use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use half::f16;
use lnmp_embedding::Vector;

/// Quantizes an embedding vector to FP16 (half-precision float)
///
/// FP16 uses 16 bits per value (vs 32 for F32), providing 2x compression
/// with minimal accuracy loss. This is ideal when high accuracy is needed
/// but storage/bandwidth is constrained.
///
/// # Arguments
/// * `emb` - The embedding vector to quantize
///
/// # Returns
/// * `Ok(QuantizedVector)` - Quantized vector with 2x compression
/// * `Err(QuantError)` - If quantization fails
///
/// # Example
/// ```
/// use lnmp_quant::fp16::quantize_fp16;
/// use lnmp_embedding::Vector;
///
/// let emb = Vector::from_f32(vec![0.1, -0.2, 0.3, -0.4]);
/// let quantized = quantize_fp16(&emb).unwrap();
/// assert_eq!(quantized.scheme, lnmp_quant::QuantScheme::FP16Passthrough);
/// ```
pub fn quantize_fp16(emb: &Vector) -> Result<QuantizedVector, QuantError> {
    // Validate input
    if emb.dtype != lnmp_embedding::EmbeddingType::F32 {
        return Err(QuantError::EncodingFailed(
            "Only F32 embeddings are supported for FP16 quantization".to_string(),
        ));
    }

    let values = emb
        .as_f32()
        .map_err(|e| QuantError::EncodingFailed(format!("Failed to convert to F32: {}", e)))?;

    if values.is_empty() {
        return Err(QuantError::InvalidDimension(
            "Cannot quantize empty vector".to_string(),
        ));
    }

    // Convert each f32 to f16 (2 bytes each)
    // Pre-allocate for performance
    let mut data = Vec::with_capacity(values.len() * 2);

    for val in values.iter() {
        let fp16 = f16::from_f32(*val);
        let bytes = fp16.to_le_bytes();
        data.extend_from_slice(&bytes);
    }

    // FP16 doesn't use scale/zero_point/min_val the same way
    // We set them to defaults since the data is self-contained
    Ok(QuantizedVector::new(
        emb.dim as u32,
        QuantScheme::FP16Passthrough,
        1.0, // Not used for FP16
        0,   // Not used for FP16
        0.0, // Not used for FP16
        data,
    ))
}

/// Dequantizes an FP16 quantized vector back to f32
///
/// Converts each 16-bit half-precision float back to 32-bit float.
///
/// # Arguments
/// * `qv` - The quantized vector to dequantize
///
/// # Returns
/// * `Ok(Vector)` - Restored f32 vector
/// * `Err(QuantError)` - If dequantization fails
pub fn dequantize_fp16(qv: &QuantizedVector) -> Result<Vector, QuantError> {
    if qv.scheme != QuantScheme::FP16Passthrough {
        return Err(QuantError::InvalidScheme(format!(
            "Expected FP16Passthrough, got {:?}",
            qv.scheme
        )));
    }

    let dim = qv.dim as usize;

    // Validate data length (should be exactly 2 bytes per value)
    if qv.data.len() != dim * 2 {
        return Err(QuantError::DataCorrupted(format!(
            "Expected {} bytes for {} dimensions, got {}",
            dim * 2,
            dim,
            qv.data.len()
        )));
    }

    let mut values = Vec::with_capacity(dim);

    // Read 2-byte chunks and convert from f16 to f32
    for chunk in qv.data.chunks_exact(2) {
        let bytes = [chunk[0], chunk[1]];
        let fp16 = f16::from_le_bytes(bytes);
        values.push(fp16.to_f32());
    }

    Ok(Vector::from_f32(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::SimilarityMetric;

    #[test]
    fn test_fp16_basic() {
        let vec = Vector::from_f32(vec![0.1, -0.2, 0.3, -0.4]);
        let quantized = quantize_fp16(&vec).unwrap();

        // Should use 2 bytes per value
        assert_eq!(quantized.data.len(), 4 * 2);
        assert_eq!(quantized.scheme, QuantScheme::FP16Passthrough);

        let restored = dequantize_fp16(&quantized).unwrap();
        assert_eq!(restored.dim, 4);
    }

    #[test]
    fn test_fp16_compression() {
        let values: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
        let vec = Vector::from_f32(values.clone());

        let quantized = quantize_fp16(&vec).unwrap();

        // Original: 512 * 4 = 2048 bytes
        // FP16: 512 * 2 = 1024 bytes
        // Compression: 2x
        let original_bytes = 512 * 4;
        let quantized_bytes = quantized.data.len();
        let compression_ratio = original_bytes as f32 / quantized_bytes as f32;

        assert_eq!(compression_ratio, 2.0);
    }

    #[test]
    fn test_fp16_accuracy() {
        // FP16 should be very accurate (near-lossless)
        let values: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let vec = Vector::from_f32(values.clone());

        let quantized = quantize_fp16(&vec).unwrap();
        let restored = dequantize_fp16(&quantized).unwrap();

        // Check cosine similarity
        let similarity = vec.similarity(&restored, SimilarityMetric::Cosine).unwrap();

        // FP16 should be very accurate
        assert!(similarity > 0.999, "Similarity: {}", similarity);

        // Check element-wise accuracy
        let restored_values = restored.as_f32().unwrap();
        for (orig, rest) in values.iter().zip(restored_values.iter()) {
            let error = (orig - rest).abs();
            // FP16 has ~3 decimal digits of precision
            assert!(error < 0.001, "Error: {} for value {}", error, orig);
        }
    }

    #[test]
    fn test_fp16_roundtrip() {
        let vec = Vector::from_f32(vec![
            0.123456, -0.789012, 0.345678, -0.901234, 0.567890, -0.234567,
        ]);

        let quantized = quantize_fp16(&vec).unwrap();
        let restored = dequantize_fp16(&quantized).unwrap();

        // Verify dimensions
        assert_eq!(restored.dim, 6);

        // Verify high accuracy
        let similarity = vec.similarity(&restored, SimilarityMetric::Cosine).unwrap();
        assert!(similarity > 0.999);
    }

    #[test]
    fn test_fp16_edge_cases() {
        // Test with edge values
        let vec = Vector::from_f32(vec![0.0, 1.0, -1.0, 0.5, -0.5]);
        let quantized = quantize_fp16(&vec).unwrap();
        let restored = dequantize_fp16(&quantized).unwrap();

        let restored_values = restored.as_f32().unwrap();
        let original_values = vec.as_f32().unwrap();

        for (orig, rest) in original_values.iter().zip(restored_values.iter()) {
            let error = (orig - rest).abs();
            assert!(error < 0.0001, "Error: {} for value {}", error, orig);
        }
    }

    #[test]
    fn test_fp16_large_vector() {
        // Test with typical embedding dimension
        let values: Vec<f32> = (0..1536).map(|i| ((i % 100) as f32) / 100.0).collect();
        let vec = Vector::from_f32(values);

        let quantized = quantize_fp16(&vec).unwrap();

        // 1536 * 2 = 3072 bytes
        assert_eq!(quantized.data.len(), 3072);
        assert_eq!(quantized.compression_ratio(), 2.0);

        let restored = dequantize_fp16(&quantized).unwrap();
        assert_eq!(restored.dim, 1536);

        // Verify high accuracy
        let similarity = vec.similarity(&restored, SimilarityMetric::Cosine).unwrap();
        assert!(similarity > 0.999);
    }

    #[test]
    fn test_fp16_empty_fails() {
        let vec = Vector::from_f32(vec![]);
        let result = quantize_fp16(&vec);
        assert!(result.is_err());
    }

    #[test]
    fn test_fp16_data_corruption() {
        // Create a valid quantized vector
        let vec = Vector::from_f32(vec![0.1, 0.2, 0.3]);
        let mut quantized = quantize_fp16(&vec).unwrap();

        // Corrupt the data (remove one byte to make length invalid)
        quantized.data.pop();

        let result = dequantize_fp16(&quantized);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QuantError::DataCorrupted(_)));
    }

    #[test]
    fn test_fp16_special_values() {
        // Test with very small and very large values
        let vec = Vector::from_f32(vec![0.000001, 10000.0, -0.000001, -10000.0]);
        let quantized = quantize_fp16(&vec).unwrap();
        let restored = dequantize_fp16(&quantized).unwrap();

        // FP16 has limited range and precision, but should handle these
        assert_eq!(restored.dim, 4);

        // Note: Very large/small values may lose precision with FP16
        // but the conversion should not fail
    }
}
