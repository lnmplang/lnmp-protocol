//! Simple test to verify all modules are accessible through the meta crate

use lnmp::prelude::*;

#[test]
fn test_core_module() {
    // Test core types
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(42),
    });

    assert_eq!(record.fields().len(), 1);
}

#[test]
fn test_codec_module() {
    // Test codec functionality
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let encoder = Encoder::new();
    let encoded = encoder.encode(&record);

    let mut parser = Parser::new(&encoded).unwrap();
    let decoded = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), decoded.fields().len());
}

#[test]
fn test_embedding_module() {
    // Test embedding types
    let vec1 = Vector::from_f32(vec![1.0, 2.0, 3.0]);
    let vec2 = Vector::from_f32(vec![1.1, 2.0, 3.2]);

    let delta = VectorDelta::from_vectors(&vec1, &vec2, 1).unwrap();

    assert!(!delta.changes.is_empty());
}

#[test]
fn test_spatial_module() {
    use lnmp::spatial::types::{Position3D, SpatialState};

    // Test spatial streaming
    let mut streamer = SpatialStreamer::new(30);

    let state = SpatialState {
        position: Some(Position3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        rotation: None,
        velocity: None,
        acceleration: None,
    };

    let frame = streamer.next_frame(&state, 0).unwrap();
    assert!(frame.header.sequence_id == 0);
}

#[test]
fn test_module_access() {
    // Verify all modules are accessible
    use lnmp::{codec, core};

    // This test just ensures all modules compile and are accessible
    let _ = core::LNMP_MAGIC;
    let _ = codec::ParsingMode::Loose;
}
