use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpatialType {
    Position2D = 0x01,
    Position3D = 0x02,
    Rotation = 0x03,
    Velocity = 0x04,
    Acceleration = 0x05,
    BoundingBox = 0x06,
    Quaternion = 0x07,
    Path = 0x08,
    Transform = 0x09,
    SpatialState = 0x0A,
    PositionDelta = 0x0B,
    RotationDelta = 0x0C,
    SpatialDelta = 0x0D,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Acceleration {
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub qx: f32,
    pub qy: f32,
    pub qz: f32,
    pub qw: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<Position3D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Position3D,
    pub rotation: Rotation,
    pub scale: Position3D, // Using Position3D for scale x, y, z
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialState {
    pub position: Option<Position3D>,
    pub rotation: Option<Rotation>,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionDelta {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotationDelta {
    pub d_pitch: f32,
    pub d_yaw: f32,
    pub d_roll: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpatialDelta {
    Position(PositionDelta),
    Rotation(RotationDelta),
    State {
        position: Option<PositionDelta>,
        rotation: Option<RotationDelta>,
        velocity: Option<Velocity>, // Velocity is absolute, or should it be delta? Usually absolute update.
        acceleration: Option<Acceleration>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictiveDelta {
    pub delta: PositionDelta,
    pub velocity: Velocity,
    pub predicted_next: Position3D,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpatialValue {
    S1(Position2D),
    S2(Position3D),
    S3(Rotation),
    S4(Velocity),
    S5(Acceleration),
    S6(BoundingBox),
    S7(Quaternion),
    S8(Path),
    S9(Transform),
    S10(SpatialState),
    S11(PositionDelta),
    S12(RotationDelta),
    S13(SpatialDelta),
}
