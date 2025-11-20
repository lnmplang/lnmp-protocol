use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EmbeddingType {
    F32 = 0x01,
    F16 = 0x02,
    I8 = 0x03,
    U8 = 0x04,
    Binary = 0x05,
}

impl fmt::Display for EmbeddingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingType::F32 => write!(f, "F32"),
            EmbeddingType::F16 => write!(f, "F16"),
            EmbeddingType::I8 => write!(f, "I8"),
            EmbeddingType::U8 => write!(f, "U8"),
            EmbeddingType::Binary => write!(f, "Binary"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SimilarityMetric {
    Cosine = 0x01,
    Euclidean = 0x02,
    DotProduct = 0x03,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vector {
    pub dtype: EmbeddingType,
    pub dim: u16,
    pub data: Vec<u8>, // Raw bytes
}

impl Vector {
    pub fn new(dtype: EmbeddingType, dim: u16, data: Vec<u8>) -> Self {
        Self { dtype, dim, data }
    }

    pub fn from_f32(data: Vec<f32>) -> Self {
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for val in &data {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        Self {
            dtype: EmbeddingType::F32,
            dim: data.len() as u16,
            data: bytes,
        }
    }

    pub fn as_f32(&self) -> Result<Vec<f32>, String> {
        if self.dtype != EmbeddingType::F32 {
            return Err(format!("Cannot convert {:?} to F32", self.dtype));
        }

        if !self.data.len().is_multiple_of(4) {
            return Err("Invalid data length for F32".to_string());
        }

        let mut res = Vec::with_capacity(self.data.len() / 4);
        for chunk in self.data.chunks_exact(4) {
            let val = f32::from_le_bytes(chunk.try_into().unwrap());
            res.push(val);
        }
        Ok(res)
    }

    pub fn similarity(&self, other: &Vector, metric: SimilarityMetric) -> Result<f32, String> {
        if self.dtype != other.dtype {
            return Err("DType mismatch".to_string());
        }
        if self.dim != other.dim {
            return Err("Dimension mismatch".to_string());
        }

        // Currently only implementing for F32
        if self.dtype == EmbeddingType::F32 {
            // Optimized: avoid intermediate Vec allocation, work directly with bytes
            match metric {
                SimilarityMetric::Cosine => {
                    let (dot, norm1_sq, norm2_sq) =
                        unsafe { Self::dot_and_norms_f32(&self.data, &other.data) };

                    let norm1 = norm1_sq.sqrt();
                    let norm2 = norm2_sq.sqrt();

                    if norm1 == 0.0 || norm2 == 0.0 {
                        return Ok(0.0);
                    }
                    Ok(dot / (norm1 * norm2))
                }
                SimilarityMetric::DotProduct => {
                    let dot = unsafe { Self::dot_product_f32(&self.data, &other.data) };
                    Ok(dot)
                }
                SimilarityMetric::Euclidean => {
                    let sum_sq =
                        unsafe { Self::euclidean_distance_sq_f32(&self.data, &other.data) };
                    Ok(sum_sq.sqrt())
                }
            }
        } else {
            Err("Similarity not implemented for this dtype yet".to_string())
        }
    }

    /// Optimized dot product for f32 from raw bytes
    /// SAFETY: Assumes data is properly aligned f32 data with length % 4 == 0
    #[inline]
    unsafe fn dot_product_f32(data1: &[u8], data2: &[u8]) -> f32 {
        let len = data1.len() / 4;
        let ptr1 = data1.as_ptr() as *const f32;
        let ptr2 = data2.as_ptr() as *const f32;

        let mut sum = 0.0f32;
        for i in 0..len {
            sum += (*ptr1.add(i)) * (*ptr2.add(i));
        }
        sum
    }

    /// Optimized euclidean distance squared for f32 from raw bytes
    /// SAFETY: Assumes data is properly aligned f32 data with length % 4 == 0
    #[inline]
    unsafe fn euclidean_distance_sq_f32(data1: &[u8], data2: &[u8]) -> f32 {
        let len = data1.len() / 4;
        let ptr1 = data1.as_ptr() as *const f32;
        let ptr2 = data2.as_ptr() as *const f32;

        let mut sum = 0.0f32;
        for i in 0..len {
            let diff = *ptr1.add(i) - *ptr2.add(i);
            sum += diff * diff;
        }
        sum
    }

    /// Optimized combined dot product and norms calculation for f32 from raw bytes
    /// Returns (dot_product, norm1_squared, norm2_squared)
    /// SAFETY: Assumes data is properly aligned f32 data with length % 4 == 0
    #[inline]
    unsafe fn dot_and_norms_f32(data1: &[u8], data2: &[u8]) -> (f32, f32, f32) {
        let len = data1.len() / 4;
        let ptr1 = data1.as_ptr() as *const f32;
        let ptr2 = data2.as_ptr() as *const f32;

        let mut dot = 0.0f32;
        let mut norm1_sq = 0.0f32;
        let mut norm2_sq = 0.0f32;

        for i in 0..len {
            let v1 = *ptr1.add(i);
            let v2 = *ptr2.add(i);
            dot += v1 * v2;
            norm1_sq += v1 * v1;
            norm2_sq += v2 * v2;
        }

        (dot, norm1_sq, norm2_sq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_conversion() {
        let data = vec![1.0, 2.0, 3.0];
        let vec = Vector::from_f32(data.clone());
        assert_eq!(vec.dtype, EmbeddingType::F32);
        assert_eq!(vec.dim, 3);
        assert_eq!(vec.as_f32().unwrap(), data);
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = Vector::from_f32(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::from_f32(vec![0.0, 1.0, 0.0]);
        assert_eq!(v1.similarity(&v2, SimilarityMetric::Cosine).unwrap(), 0.0);

        let v3 = Vector::from_f32(vec![1.0, 0.0, 0.0]);
        assert_eq!(v1.similarity(&v3, SimilarityMetric::Cosine).unwrap(), 1.0); // Should be exactly 1.0 or very close
    }
}
