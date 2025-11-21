use crate::error::QuantError;
use crate::metrics::QuantMetrics;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;

/// Quantizes an embedding vector using the specified quantization scheme
///
/// # Arguments
/// * `emb` - The embedding vector to quantize (must be F32 type)
/// * `scheme` - The quantization scheme to use
///
/// # Returns
/// * `Ok(QuantizedVector)` - Successfully quantized vector
/// * `Err(QuantError)` - If quantization fails
///
/// # Example
/// ```
/// use lnmp_quant::{quantize_embedding, QuantScheme};
/// use lnmp_embedding::Vector;
///
/// let emb = Vector::from_f32(vec![0.12, -0.45, 0.33]);
/// let quantized = quantize_embedding(&emb, QuantScheme::QInt8).unwrap();
/// ```
pub fn quantize_embedding(
    emb: &Vector,
    scheme: QuantScheme,
) -> Result<QuantizedVector, QuantError> {
    // Validate input
    if emb.dtype != lnmp_embedding::EmbeddingType::F32 {
        return Err(QuantError::EncodingFailed(
            "Only F32 embeddings are currently supported for quantization".to_string(),
        ));
    }

    if emb.dim == 0 {
        return Err(QuantError::InvalidDimension(
            "Cannot quantize zero-dimensional vector".to_string(),
        ));
    }

    match scheme {
        QuantScheme::QInt8 => quantize_qint8(emb),
        QuantScheme::QInt4 => crate::qint4::quantize_qint4(emb),
        QuantScheme::Binary => crate::binary::quantize_binary(emb),
        QuantScheme::FP16Passthrough => crate::fp16::quantize_fp16(emb),
    }
}

/// Quantizes an embedding to QInt8 format
fn quantize_qint8(emb: &Vector) -> Result<QuantizedVector, QuantError> {
    // Convert to f32 values
    let values = emb
        .as_f32()
        .map_err(|e| QuantError::EncodingFailed(format!("Failed to convert to F32: {}", e)))?;

    if values.is_empty() {
        return Err(QuantError::InvalidDimension(
            "Empty embedding vector".to_string(),
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

    // Handle edge case: all values are the same
    let (scale, zero_point) = if (max_val - min_val).abs() < 1e-10 {
        (1.0, 0)
    } else {
        // Calculate scale to map [min_val, max_val] to [-128, 127]
        let range = max_val - min_val;
        let scale = range / 255.0; // QInt8 range: -128 to 127 = 256 values, but we use 255 for symmetry

        // Calculate zero point (typically 0 for symmetric quantization)
        let zero_point = 0i8;

        (scale, zero_point)
    };

    // Pre-allocate output vector with exact capacity (memory optimization)
    let mut quantized_data = Vec::with_capacity(values.len());

    // Cache inverse scale to avoid repeated division (performance optimization)
    let inv_scale = if scale.abs() > 1e-10 {
        1.0 / scale
    } else {
        1.0
    };

    // Quantize each value
    for &value in &values {
        // Optimized: multiply by inv_scale instead of dividing by scale
        let normalized = (value - min_val) * inv_scale;
        let quantized = (normalized as i32 - 128).clamp(-128, 127) as i8;
        quantized_data.push(quantized as u8);
    }

    // Calculate metrics (approximate loss ratio based on quantization error)
    let loss_ratio = calculate_loss_ratio(&values, &quantized_data, scale, min_val);

    let _metrics = QuantMetrics::new(min_val, max_val, loss_ratio);

    Ok(QuantizedVector::new(
        emb.dim as u32,
        QuantScheme::QInt8,
        scale,
        zero_point,
        min_val,
        quantized_data,
    ))
}

/// Calculates approximate information loss ratio
fn calculate_loss_ratio(original: &[f32], quantized: &[u8], scale: f32, min_val: f32) -> f32 {
    if original.is_empty() {
        return 0.0;
    }

    let mut total_error = 0.0f32;
    let mut total_magnitude = 0.0f32;

    for (i, &orig_val) in original.iter().enumerate() {
        // Dequantize
        let q_val = quantized[i] as i8;
        let reconstructed = ((q_val as i32 + 128) as f32 * scale) + min_val;

        let error = (orig_val - reconstructed).abs();
        total_error += error;
        total_magnitude += orig_val.abs();
    }

    if total_magnitude < 1e-10 {
        0.0
    } else {
        total_error / total_magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::Vector;

    #[test]
    fn test_quantize_simple() {
        let emb = Vector::from_f32(vec![0.12, -0.45, 0.33]);
        let result = quantize_embedding(&emb, QuantScheme::QInt8);
        assert!(result.is_ok());

        let quantized = result.unwrap();
        assert_eq!(quantized.dim, 3);
        assert_eq!(quantized.scheme, QuantScheme::QInt8);
        assert_eq!(quantized.data.len(), 3);
    }

    #[test]
    fn test_quantize_large_vector() {
        let values: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0) - 0.5).collect();
        let emb = Vector::from_f32(values);
        let result = quantize_embedding(&emb, QuantScheme::QInt8);
        assert!(result.is_ok());

        let quantized = result.unwrap();
        assert_eq!(quantized.dim, 512);
        assert_eq!(quantized.data.len(), 512);
    }

    #[test]
    fn test_quantize_uniform_values() {
        let emb = Vector::from_f32(vec![0.5, 0.5, 0.5, 0.5]);
        let result = quantize_embedding(&emb, QuantScheme::QInt8);
        assert!(result.is_ok());

        let quantized = result.unwrap();
        assert_eq!(quantized.dim, 4);
        // All values should be quantized to the same value
    }

    #[test]
    fn test_quantize_empty_fails() {
        let emb = Vector::from_f32(vec![]);
        let result = quantize_embedding(&emb, QuantScheme::QInt8);
        assert!(result.is_err());
    }

    // Note: All schemes are now implemented, so there's no unsupported scheme to test
}
