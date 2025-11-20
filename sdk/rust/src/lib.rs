pub use lnmp_embedding::EmbeddingType;
pub use lnmp_embedding::Vector as LnmpEmbedding;

pub fn version() -> &'static str {
    "lnmp-sdk-rust v0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_embedding_usage() {
        let vec = LnmpEmbedding::from_f32(vec![1.0, 2.0]);
        assert_eq!(vec.dim, 2);
        assert_eq!(vec.dtype, EmbeddingType::F32);
    }
}
