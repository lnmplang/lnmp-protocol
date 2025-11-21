use crate::error::SpatialError;
use crate::types::*;
use bytes::Buf;

pub fn decode_spatial(buf: &mut &[u8]) -> Result<SpatialValue, SpatialError> {
    if buf.remaining() < 1 {
        return Err(SpatialError::DecodeError(
            "Insufficient data for SpatialType".into(),
        ));
    }

    let type_id = buf.get_u8();

    match type_id {
        0x01 => {
            // Position2D
            if buf.remaining() < 8 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Position2D".into(),
                ));
            }
            let x = buf.get_f32();
            let y = buf.get_f32();
            Ok(SpatialValue::S1(Position2D { x, y }))
        }
        0x02 => {
            // Position3D
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Position3D".into(),
                ));
            }
            let x = buf.get_f32();
            let y = buf.get_f32();
            let z = buf.get_f32();
            Ok(SpatialValue::S2(Position3D { x, y, z }))
        }
        0x03 => {
            // Rotation
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Rotation".into(),
                ));
            }
            let pitch = buf.get_f32();
            let yaw = buf.get_f32();
            let roll = buf.get_f32();
            Ok(SpatialValue::S3(Rotation { pitch, yaw, roll }))
        }
        0x04 => {
            // Velocity
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Velocity".into(),
                ));
            }
            let vx = buf.get_f32();
            let vy = buf.get_f32();
            let vz = buf.get_f32();
            Ok(SpatialValue::S4(Velocity { vx, vy, vz }))
        }
        0x05 => {
            // Acceleration
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Acceleration".into(),
                ));
            }
            let ax = buf.get_f32();
            let ay = buf.get_f32();
            let az = buf.get_f32();
            Ok(SpatialValue::S5(Acceleration { ax, ay, az }))
        }
        0x06 => {
            // BoundingBox
            if buf.remaining() < 24 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for BoundingBox".into(),
                ));
            }
            let min_x = buf.get_f32();
            let min_y = buf.get_f32();
            let min_z = buf.get_f32();
            let max_x = buf.get_f32();
            let max_y = buf.get_f32();
            let max_z = buf.get_f32();
            Ok(SpatialValue::S6(BoundingBox {
                min_x,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
            }))
        }
        0x07 => {
            // Quaternion
            if buf.remaining() < 16 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Quaternion".into(),
                ));
            }
            let qx = buf.get_f32();
            let qy = buf.get_f32();
            let qz = buf.get_f32();
            let qw = buf.get_f32();
            Ok(SpatialValue::S7(Quaternion { qx, qy, qz, qw }))
        }
        0x08 => {
            // Path
            if buf.remaining() < 4 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Path length".into(),
                ));
            }
            let len = buf.get_u32() as usize;
            if buf.remaining() < len * 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Path points".into(),
                ));
            }
            let mut points = Vec::with_capacity(len);
            for _ in 0..len {
                let x = buf.get_f32();
                let y = buf.get_f32();
                let z = buf.get_f32();
                points.push(Position3D { x, y, z });
            }
            Ok(SpatialValue::S8(Path { points }))
        }
        0x09 => {
            // Transform
            if buf.remaining() < 36 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for Transform".into(),
                ));
            }
            let px = buf.get_f32();
            let py = buf.get_f32();
            let pz = buf.get_f32();
            let rp = buf.get_f32();
            let ry = buf.get_f32();
            let rr = buf.get_f32();
            let sx = buf.get_f32();
            let sy = buf.get_f32();
            let sz = buf.get_f32();
            Ok(SpatialValue::S9(Transform {
                position: Position3D {
                    x: px,
                    y: py,
                    z: pz,
                },
                rotation: Rotation {
                    pitch: rp,
                    yaw: ry,
                    roll: rr,
                },
                scale: Position3D {
                    x: sx,
                    y: sy,
                    z: sz,
                },
            }))
        }
        0x10 | 0x0A => {
            // SpatialState (0x0A in types.rs, checking hex)
            if buf.remaining() < 1 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for SpatialState mask".into(),
                ));
            }
            let mask = buf.get_u8();

            let position = if mask & 0x01 != 0 {
                if buf.remaining() < 12 {
                    return Err(SpatialError::DecodeError(
                        "Insufficient data for SpatialState Position".into(),
                    ));
                }
                Some(Position3D {
                    x: buf.get_f32(),
                    y: buf.get_f32(),
                    z: buf.get_f32(),
                })
            } else {
                None
            };

            let rotation = if mask & 0x02 != 0 {
                if buf.remaining() < 12 {
                    return Err(SpatialError::DecodeError(
                        "Insufficient data for SpatialState Rotation".into(),
                    ));
                }
                Some(Rotation {
                    pitch: buf.get_f32(),
                    yaw: buf.get_f32(),
                    roll: buf.get_f32(),
                })
            } else {
                None
            };

            let velocity = if mask & 0x04 != 0 {
                if buf.remaining() < 12 {
                    return Err(SpatialError::DecodeError(
                        "Insufficient data for SpatialState Velocity".into(),
                    ));
                }
                Some(Velocity {
                    vx: buf.get_f32(),
                    vy: buf.get_f32(),
                    vz: buf.get_f32(),
                })
            } else {
                None
            };

            let acceleration = if mask & 0x08 != 0 {
                if buf.remaining() < 12 {
                    return Err(SpatialError::DecodeError(
                        "Insufficient data for SpatialState Acceleration".into(),
                    ));
                }
                Some(Acceleration {
                    ax: buf.get_f32(),
                    ay: buf.get_f32(),
                    az: buf.get_f32(),
                })
            } else {
                None
            };

            Ok(SpatialValue::S10(SpatialState {
                position,
                rotation,
                velocity,
                acceleration,
            }))
        }
        0x0B => {
            // PositionDelta
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for PositionDelta".into(),
                ));
            }
            let dx = buf.get_f32();
            let dy = buf.get_f32();
            let dz = buf.get_f32();
            Ok(SpatialValue::S11(PositionDelta { dx, dy, dz }))
        }
        0x0C => {
            // RotationDelta
            if buf.remaining() < 12 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for RotationDelta".into(),
                ));
            }
            let d_pitch = buf.get_f32();
            let d_yaw = buf.get_f32();
            let d_roll = buf.get_f32();
            Ok(SpatialValue::S12(RotationDelta {
                d_pitch,
                d_yaw,
                d_roll,
            }))
        }
        0x0D => {
            // SpatialDelta
            if buf.remaining() < 1 {
                return Err(SpatialError::DecodeError(
                    "Insufficient data for SpatialDelta type".into(),
                ));
            }
            let delta_type = buf.get_u8();
            match delta_type {
                0x01 => {
                    // Position
                    if buf.remaining() < 12 {
                        return Err(SpatialError::DecodeError(
                            "Insufficient data for SpatialDelta::Position".into(),
                        ));
                    }
                    Ok(SpatialValue::S13(SpatialDelta::Position(PositionDelta {
                        dx: buf.get_f32(),
                        dy: buf.get_f32(),
                        dz: buf.get_f32(),
                    })))
                }
                0x02 => {
                    // Rotation
                    if buf.remaining() < 12 {
                        return Err(SpatialError::DecodeError(
                            "Insufficient data for SpatialDelta::Rotation".into(),
                        ));
                    }
                    Ok(SpatialValue::S13(SpatialDelta::Rotation(RotationDelta {
                        d_pitch: buf.get_f32(),
                        d_yaw: buf.get_f32(),
                        d_roll: buf.get_f32(),
                    })))
                }
                0x03 => {
                    // State
                    if buf.remaining() < 1 {
                        return Err(SpatialError::DecodeError(
                            "Insufficient data for SpatialDelta::State mask".into(),
                        ));
                    }
                    let mask = buf.get_u8();

                    let position = if mask & 0x01 != 0 {
                        if buf.remaining() < 12 {
                            return Err(SpatialError::DecodeError(
                                "Insufficient data for SpatialDelta Position".into(),
                            ));
                        }
                        Some(PositionDelta {
                            dx: buf.get_f32(),
                            dy: buf.get_f32(),
                            dz: buf.get_f32(),
                        })
                    } else {
                        None
                    };

                    let rotation = if mask & 0x02 != 0 {
                        if buf.remaining() < 12 {
                            return Err(SpatialError::DecodeError(
                                "Insufficient data for SpatialDelta Rotation".into(),
                            ));
                        }
                        Some(RotationDelta {
                            d_pitch: buf.get_f32(),
                            d_yaw: buf.get_f32(),
                            d_roll: buf.get_f32(),
                        })
                    } else {
                        None
                    };

                    let velocity = if mask & 0x04 != 0 {
                        if buf.remaining() < 12 {
                            return Err(SpatialError::DecodeError(
                                "Insufficient data for SpatialDelta Velocity".into(),
                            ));
                        }
                        Some(Velocity {
                            vx: buf.get_f32(),
                            vy: buf.get_f32(),
                            vz: buf.get_f32(),
                        })
                    } else {
                        None
                    };

                    let acceleration = if mask & 0x08 != 0 {
                        if buf.remaining() < 12 {
                            return Err(SpatialError::DecodeError(
                                "Insufficient data for SpatialDelta Acceleration".into(),
                            ));
                        }
                        Some(Acceleration {
                            ax: buf.get_f32(),
                            ay: buf.get_f32(),
                            az: buf.get_f32(),
                        })
                    } else {
                        None
                    };

                    Ok(SpatialValue::S13(SpatialDelta::State {
                        position,
                        rotation,
                        velocity,
                        acceleration,
                    }))
                }
                _ => Err(SpatialError::DecodeError(format!(
                    "Unknown SpatialDelta type: {}",
                    delta_type
                ))),
            }
        }
        _ => Err(SpatialError::UnknownType(type_id)),
    }
}
