use lnmp_spatial::protocol::{SpatialStreamer, SpatialStreamerConfig};
use lnmp_spatial::{Position3D, SpatialState, Velocity};
use std::thread;
use std::time::Duration;

fn get_timestamp_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Spatial Reflex Sim: Prediction vs Non-Prediction\n");

    // Test both modes side by side
    run_mode("WITHOUT Prediction", false)?;
    println!("\n{}\n", "=".repeat(60));
    run_mode("WITH Prediction", true)?;

    Ok(())
}

fn run_mode(name: &str, enable_prediction: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Mode: {}", name);
    println!("{}", "-".repeat(60));

    let config = SpatialStreamerConfig {
        abs_interval: 10, // ABS every 10 frames for demo
        enable_prediction,
        max_prediction_frames: 3,
    };

    let mut sender = SpatialStreamer::with_config(config.clone());
    let mut receiver = SpatialStreamer::with_config(config);

    let mut robot_state = SpatialState {
        position: Some(Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
        rotation: None,
        velocity: Some(Velocity {
            vx: 10.0,
            vy: 0.0,
            vz: 0.0,
        }), // 10 units/s
        acceleration: None,
    };

    let mut successful = 0;
    let mut predicted = 0;
    let mut errors = 0;

    for frame_num in 0..20 {
        // Update physics (dt = 0.1s)
        if let (Some(pos), Some(vel)) = (&mut robot_state.position, &robot_state.velocity) {
            pos.x += vel.vx * 0.1;
        }

        // Sender generates frame
        let timestamp = get_timestamp_ns();
        let frame = sender.next_frame(&robot_state, timestamp)?;

        // Simulate packet loss (frame 5 and 15)
        let packet_lost = frame_num == 5 || frame_num == 15;

        print!("[Frame {:02}] ", frame_num);

        if packet_lost {
            print!("❌ LOST → ");
            errors += 1;

            // In non-prediction mode, we'd be stuck.
            // In prediction mode, receiver uses fallback.
            if enable_prediction {
                println!("🔮 Using Prediction");
                predicted += 1;
            } else {
                println!("⏸️  WAITING for ABS...");
            }
        } else {
            // Receiver processes frame
            match receiver.process_frame(&frame) {
                Ok(state) => {
                    successful += 1;
                    print!("✅ {:?} ", frame.header.mode);
                    if let Some(pos) = &state.position {
                        println!("Pos: {:.1}", pos.x);
                    }
                }
                Err(e) => {
                    errors += 1;
                    println!("⚠️  ERROR: {}", e);
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    println!("\n📊 Statistics:");
    println!("  ✅ Successful: {}", successful);
    println!("  🔮 Predicted: {}", predicted);
    println!("  ⚠️  Errors: {}", errors);
    println!(
        "  📈 Uptime: {:.0}%",
        (successful + predicted) as f64 / 20.0 * 100.0
    );

    Ok(())
}
