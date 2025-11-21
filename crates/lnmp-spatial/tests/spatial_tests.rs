use lnmp_spatial::*;

#[test]
fn test_encode_decode_position3d() {
    let pos = Position3D {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let val = SpatialValue::S2(pos);
    let mut buf = Vec::new();

    encode_spatial(&val, &mut buf).unwrap();

    let mut slice = buf.as_slice();
    let decoded = decode_spatial(&mut slice).unwrap();

    if let SpatialValue::S2(d_pos) = decoded {
        assert_eq!(d_pos.x, 1.0);
        assert_eq!(d_pos.y, 2.0);
        assert_eq!(d_pos.z, 3.0);
    } else {
        panic!("Wrong type decoded");
    }
}

#[test]
fn test_spatial_distance() {
    let p1 = Position3D {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let p2 = Position3D {
        x: 3.0,
        y: 4.0,
        z: 0.0,
    };
    assert_eq!(spatial_distance(&p1, &p2), 5.0);
}

#[test]
fn test_transform() {
    let p = Position3D {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    let t = Transform {
        position: Position3D {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Rotation {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        },
        scale: Position3D {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };

    let transformed = spatial_transform(&p, &t);
    assert_eq!(transformed.x, 2.0); // 1.0 (original) + 1.0 (translate)
    assert_eq!(transformed.y, 0.0);
    assert_eq!(transformed.z, 0.0);
}

#[test]
fn test_validation() {
    let constraints = SpatialConstraints::default();
    let valid_state = SpatialState {
        position: Some(Position3D {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        }),
        rotation: None,
        velocity: Some(Velocity {
            vx: 10.0,
            vy: 0.0,
            vz: 0.0,
        }),
        acceleration: None,
    };
    assert!(validate_spatial_state(&valid_state, &constraints).is_ok());

    let state = SpatialState {
        position: None,
        rotation: None,
        velocity: Some(Velocity {
            vx: 200.0,
            vy: 0.0,
            vz: 0.0,
        }), // Exceeds 100.0
        acceleration: None,
    };
    assert!(validate_spatial_state(&state, &constraints).is_err());
}

#[test]
fn test_delta() {
    use lnmp_spatial::delta::Delta;

    let start = Position3D {
        x: 10.0,
        y: 20.0,
        z: 30.0,
    };
    let end = Position3D {
        x: 11.0,
        y: 19.0,
        z: 32.0,
    };

    let delta = Position3D::compute_delta(&start, &end);
    assert!((delta.dx - 1.0).abs() < 1e-6);
    assert!((delta.dy - (-1.0)).abs() < 1e-6);
    assert!((delta.dz - 2.0).abs() < 1e-6);

    let applied = Position3D::apply_delta(&start, &delta);
    assert!((applied.x - end.x).abs() < 1e-6);
    assert!((applied.y - end.y).abs() < 1e-6);
    assert!((applied.z - end.z).abs() < 1e-6);

    // Test Encoding/Decoding of Delta
    let delta_val = SpatialValue::S11(delta);
    let mut buf = Vec::new();
    encode_spatial(&delta_val, &mut buf).unwrap();

    let mut slice = buf.as_slice();
    let decoded = decode_spatial(&mut slice).unwrap();

    if let SpatialValue::S11(d) = decoded {
        assert!((d.dx - delta.dx).abs() < 1e-6);
        assert!((d.dy - delta.dy).abs() < 1e-6);
        assert!((d.dz - delta.dz).abs() < 1e-6);
    } else {
        panic!("Decoded wrong type");
    }
}
