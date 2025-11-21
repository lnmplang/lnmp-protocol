use lnmp_spatial::protocol::{FrameMode, SpatialStreamer};
use lnmp_spatial::*;
use std::thread;
use std::time::{Duration, Instant};

/// Simulates a 1kHz control loop with packet loss and jitter analysis.
struct RobotArm {
    state: SpatialState,
}

impl RobotArm {
    fn new() -> Self {
        Self {
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
                    vx: 0.1,
                    vy: 0.0,
                    vz: 0.0,
                }), // 0.1 units/ms
                acceleration: None,
            },
        }
    }

    fn update(&mut self, dt_ms: f32) {
        if let (Some(pos), Some(vel)) = (&mut self.state.position, &self.state.velocity) {
            pos.x += vel.vx * dt_ms;
            pos.y += vel.vy * dt_ms;
            pos.z += vel.vz * dt_ms;
        }
    }
}

fn get_timestamp_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting 1kHz Hybrid Spatial Protocol Simulation...\n");

    let mut robot = RobotArm::new();
    let mut sender_streamer = SpatialStreamer::new(100); // ABS every 100 frames
    let mut receiver_streamer = SpatialStreamer::new(100);

    let _target_frequency_hz = 1000;
    let target_dt = Duration::from_micros(1000); // 1ms

    let mut abs_count = 0;
    let mut delta_count = 0;
    let mut packet_loss_count = 0;

    let start_time = Instant::now();

    for tick in 0..500 {
        // Run for 500ms
        let tick_start = Instant::now();

        // 1. Update Physics
        robot.update(1.0); // 1ms step

        // 2. Sender: Generate Frame
        let timestamp = get_timestamp_us();
        let frame = sender_streamer.next_frame(&robot.state, timestamp)?;

        // Track frame types
        match frame.header.mode {
            FrameMode::Absolute => abs_count += 1,
            FrameMode::Delta => delta_count += 1,
        }

        // 3. Simulate Packet Loss (10% loss rate)
        let packet_lost = tick % 10 == 7; // Every 10th frame (at tick 7)

        if packet_lost {
            packet_loss_count += 1;
            // Skip this frame
        } else {
            // 4. Receiver: Process Frame
            match receiver_streamer.process_frame(&frame) {
                Ok(state) => {
                    // Successful update
                    if tick % 100 == 0 {
                        println!(
                            "[Tick {:03}] Mode: {:?}, Seq: {}, Pos: {:.2},{:.2},{:.2}",
                            tick,
                            frame.header.mode,
                            frame.header.sequence_id,
                            state.position.unwrap().x,
                            state.position.unwrap().y,
                            state.position.unwrap().z
                        );
                    }
                }
                Err(e) => {
                    // Error (e.g., waiting for ABS after packet loss)
                    if tick % 100 == 0 {
                        println!("[Tick {:03}] ⚠️  ERROR: {}", tick, e);
                    }
                }
            }
        }

        // 5. Maintain 1kHz Rate
        let elapsed = tick_start.elapsed();
        if elapsed < target_dt {
            thread::sleep(target_dt - elapsed);
        }
    }

    let total_time = start_time.elapsed();

    println!("\n📊 Simulation Results:");
    println!("  Total Time: {:.2}ms", total_time.as_secs_f64() * 1000.0);
    println!("  ABS Frames: {}", abs_count);
    println!("  DELTA Frames: {}", delta_count);
    println!("  Packet Loss: {} (simulated)", packet_loss_count);
    println!(
        "  Compression Ratio: {:.2}% DELTA",
        (delta_count as f64 / (abs_count + delta_count) as f64) * 100.0
    );

    Ok(())
}
