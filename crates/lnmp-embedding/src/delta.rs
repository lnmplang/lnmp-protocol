use crate::vector::{EmbeddingType, Vector};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Represents a single change in a vector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaChange {
    pub index: u16,
    pub delta: f32,
}

/// Represents a delta update for a vector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorDelta {
    pub base_id: u16,
    pub changes: Vec<DeltaChange>,
}

impl VectorDelta {
    /// Create a new VectorDelta
    pub fn new(base_id: u16, changes: Vec<DeltaChange>) -> Self {
        Self { base_id, changes }
    }

    /// Compute delta between two vectors
    /// Only supports F32 embeddings currently
    pub fn from_vectors(old: &Vector, new: &Vector, base_id: u16) -> Result<Self, String> {
        if old.dtype != new.dtype {
            return Err("Type mismatch between vectors".to_string());
        }
        if old.dim != new.dim {
            return Err("Dimension mismatch between vectors".to_string());
        }
        if old.dtype != EmbeddingType::F32 {
            return Err("Delta only supported for F32 embeddings".to_string());
        }

        let old_values = old.as_f32()?;
        let new_values = new.as_f32()?;

        let mut changes = Vec::new();
        for (i, (old_val, new_val)) in old_values.iter().zip(new_values.iter()).enumerate() {
            if (old_val - new_val).abs() > f32::EPSILON {
                changes.push(DeltaChange {
                    index: i as u16,
                    delta: new_val - old_val,
                });
            }
        }

        Ok(Self { base_id, changes })
    }

    /// Apply delta to a base vector to produce updated vector
    pub fn apply(&self, base: &Vector) -> Result<Vector, String> {
        if base.dtype != EmbeddingType::F32 {
            return Err("Delta application only supported for F32 embeddings".to_string());
        }

        let mut values = base.as_f32()?;

        // Apply each change
        for change in &self.changes {
            let idx = change.index as usize;
            if idx >= values.len() {
                return Err(format!("Invalid index {} in delta", idx));
            }
            values[idx] += change.delta;
        }

        Ok(Vector::from_f32(values))
    }

    /// Encode delta to binary format
    /// Format: base_id (u16) | change_count (u16) | [(index: u16, delta: f32), ...]
    pub fn encode(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = Vec::new();

        buf.write_u16::<LittleEndian>(self.base_id)?;
        buf.write_u16::<LittleEndian>(self.changes.len() as u16)?;

        for change in &self.changes {
            buf.write_u16::<LittleEndian>(change.index)?;
            buf.write_f32::<LittleEndian>(change.delta)?;
        }

        Ok(buf)
    }

    /// Decode delta from binary format
    pub fn decode(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut rdr = Cursor::new(data);

        let base_id = rdr.read_u16::<LittleEndian>()?;
        let change_count = rdr.read_u16::<LittleEndian>()?;

        let mut changes = Vec::with_capacity(change_count as usize);
        for _ in 0..change_count {
            let index = rdr.read_u16::<LittleEndian>()?;
            let delta = rdr.read_f32::<LittleEndian>()?;
            changes.push(DeltaChange { index, delta });
        }

        Ok(Self { base_id, changes })
    }

    /// Get the size of the encoded delta in bytes
    pub fn encoded_size(&self) -> usize {
        4 + (self.changes.len() * 6) // header (4) + changes (6 bytes each)
    }

    /// Calculate change ratio (percentage of values changed)
    pub fn change_ratio(&self, total_dim: u16) -> f32 {
        self.changes.len() as f32 / total_dim as f32
    }

    /// Check if delta is worth using vs full vector
    /// Returns true if delta is smaller than full vector encoding
    pub fn is_beneficial(&self, full_vector_size: usize) -> bool {
        self.encoded_size() < full_vector_size
    }
}

/// Strategy for deciding between full and delta encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStrategy {
    /// Always send full vector
    AlwaysFull,
    /// Always send delta (when available)
    AlwaysDelta,
    /// Automatically decide based on change ratio
    /// Uses delta if change_ratio < threshold (default 0.3)
    Adaptive { threshold: u8 }, // threshold as percentage (0-100)
}

impl Default for UpdateStrategy {
    fn default() -> Self {
        UpdateStrategy::Adaptive { threshold: 30 }
    }
}

impl UpdateStrategy {
    /// Decide whether to use delta based on the strategy
    pub fn should_use_delta(&self, delta: &VectorDelta, vector_dim: u16) -> bool {
        match self {
            UpdateStrategy::AlwaysFull => false,
            UpdateStrategy::AlwaysDelta => true,
            UpdateStrategy::Adaptive { threshold } => {
                let change_ratio = delta.change_ratio(vector_dim);
                change_ratio < (*threshold as f32 / 100.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_creation() {
        let changes = vec![
            DeltaChange {
                index: 0,
                delta: 0.1,
            },
            DeltaChange {
                index: 5,
                delta: -0.2,
            },
        ];
        let delta = VectorDelta::new(1001, changes.clone());
        assert_eq!(delta.base_id, 1001);
        assert_eq!(delta.changes.len(), 2);
    }

    #[test]
    fn test_delta_from_vectors() {
        let old = Vector::from_f32(vec![1.0, 2.0, 3.0, 4.0]);
        let new = Vector::from_f32(vec![1.0, 2.5, 3.0, 3.5]);

        let delta = VectorDelta::from_vectors(&old, &new, 100).unwrap();

        assert_eq!(delta.base_id, 100);
        assert_eq!(delta.changes.len(), 2);
        assert_eq!(delta.changes[0].index, 1);
        assert!((delta.changes[0].delta - 0.5).abs() < f32::EPSILON);
        assert_eq!(delta.changes[1].index, 3);
        assert!((delta.changes[1].delta - (-0.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_delta_apply() {
        let base = Vector::from_f32(vec![1.0, 2.0, 3.0, 4.0]);
        let changes = vec![
            DeltaChange {
                index: 1,
                delta: 0.5,
            },
            DeltaChange {
                index: 3,
                delta: -0.5,
            },
        ];
        let delta = VectorDelta::new(100, changes);

        let result = delta.apply(&base).unwrap();
        let values = result.as_f32().unwrap();

        assert_eq!(values.len(), 4);
        assert!((values[0] - 1.0).abs() < f32::EPSILON);
        assert!((values[1] - 2.5).abs() < f32::EPSILON);
        assert!((values[2] - 3.0).abs() < f32::EPSILON);
        assert!((values[3] - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_delta_encode_decode() {
        let changes = vec![
            DeltaChange {
                index: 10,
                delta: 0.123,
            },
            DeltaChange {
                index: 20,
                delta: -0.456,
            },
        ];
        let delta = VectorDelta::new(999, changes);

        let encoded = delta.encode().unwrap();
        assert_eq!(encoded.len(), 4 + 2 * 6); // header + 2 changes

        let decoded = VectorDelta::decode(&encoded).unwrap();
        assert_eq!(decoded.base_id, delta.base_id);
        assert_eq!(decoded.changes.len(), delta.changes.len());
        assert_eq!(decoded.changes[0].index, delta.changes[0].index);
        assert!((decoded.changes[0].delta - delta.changes[0].delta).abs() < 0.0001);
    }

    #[test]
    fn test_delta_roundtrip() {
        let old = Vector::from_f32(vec![0.1; 1536]);
        let mut new_data = vec![0.1; 1536];
        // Change 1% of values
        for i in 0..15 {
            new_data[i * 100] += 0.01;
        }
        let new = Vector::from_f32(new_data);

        let delta = VectorDelta::from_vectors(&old, &new, 1).unwrap();
        let encoded = delta.encode().unwrap();
        let decoded = VectorDelta::decode(&encoded).unwrap();
        let reconstructed = decoded.apply(&old).unwrap();

        assert_eq!(new, reconstructed);
    }

    #[test]
    fn test_update_strategy() {
        let small_delta = VectorDelta::new(
            1,
            vec![DeltaChange {
                index: 0,
                delta: 0.1,
            }],
        );
        let large_delta = VectorDelta::new(
            1,
            (0..500)
                .map(|i| DeltaChange {
                    index: i,
                    delta: 0.1,
                })
                .collect(),
        );

        let strategy = UpdateStrategy::default();

        assert!(strategy.should_use_delta(&small_delta, 1536));
        assert!(!strategy.should_use_delta(&large_delta, 1536));
    }
}
