use crate::error::SpatialError;
use crate::types::*;
use bytes::BufMut;

pub fn encode_spatial(value: &SpatialValue, buf: &mut Vec<u8>) -> Result<(), SpatialError> {
    // Flag for Spatial Mode (0x07) - This might be handled at a higher level,
    // but per spec: [SPATIAL_FLAG][TYPE_ID][FLOAT32]...
    // Assuming SPATIAL_FLAG is the mode byte 0x07, which is likely written by the container.
    // Here we write [TYPE_ID][DATA]

    match value {
        SpatialValue::S1(v) => {
            buf.put_u8(SpatialType::Position2D as u8);
            buf.put_f32(v.x);
            buf.put_f32(v.y);
        }
        SpatialValue::S2(v) => {
            buf.put_u8(SpatialType::Position3D as u8);
            buf.put_f32(v.x);
            buf.put_f32(v.y);
            buf.put_f32(v.z);
        }
        SpatialValue::S3(v) => {
            buf.put_u8(SpatialType::Rotation as u8);
            buf.put_f32(v.pitch);
            buf.put_f32(v.yaw);
            buf.put_f32(v.roll);
        }
        SpatialValue::S4(v) => {
            buf.put_u8(SpatialType::Velocity as u8);
            buf.put_f32(v.vx);
            buf.put_f32(v.vy);
            buf.put_f32(v.vz);
        }
        SpatialValue::S5(v) => {
            buf.put_u8(SpatialType::Acceleration as u8);
            buf.put_f32(v.ax);
            buf.put_f32(v.ay);
            buf.put_f32(v.az);
        }
        SpatialValue::S6(v) => {
            buf.put_u8(SpatialType::BoundingBox as u8);
            buf.put_f32(v.min_x);
            buf.put_f32(v.min_y);
            buf.put_f32(v.min_z);
            buf.put_f32(v.max_x);
            buf.put_f32(v.max_y);
            buf.put_f32(v.max_z);
        }
        SpatialValue::S7(v) => {
            buf.put_u8(SpatialType::Quaternion as u8);
            buf.put_f32(v.qx);
            buf.put_f32(v.qy);
            buf.put_f32(v.qz);
            buf.put_f32(v.qw);
        }
        SpatialValue::S8(v) => {
            buf.put_u8(SpatialType::Path as u8);
            buf.put_u32(v.points.len() as u32);
            for point in &v.points {
                buf.put_f32(point.x);
                buf.put_f32(point.y);
                buf.put_f32(point.z);
            }
        }
        SpatialValue::S9(v) => {
            buf.put_u8(SpatialType::Transform as u8);
            buf.put_f32(v.position.x);
            buf.put_f32(v.position.y);
            buf.put_f32(v.position.z);
            buf.put_f32(v.rotation.pitch);
            buf.put_f32(v.rotation.yaw);
            buf.put_f32(v.rotation.roll);
            buf.put_f32(v.scale.x);
            buf.put_f32(v.scale.y);
            buf.put_f32(v.scale.z);
        }
        SpatialValue::S10(v) => {
            buf.put_u8(SpatialType::SpatialState as u8);
            // Bitmask for presence: [Pos][Rot][Vel][Acc]
            let mut mask: u8 = 0;
            if v.position.is_some() {
                mask |= 0x01;
            }
            if v.rotation.is_some() {
                mask |= 0x02;
            }
            if v.velocity.is_some() {
                mask |= 0x04;
            }
            if v.acceleration.is_some() {
                mask |= 0x08;
            }
            buf.put_u8(mask);

            if let Some(p) = v.position {
                buf.put_f32(p.x);
                buf.put_f32(p.y);
                buf.put_f32(p.z);
            }
            if let Some(r) = v.rotation {
                buf.put_f32(r.pitch);
                buf.put_f32(r.yaw);
                buf.put_f32(r.roll);
            }
            if let Some(vel) = v.velocity {
                buf.put_f32(vel.vx);
                buf.put_f32(vel.vy);
                buf.put_f32(vel.vz);
            }
            if let Some(acc) = v.acceleration {
                buf.put_f32(acc.ax);
                buf.put_f32(acc.ay);
                buf.put_f32(acc.az);
            }
        }
        SpatialValue::S11(v) => {
            buf.put_u8(SpatialType::PositionDelta as u8);
            buf.put_f32(v.dx);
            buf.put_f32(v.dy);
            buf.put_f32(v.dz);
        }
        SpatialValue::S12(v) => {
            buf.put_u8(SpatialType::RotationDelta as u8);
            buf.put_f32(v.d_pitch);
            buf.put_f32(v.d_yaw);
            buf.put_f32(v.d_roll);
        }
        SpatialValue::S13(v) => {
            buf.put_u8(SpatialType::SpatialDelta as u8);
            match v {
                SpatialDelta::Position(p) => {
                    buf.put_u8(0x01); // Type: Position
                    buf.put_f32(p.dx);
                    buf.put_f32(p.dy);
                    buf.put_f32(p.dz);
                }
                SpatialDelta::Rotation(r) => {
                    buf.put_u8(0x02); // Type: Rotation
                    buf.put_f32(r.d_pitch);
                    buf.put_f32(r.d_yaw);
                    buf.put_f32(r.d_roll);
                }
                SpatialDelta::State {
                    position,
                    rotation,
                    velocity,
                    acceleration,
                } => {
                    buf.put_u8(0x03); // Type: State
                                      // Bitmask: [Pos][Rot][Vel][Acc]
                    let mut mask: u8 = 0;
                    if position.is_some() {
                        mask |= 0x01;
                    }
                    if rotation.is_some() {
                        mask |= 0x02;
                    }
                    if velocity.is_some() {
                        mask |= 0x04;
                    }
                    if acceleration.is_some() {
                        mask |= 0x08;
                    }
                    buf.put_u8(mask);

                    if let Some(p) = position {
                        buf.put_f32(p.dx);
                        buf.put_f32(p.dy);
                        buf.put_f32(p.dz);
                    }
                    if let Some(r) = rotation {
                        buf.put_f32(r.d_pitch);
                        buf.put_f32(r.d_yaw);
                        buf.put_f32(r.d_roll);
                    }
                    if let Some(vel) = velocity {
                        buf.put_f32(vel.vx);
                        buf.put_f32(vel.vy);
                        buf.put_f32(vel.vz);
                    }
                    if let Some(acc) = acceleration {
                        buf.put_f32(acc.ax);
                        buf.put_f32(acc.ay);
                        buf.put_f32(acc.az);
                    }
                }
            }
        }
    }
    Ok(())
}
