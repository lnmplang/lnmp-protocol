//! Complete workflow example
//!
//! This example demonstrates a real-world scenario:
//! - Creating sensor data records
//! - Working with embeddings
//! - Encoding for transmission

use lnmp::core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp::embedding::Vector;
use lnmp::net::MessageKind;
use lnmp::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Complete Workflow Example ===\n");
    println!("Scenario: Smart warehouse sensor network\n");

    // 1. Create sensor data record
    println!("--- Step 1: Sensor Data Record ---");
    let mut sensor_record = LnmpRecord::new();

    sensor_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("temp-sensor-42".to_string()),
    });

    sensor_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(23.5),
    });

    sensor_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Float(65.2),
    });

    sensor_record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Int(1701234567),
    });

    println!(
        "Created sensor record with {} fields",
        sensor_record.fields().len()
    );

    // 2. Create embedding vector for semantic search
    println!("\n--- Step 2: Embedding Generation ---");
    // In real scenario, this would be from ML model
    let embedding = Vector::from_f32(vec![
        0.23, 0.65, 0.12, 0.89, 0.34, // Environmental conditions
        0.45, 0.78, 0.23, 0.56, 0.91, // Location features
    ]);
    println!("Generated embedding: dim={}", embedding.dim);

    // 3. Add embedding to record
    sensor_record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Embedding(embedding),
    });

    // 4. Encode for transmission
    println!("\n--- Step 3: Encoding ---");
    let encoder = Encoder::new();
    let encoded = encoder.encode(&sensor_record);
    println!("Encoded message size: {} bytes", encoded.len());
    println!(
        "Preview (first 100 chars):\n{}...",
        &encoded[..encoded.len().min(100)]
    );

    // 5. Network message info
    println!("\n--- Step 4: Network Transmission ---");
    let msg_kind = MessageKind::Event;
    println!("Message kind: {}", msg_kind);
    println!("Priority: {}", msg_kind.default_priority());
    println!("TTL: {}ms", msg_kind.default_ttl_ms());
    println!("📡 Message ready for transmission");

    println!("\n📊 Summary:");
    println!("  - Sensor data: {} fields", sensor_record.fields().len());
    println!("  - Embedding: 10 dimensions");
    println!("  - Encoded size: {} bytes", encoded.len());
    println!("  - Protocol: LNMP v0.5.12");

    println!("\n✅ Complete workflow executed successfully!");

    Ok(())
}
