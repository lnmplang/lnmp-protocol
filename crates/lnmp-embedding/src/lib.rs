pub mod decoder;
pub mod delta;
pub mod encoder;
pub mod vector;
pub mod view;

pub use decoder::Decoder;
pub use delta::{DeltaChange, UpdateStrategy, VectorDelta};
pub use encoder::Encoder;
pub use vector::{EmbeddingType, SimilarityMetric, Vector};
pub use view::EmbeddingView;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;
    use crate::encoder::Encoder;

    #[test]
    fn test_encode_decode() {
        let original = Vector::from_f32(vec![0.5, -0.5, 1.0]);
        let encoded = Encoder::encode(&original).expect("Failed to encode");
        let decoded = Decoder::decode(&encoded).expect("Failed to decode");

        assert_eq!(original.dim, decoded.dim);
        assert_eq!(original.dtype, decoded.dtype);
        assert_eq!(original.data, decoded.data);
    }
}
