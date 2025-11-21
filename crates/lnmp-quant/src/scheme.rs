use serde::{Deserialize, Serialize};

/// Quantization scheme for embedding vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum QuantScheme {
    /// 8-bit signed integer quantization
    /// - Range: -128 to 127
    /// - Size reduction: 4x (F32 → Int8)
    /// - Accuracy: Very high
    QInt8 = 0x01,

    /// 4-bit packed quantization (future)
    /// - Range: -8 to 7
    /// - Size reduction: 8x (F32 → 4-bit)
    /// - Accuracy: High
    #[allow(dead_code)]
    QInt4 = 0x02,

    /// 1-bit binary quantization (future)
    /// - Range: -1 or 1 (sign-based)
    /// - Size reduction: 32x (F32 → 1-bit)
    /// - Accuracy: Moderate
    #[allow(dead_code)]
    Binary = 0x03,

    /// FP16 passthrough (future)
    /// - Range: Half-precision float
    /// - Size reduction: 2x (F32 → F16)
    /// - Accuracy: Very high
    #[allow(dead_code)]
    FP16Passthrough = 0x04,
}

impl QuantScheme {
    /// Returns the expected bytes per value for this quantization scheme
    pub fn bytes_per_value(self) -> usize {
        match self {
            QuantScheme::QInt8 => 1,
            QuantScheme::QInt4 => 1,  // packed, 2 values per byte
            QuantScheme::Binary => 1, // packed, 8 values per byte
            QuantScheme::FP16Passthrough => 2,
        }
    }

    /// Returns the compression ratio compared to F32 (4 bytes)
    pub fn compression_ratio(self) -> f32 {
        4.0 / self.bytes_per_value() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_per_value() {
        assert_eq!(QuantScheme::QInt8.bytes_per_value(), 1);
        assert_eq!(QuantScheme::QInt4.bytes_per_value(), 1);
        assert_eq!(QuantScheme::Binary.bytes_per_value(), 1);
        assert_eq!(QuantScheme::FP16Passthrough.bytes_per_value(), 2);
    }

    #[test]
    fn test_compression_ratio() {
        assert_eq!(QuantScheme::QInt8.compression_ratio(), 4.0);
        assert_eq!(QuantScheme::FP16Passthrough.compression_ratio(), 2.0);
    }
}
