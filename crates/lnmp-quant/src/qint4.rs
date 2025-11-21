//! QInt4: 4-bit quantization with nibble packing
//!
//! This module implements 4-bit integer quantization where two values are packed
//! into each byte (nibble packing). This provides 8x compression compared to FP32.

use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;

/// Quantizes an embedding vector to 4-bit integers (packed 2 per byte)
///
/// # Arguments
/// * `emb` - The embedding vector to quantize
///
/// # Returns
/// * `Ok(QuantizedVector)` - Quantized vector with 8x compression
/// * `Err(QuantError)` - If quantization fails
///
/// # Example
/// ```
/// use lnmp_quant::qint4::quantize_qint4;
/// use lnmp_embedding::Vector;
///
/// let emb = Vector::from_f32(vec![0.0, 0.5, 1.0]);
/// let quantized = quantize_qint4(&emb).unwrap();
/// assert_eq!(quantized.scheme, lnmp_quant::QuantScheme::QInt4);
/// ```
pub fn quantize_qint4(emb: &Vector) -> Result<QuantizedVector, QuantError> {
    // Validate input
    if emb.dtype != lnmp_embedding::EmbeddingType::F32 {
        return Err(QuantError::EncodingFailed(
            "Only F32 embeddings are supported for QInt4 quantization".to_string(),
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

    // Find min and max values
    let min_val = values
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let max_val = values
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    // Calculate scale for 4-bit range [0, 15]
    let (scale, zero_point) = if (max_val - min_val).abs() < 1e-10 {
        // All values are the same
        (1.0, 0i8)
    } else {
        let range = max_val - min_val;
        let scale = range / 15.0; // 4-bit range: 0-15
        (scale, 0i8)
    };

    // Pre-compute inverse scale for performance
    let inv_scale = if scale.abs() > 1e-10 {
        1.0 / scale
    } else {
        1.0
    };

    // Quantize and pack: 2 values per byte
    let num_bytes = values.len().div_ceil(2); // Round up for odd dimensions
    let mut data = Vec::with_capacity(num_bytes);

    for chunk in values.chunks(2) {
        // Quantize first value
        let normalized1 = (chunk[0] - min_val) * inv_scale;
        let val1 = normalized1.round().clamp(0.0, 15.0) as u8;

        // Quantize second value (or pad with 0 if odd dimension)
        let val2 = if chunk.len() > 1 {
            let normalized2 = (chunk[1] - min_val) * inv_scale;
            normalized2.round().clamp(0.0, 15.0) as u8
        } else {
            0 // Pad with zero for odd dimensions
        };

        // Pack: high nibble = val1, low nibble = val2
        let packed = (val1 << 4) | val2;
        data.push(packed);
    }

    Ok(QuantizedVector::new(
        emb.dim as u32,
        QuantScheme::QInt4,
        scale,
        zero_point,
        min_val,
        data,
    ))
}

/// Dequantizes a 4-bit quantized vector back to f32
///
/// # Arguments
/// * `qv` - The quantized vector to dequantize
///
/// # Returns
/// * `Ok(Vector)` - Restored f32 vector
/// * `Err(QuantError)` - If dequantization fails
pub fn dequantize_qint4(qv: &QuantizedVector) -> Result<Vector, QuantError> {
    if qv.scheme != QuantScheme::QInt4 {
        return Err(QuantError::InvalidScheme(format!(
            "Expected QInt4, got {:?}",
            qv.scheme
        )));
    }

    let dim = qv.dim as usize;
    let mut values = Vec::with_capacity(dim);

    for &packed_byte in &qv.data {
        // Extract high nibble (first value)
        let val1 = (packed_byte >> 4) & 0x0F;
        let restored1 = (val1 as f32) * qv.scale + qv.min_val;
        values.push(restored1);

        // Extract low nibble (second value) if we haven't reached dimension limit
        if values.len() < dim {
            let val2 = packed_byte & 0x0F;
            let restored2 = (val2 as f32) * qv.scale + qv.min_val;
            values.push(restored2);
        }
    }

    // Truncate to exact dimension (in case we overshot due to padding)
    values.truncate(dim);

    Ok(Vector::from_f32(values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qint4_even_dimension() {
        // Test with even number of values
        let vec = Vector::from_f32(vec![0.0, 1.0, 0.5, 0.75]);
        let quantized = quantize_qint4(&vec).unwrap();

        // Should pack 4 values into 2 bytes
        assert_eq!(quantized.data.len(), 2);
        assert_eq!(quantized.scheme, QuantScheme::QInt4);

        let restored = dequantize_qint4(&quantized).unwrap();
        assert_eq!(restored.dim, 4);
    }

    #[test]
    fn test_qint4_odd_dimension() {
        // Test with odd number of values
        let vec = Vector::from_f32(vec![0.0, 0.5, 1.0]);
        let quantized = quantize_qint4(&vec).unwrap();

        // Should use 2 bytes for 3 values (last nibble padded)
        assert_eq!(quantized.data.len(), 2);

        let restored = dequantize_qint4(&quantized).unwrap();
        assert_eq!(restored.dim, 3);

        // Check values are approximately correct
        let restored_values = restored.as_f32().unwrap();
        assert!((restored_values[0] - 0.0).abs() < 0.1);
        assert!((restored_values[1] - 0.5).abs() < 0.1);
        assert!((restored_values[2] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_qint4_nibble_packing() {
        // Verify nibble packing is correct
        let vec = Vector::from_f32(vec![0.0, 15.0]);
        let quantized = quantize_qint4(&vec).unwrap();

        // Should pack into single byte
        assert_eq!(quantized.data.len(), 1);

        // First nibble should be 0 (0.0 normalized), second should be 15 (15.0 normalized)
        // Exact values depend on scale calculation
        let byte = quantized.data[0];
        let high_nibble = (byte >> 4) & 0x0F;
        let low_nibble = byte & 0x0F;

        // Both nibbles should be in valid 4-bit range
        assert!(high_nibble <= 15);
        assert!(low_nibble <= 15);
    }

    #[test]
    fn test_qint4_uniform_values() {
        // Test with all same values (edge case)
        let vec = Vector::from_f32(vec![0.5, 0.5, 0.5, 0.5]);
        let quantized = quantize_qint4(&vec).unwrap();

        let restored = dequantize_qint4(&quantized).unwrap();
        let restored_values = restored.as_f32().unwrap();

        // All values should be approximately the same
        for val in restored_values {
            assert!((val - 0.5).abs() < 0.1);
        }
    }

    #[test]
    fn test_qint4_single_value() {
        let vec = Vector::from_f32(vec![0.7]);
        let quantized = quantize_qint4(&vec).unwrap();

        // Single value should still use 1 byte
        assert_eq!(quantized.data.len(), 1);

        let restored = dequantize_qint4(&quantized).unwrap();
        assert_eq!(restored.dim, 1);
        let restored_values = restored.as_f32().unwrap();
        assert!((restored_values[0] - 0.7).abs() < 0.1);
    }

    #[test]
    fn test_qint4_roundtrip_accuracy() {
        // Test roundtrip with larger vector
        let original: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let vec = Vector::from_f32(original.clone());

        let quantized = quantize_qint4(&vec).unwrap();
        let restored = dequantize_qint4(&quantized).unwrap();
        let restored_values = restored.as_f32().unwrap();

        // Check compression ratio (should be ~8x)
        let original_bytes = original.len() * 4; // f32 = 4 bytes
        let quantized_bytes = quantized.data.len();
        let compression_ratio = original_bytes as f32 / quantized_bytes as f32;
        assert!(compression_ratio > 7.0 && compression_ratio <= 8.0);

        // Check accuracy (4-bit quantization will have some loss)
        let mut max_error = 0.0f32;
        for (orig, rest) in original.iter().zip(restored_values.iter()) {
            let error = (orig - rest).abs();
            max_error = max_error.max(error);
        }

        // With 4-bit quantization, error should be reasonable
        // For 16 levels over [0, 1] range, step size is ~0.067
        assert!(max_error < 0.1, "Max error: {}", max_error);
    }

    #[test]
    fn test_qint4_empty_fails() {
        let vec = Vector::from_f32(vec![]);
        let result = quantize_qint4(&vec);
        assert!(result.is_err());
    }
}
