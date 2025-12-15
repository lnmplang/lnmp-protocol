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

    pub fn normalize(&self) -> Result<Vector, String> {
        if self.dtype != EmbeddingType::F32 {
            return Err("Normalization not implemented for this dtype".to_string());
        }

        // Optimized for F32
        let norm_sq = Self::norm_sq_f32(&self.data);
        let norm = norm_sq.sqrt();

        if norm == 0.0 {
            return Ok(self.clone());
        }

        let mut res_data = Vec::with_capacity(self.data.len());

        for chunk in self.data.chunks_exact(4) {
            let val = f32::from_le_bytes(chunk.try_into().unwrap());
            let normalized_val = val / norm;
            res_data.extend_from_slice(&normalized_val.to_le_bytes());
        }

        Ok(Vector {
            dtype: self.dtype,
            dim: self.dim,
            data: res_data,
        })
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
                        Self::dot_and_norms_f32(&self.data, &other.data);

                    let norm1 = norm1_sq.sqrt();
                    let norm2 = norm2_sq.sqrt();

                    if norm1 == 0.0 || norm2 == 0.0 {
                        return Ok(0.0);
                    }
                    Ok(dot / (norm1 * norm2))
                }
                SimilarityMetric::DotProduct => {
                    let dot = Self::dot_product_f32(&self.data, &other.data);
                    Ok(dot)
                }
                SimilarityMetric::Euclidean => {
                    let sum_sq = Self::euclidean_distance_sq_f32(&self.data, &other.data);
                    Ok(sum_sq.sqrt())
                }
            }
        } else {
            Err("Similarity not implemented for this dtype yet".to_string())
        }
    }

    /// Optimized dot product for f32 from raw bytes
    #[inline]
    fn dot_product_f32(data1: &[u8], data2: &[u8]) -> f32 {
        let mut sum = 0.0f32;
        for (c1, c2) in data1.chunks_exact(4).zip(data2.chunks_exact(4)) {
            let v1 = f32::from_le_bytes(c1.try_into().unwrap());
            let v2 = f32::from_le_bytes(c2.try_into().unwrap());
            sum += v1 * v2;
        }
        sum
    }

    /// Optimized euclidean distance squared for f32 from raw bytes
    #[inline]
    fn euclidean_distance_sq_f32(data1: &[u8], data2: &[u8]) -> f32 {
        let mut sum = 0.0f32;
        for (c1, c2) in data1.chunks_exact(4).zip(data2.chunks_exact(4)) {
            let v1 = f32::from_le_bytes(c1.try_into().unwrap());
            let v2 = f32::from_le_bytes(c2.try_into().unwrap());
            let diff = v1 - v2;
            sum += diff * diff;
        }
        sum
    }

    /// Optimized combined dot product and norms calculation for f32 from raw bytes
    /// Returns (dot_product, norm1_squared, norm2_squared)
    #[inline]
    fn dot_and_norms_f32(data1: &[u8], data2: &[u8]) -> (f32, f32, f32) {
        let mut dot = 0.0f32;
        let mut norm1_sq = 0.0f32;
        let mut norm2_sq = 0.0f32;

        for (c1, c2) in data1.chunks_exact(4).zip(data2.chunks_exact(4)) {
            let v1 = f32::from_le_bytes(c1.try_into().unwrap());
            let v2 = f32::from_le_bytes(c2.try_into().unwrap());
            dot += v1 * v2;
            norm1_sq += v1 * v1;
            norm2_sq += v2 * v2;
        }

        (dot, norm1_sq, norm2_sq)
    }

    /// Optimized norm squared calculation for f32 from raw bytes
    #[inline]
    fn norm_sq_f32(data: &[u8]) -> f32 {
        let mut sum_sq = 0.0f32;
        for chunk in data.chunks_exact(4) {
            let val = f32::from_le_bytes(chunk.try_into().unwrap());
            sum_sq += val * val;
        }
        sum_sq
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

    #[test]
    fn test_normalize() {
        let v = Vector::from_f32(vec![3.0, 4.0]);
        let normalized = v.normalize().unwrap();
        let data = normalized.as_f32().unwrap();
        assert!((data[0] - 0.6).abs() < 1e-6);
        assert!((data[1] - 0.8).abs() < 1e-6);

        // Check that magnitude is 1
        let norm = (data[0] * data[0] + data[1] * data[1]).sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
}
