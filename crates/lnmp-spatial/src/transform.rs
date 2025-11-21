use crate::types::*;

pub fn spatial_transform(point: &Position3D, transform: &Transform) -> Position3D {
    // 1. Scale
    let sx = point.x * transform.scale.x;
    let sy = point.y * transform.scale.y;
    let sz = point.z * transform.scale.z;

    // 2. Rotate (Euler angles - simplified, assumes specific order e.g., XYZ)
    // For full robustness, quaternions should be used, but using Euler for now as per struct.
    // Rotation logic can be complex; implementing a basic rotation matrix application here.

    let (sin_x, cos_x) = transform.rotation.pitch.sin_cos();
    let (sin_y, cos_y) = transform.rotation.yaw.sin_cos();
    let (sin_z, cos_z) = transform.rotation.roll.sin_cos();

    // Rotate Z
    let x1 = sx * cos_z - sy * sin_z;
    let y1 = sx * sin_z + sy * cos_z;
    let z1 = sz;

    // Rotate Y
    let x2 = x1 * cos_y + z1 * sin_y;
    let y2 = y1;
    let z2 = -x1 * sin_y + z1 * cos_y;

    // Rotate X
    let x3 = x2;
    let y3 = y2 * cos_x - z2 * sin_x;
    let z3 = y2 * sin_x + z2 * cos_x;

    // 3. Translate
    Position3D {
        x: x3 + transform.position.x,
        y: y3 + transform.position.y,
        z: z3 + transform.position.z,
    }
}
