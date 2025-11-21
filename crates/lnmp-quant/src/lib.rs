//! # lnmp-quant
//!
//! Quantization and compression for LNMP embedding vectors.
//!
//! This crate provides efficient quantization schemes to compress embedding vectors
//! while maintaining high semantic accuracy. Multiple schemes available:
//!   - QInt8: 4x compression with ~99% accuracy
//!   - QInt4: 8x compression with ~95-97% accuracy  
//!   - Binary: 32x compression with ~85-90% similarity
//!
//! ## Quick Start
//!
//! ```rust
//! use lnmp_quant::{quantize_embedding, dequantize_embedding, QuantScheme};
//! use lnmp_embedding::Vector;
//!
//! // Create an embedding
//! let embedding = Vector::from_f32(vec![0.12, -0.45, 0.33]);
//!
//! // Quantize to QInt8
//! let quantized = quantize_embedding(&embedding, QuantScheme::QInt8).unwrap();
//! println!("Original size: {} bytes", embedding.dim * 4);
//! println!("Quantized size: {} bytes", quantized.data_size());
//! println!("Compression ratio: {:.1}x", quantized.compression_ratio());
//!
//! // Dequantize back
//! let restored = dequantize_embedding(&quantized).unwrap();
//! ```
//!
//! ## Quantization Schemes
//!
//! - **QInt8**: 8-bit signed integer quantization (4x compression, ~99% accuracy)
//! - **QInt4**: 4-bit packed quantization (8x compression, ~95-97% accuracy)
//! - **Binary**: 1-bit sign-based quantization (32x compression, ~85-90% similarity)
//! - **FP16**: Half-precision float (2x compression, ~99.9% accuracy, near-lossless)

pub mod adaptive;
pub mod batch;
pub mod binary;
pub mod decode;
pub mod encode;
pub mod error;
pub mod fp16;
pub mod metrics;
pub mod qint4;
pub mod scheme;
pub mod vector;

// Re-export main types and functions
pub use decode::dequantize_embedding;
pub use encode::quantize_embedding;
pub use error::QuantError;
pub use metrics::QuantMetrics;
pub use scheme::QuantScheme;
pub use vector::QuantizedVector;

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::{SimilarityMetric, Vector};

    #[test]
    fn test_basic_roundtrip() {
        let original = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();
        let restored = dequantize_embedding(&quantized).unwrap();

        let similarity = original
            .similarity(&restored, SimilarityMetric::Cosine)
            .unwrap();

        assert!(similarity > 0.95);
    }

    #[test]
    fn test_compression_ratio() {
        let original = Vector::from_f32(vec![0.1; 1536]);
        let quantized = quantize_embedding(&original, QuantScheme::QInt8).unwrap();

        // Original: 1536 * 4 = 6144 bytes
        // Quantized: 1536 * 1 = 1536 bytes
        assert_eq!(quantized.compression_ratio(), 4.0);
    }
}
