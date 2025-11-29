//! Comprehensive module access demonstration
//!
//! This example shows ALL available LNMP modules through the SDK


fn main() {
    println!("=== LNMP SDK - All Available Modules ===\n");

    // 1. Core - Base types and records
    println!("✅ lnmp::core - Core types");
    println!("   • LnmpRecord, LnmpField, LnmpValue");
    println!("   • TypeHint, FieldId");

    // 2. Codec - Encoding and parsing
    println!("\n✅ lnmp::codec - Encoding/Decoding");
    println!("   • Encoder, Parser");

    // 3. Embedding - Vector operations
    println!("\n✅ lnmp::embedding - Vector Operations");
    println!("   • Vector, VectorDelta");
    println!("   • DeltaChange, UpdateStrategy");
    let vec = lnmp::embedding::Vector::from_f32(vec![0.1, 0.2, 0.3]);
    println!("   Example: Created vector with dim={}", vec.dim);

    // 4. Envelope - Metadata wrapping
    println!("\n✅ lnmp::envelope - Metadata Wrapping");
    println!("   • Envelope, Metadata");

    // 5. LLB - Large Language Blocks
    println!("\n✅ lnmp::llb - Large Language Blocks");
    println!("   • LLB processing utilities");

    // 6. Net - Network behavior layer
    println!("\n✅ lnmp::net - Network Layer");
    println!("   • MessageKind, NetMessage");
    println!("   • RoutingPolicy, QoS");
    let msg_kind = lnmp::net::MessageKind::Event;
    println!(
        "   Example: {} (priority={})",
        msg_kind,
        msg_kind.default_priority()
    );

    // 7. Quant - Quantization utilities
    println!("\n✅ lnmp::quant - Quantization");
    println!("   • Quantization utilities for efficient representation");

    // 8. Sanitize - Data validation
    println!("\n✅ lnmp::sanitize - Data Sanitization");
    println!("   • Input validation and sanitization");

    // 9. SFE - Secure Function Evaluation
    println!("\n✅ lnmp::sfe - Secure Function Evaluation");
    println!("   • Privacy-preserving computation primitives");

    // 10. Spatial - 3D/Spatial streaming
    println!("\n✅ lnmp::spatial - Spatial Streaming");
    println!("   • SpatialStreamer, SpatialFrame");
    let _streamer = lnmp::spatial::protocol::SpatialStreamer::new(1000);
    println!("   Example: Created spatial streamer with 1000ms interval");

    // 11. Transport - Protocol bindings
    println!("\n✅ lnmp::transport - Transport Layer");
    println!("   • HTTP, Kafka, gRPC, NATS bindings");
    println!("   • W3C Trace Context support");

    println!("\n{}", "=".repeat(50));
    println!("🎉 Total 11 modules accessible through SDK!");
    println!("{}", "=".repeat(50));
    println!("\n📦 Usage:");
    println!("   use lnmp::module_name::Type;");
    println!("   or");
    println!("   use lnmp::prelude::*; // Commonly used types");
}
