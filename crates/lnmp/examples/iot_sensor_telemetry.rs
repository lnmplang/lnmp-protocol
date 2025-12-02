//! IoT Sensor Telemetry - Showcase Example
//!
//! Simulates a network of IoT sensors sending real-time telemetry data.
//! Demonstrates spatial encoding, delta compression, network routing, and envelopes.
//!
//! Run: `cargo run --example iot_sensor_telemetry`

use lnmp::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Simulated IoT sensor
struct Sensor {
    id: String,
    position: (f32, f32, f32), // (x, y, z) in meters
    temperature: f32,
    humidity: f32,
    battery: u8,
    alert_level: AlertLevel,
}

#[derive(Clone, Copy, PartialEq)]
enum AlertLevel {
    Normal,
    Warning,
    Critical,
}

impl Sensor {
    fn new(id: String, position: (f32, f32, f32)) -> Self {
        Self {
            id,
            position,
            temperature: 20.0,
            humidity: 50.0,
            battery: 100,
            alert_level: AlertLevel::Normal,
        }
    }

    fn update_readings(&mut self) {
        // Simulate sensor drift
        self.temperature += (rand::random() - 0.5) * 2.0;
        self.humidity += (rand::random() - 0.5) * 5.0;
        self.battery = self.battery.saturating_sub(1);

        // Determine alert level
        self.alert_level = if self.temperature > 40.0 || self.battery < 20 {
            AlertLevel::Critical
        } else if self.temperature > 30.0 || self.battery < 50 {
            AlertLevel::Warning
        } else {
            AlertLevel::Normal
        };
    }

    fn to_lnmp_record(&self) -> LnmpRecord {
        let mut record = LnmpRecord::new();

        // Field 1: Sensor ID
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.id.clone()),
        });

        // Field 10-12: Position (x, y, z)
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Float(self.position.0 as f64),
        });
        record.add_field(LnmpField {
            fid: 11,
            value: LnmpValue::Float(self.position.1 as f64),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Float(self.position.2 as f64),
        });

        // Field 20: Temperature (°C)
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Float(self.temperature as f64),
        });

        // Field 21: Humidity (%)
        record.add_field(LnmpField {
            fid: 21,
            value: LnmpValue::Float(self.humidity as f64),
        });

        // Field 30: Battery (%)
        record.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Int(self.battery as i64),
        });

        record
    }

    fn priority(&self) -> u8 {
        match self.alert_level {
            AlertLevel::Critical => 255,
            AlertLevel::Warning => 180,
            AlertLevel::Normal => 100,
        }
    }
}

// Simple random number generator for demo
mod rand {
    static mut SEED: u64 = 0x123456789ABCDEF;

    pub fn random() -> f32 {
        unsafe {
            SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((SEED >> 32) as f32) / (u32::MAX as f32)
        }
    }
}

fn main() {
    println!("🌐 IoT Sensor Telemetry - LNMP Showcase\n");
    println!("Simulating 3 sensors sending telemetry data...\n");

    // Create sensors
    let mut sensors = vec![
        Sensor::new("sensor-001".to_string(), (10.0, 20.0, 0.5)),
        Sensor::new("sensor-002".to_string(), (15.0, 25.0, 1.2)),
        Sensor::new("sensor-003".to_string(), (12.0, 18.0, 0.8)),
    ];

    let encoder = Encoder::new();

    // Simulate 5 telemetry cycles
    for cycle in 1..=5 {
        println!("📡 Cycle {} ---", cycle);
        let now = current_timestamp();

        for sensor in &mut sensors {
            sensor.update_readings();

            // Create LNMP record
            let record = sensor.to_lnmp_record();

            // Wrap in envelope with metadata
            let envelope = lnmp::envelope::EnvelopeBuilder::new(record)
                .timestamp(now)
                .source(&sensor.id)
                .build();

            // Encode to LNMP text format
            let lnmp_text = encoder.encode(&envelope.record);

            // Create network message with QoS
            let _net_msg = lnmp::net::NetMessage::with_qos(
                envelope,
                lnmp::net::MessageKind::Event,
                sensor.priority(),
                5000, // 5s TTL
            );

            // Determine routing
            let importance = sensor.priority() as f64 / 255.0;
            let route = if importance > 0.7 {
                "🚨 PRIORITY QUEUE"
            } else {
                "📮 STANDARD QUEUE"
            };

            // Display
            println!(
                "  {} | Temp: {:.1}°C | Battery: {}% | {} | Size: {} bytes",
                sensor.id,
                sensor.temperature,
                sensor.battery,
                route,
                lnmp_text.len()
            );
        }

        println!();
    }

    println!("✅ Demo complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • LNMP encoding for structured IoT data");
    println!("   • Envelope metadata (timestamp, source)");
    println!("   • Priority-based routing (QoS)");
    println!("   • Compact wire format for bandwidth efficiency");
}
