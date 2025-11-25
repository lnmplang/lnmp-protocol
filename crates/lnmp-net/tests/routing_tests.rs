//! Integration tests for lnmp-net routing functionality

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_net::{MessageKind, NetMessage, RoutingDecision, RoutingPolicy};

fn sample_record() -> LnmpRecord {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 42,
        value: LnmpValue::Int(12345),
    });
    record
}

#[test]
fn test_eco_profile_alert_routing() {
    let policy = RoutingPolicy::default();

    // High-priority alert should always route to LLM
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let alert = NetMessage::new(envelope, MessageKind::Alert);

    assert_eq!(
        policy.decide(&alert, 2000).unwrap(),
        RoutingDecision::SendToLLM
    );
}

#[test]
fn test_eco_profile_expired_drop() {
    let policy = RoutingPolicy::default();

    // Create event with short TTL that has expired
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let event = NetMessage::with_qos(envelope, MessageKind::Event, 150, 2000);

    // Age = 6000ms > TTL 2000ms
    assert_eq!(policy.decide(&event, 7000).unwrap(), RoutingDecision::Drop);
}

#[test]
fn test_eco_profile_high_importance_event() {
    let policy = RoutingPolicy::default(); // threshold = 0.7

    // Very high priority event should route to LLM
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let event = NetMessage::with_qos(envelope, MessageKind::Event, 240, 10000);

    assert_eq!(
        policy.decide(&event, 2000).unwrap(),
        RoutingDecision::SendToLLM
    );
}

#[test]
fn test_eco_profile_low_importance_event() {
    let policy = RoutingPolicy::default();

    // Low priority event should process locally
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let event = NetMessage::with_qos(envelope, MessageKind::Event, 40, 10000);

    assert_eq!(
        policy.decide(&event, 2000).unwrap(),
        RoutingDecision::ProcessLocally
    );
}

#[test]
fn test_command_processing() {
    let policy = RoutingPolicy::default();

    // Commands should process locally by default
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let command = NetMessage::new(envelope, MessageKind::Command);

    assert_eq!(
        policy.decide(&command, 2000).unwrap(),
        RoutingDecision::ProcessLocally
    );
}

#[test]
fn test_query_processing() {
    let policy = RoutingPolicy::default();

    // Queries should process locally by default
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let query = NetMessage::new(envelope, MessageKind::Query);

    assert_eq!(
        policy.decide(&query, 2000).unwrap(),
        RoutingDecision::ProcessLocally
    );
}

#[test]
fn test_state_routing() {
    let policy = RoutingPolicy::default();

    // High-priority state should route to LLM
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let state = NetMessage::with_qos(envelope, MessageKind::State, 230, 10000);

    assert_eq!(
        policy.decide(&state, 2000).unwrap(),
        RoutingDecision::SendToLLM
    );

    // Low-priority state should process locally
    let envelope2 = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let state2 = NetMessage::with_qos(envelope2, MessageKind::State, 50, 10000);

    assert_eq!(
        policy.decide(&state2, 2000).unwrap(),
        RoutingDecision::ProcessLocally
    );
}

#[test]
fn test_custom_threshold() {
    // Very restrictive policy (0.95 threshold)
    let strict_policy = RoutingPolicy::new(0.95);

    // Even high priority event should process locally
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let event = NetMessage::with_qos(envelope, MessageKind::Event, 200, 10000);

    assert_eq!(
        strict_policy.decide(&event, 2000).unwrap(),
        RoutingDecision::ProcessLocally
    );

    // Very permissive policy (0.3 threshold)
    let permissive_policy = RoutingPolicy::new(0.3);

    let envelope2 = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let event2 = NetMessage::with_qos(envelope2, MessageKind::Event, 100, 10000);

    assert_eq!(
        permissive_policy.decide(&event2, 2000).unwrap(),
        RoutingDecision::SendToLLM
    );
}

#[test]
fn test_policy_configuration() {
    // Disable alert auto-routing
    let policy = RoutingPolicy::default().with_always_route_alerts(false);

    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    // Low-priority alert
    let alert = NetMessage::with_qos(envelope, MessageKind::Alert, 100, 1000);

    // Should not auto-route
    let decision = policy.decide(&alert, 2000).unwrap();
    // With disabled alert routing, low priority alert won't auto-send
    assert_ne!(decision, RoutingDecision::SendToLLM);
}

#[test]
fn test_base_importance_calculation() {
    let policy = RoutingPolicy::default();

    // High priority message
    let envelope = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let high_msg = NetMessage::with_qos(envelope, MessageKind::Event, 255, 10000);

    let high_importance = policy.base_importance(&high_msg, 2000).unwrap();
    assert!(
        high_importance > 0.5,
        "High priority should yield high importance: {}",
        high_importance
    );

    // Low priority message
    let envelope2 = EnvelopeBuilder::new(sample_record())
        .timestamp(1000)
        .build();
    let low_msg = NetMessage::with_qos(envelope2, MessageKind::Event, 20, 10000);

    let low_importance = policy.base_importance(&low_msg, 2000).unwrap();
    assert!(
        low_importance < high_importance,
        "Low priority should yield lower importance"
    );
}

#[test]
fn test_mixed_scenario() {
    let policy = RoutingPolicy::default();
    let now_ms = 10000;

    // Alert: should route to LLM
    let alert_env = EnvelopeBuilder::new(sample_record())
        .timestamp(now_ms - 500)
        .build();
    let alert = NetMessage::new(alert_env, MessageKind::Alert);
    assert_eq!(
        policy.decide(&alert, now_ms).unwrap(),
        RoutingDecision::SendToLLM
    );

    // Expired event: should drop
    let expired_env = EnvelopeBuilder::new(sample_record())
        .timestamp(now_ms - 10000)
        .build();
    let expired = NetMessage::with_qos(expired_env, MessageKind::Event, 150, 2000);
    assert_eq!(
        policy.decide(&expired, now_ms).unwrap(),
        RoutingDecision::Drop
    );

    // Fresh high-priority event: should route to LLM
    let fresh_env = EnvelopeBuilder::new(sample_record())
        .timestamp(now_ms - 1000)
        .build();
    let fresh = NetMessage::with_qos(fresh_env, MessageKind::Event, 240, 10000);
    assert_eq!(
        policy.decide(&fresh, now_ms).unwrap(),
        RoutingDecision::SendToLLM
    );

    // Low-priority event: should process locally
    let low_env = EnvelopeBuilder::new(sample_record())
        .timestamp(now_ms - 1000)
        .build();
    let low = NetMessage::with_qos(low_env, MessageKind::Event, 30, 10000);
    assert_eq!(
        policy.decide(&low, now_ms).unwrap(),
        RoutingDecision::ProcessLocally
    );

    // Command: should process locally
    let cmd_env = EnvelopeBuilder::new(sample_record())
        .timestamp(now_ms - 500)
        .build();
    let cmd = NetMessage::new(cmd_env, MessageKind::Command);
    assert_eq!(
        policy.decide(&cmd, now_ms).unwrap(),
        RoutingDecision::ProcessLocally
    );
}
