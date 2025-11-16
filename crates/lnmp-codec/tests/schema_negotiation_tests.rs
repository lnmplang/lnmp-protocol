use lnmp_codec::binary::{
    Capabilities, ErrorCode, FeatureFlags, NegotiationError, NegotiationMessage,
    NegotiationResponse, NegotiationState, SchemaNegotiator, TypeTag,
};
use std::collections::HashMap;

#[test]
fn test_successful_negotiation_flow() {
    // Client side
    let mut client = SchemaNegotiator::v0_5();
    let mut client_mappings = HashMap::new();
    client_mappings.insert(1, "user_id".to_string());
    client_mappings.insert(2, "username".to_string());
    client = client.with_fid_mappings(client_mappings.clone());

    // Server side
    let mut server = SchemaNegotiator::v0_5();
    server = server.with_fid_mappings(client_mappings.clone());

    // Step 1: Client initiates
    let caps_msg = client.initiate().unwrap();
    assert_eq!(client.state(), &NegotiationState::CapabilitiesSent);

    // Step 2: Server receives capabilities and sends ack
    let server_response = server.handle_message(caps_msg).unwrap();
    assert_eq!(server.state(), &NegotiationState::CapabilitiesReceived);

    let ack_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    // Step 3: Client receives ack and sends schema selection
    let client_response = client.handle_message(ack_msg).unwrap();
    assert_eq!(client.state(), &NegotiationState::SchemaSelected);

    let select_msg = match client_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    // Step 4: Server receives schema selection and sends ready
    let server_response = server.handle_message(select_msg).unwrap();
    assert_eq!(server.state(), &NegotiationState::SchemaSelected);

    let ready_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    // Step 5: Client receives ready and completes
    let client_response = client.handle_message(ready_msg).unwrap();
    assert_eq!(client.state(), &NegotiationState::Ready);
    assert!(client.is_ready());

    match client_response {
        NegotiationResponse::Complete(session) => {
            assert_eq!(session.session_id, 1);
            assert!(session.agreed_features.supports_nested);
            assert!(session.agreed_features.supports_streaming);
            assert!(session.agreed_features.supports_delta);
            assert!(session.agreed_features.supports_llb);
        }
        _ => panic!("Expected Complete response"),
    }
}

#[test]
fn test_fid_conflict_detection() {
    let mut client_mappings = HashMap::new();
    client_mappings.insert(1, "user_id".to_string());
    client_mappings.insert(2, "username".to_string());

    let mut server_mappings = HashMap::new();
    server_mappings.insert(1, "userId".to_string()); // Conflict!
    server_mappings.insert(2, "username".to_string());

    let mut client = SchemaNegotiator::v0_5().with_fid_mappings(client_mappings);
    let mut server = SchemaNegotiator::v0_5().with_fid_mappings(server_mappings);

    // Start negotiation
    let caps_msg = client.initiate().unwrap();
    let server_response = server.handle_message(caps_msg).unwrap();

    let ack_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ack_msg).unwrap();

    let select_msg = match client_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    // Server should detect FID conflict
    let result = server.handle_message(select_msg);
    assert!(result.is_err());

    match result {
        Err(NegotiationError::FidConflict { fid, name1, name2 }) => {
            assert_eq!(fid, 1);
            assert_eq!(name1, "userId");
            assert_eq!(name2, "user_id");
        }
        _ => panic!("Expected FidConflict error"),
    }
}

#[test]
fn test_type_mismatch_detection() {
    let mut expected_types = HashMap::new();
    expected_types.insert(1, TypeTag::Int);
    expected_types.insert(2, TypeTag::String);
    expected_types.insert(3, TypeTag::Bool);

    let mut actual_types = HashMap::new();
    actual_types.insert(1, TypeTag::Float); // Mismatch
    actual_types.insert(2, TypeTag::String);
    actual_types.insert(3, TypeTag::Int); // Mismatch

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected_types, &actual_types);
    assert_eq!(mismatches.len(), 2);

    // Check first mismatch
    match &mismatches[0] {
        NegotiationError::TypeMismatch {
            fid,
            expected,
            found,
        } => {
            assert_eq!(*fid, 1);
            assert_eq!(*expected, TypeTag::Int);
            assert_eq!(*found, TypeTag::Float);
        }
        _ => panic!("Expected TypeMismatch"),
    }

    // Check second mismatch
    match &mismatches[1] {
        NegotiationError::TypeMismatch {
            fid,
            expected,
            found,
        } => {
            assert_eq!(*fid, 3);
            assert_eq!(*expected, TypeTag::Bool);
            assert_eq!(*found, TypeTag::Int);
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_feature_flag_negotiation() {
    // Client with full v0.5 features
    let client_caps = Capabilities::v0_5();
    let mut client = SchemaNegotiator::new(client_caps);

    // Server with limited features
    let server_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: false, // Not supported
        supports_delta: true,
        supports_llb: false, // Not supported
        requires_checksums: true,
        requires_canonical: true,
    };
    let server_caps = Capabilities::new(0x05, server_features, vec![TypeTag::Int, TypeTag::String]);
    let mut server = SchemaNegotiator::new(server_caps);

    // Complete negotiation
    let caps_msg = client.initiate().unwrap();
    let server_response = server.handle_message(caps_msg).unwrap();

    let ack_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ack_msg).unwrap();
    assert_eq!(client.state(), &NegotiationState::SchemaSelected);

    let select_msg = match client_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let server_response = server.handle_message(select_msg).unwrap();

    let ready_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ready_msg).unwrap();

    // Check agreed features (intersection)
    match client_response {
        NegotiationResponse::Complete(session) => {
            assert!(session.agreed_features.supports_nested); // Both support
            assert!(!session.agreed_features.supports_streaming); // Server doesn't support
            assert!(session.agreed_features.supports_delta); // Both support
            assert!(!session.agreed_features.supports_llb); // Server doesn't support
            assert!(session.agreed_features.requires_checksums); // OR logic
            assert!(session.agreed_features.requires_canonical); // OR logic
        }
        _ => panic!("Expected Complete response"),
    }
}

#[test]
fn test_protocol_version_mismatch_handling() {
    let mut client = SchemaNegotiator::v0_5();
    let mut server = SchemaNegotiator::v0_4();

    // Client sends v0.5 capabilities
    let caps_msg = client.initiate().unwrap();

    // Server should reject due to version mismatch
    let result = server.handle_message(caps_msg);
    assert!(result.is_err());

    match result {
        Err(NegotiationError::ProtocolVersionMismatch { local, remote }) => {
            assert_eq!(local, 0x04);
            assert_eq!(remote, 0x05);
        }
        _ => panic!("Expected ProtocolVersionMismatch error"),
    }
}

#[test]
fn test_invalid_state_transitions() {
    let mut negotiator = SchemaNegotiator::v0_5();

    // Try to handle SelectSchema before capabilities exchange
    let mut mappings = HashMap::new();
    mappings.insert(1, "test".to_string());

    let select_msg = NegotiationMessage::SelectSchema {
        schema_id: "test".to_string(),
        fid_mappings: mappings,
    };

    let result = negotiator.handle_message(select_msg);
    assert!(result.is_err());

    match result {
        Err(NegotiationError::InvalidState { current, expected }) => {
            assert_eq!(current, NegotiationState::Initial);
            assert_eq!(expected, NegotiationState::CapabilitiesReceived);
        }
        _ => panic!("Expected InvalidState error"),
    }
}

#[test]
fn test_error_message_handling() {
    let mut negotiator = SchemaNegotiator::v0_5();

    let error_msg = NegotiationMessage::Error {
        code: ErrorCode::Generic,
        message: "Test error message".to_string(),
    };

    let result = negotiator.handle_message(error_msg).unwrap();

    match negotiator.state() {
        NegotiationState::Failed(msg) => {
            assert_eq!(msg, "Test error message");
        }
        _ => panic!("Expected Failed state"),
    }

    match result {
        NegotiationResponse::Failed(msg) => {
            assert_eq!(msg, "Test error message");
        }
        _ => panic!("Expected Failed response"),
    }
}

#[test]
fn test_v0_4_compatibility_negotiation() {
    let mut client = SchemaNegotiator::v0_4();
    let mut server = SchemaNegotiator::v0_4();

    // Complete negotiation with v0.4
    let caps_msg = client.initiate().unwrap();
    let server_response = server.handle_message(caps_msg).unwrap();

    let ack_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ack_msg).unwrap();
    assert_eq!(client.state(), &NegotiationState::SchemaSelected);

    let select_msg = match client_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let server_response = server.handle_message(select_msg).unwrap();

    let ready_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ready_msg).unwrap();

    // Verify v0.4 features
    match client_response {
        NegotiationResponse::Complete(session) => {
            assert!(!session.agreed_features.supports_nested);
            assert!(!session.agreed_features.supports_streaming);
            assert!(!session.agreed_features.supports_delta);
            assert!(!session.agreed_features.supports_llb);
            assert!(session.agreed_features.requires_canonical);
        }
        _ => panic!("Expected Complete response"),
    }
}

#[test]
fn test_multiple_fid_conflicts() {
    let mut local = HashMap::new();
    local.insert(1, "user_id".to_string());
    local.insert(2, "username".to_string());
    local.insert(3, "email".to_string());

    let mut remote = HashMap::new();
    remote.insert(1, "userId".to_string()); // Conflict
    remote.insert(2, "userName".to_string()); // Conflict
    remote.insert(3, "email".to_string());

    let conflicts = SchemaNegotiator::detect_conflicts(&local, &remote);
    assert_eq!(conflicts.len(), 2);
}

#[test]
fn test_empty_fid_mappings_negotiation() {
    let mut client = SchemaNegotiator::v0_5();
    let mut server = SchemaNegotiator::v0_5();

    // Complete negotiation with empty mappings
    let caps_msg = client.initiate().unwrap();
    let server_response = server.handle_message(caps_msg).unwrap();

    let ack_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ack_msg).unwrap();
    assert_eq!(client.state(), &NegotiationState::SchemaSelected);

    let select_msg = match client_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let server_response = server.handle_message(select_msg).unwrap();

    let ready_msg = match server_response {
        NegotiationResponse::SendMessage(msg) => msg,
        _ => panic!("Expected SendMessage"),
    };

    let client_response = client.handle_message(ready_msg).unwrap();

    match client_response {
        NegotiationResponse::Complete(session) => {
            assert!(session.fid_mappings.is_empty());
        }
        _ => panic!("Expected Complete response"),
    }
}
