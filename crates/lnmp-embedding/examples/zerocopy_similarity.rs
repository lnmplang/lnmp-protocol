//! Zero-Copy Embedding Similarity Example
//!
//! Demonstrates zero-copy embedding operations with SIMD-optimized similarity.

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_embedding::{EmbeddingView, SimilarityMetric, Vector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Zero-Copy Embedding Similarity Demo ===\n");

    // Create sample embeddings (256-dim like CLIP)
    let emb1_data: Vec<f32> = (0..256).map(|i| (i as f32) / 256.0).collect();
    let emb2_data: Vec<f32> = (0..256).map(|i| ((255 - i) as f32) / 256.0).collect();
    let emb3_data: Vec<f32> = emb1_data.clone(); // Identical to emb1

    let vec1 = Vector::from_f32(emb1_data);
    let vec2 = Vector::from_f32(emb2_data);
    let vec3 = Vector::from_f32(emb3_data);

    // Create LNMP records with embeddings
    let rec1 = create_record_with_embedding(vec1, "image1.jpg");
    let rec2 = create_record_with_embedding(vec2, "image2.jpg");
    let rec3 = create_record_with_embedding(vec3, "image3.jpg");

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let bytes1 = encoder.encode(&rec1)?;
    let bytes2 = encoder.encode(&rec2)?;
    let bytes3 = encoder.encode(&rec3)?;

    println!("📦 Encoded 3 records:");
    println!("   Record 1: {} bytes", bytes1.len());
    println!("   Record 2: {} bytes", bytes2.len());
    println!("   Record 3: {} bytes\n", bytes3.len());

    // Zero-copy similarity computation
    let decoder = BinaryDecoder::new();

    println!("🔬 Zero-Copy Similarity Computation:\n");

    compute_similarity_zerocopy(&decoder, &bytes1, &bytes2, "image1 vs image2")?;
    compute_similarity_zerocopy(&decoder, &bytes1, &bytes3, "image1 vs image3")?;
    compute_similarity_zerocopy(&decoder, &bytes2, &bytes3, "image2 vs image3")?;

    // Performance benchmark
    benchmark_zerocopy_similarity(&bytes1, &bytes2)?;

    Ok(())
}

fn create_record_with_embedding(embedding: Vector, filename: &str) -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 20, // filename
        value: LnmpValue::String(filename.to_string()),
    });
    record.add_field(LnmpField {
        fid: 512, // embedding
        value: LnmpValue::Embedding(embedding),
    });
    record
}

fn compute_similarity_zerocopy(
    decoder: &BinaryDecoder,
    bytes1: &[u8],
    bytes2: &[u8],
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Decode views (zero-copy for strings/embeddings!)
    let view1 = decoder.decode_view(bytes1)?;
    let view2 = decoder.decode_view(bytes2)?;

    // Extract embedding fields
    let emb_bytes1 = extract_embedding_bytes(&view1)?;
    let emb_bytes2 = extract_embedding_bytes(&view2)?;

    // Create zero-copy embedding views
    let emb_view1 = EmbeddingView::from_bytes(emb_bytes1)?;
    let emb_view2 = EmbeddingView::from_bytes(emb_bytes2)?;

    // Compute similarities (zero-copy, SIMD-optimized!)
    let cosine = emb_view1.cosine_similarity(&emb_view2)?;
    let dot = emb_view1.dot_product(&emb_view2)?;
    let euclidean = emb_view1.euclidean_distance(&emb_view2)?;

    println!("📊 {} (dim={}):", description, emb_view1.dim);
    println!("   Cosine Similarity: {:.4}", cosine);
    println!("   Dot Product:       {:.4}", dot);
    println!("   Euclidean Distance: {:.4}\n", euclidean);

    Ok(())
}

fn extract_embedding_bytes<'a>(
    view: &lnmp_core::LnmpRecordView<'a>,
) -> Result<&'a [u8], Box<dyn std::error::Error>> {
    if let Some(field) = view.get_field(512) {
        if let lnmp_core::LnmpValueView::Embedding(emb_bytes) = &field.value {
            return Ok(emb_bytes);
        }
    }
    Err("Embedding field not found".into())
}

fn benchmark_zerocopy_similarity(
    bytes1: &[u8],
    bytes2: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    let iterations = 10_000;
    let decoder = BinaryDecoder::new();

    println!("=== Performance Benchmark ({} iterations) ===", iterations);

    // Zero-copy approach
    let start = Instant::now();
    for _ in 0..iterations {
        let view1 = decoder.decode_view(bytes1)?;
        let view2 = decoder.decode_view(bytes2)?;

        let emb_bytes1 = extract_embedding_bytes(&view1)?;
        let emb_bytes2 = extract_embedding_bytes(&view2)?;

        let emb_view1 = EmbeddingView::from_bytes(emb_bytes1)?;
        let emb_view2 = EmbeddingView::from_bytes(emb_bytes2)?;

        let _ = emb_view1.cosine_similarity(&emb_view2)?;
    }
    let zerocopy_time = start.elapsed();

    // Standard decode approach
    let start = Instant::now();
    for _ in 0..iterations {
        let rec1 = decoder.decode(bytes1)?;
        let rec2 = decoder.decode(bytes2)?;

        if let (Some(f1), Some(f2)) = (rec1.get_field(512), rec2.get_field(512)) {
            if let (LnmpValue::Embedding(e1), LnmpValue::Embedding(e2)) = (&f1.value, &f2.value) {
                let _ = e1.similarity(e2, SimilarityMetric::Cosine)?;
            }
        }
    }
    let standard_time = start.elapsed();

    println!(
        "Standard decode + similarity: {:?} ({:.2} μs/iter)",
        standard_time,
        standard_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Zero-copy view + similarity:  {:?} ({:.2} μs/iter)",
        zerocopy_time,
        zerocopy_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "Speedup: {:.2}x faster",
        standard_time.as_secs_f64() / zerocopy_time.as_secs_f64()
    );
    println!("\n💾 Memory Savings: Zero allocations for embedding data access!");

    Ok(())
}
