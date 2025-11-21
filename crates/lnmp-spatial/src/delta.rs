use crate::types::*;

/// Trait for types that support delta encoding.
pub trait Delta: Sized {
    type DeltaType;

    /// Computes the delta required to go from `start` to `end`.
    /// `start + delta = end`
    fn compute_delta(start: &Self, end: &Self) -> Self::DeltaType;

    /// Applies a delta to `start` to get `end`.
    /// `start + delta = end`
    fn apply_delta(start: &Self, delta: &Self::DeltaType) -> Self;
}

impl Delta for Position3D {
    type DeltaType = PositionDelta;

    fn compute_delta(start: &Self, end: &Self) -> Self::DeltaType {
        PositionDelta {
            dx: end.x - start.x,
            dy: end.y - start.y,
            dz: end.z - start.z,
        }
    }

    fn apply_delta(start: &Self, delta: &Self::DeltaType) -> Self {
        Self {
            x: start.x + delta.dx,
            y: start.y + delta.dy,
            z: start.z + delta.dz,
        }
    }
}

impl Delta for Rotation {
    type DeltaType = RotationDelta;

    fn compute_delta(start: &Self, end: &Self) -> Self::DeltaType {
        RotationDelta {
            d_pitch: end.pitch - start.pitch,
            d_yaw: end.yaw - start.yaw,
            d_roll: end.roll - start.roll,
        }
    }

    fn apply_delta(start: &Self, delta: &Self::DeltaType) -> Self {
        Self {
            pitch: start.pitch + delta.d_pitch,
            yaw: start.yaw + delta.d_yaw,
            roll: start.roll + delta.d_roll,
        }
    }
}

impl Delta for SpatialState {
    type DeltaType = SpatialDelta;

    fn compute_delta(start: &Self, end: &Self) -> Self::DeltaType {
        let pos_delta = match (&start.position, &end.position) {
            (Some(s), Some(e)) => Some(Position3D::compute_delta(s, e)),
            _ => None,
        };

        let rot_delta = match (&start.rotation, &end.rotation) {
            (Some(s), Some(e)) => Some(Rotation::compute_delta(s, e)),
            _ => None,
        };

        // Velocity and Acceleration are usually treated as absolute updates in telemetry,
        // but we could delta them too. For now, we'll just carry over the new value if it changed.
        // However, SpatialDelta::State definition uses Option<Velocity>.
        // Let's assume if it's present in `end`, we send it.

        SpatialDelta::State {
            position: pos_delta,
            rotation: rot_delta,
            velocity: end.velocity,
            acceleration: end.acceleration,
        }
    }

    fn apply_delta(start: &Self, delta: &Self::DeltaType) -> Self {
        match delta {
            SpatialDelta::State {
                position,
                rotation,
                velocity,
                acceleration,
            } => {
                let new_pos = match (start.position, position) {
                    (Some(p), Some(d)) => Some(Position3D::apply_delta(&p, d)),
                    (p, _) => p,
                };

                let new_rot = match (start.rotation, rotation) {
                    (Some(r), Some(d)) => Some(Rotation::apply_delta(&r, d)),
                    (r, _) => r,
                };

                Self {
                    position: new_pos,
                    rotation: new_rot,
                    velocity: velocity.or(start.velocity),
                    acceleration: acceleration.or(start.acceleration),
                }
            }
            _ => start.clone(), // Should not happen if types match
        }
    }
}
