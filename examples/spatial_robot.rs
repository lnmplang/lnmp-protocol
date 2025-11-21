use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_embedding::Vector;
use lnmp_spatial::*;

/// Represents a physical object with both semantic meaning (embedding) and physical location.
struct SmartObject {
    id: String,
    embedding: Vector,
    position: Position3D,
}

/// Represents a robot with spatial awareness.
struct Robot {
    state: SpatialState,
    constraints: SpatialConstraints,
}

impl Robot {
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
                    vx: 0.0,
                    vy: 0.0,
                    vz: 0.0,
                }),
                acceleration: Some(Acceleration {
                    ax: 0.0,
                    ay: 0.0,
                    az: 0.0,
                }),
            },
            constraints: SpatialConstraints::default(),
        }
    }

    fn execute_command(&mut self, target_pos: Position3D) -> Result<(), SpatialError> {
        println!("🤖 Robot receiving command: Move to {:?}", target_pos);

        // 1. Validate current state
        validate_spatial_state(&self.state, &self.constraints)?;

        // 2. Calculate path (simplified: direct line)
        let current_pos = self.state.position.unwrap();
        let distance = spatial_distance(&current_pos, &target_pos);

        println!("📏 Distance to target: {:.2} units", distance);

        // 3. Simulate movement (update position)
        // In a real system, this would involve velocity/acceleration over time.
        // Here we just "teleport" for the example, but we check if the jump is "safe"
        // (simulating a max step size check).

        if distance > 1000.0 {
            return Err(SpatialError::ValidationError(
                "Target too far for single step".into(),
            ));
        }

        self.state.position = Some(target_pos);
        println!(
            "✅ Movement complete. New Position: {:?}",
            self.state.position.unwrap()
        );

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting LNMP-Spatial Robot Simulation...\n");

    // 1. Setup Environment
    // Create a "Red Box" object
    // Embedding is dummy data for example
    let red_box = SmartObject {
        id: "obj_001".to_string(),
        embedding: Vector::from_f32(vec![0.9, 0.1, 0.0]),
        position: Position3D {
            x: 50.0,
            y: 50.0,
            z: 0.0,
        },
    };

    println!(
        "📦 Object Detected: {} at {:?}",
        red_box.id, red_box.position
    );

    // 2. Create an LNMP Record representing a "Move Command"
    // This record combines:
    // - F1: Target Object ID (String)
    // - F2: Target Embedding (Vector) - Semantic confirmation
    // - F10: Target Position (Spatial) - Physical destination

    let mut command_record = LnmpRecord::new();
    command_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String(red_box.id.clone()),
    });
    command_record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Embedding(red_box.embedding.clone()),
    });

    // We need to encode the Spatial type into a byte buffer to store it in LnmpValue?
    // Wait, LnmpValue doesn't have a direct "Spatial" variant yet in lnmp-core.
    // We usually store binary data in LnmpValue::String (base64) or we need a Binary variant.
    // Looking at lnmp-core, there is no Byte array variant exposed directly except maybe String?
    // Actually, for this example, let's assume we extend LnmpValue or use a custom field.
    // OR, we can use the new `LnmpValue::NestedRecord` to structure it,
    // BUT `lnmp-spatial` is a binary protocol.

    // Let's simulate the binary payload of the spatial data.
    let mut spatial_payload = Vec::new();
    let target_pos_val = SpatialValue::S2(red_box.position);
    encode_spatial(&target_pos_val, &mut spatial_payload)?;

    // For now, we'll store it as a hex string in the record for demonstration,
    // since LnmpValue doesn't have a raw bytes type (it has String).
    // In a real binary LNMP container, this would be a raw byte field.
    let hex_payload = hex::encode(&spatial_payload);

    command_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String(hex_payload),
    });

    println!("📝 Command Record Created: {:?}", command_record);

    // 3. Robot Execution Loop
    let mut robot = Robot::new();

    // Decode the command
    // In a real scenario, we'd parse the LNMP container.
    // Here we extract F10.
    if let Some(field) = command_record.get_field(10) {
        if let LnmpValue::String(hex_str) = &field.value {
            let bytes = hex::decode(hex_str)?;
            let mut slice = bytes.as_slice();
            let decoded_spatial = decode_spatial(&mut slice)?;

            if let SpatialValue::S2(target_pos) = decoded_spatial {
                robot.execute_command(target_pos)?;
            }
        }
    }

    Ok(())
}
