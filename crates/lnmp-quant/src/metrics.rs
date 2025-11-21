use serde::{Deserialize, Serialize};

/// Metrics and debug information for quantization operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantMetrics {
    /// Original minimum value in the embedding
    pub original_min: f32,

    /// Original maximum value in the embedding
    pub original_max: f32,

    /// Approximate information loss ratio (0.0 = no loss, 1.0 = complete loss)
    /// Calculated based on quantization error
    pub loss_ratio: f32,
}

impl QuantMetrics {
    /// Creates new quantization metrics
    pub fn new(original_min: f32, original_max: f32, loss_ratio: f32) -> Self {
        Self {
            original_min,
            original_max,
            loss_ratio,
        }
    }

    /// Returns the dynamic range of the original data
    pub fn dynamic_range(&self) -> f32 {
        self.original_max - self.original_min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = QuantMetrics::new(-1.0, 1.0, 0.05);

        assert_eq!(metrics.original_min, -1.0);
        assert_eq!(metrics.original_max, 1.0);
        assert_eq!(metrics.loss_ratio, 0.05);
        assert_eq!(metrics.dynamic_range(), 2.0);
    }
}
