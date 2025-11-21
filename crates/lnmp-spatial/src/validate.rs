use crate::error::SpatialError;
use crate::types::*;

pub struct SpatialConstraints {
    pub max_velocity: f32,
    pub max_acceleration: f32,
    pub max_rotation_speed: f32, // TAU radians/second
    pub max_coordinate: f32,
}

impl Default for SpatialConstraints {
    fn default() -> Self {
        Self {
            max_velocity: 100.0,
            max_acceleration: 50.0,
            max_rotation_speed: std::f32::consts::TAU,
            max_coordinate: 1000.0,
        }
    }
}

pub fn validate_spatial_state(
    state: &SpatialState,
    constraints: &SpatialConstraints,
) -> Result<(), SpatialError> {
    if let Some(pos) = &state.position {
        if pos.x.abs() > constraints.max_coordinate
            || pos.y.abs() > constraints.max_coordinate
            || pos.z.abs() > constraints.max_coordinate
        {
            return Err(SpatialError::ValidationError(
                "Position exceeds max coordinate value".into(),
            ));
        }
    }

    if let Some(vel) = &state.velocity {
        let speed = (vel.vx * vel.vx + vel.vy * vel.vy + vel.vz * vel.vz).sqrt();
        if speed > constraints.max_velocity {
            return Err(SpatialError::ValidationError(format!(
                "Velocity {} exceeds max {}",
                speed, constraints.max_velocity
            )));
        }
    }

    if let Some(acc) = &state.acceleration {
        let mag = (acc.ax * acc.ax + acc.ay * acc.ay + acc.az * acc.az).sqrt();
        if mag > constraints.max_acceleration {
            return Err(SpatialError::ValidationError(format!(
                "Acceleration {} exceeds max {}",
                mag, constraints.max_acceleration
            )));
        }
    }

    Ok(())
}
