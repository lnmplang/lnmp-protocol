use crate::types::*;

pub fn spatial_distance(p1: &Position3D, p2: &Position3D) -> f32 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let dz = p2.z - p1.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn normalize_vector(v: &Position3D) -> Position3D {
    let mag = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if mag == 0.0 {
        Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    } else {
        Position3D {
            x: v.x / mag,
            y: v.y / mag,
            z: v.z / mag,
        }
    }
}

pub fn spatial_intersect(box1: &BoundingBox, box2: &BoundingBox) -> bool {
    (box1.min_x <= box2.max_x && box1.max_x >= box2.min_x)
        && (box1.min_y <= box2.max_y && box1.max_y >= box2.min_y)
        && (box1.min_z <= box2.max_z && box1.max_z >= box2.min_z)
}
