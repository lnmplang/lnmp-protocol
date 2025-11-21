use crate::scheme::QuantScheme;
use serde::{Deserialize, Serialize};

/// Quantized embedding vector with compression parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantizedVector {
    /// Vector dimension (number of elements)
    pub dim: u32,

    /// Quantization scheme used
    pub scheme: QuantScheme,

    /// Scaling factor for dequantization
    pub scale: f32,

    /// Zero-point offset for optimal range utilization
    pub zero_point: i8,

    /// Minimum value in the original data (for reconstruction)
    pub min_val: f32,

    /// Packed quantized data (actual byte representation)
    pub data: Vec<u8>,
}

impl QuantizedVector {
    /// Creates a new quantized vector
    pub fn new(
        dim: u32,
        scheme: QuantScheme,
        scale: f32,
        zero_point: i8,
        min_val: f32,
        data: Vec<u8>,
    ) -> Self {
        Self {
            dim,
            scheme,
            scale,
            zero_point,
            min_val,
            data,
        }
    }

    /// Returns the size in bytes (excluding metadata)
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// Returns the compression ratio compared to F32 representation
    pub fn compression_ratio(&self) -> f32 {
        let original_size = self.dim as f32 * 4.0; // F32 = 4 bytes
        let compressed_size = self.data.len() as f32;
        original_size / compressed_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantized_vector_creation() {
        let qv = QuantizedVector::new(128, QuantScheme::QInt8, 0.01, 0, 0.0, vec![0u8; 128]);

        assert_eq!(qv.dim, 128);
        assert_eq!(qv.scheme, QuantScheme::QInt8);
        assert_eq!(qv.scale, 0.01);
        assert_eq!(qv.zero_point, 0);
        assert_eq!(qv.min_val, 0.0);
        assert_eq!(qv.data_size(), 128);
    }

    #[test]
    fn test_compression_ratio() {
        let qv = QuantizedVector::new(512, QuantScheme::QInt8, 0.01, 0, 0.0, vec![0u8; 512]);

        // F32: 512 * 4 = 2048 bytes
        // QInt8: 512 * 1 = 512 bytes
        // Ratio: 2048 / 512 = 4.0
        assert_eq!(qv.compression_ratio(), 4.0);
    }
}
