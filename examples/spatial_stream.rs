use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_spatial::delta::Delta;
use lnmp_spatial::*;
use std::thread;
use std::time::Duration;

/// Simulates a drone flying a path and streaming telemetry.
struct Drone {
    id: String,
    state: SpatialState,
}

impl Drone {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            state: SpatialState {
                position: Some(Position3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                rotation: Some(Rotation {
                    pitch: 0.0,
                    yaw: 0.0,
                    roll: 0.0,
                }),
                velocity: Some(Velocity {
                    vx: 1.0,
                    vy: 0.0,
                    vz: 0.1,
                }), // Moving X+ and Z+
                acceleration: None,
            },
        }
    }

    fn update(&mut self, dt: f32) {
        if let (Some(pos), Some(vel)) = (&mut self.state.position, &self.state.velocity) {
            pos.x += vel.vx * dt;
            pos.y += vel.vy * dt;
            pos.z += vel.vz * dt;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Starting Spatial Telemetry Stream...");

    let mut drone = Drone::new("drone_alpha");
    let mut last_state = drone.state.clone();

    // Simulate streaming loop
    for i in 0..5 {
        // 1. Update Physics
        drone.update(1.0); // 1 second step

        // 2. Compute Delta
        let delta = SpatialState::compute_delta(&last_state, &drone.state);

        // 3. Encode Delta (Simulating binary payload)
        let delta_val = SpatialValue::S13(delta.clone());
        let mut payload = Vec::new();
        encode_spatial(&delta_val, &mut payload)?;

        // 4. Create LNMP Record (Stream Frame)
        // In a real stream, this would be wrapped in a CHUNK frame.
        // Here we just show the record structure.
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(drone.id.clone()),
        });
        record.add_field(LnmpField {
            fid: 10, // Spatial Data
            value: LnmpValue::String(hex::encode(&payload)),
        });

        println!("\n[Tick {}] Transmitting Telemetry...", i);
        println!("  Current Pos: {:?}", drone.state.position.unwrap());
        if let SpatialDelta::State {
            position: Some(p), ..
        } = delta
        {
            println!(
                "  Delta Sent:  dx={:.2}, dy={:.2}, dz={:.2}",
                p.dx, p.dy, p.dz
            );
        }
        println!("  Payload Size: {} bytes", payload.len());

        // 5. Receiver Side (Decode & Apply)
        // Simulate receiver applying delta to their local shadow state
        let decoded_val = decode_spatial(&mut payload.as_slice())?;
        if let SpatialValue::S13(received_delta) = decoded_val {
            // Manual cast for example since we know type
            if let SpatialDelta::State { .. } = received_delta {
                let applied = SpatialState::apply_delta(&last_state, &received_delta);
                assert_eq!(applied.position.unwrap().x, drone.state.position.unwrap().x);
                println!(
                    "  ✅ Receiver State Synced: {:?}",
                    applied.position.unwrap()
                );
            }
        }

        last_state = drone.state.clone();
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
