//! Binary: 1-bit quantization for maximum compression
//!
//! This module implements binary (1-bit) quantization where each value is represented
//! by a single bit based on its sign relative to the mean. This provides 32x compression
//! and is particularly effective for similarity search tasks.

use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;

/// Quantizes an embedding vector to binary (1-bit per value)
///
/// Binary quantization uses a simple threshold-based approach:
/// - Calculate the mean of all values
/// - Each value above mean → bit 1
/// - Each value below mean → bit 0
///
/// This is extremely lossy but preserves relative ordering and is fast for similarity.
///
/// # Arguments
/// * `emb` - The embedding vector to quantize
///
/// # Returns
/// * `Ok(QuantizedVector)` - Quantized vector with 32x compression
/// * `Err(QuantError)` - If quantization fails
///
/// # Example
/// ```
/// use lnmp_quant::binary::quantize_binary;
/// use lnmp_embedding::Vector;
///
/// let emb = Vector::from_f32(vec![0.1, -0.2, 0.3, -0.4]);
/// let quantized = quantize_binary(&emb).unwrap();
/// assert_eq!(quantized.scheme, lnmp_quant::QuantScheme::Binary);
/// ```
pub fn quantize_binary(emb: &Vector) -> Result<QuantizedVector, QuantError> {
    // Validate input
    if emb.dtype != lnmp_embedding::EmbeddingType::F32 {
        return Err(QuantError::EncodingFailed(
            "Only F32 embeddings are supported for binary quantization".to_string(),
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

    // Calculate mean for thresholding
    let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;

    // Pack bits: 8 values per byte
    let num_bytes = values.len().div_ceil(8); // Round up
    let mut data = Vec::with_capacity(num_bytes);

    for chunk in values.chunks(8) {
        let mut byte = 0u8;
        for (i, &val) in chunk.iter().enumerate() {
            if val >= mean {
                byte |= 1 << i; // Set bit if value >= mean
            }
            // Otherwise bit remains 0
        }
        data.push(byte);
    }

    // Store mean in min_val field for use during dequantization
    // Scale and zero_point are not really used for binary quantization
    Ok(QuantizedVector::new(
        emb.dim as u32,
        QuantScheme::Binary,
        1.0,  // Not used for binary
        0,    // Not used for binary
        mean, // Store mean as min_val for reconstruction
        data,
    ))
}

/// Dequantizes a binary quantized vector back to f32
///
/// Converts each bit back to either +1.0 or -1.0:
/// - Bit = 1 → +1.0 (above mean)
/// - Bit = 0 → -1.0 (below mean)
///
/// This creates a normalized vector suitable for similarity comparison.
///
/// # Arguments
/// * `qv` - The quantized vector to dequantize
///
/// # Returns
/// * `Ok(Vector)` - Restored f32 vector with normalized values
/// * `Err(QuantError)` - If dequantization fails
pub fn dequantize_binary(qv: &QuantizedVector) -> Result<Vector, QuantError> {
    if qv.scheme != QuantScheme::Binary {
        return Err(QuantError::InvalidScheme(format!(
            "Expected Binary, got {:?}",
            qv.scheme
        )));
    }

    let dim = qv.dim as usize;
    let mut values = Vec::with_capacity(dim);

    for &byte in &qv.data {
        for i in 0..8 {
            if values.len() >= dim {
                break; // Stop if we've reached the target dimension
            }

            let bit = (byte >> i) & 1;
            // Convert to +1 or -1 for normalized representation
            let value = if bit == 1 { 1.0 } else { -1.0 };
            values.push(value);
        }
    }

    // Truncate to exact dimension
    values.truncate(dim);

    Ok(Vector::from_f32(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::SimilarityMetric;

    #[test]
    fn test_binary_basic() {
        // Simple test with known values
        let vec = Vector::from_f32(vec![0.5, -0.5, 0.3, -0.3]);
        let quantized = quantize_binary(&vec).unwrap();

        // Should pack 4 values into 1 byte
        assert_eq!(quantized.data.len(), 1);
        assert_eq!(quantized.scheme, QuantScheme::Binary);

        let restored = dequantize_binary(&quantized).unwrap();
        assert_eq!(restored.dim, 4);
    }

    #[test]
    fn test_binary_compression() {
        // Test with larger vector to verify compression
        let values: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0 - 0.5).collect();
        let vec = Vector::from_f32(values.clone());

        let quantized = quantize_binary(&vec).unwrap();

        // Original: 512 * 4 = 2048 bytes
        // Binary: 512 / 8 = 64 bytes
        // Compression: 32x
        let original_bytes = 512 * 4;
        let quantized_bytes = quantized.data.len();
        let compression_ratio = original_bytes as f32 / quantized_bytes as f32;

        assert_eq!(compression_ratio, 32.0);
    }

    #[test]
    fn test_binary_bit_packing() {
        // Verify bit packing is correct
        // Mean of [1.0, -1.0] = 0.0
        // 1.0 >= 0.0 → bit 1
        // -1.0 < 0.0 → bit 0
        let vec = Vector::from_f32(vec![1.0, -1.0]);
        let quantized = quantize_binary(&vec).unwrap();

        assert_eq!(quantized.data.len(), 1);
        // First bit should be set (1.0), second bit should be clear (-1.0)
        let byte = quantized.data[0];
        assert_eq!(byte & 0x01, 1); // First bit set
        assert_eq!(byte & 0x02, 0); // Second bit clear
    }

    #[test]
    fn test_binary_odd_dimension() {
        // Test with dimension not divisible by 8
        let vec = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        let quantized = quantize_binary(&vec).unwrap();

        // 5 values need 1 byte (with 3 unused bits)
        assert_eq!(quantized.data.len(), 1);

        let restored = dequantize_binary(&quantized).unwrap();
        assert_eq!(restored.dim, 5);
    }

    #[test]
    fn test_binary_roundtrip_similarity() {
        // Binary quantization is very lossy, but should preserve relative similarity
        let vec1 = Vector::from_f32(vec![0.5, 0.3, -0.2, 0.8, -0.4, 0.1]);
        let vec2 = Vector::from_f32(vec![0.4, 0.35, -0.15, 0.75, -0.35, 0.15]);

        // Original similarity
        let original_similarity = vec1.similarity(&vec2, SimilarityMetric::Cosine).unwrap();

        // Quantize both
        let q1 = quantize_binary(&vec1).unwrap();
        let q2 = quantize_binary(&vec2).unwrap();

        // Dequantize
        let r1 = dequantize_binary(&q1).unwrap();
        let r2 = dequantize_binary(&q2).unwrap();

        // Binary similarity
        let binary_similarity = r1.similarity(&r2, SimilarityMetric::Cosine).unwrap();

        // Binary quantization preserves general similarity patterns
        // Both should be positive (similar vectors)
        assert!(original_similarity > 0.5);
        assert!(binary_similarity > 0.0);

        // Print for information
        println!("Original similarity: {}", original_similarity);
        println!("Binary similarity: {}", binary_similarity);
    }

    #[test]
    fn test_binary_uniform_values() {
        // All same values → mean equals that value
        // All values will be exactly at mean, implementation defines behavior
        let vec = Vector::from_f32(vec![0.5, 0.5, 0.5, 0.5]);
        let quantized = quantize_binary(&vec).unwrap();

        let restored = dequantize_binary(&quantized).unwrap();
        assert_eq!(restored.dim, 4);
        // All values should be +1 or -1
        let restored_values = restored.as_f32().unwrap();
        for val in restored_values {
            assert!(val == 1.0 || val == -1.0);
        }
    }

    #[test]
    fn test_binary_normalized_output() {
        // Binary dequantization always outputs +1 or -1
        let vec = Vector::from_f32(vec![0.1, -0.5, 0.8, -0.2, 0.4]);
        let quantized = quantize_binary(&vec).unwrap();
        let restored = dequantize_binary(&quantized).unwrap();

        let restored_values = restored.as_f32().unwrap();
        for val in restored_values {
            assert!(val == 1.0 || val == -1.0, "Value: {}", val);
        }
    }

    #[test]
    fn test_binary_empty_fails() {
        let vec = Vector::from_f32(vec![]);
        let result = quantize_binary(&vec);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_large_vector() {
        // Test with typical embedding dimension
        let values: Vec<f32> = (0..1536)
            .map(|i| ((i % 100) as f32) / 100.0 - 0.5)
            .collect();
        let vec = Vector::from_f32(values);

        let quantized = quantize_binary(&vec).unwrap();

        // 1536 values / 8 = 192 bytes
        assert_eq!(quantized.data.len(), 192);

        let restored = dequantize_binary(&quantized).unwrap();
        assert_eq!(restored.dim, 1536);

        // Verify all values are normalized
        let restored_values = restored.as_f32().unwrap();
        for val in restored_values {
            assert!(val == 1.0 || val == -1.0);
        }
    }
}
