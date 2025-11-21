//! Adaptive quantization module
//!
//!
//! This module provides functionality to automatically select the best quantization scheme
//! based on high-level requirements (accuracy vs compression).
//!
//! # Performance
//! Adaptive quantization introduces **negligible overhead** (zero cost in most cases) compared
//! to direct scheme usage, as the selection logic is effectively inlined by the compiler.

use crate::encode::quantize_embedding;
use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;

/// Target accuracy levels for adaptive quantization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccuracyTarget {
    /// Maximum accuracy (~99.9%), uses FP16 (2x compression)
    Maximum,
    /// High accuracy (~99%), uses QInt8 (4x compression)
    High,
    /// Balanced accuracy (~95-97%), uses QInt4 (8x compression)
    Balanced,
    /// Compact storage (~85-90%), uses Binary (32x compression)
    Compact,
}

/// Target compression levels for adaptive quantization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionTarget {
    /// Conservative compression (2x), prioritizes accuracy (FP16)
    Conservative,
    /// Moderate compression (4x), good balance (QInt8)
    Moderate,
    /// Aggressive compression (8x), prioritizes size (QInt4)
    Aggressive,
    /// Maximum compression (32x), minimal size (Binary)
    Maximum,
}

/// Quantize an embedding based on an accuracy target
///
/// # Arguments
/// * `emb` - The embedding vector to quantize
/// * `target` - The desired accuracy level
///
/// # Returns
/// * `Ok(QuantizedVector)` - The quantized vector using the selected scheme
/// * `Err(QuantError)` - If quantization fails
///
/// # Example
/// ```
/// use lnmp_quant::adaptive::{quantize_adaptive, AccuracyTarget};
/// use lnmp_embedding::Vector;
///
/// let vec = Vector::from_f32(vec![0.1, 0.2, 0.3]);
/// let q = quantize_adaptive(&vec, AccuracyTarget::High).unwrap();
/// ```
pub fn quantize_adaptive(
    emb: &Vector,
    target: AccuracyTarget,
) -> Result<QuantizedVector, QuantError> {
    let scheme = match target {
        AccuracyTarget::Maximum => QuantScheme::FP16Passthrough,
        AccuracyTarget::High => QuantScheme::QInt8,
        AccuracyTarget::Balanced => QuantScheme::QInt4,
        AccuracyTarget::Compact => QuantScheme::Binary,
    };

    quantize_embedding(emb, scheme)
}

/// Quantize an embedding based on a compression target
///
/// # Arguments
/// * `emb` - The embedding vector to quantize
/// * `target` - The desired compression level
///
/// # Returns
/// * `Ok(QuantizedVector)` - The quantized vector using the selected scheme
/// * `Err(QuantError)` - If quantization fails
pub fn quantize_with_target(
    emb: &Vector,
    target: CompressionTarget,
) -> Result<QuantizedVector, QuantError> {
    let scheme = match target {
        CompressionTarget::Conservative => QuantScheme::FP16Passthrough,
        CompressionTarget::Moderate => QuantScheme::QInt8,
        CompressionTarget::Aggressive => QuantScheme::QInt4,
        CompressionTarget::Maximum => QuantScheme::Binary,
    };

    quantize_embedding(emb, scheme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::Vector;

    #[test]
    fn test_adaptive_accuracy_selection() {
        let vec = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4]);

        // Maximum -> FP16
        let q = quantize_adaptive(&vec, AccuracyTarget::Maximum).unwrap();
        assert_eq!(q.scheme, QuantScheme::FP16Passthrough);

        // High -> QInt8
        let q = quantize_adaptive(&vec, AccuracyTarget::High).unwrap();
        assert_eq!(q.scheme, QuantScheme::QInt8);

        // Balanced -> QInt4
        let q = quantize_adaptive(&vec, AccuracyTarget::Balanced).unwrap();
        assert_eq!(q.scheme, QuantScheme::QInt4);

        // Compact -> Binary
        let q = quantize_adaptive(&vec, AccuracyTarget::Compact).unwrap();
        assert_eq!(q.scheme, QuantScheme::Binary);
    }

    #[test]
    fn test_adaptive_compression_selection() {
        let vec = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4]);

        // Conservative -> FP16
        let q = quantize_with_target(&vec, CompressionTarget::Conservative).unwrap();
        assert_eq!(q.scheme, QuantScheme::FP16Passthrough);

        // Moderate -> QInt8
        let q = quantize_with_target(&vec, CompressionTarget::Moderate).unwrap();
        assert_eq!(q.scheme, QuantScheme::QInt8);

        // Aggressive -> QInt4
        let q = quantize_with_target(&vec, CompressionTarget::Aggressive).unwrap();
        assert_eq!(q.scheme, QuantScheme::QInt4);

        // Maximum -> Binary
        let q = quantize_with_target(&vec, CompressionTarget::Maximum).unwrap();
        assert_eq!(q.scheme, QuantScheme::Binary);
    }
}
