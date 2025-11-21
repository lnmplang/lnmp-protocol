//! Batch quantization module
//!
//!
//! This module provides functionality to efficiently quantize multiple embeddings in a batch.
//!
//! # Performance
//! The batch API introduces a small overhead (~3-13%) compared to a raw loop due to
//! statistics tracking and result collection. For 512-dim vectors, this is approximately
//! 150ns per vector. For maximum raw throughput in tight loops, consider using
//! `quantize_embedding` directly.

use crate::adaptive::{quantize_adaptive, AccuracyTarget};
use crate::encode::quantize_embedding;
use crate::error::QuantError;
use crate::scheme::QuantScheme;
use crate::vector::QuantizedVector;
use lnmp_embedding::Vector;
use std::time::{Duration, Instant};

/// Statistics for a batch quantization operation
#[derive(Debug, Clone, Copy, Default)]
pub struct BatchStats {
    /// Total number of vectors processed
    pub total: usize,
    /// Number of vectors successfully quantized
    pub succeeded: usize,
    /// Number of vectors that failed quantization
    pub failed: usize,
    /// Total time taken for the operation
    pub total_time: Duration,
}

/// Result of a batch quantization operation
#[derive(Debug)]
pub struct BatchResult {
    /// List of successfully quantized vectors (in order, failures are skipped or handled)
    /// Note: This implementation returns Results in the vector to maintain index alignment
    pub results: Vec<Result<QuantizedVector, QuantError>>,
    /// Statistics about the operation
    pub stats: BatchStats,
}

/// Quantize a batch of embeddings using a specific scheme
///
/// # Arguments
/// * `embeddings` - Slice of embedding vectors to quantize
/// * `scheme` - The quantization scheme to use
///
/// # Returns
/// * `BatchResult` - Contains results and statistics
///
/// # Example
/// ```
/// use lnmp_quant::batch::quantize_batch;
/// use lnmp_quant::QuantScheme;
/// use lnmp_embedding::Vector;
///
/// let vecs = vec![Vector::from_f32(vec![0.1]), Vector::from_f32(vec![0.2])];
/// let result = quantize_batch(&vecs, QuantScheme::QInt8);
/// println!("Processed {} vectors", result.stats.total);
/// ```
pub fn quantize_batch(embeddings: &[Vector], scheme: QuantScheme) -> BatchResult {
    let start_time = Instant::now();
    let mut results = Vec::with_capacity(embeddings.len());
    let mut succeeded = 0;
    let mut failed = 0;

    for emb in embeddings {
        let result = quantize_embedding(emb, scheme);
        if result.is_ok() {
            succeeded += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    BatchResult {
        results,
        stats: BatchStats {
            total: embeddings.len(),
            succeeded,
            failed,
            total_time: start_time.elapsed(),
        },
    }
}

/// Quantize a batch of embeddings using adaptive selection
///
/// # Arguments
/// * `embeddings` - Slice of embedding vectors to quantize
/// * `target` - The desired accuracy target
///
/// # Returns
/// * `BatchResult` - Contains results and statistics
pub fn quantize_batch_adaptive(embeddings: &[Vector], target: AccuracyTarget) -> BatchResult {
    let start_time = Instant::now();
    let mut results = Vec::with_capacity(embeddings.len());
    let mut succeeded = 0;
    let mut failed = 0;

    for emb in embeddings {
        let result = quantize_adaptive(emb, target);
        if result.is_ok() {
            succeeded += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    BatchResult {
        results,
        stats: BatchStats {
            total: embeddings.len(),
            succeeded,
            failed,
            total_time: start_time.elapsed(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_embedding::Vector;

    #[test]
    fn test_batch_quantization() {
        let vecs = vec![
            Vector::from_f32(vec![0.1, 0.2]),
            Vector::from_f32(vec![0.3, 0.4]),
            Vector::from_f32(vec![0.5, 0.6]),
        ];

        let result = quantize_batch(&vecs, QuantScheme::QInt8);

        assert_eq!(result.stats.total, 3);
        assert_eq!(result.stats.succeeded, 3);
        assert_eq!(result.stats.failed, 0);
        assert_eq!(result.results.len(), 3);

        for res in result.results {
            assert!(res.is_ok());
            assert_eq!(res.unwrap().scheme, QuantScheme::QInt8);
        }
    }

    #[test]
    fn test_batch_adaptive() {
        let vecs = vec![
            Vector::from_f32(vec![0.1, 0.2]),
            Vector::from_f32(vec![0.3, 0.4]),
        ];

        let result = quantize_batch_adaptive(&vecs, AccuracyTarget::Compact);

        assert_eq!(result.stats.total, 2);
        assert_eq!(result.stats.succeeded, 2);
        assert_eq!(result.results.len(), 2);

        for res in result.results {
            assert!(res.is_ok());
            assert_eq!(res.unwrap().scheme, QuantScheme::Binary);
        }
    }

    #[test]
    fn test_batch_with_errors() {
        // Create a vector that might fail (though currently most valid vectors succeed)
        // We'll use an empty vector which should fail
        let vecs = vec![
            Vector::from_f32(vec![0.1]),
            Vector::from_f32(vec![]), // Should fail
            Vector::from_f32(vec![0.2]),
        ];

        let result = quantize_batch(&vecs, QuantScheme::QInt8);

        assert_eq!(result.stats.total, 3);
        assert_eq!(result.stats.succeeded, 2);
        assert_eq!(result.stats.failed, 1);

        assert!(result.results[0].is_ok());
        assert!(result.results[1].is_err());
        assert!(result.results[2].is_ok());
    }
}
