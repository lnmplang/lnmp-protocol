//! Integration tests for lnmp-net message functionality

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_net::{MessageKind, NetMessage, NetMessageBuilder};

fn sample_record() -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 42,
        value: LnmpValue::Int(12345),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record
}

#[test]
fn test_message_lifecycle() {
    // Create envelope with timestamp
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .source("test-sensor")
        .trace_id("trace-abc")
        .sequence(1)
        .build();

    // Create message
    let msg = NetMessage::new(envelope, MessageKind::Event);

    // Verify defaults
    assert_eq!(msg.kind, MessageKind::Event);
    assert_eq!(msg.priority, 100); // Event default
    assert_eq!(msg.ttl_ms, 5000); // Event default

    // Check not expired (age 1s < TTL 5s)
    assert!(!msg.is_expired(2000).unwrap());

    // Check expired (age 6s > TTL 5s)
    assert!(msg.is_expired(7000).unwrap());

    // Verify accessors
    assert_eq!(msg.source(), Some("test-sensor"));
    assert_eq!(msg.trace_id(), Some("trace-abc"));
    assert_eq!(msg.timestamp(), Some(1000));
    assert_eq!(msg.age_ms(3000), Some(2000));
}

#[test]
fn test_builder_pattern() {
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .source("builder-test")
        .build();

    let msg = NetMessageBuilder::new(envelope, MessageKind::Alert)
        .priority(255)
        .ttl_ms(1000)
        .class("safety")
        .build();

    assert_eq!(msg.kind, MessageKind::Alert);
    assert_eq!(msg.priority, 255);
    assert_eq!(msg.ttl_ms, 1000);
    assert_eq!(msg.class, Some("safety".to_string()));
    assert_eq!(msg.source(), Some("builder-test"));
}

#[test]
fn test_all_message_kinds() {
    for kind in MessageKind::all() {
        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessage::new(envelope, kind);

        // Verify each kind has appropriate defaults
        match kind {
            MessageKind::Alert => {
                assert_eq!(msg.priority, 255);
                assert_eq!(msg.ttl_ms, 1000);
            }
            MessageKind::Command => {
                assert_eq!(msg.priority, 150);
                assert_eq!(msg.ttl_ms, 2000);
            }
            MessageKind::Query => {
                assert_eq!(msg.priority, 120);
                assert_eq!(msg.ttl_ms, 5000);
            }
            MessageKind::State => {
                assert_eq!(msg.priority, 100);
                assert_eq!(msg.ttl_ms, 10000);
            }
            MessageKind::Event => {
                assert_eq!(msg.priority, 100);
                assert_eq!(msg.ttl_ms, 5000);
            }
        }
    }
}

#[test]
fn test_message_without_timestamp() {
    // Envelope without timestamp
    let envelope = EnvelopeBuilder::new(sample_record())
        .source("no-timestamp-node")
        .build();

    let msg = NetMessage::new(envelope, MessageKind::Event);

    // is_expired should return error
    assert!(msg.is_expired(5000).is_err());

    // age_ms should return None
    assert_eq!(msg.age_ms(5000), None);

    // timestamp should be None
    assert_eq!(msg.timestamp(), None);
}

#[test]
fn test_validation() {
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .source("valid-node")
        .build();

    let msg = NetMessage::new(envelope, MessageKind::State);

    // Valid message should pass validation
    assert!(msg.validate().is_ok());
}

#[test]
fn test_message_with_class() {
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();

    let msg = NetMessageBuilder::new(envelope, MessageKind::Event)
        .class("health")
        .build();

    assert_eq!(msg.class, Some("health".to_string()));

    // Create another with different class
    let envelope2 = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();

    let msg2 = NetMessageBuilder::new(envelope2, MessageKind::Alert)
        .class("traffic")
        .build();

    assert_eq!(msg2.class, Some("traffic".to_string()));
}
