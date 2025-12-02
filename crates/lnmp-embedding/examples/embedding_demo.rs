#![allow(clippy::useless_vec)]
//! LNMP Embedding Demo
//!
//! This example demonstrates the complete LNMP-Embedding feature including:
//! - Creating vector embeddings of different types (F32, F16, I8, U8, Binary)
//! - Encoding and decoding embeddings to/from binary format
//! - Calculating similarity metrics (Cosine, Euclidean, DotProduct)
//! - Using embeddings in LNMP records
//! - Container mode serialization with Embedding mode
//! - Round-trip verification and data integrity checks

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_codec::{ContainerBuilder, ContainerFrame};
use lnmp_core::{LnmpField, LnmpFileMode, LnmpRecord, LnmpValue};
use lnmp_embedding::{Decoder, EmbeddingType, Encoder, SimilarityMetric, Vector};

fn main() {
    println!("=== LNMP Embedding Demo ===\n");

    // Demo 1: Basic Vector Operations
    demo_vector_operations();

    // Demo 2: Similarity Calculations
    demo_similarity_calculations();

    // Demo 3: Binary Encoding/Decoding
    demo_binary_encoding();

    // Demo 4: Integration with LNMP Records
    demo_lnmp_record_integration();

    // Demo 5: Container Mode Serialization
    demo_container_mode();

    // Demo 6: Different Embedding Types
    demo_different_types();

    // Demo 7: Round-trip Verification
    demo_roundtrip_verification();

    println!("\n=== All demos completed successfully! ===");
}

fn demo_vector_operations() {
    println!("--- Demo 1: Basic Vector Operations ---");

    // Create F32 embedding
    let vec = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    println!("Created F32 vector: dim={}, type={:?}", vec.dim, vec.dtype);

    // Convert to F32 values
    let values = vec.as_f32().unwrap();
    println!("Values: {:?}", values);

    // Check properties
    println!("Data size: {} bytes", vec.data.len());
    println!();
}

fn demo_similarity_calculations() {
    println!("--- Demo 2: Similarity Calculations ---");

    let vec1 = Vector::from_f32(vec![1.0, 0.0, 0.0]);
    let vec2 = Vector::from_f32(vec![0.0, 1.0, 0.0]);
    let vec3 = Vector::from_f32(vec![1.0, 0.0, 0.0]);

    // Cosine similarity
    let cosine_12 = vec1.similarity(&vec2, SimilarityMetric::Cosine).unwrap();
    let cosine_13 = vec1.similarity(&vec3, SimilarityMetric::Cosine).unwrap();
    println!("Cosine similarity(vec1, vec2): {:.4}", cosine_12);
    println!("Cosine similarity(vec1, vec3): {:.4}", cosine_13);

    // Euclidean distance
    let euclidean_12 = vec1.similarity(&vec2, SimilarityMetric::Euclidean).unwrap();
    let euclidean_13 = vec1.similarity(&vec3, SimilarityMetric::Euclidean).unwrap();
    println!("Euclidean distance(vec1, vec2): {:.4}", euclidean_12);
    println!("Euclidean distance(vec1, vec3): {:.4}", euclidean_13);

    // Dot product
    let dot_12 = vec1
        .similarity(&vec2, SimilarityMetric::DotProduct)
        .unwrap();
    let dot_13 = vec1
        .similarity(&vec3, SimilarityMetric::DotProduct)
        .unwrap();
    println!("Dot product(vec1, vec2): {:.4}", dot_12);
    println!("Dot product(vec1, vec3): {:.4}", dot_13);
    println!();
}

fn demo_binary_encoding() {
    println!("--- Demo 3: Binary Encoding/Decoding ---");

    let original = Vector::from_f32(vec![0.5, 0.25, 0.75, 1.0]);
    println!(
        "Original vector: dim={}, type={:?}",
        original.dim, original.dtype
    );

    // Encode to binary
    let binary = Encoder::encode(&original).unwrap();
    println!("Encoded to {} bytes", binary.len());

    // Decode from binary
    let decoded = Decoder::decode(&binary).unwrap();
    println!(
        "Decoded vector: dim={}, type={:?}",
        decoded.dim, decoded.dtype
    );

    // Verify equality
    assert_eq!(original, decoded);
    println!("✓ Binary round-trip verified");
    println!();
}

fn demo_lnmp_record_integration() {
    println!("--- Demo 4: Integration with LNMP Records ---");

    let mut record = LnmpRecord::new();

    // Add a user ID
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(12345),
    });

    // Add username
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("alice".to_string()),
    });

    // Add embedding vector for semantic search
    let embedding = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Embedding(embedding),
    });

    println!("Created record with {} fields", record.fields().len());

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    println!("Encoded record to {} bytes", binary.len());

    // Decode back
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();
    println!("Decoded record with {} fields", decoded.fields().len());

    // Verify embedding field
    let embedding_field = decoded.get_field(100).unwrap();
    if let LnmpValue::Embedding(vec) = &embedding_field.value {
        println!(
            "✓ Embedding field preserved: dim={}, type={:?}",
            vec.dim, vec.dtype
        );
    }
    println!();
}

fn demo_container_mode() {
    println!("--- Demo 5: Container Mode Serialization ---");

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Embedding(Vector::from_f32(vec![1.0, 2.0, 3.0])),
    });

    // Create container with Embedding mode
    let builder = ContainerBuilder::new(LnmpFileMode::Embedding);
    let container = builder.encode_record(&record).unwrap();
    println!("Created container with {} bytes", container.len());

    // Parse container
    let frame = ContainerFrame::parse(&container).unwrap();
    println!("Container mode: {:?}", frame.header().mode);
    assert_eq!(frame.header().mode, LnmpFileMode::Embedding);
    println!("✓ Container mode verified");
    println!();
}

fn demo_different_types() {
    println!("--- Demo 6: Different Embedding Types ---");

    // F32 (most common)
    let f32_vec = Vector::from_f32(vec![0.1, 0.2, 0.3]);
    println!("F32: dim={}, bytes={}", f32_vec.dim, f32_vec.data.len());

    // Create I8, U8, and Binary vectors manually using the new() constructor
    let i8_vec = Vector::new(EmbeddingType::I8, 4, vec![10u8, 20, 30, 40]);
    println!("I8: dim={}, bytes={}", i8_vec.dim, i8_vec.data.len());

    let u8_vec = Vector::new(EmbeddingType::U8, 3, vec![100u8, 150, 200]);
    println!("U8: dim={}, bytes={}", u8_vec.dim, u8_vec.data.len());

    let binary_vec = Vector::new(EmbeddingType::Binary, 4, vec![0xFFu8, 0x00, 0xAA, 0x55]);
    println!(
        "Binary: dim={}, bytes={}",
        binary_vec.dim,
        binary_vec.data.len()
    );

    // Encode and decode each
    for vec in [&f32_vec, &i8_vec, &u8_vec, &binary_vec] {
        let binary = Encoder::encode(vec).unwrap();
        let decoded = Decoder::decode(&binary).unwrap();
        assert_eq!(vec, &decoded);
    }

    println!("✓ All types verified");
    println!();
}

fn demo_roundtrip_verification() {
    println!("--- Demo 7: Round-trip Verification ---");

    // Create embeddings with different dimensions
    let vectors = vec![
        Vector::from_f32(vec![0.1; 128]),  // 128-dim
        Vector::from_f32(vec![0.2; 256]),  // 256-dim
        Vector::from_f32(vec![0.3; 512]),  // 512-dim
        Vector::from_f32(vec![0.4; 768]),  // 768-dim (BERT)
        Vector::from_f32(vec![0.5; 1536]), // 1536-dim (OpenAI)
    ];

    for (idx, vec) in vectors.iter().enumerate() {
        // Encode
        let binary = Encoder::encode(vec).unwrap();
        let encoded_size = binary.len();

        // Decode
        let decoded = Decoder::decode(&binary).unwrap();

        // Verify
        assert_eq!(vec.dim, decoded.dim);
        assert_eq!(vec.dtype, decoded.dtype);
        assert_eq!(vec.data, decoded.data);

        println!(
            "Vector {}: dim={}, encoded_size={} bytes, ✓ verified",
            idx + 1,
            vec.dim,
            encoded_size
        );
    }

    println!("\n✓ All round-trip tests passed");
    println!();
}
