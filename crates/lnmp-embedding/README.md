# lnmp-embedding

Native vector embedding support for the LNMP protocol.

This crate provides the core data structures and logic for handling vector embeddings within LNMP, enabling efficient storage, transport, and processing of high-dimensional vectors for AI and ML applications.

## Features

- **Multiple Data Types**: Supports `F32`, `F16`, `I8`, `U8`, and `Binary` embeddings.
- **Similarity Metrics**: Built-in calculation for Cosine Similarity, Euclidean Distance, and Dot Product.
- ** efficient Serialization**: Optimized binary format for minimal overhead.

## Usage

```rust
use lnmp_embedding::{Vector, EmbeddingType, SimilarityMetric};

// Create a vector from f32 data
let v1 = Vector::from_f32(vec![1.0, 0.0, 0.0]);
let v2 = Vector::from_f32(vec![0.0, 1.0, 0.0]);

// Calculate similarity
let similarity = v1.similarity(&v2, SimilarityMetric::Cosine).unwrap();
assert_eq!(similarity, 0.0);
```

## Integration

This crate is designed to be used with `lnmp-core` and the LNMP SDKs. It is the underlying implementation for the `Embedding` mode (0x06) and `LnmpValue::Embedding` variant.
