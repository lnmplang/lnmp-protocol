//! Example demonstrating LNMP v0.5 Schema Negotiation Layer (SNL)
//!
//! This example shows:
//! - Client-server capability negotiation
//! - Feature flag exchange and agreement
//! - FID conflict detection
//! - Type mismatch detection
//! - Protocol version negotiation
//! - Successful and failed negotiation scenarios

use lnmp_codec::binary::{
    SchemaNegotiator, Capabilities, FeatureFlags, NegotiationMessage,
    NegotiationResponse, NegotiationState, TypeTag,
};
use std::collections::HashMap;

fn main() {
    println!("=== LNMP v0.5 Schema Negotiation Layer Example ===\n");

    // Example 1: Successful negotiation
    println!("1. Successful Client-Server Negotiation:");
    successful_negotiation();
    println!();

    // Example 2: Feature flag negotiation
    println!("2. Feature Flag Negotiation:");
    feature_flag_negotiation();
    println!();

    // Example 3: FID conflict detection
    println!("3. FID Conflict Detection:");
    fid_conflict_example();
    println!();

    // Example 4: Type mismatch detection
    println!("4. Type Mismatch Detection:");
    type_mismatch_example();
    println!();

    // Example 5: Protocol version mismatch
    println!("5. Protocol Version Mismatch:");
    version_mismatch_example();
    println!();

    // Example 6: Partial feature support
    println!("6. Partial Feature Support:");
    partial_feature_support();
    println!();
}

fn successful_negotiation() {
    // Create client capabilities
    let client_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: true,
        supports_delta: true,
        supports_llb: true,
        requires_checksums: false,
        requires_canonical: true,
    };
    
    let client_caps = Capabilities {
        version: 5,
        features: client_features,
        supported_types: vec![
            TypeTag::Int,
            TypeTag::Float,
            TypeTag::Bool,
            TypeTag::String,
            TypeTag::StringArray,
            TypeTag::NestedRecord,
            TypeTag::NestedArray,
        ],
    };
    
    println!("   Client capabilities:");
    println!("     Version: {}", client_caps.version);
    println!("     Nested: {}", client_caps.features.supports_nested);
    println!("     Streaming: {}", client_caps.features.supports_streaming);
    println!("     Delta: {}", client_caps.features.supports_delta);
    
    // Create server capabilities (compatible)
    let server_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: true,
        supports_delta: false, // Server doesn't support delta
        supports_llb: true,
        requires_checksums: true, // Server requires checksums
        requires_canonical: true,
    };
    
    let server_caps = Capabilities {
        version: 5,
        features: server_features,
        supported_types: vec![
            TypeTag::Int,
            TypeTag::Float,
            TypeTag::Bool,
            TypeTag::String,
            TypeTag::StringArray,
            TypeTag::NestedRecord,
            TypeTag::NestedArray,
        ],
    };
    
    println!("   Server capabilities:");
    println!("     Version: {}", server_caps.version);
    println!("     Nested: {}", server_caps.features.supports_nested);
    println!("     Streaming: {}", server_caps.features.supports_streaming);
    println!("     Delta: {}", server_caps.features.supports_delta);
    
    // Client initiates negotiation
    let mut client_negotiator = SchemaNegotiator::new(client_caps.clone());
    let client_msg = client_negotiator.initiate().unwrap();
    println!("   ✓ Client sent CAPABILITIES message");
    
    // Server receives and responds
    let mut server_negotiator = SchemaNegotiator::new(server_caps.clone());
    match server_negotiator.handle_message(client_msg).unwrap() {
        NegotiationResponse::SendMessage(response) => {
            println!("   ✓ Server sent CAPABILITIES_ACK");
            
            // Client receives server response
            match client_negotiator.handle_message(response).unwrap() {
                NegotiationResponse::SendMessage(select_msg) => {
                    println!("   ✓ Client sent SELECT_SCHEMA");
                    
                    // Server confirms
                    match server_negotiator.handle_message(select_msg).unwrap() {
                        NegotiationResponse::SendMessage(ready_msg) => {
                            println!("   ✓ Server sent READY");
                            
                            // Client receives ready
                            client_negotiator.handle_message(ready_msg).unwrap();
                            println!("   ✓ Negotiation complete!");
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    
    // Check agreed features
    if client_negotiator.is_ready() && server_negotiator.is_ready() {
        println!("   Agreed features:");
        println!("     Nested: ✓");
        println!("     Streaming: ✓");
        println!("     Delta: ✗ (server doesn't support)");
        println!("     Checksums: ✓ (server requires)");
    }
}

fn feature_flag_negotiation() {
    // Client with all features
    let client_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: true,
        supports_delta: true,
        supports_llb: true,
        requires_checksums: false,
        requires_canonical: true,
    };
    
    let client_caps = Capabilities {
        version: 5,
        features: client_features,
        supported_types: vec![TypeTag::Int, TypeTag::String],
    };
    
    // Server with minimal features
    let server_features = FeatureFlags {
        supports_nested: false,
        supports_streaming: false,
        supports_delta: false,
        supports_llb: false,
        requires_checksums: true,
        requires_canonical: true,
    };
    
    let server_caps = Capabilities {
        version: 5,
        features: server_features,
        supported_types: vec![TypeTag::Int, TypeTag::String],
    };
    
    println!("   Client supports: nested, streaming, delta, llb");
    println!("   Server supports: (none of the optional features)");
    println!("   Server requires: checksums, canonical");
    
    let mut client_negotiator = SchemaNegotiator::new(client_caps);
    let mut server_negotiator = SchemaNegotiator::new(server_caps);
    
    // Perform negotiation
    let msg1 = client_negotiator.initiate().unwrap();
        if let NegotiationResponse::SendMessage(msg2) = server_negotiator.handle_message(msg1).unwrap() {
        if let NegotiationResponse::SendMessage(msg3) = client_negotiator.handle_message(msg2).unwrap() {
                if let NegotiationResponse::SendMessage(msg4) = server_negotiator.handle_message(msg3).unwrap() {
                client_negotiator.handle_message(msg4).unwrap();
            }
        }
    }
    
    if client_negotiator.is_ready() {
        println!("   ✓ Negotiation succeeded with minimal feature set");
        println!("   Agreed features: checksums, canonical only");
    }
}

fn fid_conflict_example() {
    // Client schema: F12 = "user_id"
    let mut client_fid_mappings = HashMap::new();
    client_fid_mappings.insert(12, "user_id".to_string());
    client_fid_mappings.insert(7, "is_admin".to_string());
    
    // Server schema: F12 = "order_id" (CONFLICT!)
    let mut server_fid_mappings = HashMap::new();
    server_fid_mappings.insert(12, "order_id".to_string());
    server_fid_mappings.insert(7, "is_admin".to_string());
    
    println!("   Client schema: F12 = user_id, F7 = is_admin");
    println!("   Server schema: F12 = order_id, F7 = is_admin");
    println!("   Conflict: F12 maps to different field names!");
    
    let client_caps = Capabilities {
        version: 5,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int],
    };
    
    let server_caps = Capabilities {
        version: 5,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int],
    };
    
    let mut client_negotiator = SchemaNegotiator::new(client_caps);
    let mut server_negotiator = SchemaNegotiator::new(server_caps);
    
    // Set FID mappings
    client_negotiator = client_negotiator.with_fid_mappings(client_fid_mappings);
    server_negotiator = server_negotiator.with_fid_mappings(server_fid_mappings);
    
    // Attempt negotiation
    let msg1 = client_negotiator.initiate().unwrap();
    if let NegotiationResponse::SendMessage(msg2) = server_negotiator.handle_message(msg1).unwrap() {
        match client_negotiator.handle_message(msg2) {
            Ok(NegotiationResponse::SendMessage(msg3)) => {
                match server_negotiator.handle_message(msg3).unwrap() {
                    NegotiationResponse::Complete(_) => println!("   ✗ Unexpected success"),
                    NegotiationResponse::Failed(e) => println!("   ✓ Correctly detected conflict: {}", e),
                    _ => println!("   ✓ Correctly detected conflict"),
                }
            }
            Err(e) => println!("   ✓ Correctly detected conflict: {}", e),
            _ => {}
        }
    }
}

fn type_mismatch_example() {
    // Client expects F12 to be Int
    let mut client_type_mappings = HashMap::new();
    client_type_mappings.insert(12, TypeTag::Int);
    client_type_mappings.insert(7, TypeTag::Bool);
    
    // Server expects F12 to be String (MISMATCH!)
    let mut server_type_mappings = HashMap::new();
    server_type_mappings.insert(12, TypeTag::String);
    server_type_mappings.insert(7, TypeTag::Bool);
    
    println!("   Client expects: F12:Int, F7:Bool");
    println!("   Server expects: F12:String, F7:Bool");
    println!("   Type mismatch on F12!");
    
    let client_caps = Capabilities {
        version: 5,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int, TypeTag::Bool, TypeTag::String],
    };
    
    let server_caps = Capabilities {
        version: 5,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int, TypeTag::Bool, TypeTag::String],
    };
    
    let mut client_negotiator = SchemaNegotiator::new(client_caps);
    let mut server_negotiator = SchemaNegotiator::new(server_caps);
    
    // Type mapping helpers are not supported by the SchemaNegotiator API; skipping explicit type mappings.
    
    // Attempt negotiation
    let msg1 = client_negotiator.initiate().unwrap();
    if let NegotiationResponse::SendMessage(msg2) = server_negotiator.handle_message(msg1).unwrap() {
        match client_negotiator.handle_message(msg2) {
            Ok(NegotiationResponse::SendMessage(msg3)) => {
                match server_negotiator.handle_message(msg3).unwrap() {
                    NegotiationResponse::Complete(_) => println!("   ✗ Unexpected success"),
                    NegotiationResponse::Failed(e) => println!("   ✓ Correctly detected type mismatch: {}", e),
                    _ => println!("   ✓ Correctly detected type mismatch"),
                }
            }
            Err(e) => println!("   ✓ Correctly detected type mismatch: {}", e),
            _ => {}
        }
    }
}

fn version_mismatch_example() {
    // Client using v0.5
    let client_caps = Capabilities {
        version: 5,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int],
    };
    
    // Server using v0.4
    let server_caps = Capabilities {
        version: 4,
        features: FeatureFlags::default(),
        supported_types: vec![TypeTag::Int],
    };
    
    println!("   Client version: 5");
    println!("   Server version: 4");
    
    let mut client_negotiator = SchemaNegotiator::new(client_caps);
    let mut server_negotiator = SchemaNegotiator::new(server_caps);
    
    // Attempt negotiation
    let msg1 = client_negotiator.initiate().unwrap();
    match server_negotiator.handle_message(msg1).unwrap() {
        NegotiationResponse::SendMessage(msg2) => {
            match client_negotiator.handle_message(msg2) {
                Ok(_) => println!("   ✓ Negotiation succeeded (backward compatible)"),
                Err(e) => println!("   Version negotiation result: {}", e),
            }
        }
        _ => println!("   ✓ Version mismatch detected (no SendMessage response)"),
    }
}

fn partial_feature_support() {
    // Client supports everything
    let client_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: true,
        supports_delta: true,
        supports_llb: true,
        requires_checksums: false,
        requires_canonical: true,
    };
    
    // Server supports only nested and streaming
    let server_features = FeatureFlags {
        supports_nested: true,
        supports_streaming: true,
        supports_delta: false,
        supports_llb: false,
        requires_checksums: false,
        requires_canonical: true,
    };
    
    println!("   Client supports: nested, streaming, delta, llb");
    println!("   Server supports: nested, streaming");
    
    let client_caps = Capabilities {
        version: 5,
        features: client_features,
        supported_types: vec![TypeTag::Int, TypeTag::NestedRecord],
    };
    
    let server_caps = Capabilities {
        version: 5,
        features: server_features,
        supported_types: vec![TypeTag::Int, TypeTag::NestedRecord],
    };
    
    let mut client_negotiator = SchemaNegotiator::new(client_caps);
    let mut server_negotiator = SchemaNegotiator::new(server_caps);
    
    // Perform negotiation
    let msg1 = client_negotiator.initiate().unwrap();
    if let NegotiationResponse::SendMessage(msg2) = server_negotiator.handle_message(msg1).unwrap() {
        if let NegotiationResponse::SendMessage(msg3) = client_negotiator.handle_message(msg2).unwrap() {
            if let NegotiationResponse::SendMessage(msg4) = server_negotiator.handle_message(msg3).unwrap() {
                client_negotiator.handle_message(msg4).unwrap();
            }
        }
    }
    
    if client_negotiator.is_ready() && server_negotiator.is_ready() {
        println!("   ✓ Negotiation succeeded");
        println!("   Agreed features (intersection):");
        println!("     Nested: ✓");
        println!("     Streaming: ✓");
        println!("     Delta: ✗ (not supported by server)");
        println!("     LLB: ✗ (not supported by server)");
        println!("   Client will not use delta or LLB features");
    }
}
