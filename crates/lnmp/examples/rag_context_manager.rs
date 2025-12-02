//! RAG Context Manager - Showcase Example
//!
//! Document management system for Retrieval-Augmented Generation.
//! Demonstrates context profiling, embedding deltas, and LLM optimization.
//!
//! Run: `cargo run --example rag_context_manager`

use lnmp::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Document in the RAG system
struct Document {
    id: String,
    content: String,
    embedding: Vec<f32>,
    timestamp: u64,
    source: String,
    importance: u8,
}

impl Document {
    fn new(id: String, content: String, source: String) -> Self {
        // Simulate embedding (in real system, would call embedding model)
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // 5D for demo

        Self {
            id,
            content,
            embedding,
            timestamp: current_timestamp(),
            source,
            importance: 150,
        }
    }

    fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        // Simulate new embedding
        self.embedding = vec![0.11, 0.21, 0.31, 0.41, 0.51];
        self.timestamp = current_timestamp();
    }

    fn to_lnmp_envelope(&self) -> lnmp::envelope::LnmpEnvelope {
        let mut record = LnmpRecord::new();

        // Document ID
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.id.clone()),
        });

        // Content
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::String(self.content.clone()),
        });

        // Embedding as float array (convert f32 to f64 for FloatArray)
        let embedding_f64: Vec<f64> = self.embedding.iter().map(|&x| x as f64).collect();
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::FloatArray(embedding_f64),
        });

        lnmp::envelope::EnvelopeBuilder::new(record)
            .timestamp(self.timestamp)
            .source(&self.source)
            .build()
    }
}

fn main() {
    println!("🤖 RAG Context Manager - LNMP Showcase\n");

    // Create document store
    let mut docs = vec![
        Document::new(
            "doc-001".to_string(),
            "LNMP is a minimal protocol for LLMs".to_string(),
            "knowledge-base".to_string(),
        ),
        Document::new(
            "doc-002".to_string(),
            "Spatial encoding enables real-time robotics".to_string(),
            "knowledge-base".to_string(),
        ),
        Document::new(
            "doc-003".to_string(),
            "Context profiling helps RAG prioritization".to_string(),
            "knowledge-base".to_string(),
        ),
    ];

    // Set importance levels
    docs[0].importance = 200; // High
    docs[1].importance = 180; // Medium-high
    docs[2].importance = 220; // Very high (this example!)

    println!(
        "📚 Document Store initialized with {} documents\n",
        docs.len()
    );

    // Create context scorer
    let scorer_config = lnmp::sfe::ContextScorerConfig::new()
        .with_freshness_decay(24.0)
        .add_trusted_source("knowledge-base".to_string());

    let scorer = lnmp::sfe::ContextScorer::with_config(scorer_config);

    let now = current_timestamp();

    println!("📊 Context Profiling:");
    for doc in &docs {
        let envelope = doc.to_lnmp_envelope();
        let profile = scorer.score_envelope(&envelope, now);
        let composite = profile.composite_score();

        println!(
            "  {} | Freshness: {:.3} | Importance: {} | Composite: {:.3}",
            doc.id, profile.freshness_score, doc.importance, composite
        );
    }

    // Simulate document update
    println!("\n🔄 Updating doc-002 content...");
    let old_embedding = docs[1].embedding.clone();
    docs[1].update_content("Spatial encoding updated with new examples".to_string());
    let new_embedding = docs[1].embedding.clone();

    // Calculate embedding delta
    let old_vec = lnmp::embedding::Vector::from_f32(old_embedding);
    let new_vec = lnmp::embedding::Vector::from_f32(new_embedding);

    let delta = lnmp::embedding::VectorDelta::from_vectors(&old_vec, &new_vec, 1).unwrap();

    println!("  Delta changes: {} dimensions", delta.changes.len());
    println!("  Change ratio: {:.1}%", delta.change_ratio(5) * 100.0);

    // Encode delta for transmission
    let delta_bytes = delta.encode().unwrap();
    let full_bytes = new_vec.data.len();

    println!("  Full embedding: {} bytes", full_bytes);
    println!("  Delta encoding: {} bytes", delta_bytes.len());
    println!(
        "  Bandwidth saved: {:.1}%",
        (1.0 - delta_bytes.len() as f64 / full_bytes as f64) * 100.0
    );

    // Re-evaluate context scores after update
    println!("\n📊 Re-scoring after update:");
    let updated_envelope = docs[1].to_lnmp_envelope();
    let updated_profile = scorer.score_envelope(&updated_envelope, now);

    println!(
        "  {} | Freshness: {:.3} | Composite: {:.3} (↑ due to update)",
        docs[1].id,
        updated_profile.freshness_score,
        updated_profile.composite_score()
    );

    println!("\n✅ Demo complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Context profiling for RAG prioritization");
    println!("   • Embedding delta encoding (bandwidth optimization)");
    println!("   • Freshness-based scoring for cache decisions");
    println!("   • Source trust evaluation");
    println!("   • Meta crate integration (lnmp::*)");
}
