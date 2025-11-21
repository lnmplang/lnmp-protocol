use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;

/// Dequantizes a quantized vector back to approximate F32 representation
///
/// # Arguments
/// * `q` - The quantized vector to dequantize
///
/// # Returns
/// * `Ok(Vector)` - Successfully dequantized vector (F32 type)
/// * `Err(QuantError)` - If dequantization fails
///
/// # Example
/// ```
/// use lnmp_quant::{quantize_embedding, dequantize_embedding, QuantScheme};
/// use lnmp_embedding::Vector;
///
/// let original = Vector::from_f32(vec![0.12, -0.45, 0.33]);
/// let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
/// let restored = dequantize_embedding(&quantized).unwrap();
/// ```
pub fn dequantize_embedding(q: &QuantizedVector) -> Result<Vector, QuantError> {
    if q.dim == 0 {
        return Err(QuantError::InvalidDimension(
            "Cannot dequantize zero-dimensional vector".to_string(),
        ));
    }

    match q.scheme {
        QuantScheme::QInt8 => dequantize_qint8(q),
        QuantScheme::QInt4 => crate::qint4::dequantize_qint4(q),
        QuantScheme::Binary => crate::binary::dequantize_binary(q),
        QuantScheme::FP16Passthrough => crate::fp16::dequantize_fp16(q),
    }
}

/// Dequantizes a QInt8 quantized vector
fn dequantize_qint8(q: &QuantizedVector) -> Result<Vector, QuantError> {
    // Validate data length
    if q.data.len() != q.dim as usize {
        return Err(QuantError::DataCorrupted(format!(
            "Data length mismatch: expected {} bytes, got {}",
            q.dim,
            q.data.len()
        )));
    }

    // Dequantize each value using the stored min_val
    let mut values = Vec::with_capacity(q.dim as usize);

    for &quantized_byte in &q.data {
        let quantized = quantized_byte as i8;
        // Reverse the quantization: value = (quantized + 128) * scale + min_val
        let value = ((quantized as i32 + 128) as f32 * q.scale) + q.min_val;
        values.push(value);
    }

    // Convert to embedding vector
    Ok(Vector::from_f32(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::quantize_embedding;
    use lnmp_embedding::{SimilarityMetric, Vector};

    #[test]
    fn test_dequantize_simple() {
        let original = Vector::from_f32(vec![0.12, -0.45, 0.33]);
        let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
        let result = dequantize_embedding(&quantized);

        assert!(result.is_ok());
        let restored = result.unwrap();
        assert_eq!(restored.dim, 3);
        // Verify it's F32 by successfully converting
        assert!(restored.as_f32().is_ok());
    }

    #[test]
    fn test_roundtrip_accuracy() {
        let original =
            Vector::from_f32(vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8, 0.9, -1.0]);
        let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
        let restored = dequantize_embedding(&quantized).unwrap();

        // Check cosine similarity
        let similarity = original
            .similarity(&restored, SimilarityMetric::Cosine)
            .unwrap();

        // Should be very close to 1.0 (near-perfect similarity)
        assert!(similarity > 0.98, "Cosine similarity: {}", similarity);
    }

    #[test]
    fn test_roundtrip_large_vector() {
        let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
        let original = Vector::from_f32(values);

        let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
        let restored = dequantize_embedding(&quantized).unwrap();

        let similarity = original
            .similarity(&restored, SimilarityMetric::Cosine)
            .unwrap();

        assert!(similarity > 0.99, "Cosine similarity: {}", similarity);
    }

    #[test]
    fn test_dequantize_corrupted_data() {
        // Create a quantized vector with mismatched dimensions
        let qv = QuantizedVector::new(10, QuantScheme::QInt8, 0.01, 0, 0.0, vec![0u8; 5]);
        let result = dequantize_embedding(&qv);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QuantError::DataCorrupted(_)));
    }
}
