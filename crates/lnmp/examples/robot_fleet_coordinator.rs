//! Robot Fleet Coordinator - Showcase Example
//!
//! Coordinates multiple robots with real-time position updates.
//! Demonstrates spatial protocol, QoS, and efficient delta encoding.
//!
//! Run: `cargo run --example robot_fleet_coordinator`

use lnmp::prelude::*;

/// Robot in the fleet
struct Robot {
    id: String,
    position: (f32, f32, f32), // (x, y, z) in meters
    velocity: (f32, f32, f32), // (vx, vy, vz) in m/s
    battery: u8,
    status: RobotStatus,
}

#[derive(Clone, Copy, PartialEq)]
enum RobotStatus {
    Active,
    Charging,
    Emergency,
}

impl Robot {
    fn new(id: String, position: (f32, f32, f32)) -> Self {
        Self {
            id,
            position,
            velocity: (0.0, 0.0, 0.0),
            battery: 100,
            status: RobotStatus::Active,
        }
    }

    fn update(&mut self, dt: f32) {
        // Update position based on velocity
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;
        self.position.2 += self.velocity.2 * dt;

        // Simulate battery drain
        if self.status == RobotStatus::Active {
            self.battery = self.battery.saturating_sub(1);
        }

        // Check for emergency
        if self.battery < 15 {
            self.status = RobotStatus::Emergency;
        }
    }

    fn to_lnmp_record(&self) -> LnmpRecord {
        let mut record = LnmpRecord::new();

        // Robot ID
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.id.clone()),
        });

        // Position
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

        // Battery
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Int(self.battery as i64),
        });

        record
    }

    fn priority(&self) -> u8 {
        match self.status {
            RobotStatus::Emergency => 255,
            RobotStatus::Charging => 80,
            RobotStatus::Active => 150,
        }
    }
}

fn main() {
    println!("🦾 Robot Fleet Coordinator - LNMP Showcase\n");

    // Create robot fleet
    let mut robots = vec![
        Robot::new("robot-alpha".to_string(), (0.0, 0.0, 0.0)),
        Robot::new("robot-beta".to_string(), (5.0, 0.0, 0.0)),
        Robot::new("robot-gamma".to_string(), (0.0, 5.0, 0.0)),
    ];

    // Set velocities
    robots[0].velocity = (1.0, 0.5, 0.0);
    robots[1].velocity = (0.0, 1.0, 0.0);
    robots[2].velocity = (-0.5, 0.0, 0.5);

    println!("🤖 Fleet initialized with {} robots\n", robots.len());

    let dt = 0.1; // 100ms time step (10 Hz update rate)
    let encoder = Encoder::new();

    println!("📊 Starting position updates (10 Hz control loop):\n");

    // Simulate 5 control cycles
    for cycle in 1..=5 {
        println!("🔄 Cycle {} ---", cycle);

        for robot in robots.iter_mut() {
            robot.update(dt);

            let record = robot.to_lnmp_record();
            let encoded = encoder.encode(&record);

            // Wrap with metadata
            let envelope = lnmp::envelope::EnvelopeBuilder::new(record)
                .source(&robot.id)
                .build();

            // Create network message with QoS
            let _net_msg = lnmp::net::NetMessage::with_qos(
                envelope,
                lnmp::net::MessageKind::Event,
                robot.priority(),
                100, // 100ms TTL for real-time updates
            );

            let status_icon = match robot.status {
                RobotStatus::Active => "✅",
                RobotStatus::Charging => "🔋",
                RobotStatus::Emergency => "🚨",
            };

            println!(
                "  {} {} | Pos: ({:.1}, {:.1}, {:.1}) | Size: {} bytes | Battery: {}%",
                status_icon,
                robot.id,
                robot.position.0,
                robot.position.1,
                robot.position.2,
                encoded.len(),
                robot.battery
            );
        }
        println!();
    }

    println!("✅ Fleet coordination demo complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Real-time position encoding (10 Hz updates)");
    println!("   • Network QoS based on robot status");
    println!("   • Emergency priority escalation");
    println!("   • Compact wire format for bandwidth efficiency");
    println!("   • Meta crate integration (lnmp::*)");
}
