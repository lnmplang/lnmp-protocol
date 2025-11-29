//! Spatial protocol example
//!
//! This example demonstrates:
//! - Creating spatial frames
//! - Using SpatialStreamer
//! - LNMP encoding for spatial data

use lnmp::codec::Encoder;
use lnmp::core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp::spatial::protocol::SpatialStreamer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Spatial Protocol Example ===\n");

    // 1. Create a spatial streamer
    println!("--- Spatial Streamer ---");
    let _streamer = SpatialStreamer::new(1000); // 1000ms interval
    println!("Created spatial streamer");

    // 2. Create a spatial record
    println!("\n--- Spatial Data Record ---");
    let mut spatial_record = LnmpRecord::new();

    spatial_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("drone-001".to_string()),
    });

    spatial_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(1.5), // position_x
    });

    spatial_record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Float(2.0), // position_y
    });

    spatial_record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(5.0), // position_z
    });

    println!(
        "Created spatial record with {} fields",
        spatial_record.fields().len()
    );

    // 3. Encode the spatial record
    println!("\n--- LNMP Encoding ---");
    let encoder = Encoder::new();
    let encoded = encoder.encode(&spatial_record);
    println!("Encoded spatial data:\n{}", encoded);

    // 4. Spatial streaming info
    println!("\n--- Spatial Streaming ---");
    println!("Spatial streamer provides hybrid protocol:");
    println!("  • High-frequency binary updates for position/velocity");
    println!("  • Low-frequency LNMP messages for metadata");
    println!("  • Predictive delta compression for efficiency");

    println!("\n✅ Spatial protocol operations completed successfully!");

    Ok(())
}
