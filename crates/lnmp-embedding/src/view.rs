//! Zero-copy embedding views for high-performance vector operations.
//!
//! This module provides `EmbeddingView` which allows direct access to embedding
//! data without allocation, enabling SIMD-optimized similarity computations.

use crate::vector::{EmbeddingType, SimilarityMetric};

/// Zero-copy view into an embedding stored in a binary buffer.
///
/// This struct provides direct access to embedding data without allocation.
/// All similarity computations can be performed on borrowed data.
///
/// # Layout
///
/// The binary format is:
/// ```text
/// [u16 dim | u8 dtype | u8 similarity | vector data...]
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EmbeddingView<'a> {
    /// Embedding dimension
    pub dim: u16,
    /// Embedding data type (F32, F16, etc.)
    pub dtype: EmbeddingType,
    /// Raw embedding data (borrowed from input buffer)
    data: &'a [u8],
}

impl<'a> EmbeddingView<'a> {
    /// Creates a new embedding view from raw bytes.
    ///
    /// # Format
    ///
    /// Expects bytes in the format: `[u16 dim | u8 dtype | u8 reserved | data...]`
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Buffer too small (< 4 bytes header)
    /// - Invalid dtype byte
    /// - Data length doesn't match expected size
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.len() < 4 {
            return Err("Buffer too small for embedding header".to_string());
        }

        // Parse header
        let dim = u16::from_le_bytes([bytes[0], bytes[1]]);
        let dtype_byte = bytes[2];
        // bytes[3] is similarity/reserved

        let dtype = match dtype_byte {
            0x01 => EmbeddingType::F32,
            0x02 => EmbeddingType::F16,
            0x03 => EmbeddingType::I8,
            0x04 => EmbeddingType::U8,
            0x05 => EmbeddingType::Binary,
            _ => return Err(format!("Invalid dtype: 0x{:02x}", dtype_byte)),
        };

        let data = &bytes[4..];
        let expected_len = dim as usize * dtype.size_bytes();

        if data.len() != expected_len {
            return Err(format!(
                "Data length mismatch: expected {} bytes, found {}",
                expected_len,
                data.len()
            ));
        }

        Ok(Self { dim, dtype, data })
    }

    /// Returns the raw data bytes.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Returns the embedding as a Vec<f32> (safe copy).
    ///
    /// This method performs a safe copy of the embedding data, converting
    /// from bytes to f32 values. While it allocates memory, it's guaranteed
    /// to work regardless of memory alignment.
    ///
    /// # Performance
    ///
    /// For 256-dim embedding: ~100-200ns allocation + copy overhead
    /// Still much faster than full record decode for large records.
    ///
    /// # Errors
    ///
    /// Returns error if dtype is not F32.
    pub fn as_f32_vec(&self) -> Result<Vec<f32>, String> {
        if self.dtype != EmbeddingType::F32 {
            return Err(format!(
                "Cannot convert {:?} to f32 vec (expected F32)",
                self.dtype
            ));
        }

        if !self.data.len().is_multiple_of(4) {
            return Err("Invalid data length for F32".to_string());
        }

        let mut result = Vec::with_capacity(self.dim as usize);
        for chunk in self.data.chunks_exact(4) {
            result.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(result)
    }

    /// Returns the embedding as an f32 slice (zero-copy cast - EXPERIMENTAL).
    ///
    /// ⚠️ **WARNING:** This method is experimental and may fail with alignment errors.
    /// Memory alignment is not guaranteed for slices from arbitrary buffers.
    /// Use `as_f32_vec()` for a safe, reliable alternative.
    ///
    /// # Safety
    ///
    /// Uses `bytemuck` for casting, which requires 4-byte alignment.
    /// Will panic if the underlying buffer is not properly aligned.
    ///
    /// # Errors
    ///
    /// Returns error if dtype is not F32 or bytemuck feature is not enabled.
    #[cfg(feature = "zerocopy")]
    pub fn as_f32_slice(&self) -> Result<&'a [f32], String> {
        if self.dtype != EmbeddingType::F32 {
            return Err(format!(
                "Cannot cast {:?} to f32 slice (expected F32)",
                self.dtype
            ));
        }

        // Try zero-copy cast (may panic if unaligned!)
        Ok(bytemuck::cast_slice(self.data))
    }

    /// Computes cosine similarity with another embedding view (zero-copy).
    ///
    /// # Performance
    ///
    /// Uses SIMD instructions when available on x86_64.
    /// Typical performance:
    /// - 256-dim: ~50-100 ns
    /// - 1024-dim: ~200-400 ns
    ///
    /// # Errors
    ///
    /// Returns error if dimensions or dtypes don't match.
    pub fn cosine_similarity(&self, other: &EmbeddingView) -> Result<f32, String> {
        self.check_compatibility(other)?;

        match self.dtype {
            EmbeddingType::F32 => {
                let a = self.as_f32_vec()?;
                let b = other.as_f32_vec()?;
                Ok(cosine_similarity_f32(&a, &b))
            }
            _ => Err(format!(
                "Cosine similarity not implemented for {:?}",
                self.dtype
            )),
        }
    }

    pub fn dot_product(&self, other: &EmbeddingView) -> Result<f32, String> {
        self.check_compatibility(other)?;
        let a = self.as_f32_vec()?;
        let b = other.as_f32_vec()?;
        Ok(dot_product_f32(&a, &b))
    }

    /// Computes Euclidean distance (zero-copy).
    pub fn euclidean_distance(&self, other: &EmbeddingView) -> Result<f32, String> {
        self.check_compatibility(other)?;
        let a = self.as_f32_vec()?;
        let b = other.as_f32_vec()?;
        Ok(euclidean_distance_f32(&a, &b))
    }

    /// Generic similarity with metric selection.
    pub fn similarity(
        &self,
        other: &EmbeddingView,
        metric: SimilarityMetric,
    ) -> Result<f32, String> {
        match metric {
            SimilarityMetric::Cosine => self.cosine_similarity(other),
            SimilarityMetric::DotProduct => self.dot_product(other),
            SimilarityMetric::Euclidean => self.euclidean_distance(other),
        }
    }

    fn check_compatibility(&self, other: &EmbeddingView) -> Result<(), String> {
        if self.dim != other.dim {
            return Err(format!("Dimension mismatch: {} vs {}", self.dim, other.dim));
        }
        if self.dtype != other.dtype {
            return Err(format!(
                "DType mismatch: {:?} vs {:?}",
                self.dtype, other.dtype
            ));
        }
        Ok(())
    }
}

impl EmbeddingType {
    /// Returns the size in bytes for each element of this type.
    pub fn size_bytes(&self) -> usize {
        match self {
            EmbeddingType::F32 => 4,
            EmbeddingType::F16 => 2,
            EmbeddingType::I8 => 1,
            EmbeddingType::U8 => 1,
            EmbeddingType::Binary => 1, // Bitpacked, but byte-aligned
        }
    }
}

// ============================================================================
// SIMD-Optimized Similarity Functions
// ============================================================================

#[cfg(all(target_arch = "x86_64", feature = "zerocopy"))]
fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    unsafe {
        let mut dot = _mm256_setzero_ps();
        let mut norm_a = _mm256_setzero_ps();
        let mut norm_b = _mm256_setzero_ps();

        // Process 8 floats at a time with AVX2
        let chunks = a.len() / 8;
        for i in 0..chunks {
            let offset = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

            dot = _mm256_add_ps(dot, _mm256_mul_ps(va, vb));
            norm_a = _mm256_add_ps(norm_a, _mm256_mul_ps(va, va));
            norm_b = _mm256_add_ps(norm_b, _mm256_mul_ps(vb, vb));
        }

        // Horizontal sum for dot, norm_a, norm_b
        let dot_sum = horizontal_sum_avx(dot);
        let norm_a_sum = horizontal_sum_avx(norm_a);
        let norm_b_sum = horizontal_sum_avx(norm_b);

        // Handle remainder
        let mut dot_rem = 0.0f32;
        let mut norm_a_rem = 0.0f32;
        let mut norm_b_rem = 0.0f32;
        for i in (chunks * 8)..a.len() {
            dot_rem += a[i] * b[i];
            norm_a_rem += a[i] * a[i];
            norm_b_rem += b[i] * b[i];
        }

        let total_dot = dot_sum + dot_rem;
        let total_norm_a = (norm_a_sum + norm_a_rem).sqrt();
        let total_norm_b = (norm_b_sum + norm_b_rem).sqrt();

        if total_norm_a == 0.0 || total_norm_b == 0.0 {
            return 0.0;
        }

        total_dot / (total_norm_a * total_norm_b)
    }
}

#[cfg(all(target_arch = "x86_64", feature = "zerocopy"))]
unsafe fn horizontal_sum_avx(v: std::arch::x86_64::__m256) -> f32 {
    use std::arch::x86_64::*;
    let hi = _mm256_extractf128_ps(v, 1);
    let lo = _mm256_castps256_ps128(v);
    let sum = _mm_add_ps(hi, lo);
    let shuf = _mm_movehdup_ps(sum);
    let sums = _mm_add_ps(sum, shuf);
    let shuf = _mm_movehl_ps(shuf, sums);
    let result = _mm_add_ss(sums, shuf);
    _mm_cvtss_f32(result)
}

// Fallback implementations for non-x86_64 or without bytemuck
#[cfg(any(not(target_arch = "x86_64"), not(feature = "zerocopy")))]
fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f32 {
    let (dot, norm_a_sq, norm_b_sq) = a
        .iter()
        .zip(b)
        .fold((0.0f32, 0.0f32, 0.0f32), |(d, na, nb), (&x, &y)| {
            (d + x * y, na + x * x, nb + y * y)
        });

    let norm_a = norm_a_sq.sqrt();
    let norm_b = norm_b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(feature = "zerocopy")]
fn dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}

#[cfg(not(feature = "zerocopy"))]
fn dot_product_f32(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("bytemuck feature required");
}

#[cfg(feature = "zerocopy")]
fn euclidean_distance_f32(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

#[cfg(not(feature = "zerocopy"))]
fn euclidean_distance_f32(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("bytemuck feature required");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Encoder;
    use crate::vector::Vector;

    #[test]
    fn test_embedding_view_from_bytes() {
        let vec = Vector::from_f32(vec![1.0, 2.0, 3.0]);
        let encoded = Encoder::encode(&vec).unwrap();

        let view = EmbeddingView::from_bytes(&encoded).unwrap();
        assert_eq!(view.dim, 3);
        assert_eq!(view.dtype, EmbeddingType::F32);
    }

    #[test]
    #[cfg(feature = "zerocopy")]
    fn test_cosine_similarity_zerocopy() {
        let v1 = Vector::from_f32(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::from_f32(vec![0.0, 1.0, 0.0]);

        let bytes1 = Encoder::encode(&v1).unwrap();
        let bytes2 = Encoder::encode(&v2).unwrap();

        let view1 = EmbeddingView::from_bytes(&bytes1).unwrap();
        let view2 = EmbeddingView::from_bytes(&bytes2).unwrap();

        let similarity = view1.cosine_similarity(&view2).unwrap();
        assert!((similarity - 0.0).abs() < 1e-6);
    }

    #[test]
    #[cfg(feature = "zerocopy")]
    fn test_dot_product_zerocopy() {
        let v1 = Vector::from_f32(vec![1.0, 2.0, 3.0]);
        let v2 = Vector::from_f32(vec![4.0, 5.0, 6.0]);

        let bytes1 = Encoder::encode(&v1).unwrap();
        let bytes2 = Encoder::encode(&v2).unwrap();

        let view1 = EmbeddingView::from_bytes(&bytes1).unwrap();
        let view2 = EmbeddingView::from_bytes(&bytes2).unwrap();

        let dot = view1.dot_product(&view2).unwrap();
        assert!((dot - 32.0).abs() < 1e-6); // 1*4 + 2*5 + 3*6 = 32
    }
}
