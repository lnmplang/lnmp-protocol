//! Schema Negotiation Layer (SNL) for LNMP v0.5.
//!
//! This module provides capability exchange and schema version negotiation
//! between communicating parties to ensure compatibility and detect conflicts.

use super::types::TypeTag;
use std::collections::HashMap;

/// Feature flags for optional protocol features
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlags {
    /// Support for nested structures (v0.5+)
    pub supports_nested: bool,
    /// Support for streaming frame layer
    pub supports_streaming: bool,
    /// Support for delta encoding
    pub supports_delta: bool,
    /// Support for LLM optimization layer
    pub supports_llb: bool,
    /// Require checksums for data integrity
    pub requires_checksums: bool,
    /// Require canonical field ordering
    pub requires_canonical: bool,
}

impl FeatureFlags {
    /// Creates a new FeatureFlags with all features disabled
    pub fn new() -> Self {
        Self {
            supports_nested: false,
            supports_streaming: false,
            supports_delta: false,
            supports_llb: false,
            requires_checksums: false,
            requires_canonical: false,
        }
    }

    /// Creates FeatureFlags with all v0.5 features enabled
    pub fn v0_5_full() -> Self {
        Self {
            supports_nested: true,
            supports_streaming: true,
            supports_delta: true,
            supports_llb: true,
            requires_checksums: true,
            requires_canonical: true,
        }
    }

    /// Creates FeatureFlags with only v0.4 compatible features
    pub fn v0_4_compatible() -> Self {
        Self {
            supports_nested: false,
            supports_streaming: false,
            supports_delta: false,
            supports_llb: false,
            requires_checksums: false,
            requires_canonical: true,
        }
    }

    /// Computes the intersection of two feature sets (agreed features)
    pub fn intersect(&self, other: &FeatureFlags) -> FeatureFlags {
        FeatureFlags {
            supports_nested: self.supports_nested && other.supports_nested,
            supports_streaming: self.supports_streaming && other.supports_streaming,
            supports_delta: self.supports_delta && other.supports_delta,
            supports_llb: self.supports_llb && other.supports_llb,
            requires_checksums: self.requires_checksums || other.requires_checksums,
            requires_canonical: self.requires_canonical || other.requires_canonical,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Capabilities structure containing version and supported types
#[derive(Debug, Clone, PartialEq)]
pub struct Capabilities {
    /// Protocol version (e.g., 0x05 for v0.5)
    pub version: u8,
    /// Feature flags
    pub features: FeatureFlags,
    /// Supported type tags
    pub supported_types: Vec<TypeTag>,
}

impl Capabilities {
    /// Creates a new Capabilities structure
    pub fn new(version: u8, features: FeatureFlags, supported_types: Vec<TypeTag>) -> Self {
        Self {
            version,
            features,
            supported_types,
        }
    }

    /// Creates v0.5 capabilities with full feature support
    pub fn v0_5() -> Self {
        Self {
            version: 0x05,
            features: FeatureFlags::v0_5_full(),
            supported_types: vec![
                TypeTag::Int,
                TypeTag::Float,
                TypeTag::Bool,
                TypeTag::String,
                TypeTag::StringArray,
                TypeTag::NestedRecord,
                TypeTag::NestedArray,
            ],
        }
    }

    /// Creates v0.4 capabilities
    pub fn v0_4() -> Self {
        Self {
            version: 0x04,
            features: FeatureFlags::v0_4_compatible(),
            supported_types: vec![
                TypeTag::Int,
                TypeTag::Float,
                TypeTag::Bool,
                TypeTag::String,
                TypeTag::StringArray,
            ],
        }
    }

    /// Checks if a specific type tag is supported
    pub fn supports_type(&self, type_tag: TypeTag) -> bool {
        self.supported_types.contains(&type_tag)
    }
}

/// Negotiation message types
#[derive(Debug, Clone, PartialEq)]
pub enum NegotiationMessage {
    /// Initial capabilities message from client
    Capabilities {
        /// Protocol version
        version: u8,
        /// Feature flags
        features: FeatureFlags,
        /// Supported type tags
        supported_types: Vec<TypeTag>,
    },

    /// Capabilities acknowledgment from server
    CapabilitiesAck {
        /// Protocol version
        version: u8,
        /// Feature flags
        features: FeatureFlags,
    },

    /// Schema selection message from client
    SelectSchema {
        /// Schema identifier
        schema_id: String,
        /// FID to field name mappings
        fid_mappings: HashMap<u16, String>,
    },

    /// Ready message indicating negotiation complete
    Ready {
        /// Session identifier
        session_id: u64,
    },

    /// Error message
    Error {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
    },
}

/// Error codes for negotiation failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// FID conflict detected
    FidConflict = 0x01,
    /// Type mismatch detected
    TypeMismatch = 0x02,
    /// Unsupported feature requested
    UnsupportedFeature = 0x03,
    /// Protocol version mismatch
    ProtocolVersionMismatch = 0x04,
    /// Invalid state transition
    InvalidState = 0x05,
    /// Generic error
    Generic = 0xFF,
}

impl ErrorCode {
    /// Converts a byte to an ErrorCode
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(ErrorCode::FidConflict),
            0x02 => Some(ErrorCode::TypeMismatch),
            0x03 => Some(ErrorCode::UnsupportedFeature),
            0x04 => Some(ErrorCode::ProtocolVersionMismatch),
            0x05 => Some(ErrorCode::InvalidState),
            0xFF => Some(ErrorCode::Generic),
            _ => None,
        }
    }

    /// Converts the ErrorCode to a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Negotiation state for the state machine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationState {
    /// Initial state before negotiation starts
    Initial,
    /// Capabilities message sent, waiting for acknowledgment
    CapabilitiesSent,
    /// Capabilities received from remote party
    CapabilitiesReceived,
    /// Schema selected, waiting for ready confirmation
    SchemaSelected,
    /// Negotiation complete and ready for communication
    Ready,
    /// Negotiation failed with error message
    Failed(String),
}

/// Negotiation session containing agreed parameters
#[derive(Debug, Clone, PartialEq)]
pub struct NegotiationSession {
    /// Unique session identifier
    pub session_id: u64,
    /// Local capabilities
    pub local_caps: Capabilities,
    /// Remote capabilities
    pub remote_caps: Capabilities,
    /// Agreed feature flags (intersection)
    pub agreed_features: FeatureFlags,
    /// FID to field name mappings
    pub fid_mappings: HashMap<u16, String>,
}

impl NegotiationSession {
    /// Creates a new negotiation session
    pub fn new(
        session_id: u64,
        local_caps: Capabilities,
        remote_caps: Capabilities,
        fid_mappings: HashMap<u16, String>,
    ) -> Self {
        let agreed_features = local_caps.features.intersect(&remote_caps.features);
        Self {
            session_id,
            local_caps,
            remote_caps,
            agreed_features,
            fid_mappings,
        }
    }
}

/// Schema negotiator state machine
#[derive(Debug, Clone)]
pub struct SchemaNegotiator {
    /// Local capabilities
    local_capabilities: Capabilities,
    /// Remote capabilities (if received)
    remote_capabilities: Option<Capabilities>,
    /// Current negotiation state
    state: NegotiationState,
    /// Session ID counter
    next_session_id: u64,
    /// FID mappings
    fid_mappings: HashMap<u16, String>,
}

/// Response from handling a negotiation message
#[derive(Debug, Clone, PartialEq)]
pub enum NegotiationResponse {
    /// Send this message to the remote party
    SendMessage(NegotiationMessage),
    /// Negotiation complete with session
    Complete(NegotiationSession),
    /// Negotiation failed
    Failed(String),
    /// No action needed
    None,
}

/// Error type for negotiation operations
#[derive(Debug, Clone, PartialEq)]
pub enum NegotiationError {
    /// FID conflict detected
    FidConflict {
        /// Conflicting FID
        fid: u16,
        /// First field name
        name1: String,
        /// Second field name
        name2: String,
    },
    /// Type mismatch detected
    TypeMismatch {
        /// FID with type mismatch
        fid: u16,
        /// Expected type tag
        expected: TypeTag,
        /// Found type tag
        found: TypeTag,
    },
    /// Unsupported feature requested
    UnsupportedFeature {
        /// Feature name
        feature: String,
    },
    /// Protocol version mismatch
    ProtocolVersionMismatch {
        /// Local version
        local: u8,
        /// Remote version
        remote: u8,
    },
    /// Invalid state transition
    InvalidState {
        /// Current state
        current: NegotiationState,
        /// Expected state
        expected: NegotiationState,
    },
}

impl std::fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NegotiationError::FidConflict { fid, name1, name2 } => {
                write!(
                    f,
                    "FID conflict: FID {} maps to both '{}' and '{}'",
                    fid, name1, name2
                )
            }
            NegotiationError::TypeMismatch {
                fid,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Type mismatch for FID {}: expected {:?}, found {:?}",
                    fid, expected, found
                )
            }
            NegotiationError::UnsupportedFeature { feature } => {
                write!(f, "Unsupported feature: {}", feature)
            }
            NegotiationError::ProtocolVersionMismatch { local, remote } => {
                write!(
                    f,
                    "Protocol version mismatch: local 0x{:02X}, remote 0x{:02X}",
                    local, remote
                )
            }
            NegotiationError::InvalidState { current, expected } => {
                write!(
                    f,
                    "Invalid state transition: current {:?}, expected {:?}",
                    current, expected
                )
            }
        }
    }
}

impl std::error::Error for NegotiationError {}

impl SchemaNegotiator {
    /// Creates a new schema negotiator with local capabilities
    pub fn new(local_capabilities: Capabilities) -> Self {
        Self {
            local_capabilities,
            remote_capabilities: None,
            state: NegotiationState::Initial,
            next_session_id: 1,
            fid_mappings: HashMap::new(),
        }
    }

    /// Creates a negotiator with v0.5 capabilities
    pub fn v0_5() -> Self {
        Self::new(Capabilities::v0_5())
    }

    /// Creates a negotiator with v0.4 capabilities
    pub fn v0_4() -> Self {
        Self::new(Capabilities::v0_4())
    }

    /// Sets FID mappings for the negotiator
    pub fn with_fid_mappings(mut self, mappings: HashMap<u16, String>) -> Self {
        self.fid_mappings = mappings;
        self
    }

    /// Initiates negotiation by sending capabilities message
    pub fn initiate(&mut self) -> Result<NegotiationMessage, NegotiationError> {
        if self.state != NegotiationState::Initial {
            return Err(NegotiationError::InvalidState {
                current: self.state.clone(),
                expected: NegotiationState::Initial,
            });
        }

        self.state = NegotiationState::CapabilitiesSent;

        Ok(NegotiationMessage::Capabilities {
            version: self.local_capabilities.version,
            features: self.local_capabilities.features.clone(),
            supported_types: self.local_capabilities.supported_types.clone(),
        })
    }

    /// Handles an incoming negotiation message
    pub fn handle_message(
        &mut self,
        message: NegotiationMessage,
    ) -> Result<NegotiationResponse, NegotiationError> {
        match message {
            NegotiationMessage::Capabilities {
                version,
                features,
                supported_types,
            } => self.handle_capabilities(version, features, supported_types),

            NegotiationMessage::CapabilitiesAck { version, features } => {
                self.handle_capabilities_ack(version, features)
            }

            NegotiationMessage::SelectSchema {
                schema_id,
                fid_mappings,
            } => self.handle_select_schema(schema_id, fid_mappings),

            NegotiationMessage::Ready { session_id } => self.handle_ready(session_id),

            NegotiationMessage::Error { code: _, message } => {
                self.state = NegotiationState::Failed(message.clone());
                Ok(NegotiationResponse::Failed(message))
            }
        }
    }

    /// Returns true if negotiation is complete and ready
    pub fn is_ready(&self) -> bool {
        self.state == NegotiationState::Ready
    }

    /// Returns the current negotiation state
    pub fn state(&self) -> &NegotiationState {
        &self.state
    }

    /// Returns the local capabilities
    pub fn local_capabilities(&self) -> &Capabilities {
        &self.local_capabilities
    }

    /// Returns the remote capabilities if received
    pub fn remote_capabilities(&self) -> Option<&Capabilities> {
        self.remote_capabilities.as_ref()
    }

    // Private helper methods

    fn handle_capabilities(
        &mut self,
        version: u8,
        features: FeatureFlags,
        supported_types: Vec<TypeTag>,
    ) -> Result<NegotiationResponse, NegotiationError> {
        // Check version compatibility
        if version != self.local_capabilities.version {
            return Err(NegotiationError::ProtocolVersionMismatch {
                local: self.local_capabilities.version,
                remote: version,
            });
        }

        // Store remote capabilities
        self.remote_capabilities = Some(Capabilities::new(
            version,
            features.clone(),
            supported_types,
        ));
        self.state = NegotiationState::CapabilitiesReceived;

        // Send acknowledgment
        Ok(NegotiationResponse::SendMessage(
            NegotiationMessage::CapabilitiesAck {
                version: self.local_capabilities.version,
                features: self.local_capabilities.features.clone(),
            },
        ))
    }

    fn handle_capabilities_ack(
        &mut self,
        version: u8,
        features: FeatureFlags,
    ) -> Result<NegotiationResponse, NegotiationError> {
        if self.state != NegotiationState::CapabilitiesSent {
            return Err(NegotiationError::InvalidState {
                current: self.state.clone(),
                expected: NegotiationState::CapabilitiesSent,
            });
        }

        // Check version compatibility
        if version != self.local_capabilities.version {
            return Err(NegotiationError::ProtocolVersionMismatch {
                local: self.local_capabilities.version,
                remote: version,
            });
        }

        // Store remote capabilities (from ack)
        self.remote_capabilities = Some(Capabilities::new(
            version,
            features,
            self.local_capabilities.supported_types.clone(),
        ));

        // Transition to SchemaSelected after sending SelectSchema
        self.state = NegotiationState::SchemaSelected;

        // Send schema selection
        Ok(NegotiationResponse::SendMessage(
            NegotiationMessage::SelectSchema {
                schema_id: "default".to_string(),
                fid_mappings: self.fid_mappings.clone(),
            },
        ))
    }

    fn handle_select_schema(
        &mut self,
        _schema_id: String,
        fid_mappings: HashMap<u16, String>,
    ) -> Result<NegotiationResponse, NegotiationError> {
        if self.state != NegotiationState::CapabilitiesReceived {
            return Err(NegotiationError::InvalidState {
                current: self.state.clone(),
                expected: NegotiationState::CapabilitiesReceived,
            });
        }

        // Detect FID conflicts
        self.detect_fid_conflicts(&fid_mappings)?;

        // Store mappings and update state
        self.fid_mappings = fid_mappings;
        self.state = NegotiationState::SchemaSelected;

        // Generate session ID and send ready
        let session_id = self.next_session_id;
        self.next_session_id += 1;

        Ok(NegotiationResponse::SendMessage(
            NegotiationMessage::Ready { session_id },
        ))
    }

    fn handle_ready(&mut self, session_id: u64) -> Result<NegotiationResponse, NegotiationError> {
        if self.state != NegotiationState::SchemaSelected {
            return Err(NegotiationError::InvalidState {
                current: self.state.clone(),
                expected: NegotiationState::SchemaSelected,
            });
        }

        self.state = NegotiationState::Ready;

        // Create negotiation session
        let remote_caps = self
            .remote_capabilities
            .clone()
            .expect("Remote capabilities should be set");

        let session = NegotiationSession::new(
            session_id,
            self.local_capabilities.clone(),
            remote_caps,
            self.fid_mappings.clone(),
        );

        Ok(NegotiationResponse::Complete(session))
    }

    fn detect_fid_conflicts(
        &self,
        remote_mappings: &HashMap<u16, String>,
    ) -> Result<(), NegotiationError> {
        for (fid, remote_name) in remote_mappings {
            if let Some(local_name) = self.fid_mappings.get(fid) {
                if local_name != remote_name {
                    return Err(NegotiationError::FidConflict {
                        fid: *fid,
                        name1: local_name.clone(),
                        name2: remote_name.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Detects FID conflicts between two mapping sets
    ///
    /// Returns a list of all conflicts found
    pub fn detect_conflicts(
        local_mappings: &HashMap<u16, String>,
        remote_mappings: &HashMap<u16, String>,
    ) -> Vec<NegotiationError> {
        let mut conflicts = Vec::new();

        for (fid, remote_name) in remote_mappings {
            if let Some(local_name) = local_mappings.get(fid) {
                if local_name != remote_name {
                    conflicts.push(NegotiationError::FidConflict {
                        fid: *fid,
                        name1: local_name.clone(),
                        name2: remote_name.clone(),
                    });
                }
            }
        }

        conflicts
    }

    /// Detects type mismatches between expected and actual types for FIDs
    ///
    /// Returns a list of all type mismatches found
    pub fn detect_type_mismatches(
        expected_types: &HashMap<u16, TypeTag>,
        actual_types: &HashMap<u16, TypeTag>,
    ) -> Vec<NegotiationError> {
        let mut mismatches = Vec::new();

        for (fid, actual_type) in actual_types {
            if let Some(expected_type) = expected_types.get(fid) {
                if expected_type != actual_type {
                    mismatches.push(NegotiationError::TypeMismatch {
                        fid: *fid,
                        expected: *expected_type,
                        found: *actual_type,
                    });
                }
            }
        }

        // Ensure deterministic ordering of mismatches by sorting by fid.
        // HashMap iteration order is not stable across runs, so tests that
        // depend on the ordering of returned mismatches would be flaky.
        mismatches.sort_by(|a, b| {
            let fid_a = match a {
                NegotiationError::TypeMismatch { fid, .. } => *fid,
                _ => 0,
            };
            let fid_b = match b {
                NegotiationError::TypeMismatch { fid, .. } => *fid,
                _ => 0,
            };
            fid_a.cmp(&fid_b)
        });

        mismatches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_new() {
        let flags = FeatureFlags::new();
        assert!(!flags.supports_nested);
        assert!(!flags.supports_streaming);
        assert!(!flags.supports_delta);
        assert!(!flags.supports_llb);
        assert!(!flags.requires_checksums);
        assert!(!flags.requires_canonical);
    }

    #[test]
    fn test_feature_flags_v0_5_full() {
        let flags = FeatureFlags::v0_5_full();
        assert!(flags.supports_nested);
        assert!(flags.supports_streaming);
        assert!(flags.supports_delta);
        assert!(flags.supports_llb);
        assert!(flags.requires_checksums);
        assert!(flags.requires_canonical);
    }

    #[test]
    fn test_feature_flags_v0_4_compatible() {
        let flags = FeatureFlags::v0_4_compatible();
        assert!(!flags.supports_nested);
        assert!(!flags.supports_streaming);
        assert!(!flags.supports_delta);
        assert!(!flags.supports_llb);
        assert!(!flags.requires_checksums);
        assert!(flags.requires_canonical);
    }

    #[test]
    fn test_feature_flags_intersect() {
        let flags1 = FeatureFlags {
            supports_nested: true,
            supports_streaming: true,
            supports_delta: false,
            supports_llb: true,
            requires_checksums: false,
            requires_canonical: true,
        };

        let flags2 = FeatureFlags {
            supports_nested: true,
            supports_streaming: false,
            supports_delta: true,
            supports_llb: true,
            requires_checksums: true,
            requires_canonical: false,
        };

        let intersection = flags1.intersect(&flags2);
        assert!(intersection.supports_nested);
        assert!(!intersection.supports_streaming);
        assert!(!intersection.supports_delta);
        assert!(intersection.supports_llb);
        assert!(intersection.requires_checksums); // OR logic
        assert!(intersection.requires_canonical); // OR logic
    }

    #[test]
    fn test_feature_flags_intersect_all_enabled() {
        let flags1 = FeatureFlags::v0_5_full();
        let flags2 = FeatureFlags::v0_5_full();

        let intersection = flags1.intersect(&flags2);
        assert!(intersection.supports_nested);
        assert!(intersection.supports_streaming);
        assert!(intersection.supports_delta);
        assert!(intersection.supports_llb);
        assert!(intersection.requires_checksums);
        assert!(intersection.requires_canonical);
    }

    #[test]
    fn test_feature_flags_intersect_all_disabled() {
        let flags1 = FeatureFlags::new();
        let flags2 = FeatureFlags::new();

        let intersection = flags1.intersect(&flags2);
        assert!(!intersection.supports_nested);
        assert!(!intersection.supports_streaming);
        assert!(!intersection.supports_delta);
        assert!(!intersection.supports_llb);
        assert!(!intersection.requires_checksums);
        assert!(!intersection.requires_canonical);
    }

    #[test]
    fn test_feature_flags_intersect_v0_5_with_v0_4() {
        let v0_5 = FeatureFlags::v0_5_full();
        let v0_4 = FeatureFlags::v0_4_compatible();

        let intersection = v0_5.intersect(&v0_4);
        // Only features supported by both
        assert!(!intersection.supports_nested);
        assert!(!intersection.supports_streaming);
        assert!(!intersection.supports_delta);
        assert!(!intersection.supports_llb);
        // Requirements use OR logic
        assert!(intersection.requires_checksums); // v0.5 requires it
        assert!(intersection.requires_canonical); // Both require it
    }

    #[test]
    fn test_negotiation_session_agreed_features() {
        let local_caps = Capabilities {
            version: 0x05,
            features: FeatureFlags {
                supports_nested: true,
                supports_streaming: true,
                supports_delta: false,
                supports_llb: true,
                requires_checksums: false,
                requires_canonical: true,
            },
            supported_types: vec![TypeTag::Int],
        };

        let remote_caps = Capabilities {
            version: 0x05,
            features: FeatureFlags {
                supports_nested: true,
                supports_streaming: false,
                supports_delta: true,
                supports_llb: true,
                requires_checksums: true,
                requires_canonical: false,
            },
            supported_types: vec![TypeTag::Int],
        };

        let session = NegotiationSession::new(1, local_caps, remote_caps, HashMap::new());

        // Verify agreed features are the intersection
        assert!(session.agreed_features.supports_nested);
        assert!(!session.agreed_features.supports_streaming);
        assert!(!session.agreed_features.supports_delta);
        assert!(session.agreed_features.supports_llb);
        assert!(session.agreed_features.requires_checksums);
        assert!(session.agreed_features.requires_canonical);
    }

    #[test]
    fn test_capabilities_new() {
        let features = FeatureFlags::new();
        let types = vec![TypeTag::Int, TypeTag::String];
        let caps = Capabilities::new(0x05, features.clone(), types.clone());

        assert_eq!(caps.version, 0x05);
        assert_eq!(caps.features, features);
        assert_eq!(caps.supported_types, types);
    }

    #[test]
    fn test_capabilities_v0_5() {
        let caps = Capabilities::v0_5();
        assert_eq!(caps.version, 0x05);
        assert!(caps.features.supports_nested);
        assert!(caps.supports_type(TypeTag::NestedRecord));
        assert!(caps.supports_type(TypeTag::NestedArray));
    }

    #[test]
    fn test_capabilities_v0_4() {
        let caps = Capabilities::v0_4();
        assert_eq!(caps.version, 0x04);
        assert!(!caps.features.supports_nested);
        assert!(!caps.supports_type(TypeTag::NestedRecord));
        assert!(!caps.supports_type(TypeTag::NestedArray));
        assert!(caps.supports_type(TypeTag::Int));
        assert!(caps.supports_type(TypeTag::String));
    }

    #[test]
    fn test_capabilities_supports_type() {
        let caps = Capabilities::v0_5();
        assert!(caps.supports_type(TypeTag::Int));
        assert!(caps.supports_type(TypeTag::Float));
        assert!(caps.supports_type(TypeTag::Bool));
        assert!(caps.supports_type(TypeTag::String));
        assert!(caps.supports_type(TypeTag::StringArray));
        assert!(caps.supports_type(TypeTag::NestedRecord));
        assert!(caps.supports_type(TypeTag::NestedArray));
        assert!(!caps.supports_type(TypeTag::Reserved08));
    }

    #[test]
    fn test_negotiation_message_capabilities() {
        let msg = NegotiationMessage::Capabilities {
            version: 0x05,
            features: FeatureFlags::v0_5_full(),
            supported_types: vec![TypeTag::Int, TypeTag::String],
        };

        match msg {
            NegotiationMessage::Capabilities { version, .. } => {
                assert_eq!(version, 0x05);
            }
            _ => panic!("Expected Capabilities variant"),
        }
    }

    #[test]
    fn test_negotiation_message_capabilities_ack() {
        let msg = NegotiationMessage::CapabilitiesAck {
            version: 0x05,
            features: FeatureFlags::new(),
        };

        match msg {
            NegotiationMessage::CapabilitiesAck { version, .. } => {
                assert_eq!(version, 0x05);
            }
            _ => panic!("Expected CapabilitiesAck variant"),
        }
    }

    #[test]
    fn test_negotiation_message_select_schema() {
        let mut mappings = HashMap::new();
        mappings.insert(1, "user_id".to_string());
        mappings.insert(2, "username".to_string());

        let msg = NegotiationMessage::SelectSchema {
            schema_id: "user_schema_v1".to_string(),
            fid_mappings: mappings.clone(),
        };

        match msg {
            NegotiationMessage::SelectSchema {
                schema_id,
                fid_mappings,
            } => {
                assert_eq!(schema_id, "user_schema_v1");
                assert_eq!(fid_mappings.len(), 2);
                assert_eq!(fid_mappings.get(&1), Some(&"user_id".to_string()));
            }
            _ => panic!("Expected SelectSchema variant"),
        }
    }

    #[test]
    fn test_negotiation_message_ready() {
        let msg = NegotiationMessage::Ready { session_id: 12345 };

        match msg {
            NegotiationMessage::Ready { session_id } => {
                assert_eq!(session_id, 12345);
            }
            _ => panic!("Expected Ready variant"),
        }
    }

    #[test]
    fn test_negotiation_message_error() {
        let msg = NegotiationMessage::Error {
            code: ErrorCode::FidConflict,
            message: "FID 7 conflict".to_string(),
        };

        match msg {
            NegotiationMessage::Error { code, message } => {
                assert_eq!(code, ErrorCode::FidConflict);
                assert_eq!(message, "FID 7 conflict");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_error_code_from_u8() {
        assert_eq!(ErrorCode::from_u8(0x01), Some(ErrorCode::FidConflict));
        assert_eq!(ErrorCode::from_u8(0x02), Some(ErrorCode::TypeMismatch));
        assert_eq!(
            ErrorCode::from_u8(0x03),
            Some(ErrorCode::UnsupportedFeature)
        );
        assert_eq!(
            ErrorCode::from_u8(0x04),
            Some(ErrorCode::ProtocolVersionMismatch)
        );
        assert_eq!(ErrorCode::from_u8(0x05), Some(ErrorCode::InvalidState));
        assert_eq!(ErrorCode::from_u8(0xFF), Some(ErrorCode::Generic));
        assert_eq!(ErrorCode::from_u8(0x99), None);
    }

    #[test]
    fn test_error_code_to_u8() {
        assert_eq!(ErrorCode::FidConflict.to_u8(), 0x01);
        assert_eq!(ErrorCode::TypeMismatch.to_u8(), 0x02);
        assert_eq!(ErrorCode::UnsupportedFeature.to_u8(), 0x03);
        assert_eq!(ErrorCode::ProtocolVersionMismatch.to_u8(), 0x04);
        assert_eq!(ErrorCode::InvalidState.to_u8(), 0x05);
        assert_eq!(ErrorCode::Generic.to_u8(), 0xFF);
    }

    #[test]
    fn test_error_code_round_trip() {
        let codes = vec![
            ErrorCode::FidConflict,
            ErrorCode::TypeMismatch,
            ErrorCode::UnsupportedFeature,
            ErrorCode::ProtocolVersionMismatch,
            ErrorCode::InvalidState,
            ErrorCode::Generic,
        ];

        for code in codes {
            let byte = code.to_u8();
            let parsed = ErrorCode::from_u8(byte).unwrap();
            assert_eq!(parsed, code);
        }
    }
}

#[test]
fn test_detect_type_mismatches_no_mismatches() {
    let mut expected = HashMap::new();
    expected.insert(1, TypeTag::Int);
    expected.insert(2, TypeTag::String);

    let mut actual = HashMap::new();
    actual.insert(1, TypeTag::Int);
    actual.insert(2, TypeTag::String);

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert!(mismatches.is_empty());
}

#[test]
fn test_detect_type_mismatches_single_mismatch() {
    let mut expected = HashMap::new();
    expected.insert(1, TypeTag::Int);
    expected.insert(2, TypeTag::String);

    let mut actual = HashMap::new();
    actual.insert(1, TypeTag::Float); // Mismatch
    actual.insert(2, TypeTag::String);

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert_eq!(mismatches.len(), 1);

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
}

#[test]
fn test_detect_type_mismatches_multiple_mismatches() {
    let mut expected = HashMap::new();
    expected.insert(1, TypeTag::Int);
    expected.insert(2, TypeTag::String);
    expected.insert(3, TypeTag::Bool);

    let mut actual = HashMap::new();
    actual.insert(1, TypeTag::Float); // Mismatch
    actual.insert(2, TypeTag::Bool); // Mismatch
    actual.insert(3, TypeTag::Bool);

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert_eq!(mismatches.len(), 2);
}

#[test]
fn test_detect_type_mismatches_partial_overlap() {
    let mut expected = HashMap::new();
    expected.insert(1, TypeTag::Int);
    expected.insert(2, TypeTag::String);

    let mut actual = HashMap::new();
    actual.insert(2, TypeTag::String);
    actual.insert(3, TypeTag::Bool); // Not in expected, no mismatch

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert!(mismatches.is_empty());
}

#[test]
fn test_detect_type_mismatches_empty_types() {
    let expected = HashMap::new();
    let actual = HashMap::new();

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert!(mismatches.is_empty());
}

#[test]
fn test_detect_type_mismatches_nested_types() {
    let mut expected = HashMap::new();
    expected.insert(1, TypeTag::NestedRecord);
    expected.insert(2, TypeTag::NestedArray);

    let mut actual = HashMap::new();
    actual.insert(1, TypeTag::String); // Mismatch
    actual.insert(2, TypeTag::NestedArray);

    let mismatches = SchemaNegotiator::detect_type_mismatches(&expected, &actual);
    assert_eq!(mismatches.len(), 1);

    match &mismatches[0] {
        NegotiationError::TypeMismatch {
            fid,
            expected,
            found,
        } => {
            assert_eq!(*fid, 1);
            assert_eq!(*expected, TypeTag::NestedRecord);
            assert_eq!(*found, TypeTag::String);
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_detect_conflicts_no_conflicts() {
    let mut local = HashMap::new();
    local.insert(1, "user_id".to_string());
    local.insert(2, "username".to_string());

    let mut remote = HashMap::new();
    remote.insert(1, "user_id".to_string());
    remote.insert(2, "username".to_string());

    let conflicts = SchemaNegotiator::detect_conflicts(&local, &remote);
    assert!(conflicts.is_empty());
}

#[test]
fn test_detect_conflicts_single_conflict() {
    let mut local = HashMap::new();
    local.insert(1, "user_id".to_string());
    local.insert(2, "username".to_string());

    let mut remote = HashMap::new();
    remote.insert(1, "userId".to_string()); // Conflict
    remote.insert(2, "username".to_string());

    let conflicts = SchemaNegotiator::detect_conflicts(&local, &remote);
    assert_eq!(conflicts.len(), 1);

    match &conflicts[0] {
        NegotiationError::FidConflict { fid, name1, name2 } => {
            assert_eq!(*fid, 1);
            assert_eq!(name1, "user_id");
            assert_eq!(name2, "userId");
        }
        _ => panic!("Expected FidConflict"),
    }
}

#[test]
fn test_detect_conflicts_multiple_conflicts() {
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
fn test_detect_conflicts_partial_overlap() {
    let mut local = HashMap::new();
    local.insert(1, "user_id".to_string());
    local.insert(2, "username".to_string());

    let mut remote = HashMap::new();
    remote.insert(2, "username".to_string());
    remote.insert(3, "email".to_string()); // Not in local, no conflict

    let conflicts = SchemaNegotiator::detect_conflicts(&local, &remote);
    assert!(conflicts.is_empty());
}

#[test]
fn test_detect_conflicts_empty_mappings() {
    let local = HashMap::new();
    let remote = HashMap::new();

    let conflicts = SchemaNegotiator::detect_conflicts(&local, &remote);
    assert!(conflicts.is_empty());
}

#[test]
fn test_negotiation_state_equality() {
    assert_eq!(NegotiationState::Initial, NegotiationState::Initial);
    assert_eq!(
        NegotiationState::CapabilitiesSent,
        NegotiationState::CapabilitiesSent
    );
    assert_ne!(NegotiationState::Initial, NegotiationState::Ready);
}

#[test]
fn test_negotiation_session_new() {
    let local_caps = Capabilities::v0_5();
    let remote_caps = Capabilities::v0_5();
    let mut mappings = HashMap::new();
    mappings.insert(1, "user_id".to_string());

    let session = NegotiationSession::new(
        123,
        local_caps.clone(),
        remote_caps.clone(),
        mappings.clone(),
    );

    assert_eq!(session.session_id, 123);
    assert_eq!(session.local_caps, local_caps);
    assert_eq!(session.remote_caps, remote_caps);
    assert_eq!(session.fid_mappings, mappings);
    assert!(session.agreed_features.supports_nested);
}

#[test]
fn test_schema_negotiator_new() {
    let caps = Capabilities::v0_5();
    let negotiator = SchemaNegotiator::new(caps.clone());

    assert_eq!(negotiator.local_capabilities(), &caps);
    assert_eq!(negotiator.state(), &NegotiationState::Initial);
    assert!(negotiator.remote_capabilities().is_none());
    assert!(!negotiator.is_ready());
}

#[test]
fn test_schema_negotiator_v0_5() {
    let negotiator = SchemaNegotiator::v0_5();
    assert_eq!(negotiator.local_capabilities().version, 0x05);
    assert!(negotiator.local_capabilities().features.supports_nested);
}

#[test]
fn test_schema_negotiator_v0_4() {
    let negotiator = SchemaNegotiator::v0_4();
    assert_eq!(negotiator.local_capabilities().version, 0x04);
    assert!(!negotiator.local_capabilities().features.supports_nested);
}

#[test]
fn test_schema_negotiator_with_fid_mappings() {
    let mut mappings = HashMap::new();
    mappings.insert(1, "user_id".to_string());
    mappings.insert(2, "username".to_string());

    let negotiator = SchemaNegotiator::v0_5().with_fid_mappings(mappings.clone());
    assert_eq!(negotiator.fid_mappings, mappings);
}

#[test]
fn test_schema_negotiator_initiate() {
    let mut negotiator = SchemaNegotiator::v0_5();
    let result = negotiator.initiate();

    assert!(result.is_ok());
    assert_eq!(negotiator.state(), &NegotiationState::CapabilitiesSent);

    match result.unwrap() {
        NegotiationMessage::Capabilities { version, .. } => {
            assert_eq!(version, 0x05);
        }
        _ => panic!("Expected Capabilities message"),
    }
}

#[test]
fn test_schema_negotiator_initiate_invalid_state() {
    let mut negotiator = SchemaNegotiator::v0_5();
    negotiator.initiate().unwrap();

    // Try to initiate again
    let result = negotiator.initiate();
    assert!(result.is_err());
    match result {
        Err(NegotiationError::InvalidState { .. }) => {}
        _ => panic!("Expected InvalidState error"),
    }
}

#[test]
fn test_schema_negotiator_handle_capabilities() {
    let mut negotiator = SchemaNegotiator::v0_5();

    let msg = NegotiationMessage::Capabilities {
        version: 0x05,
        features: FeatureFlags::v0_5_full(),
        supported_types: vec![TypeTag::Int, TypeTag::String],
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_ok());
    assert_eq!(negotiator.state(), &NegotiationState::CapabilitiesReceived);

    match result.unwrap() {
        NegotiationResponse::SendMessage(NegotiationMessage::CapabilitiesAck {
            version, ..
        }) => {
            assert_eq!(version, 0x05);
        }
        _ => panic!("Expected SendMessage with CapabilitiesAck"),
    }
}

#[test]
fn test_schema_negotiator_handle_capabilities_version_mismatch() {
    let mut negotiator = SchemaNegotiator::v0_5();

    let msg = NegotiationMessage::Capabilities {
        version: 0x04, // Mismatch
        features: FeatureFlags::v0_4_compatible(),
        supported_types: vec![TypeTag::Int],
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_err());
    match result {
        Err(NegotiationError::ProtocolVersionMismatch { local, remote }) => {
            assert_eq!(local, 0x05);
            assert_eq!(remote, 0x04);
        }
        _ => panic!("Expected ProtocolVersionMismatch error"),
    }
}

#[test]
fn test_schema_negotiator_handle_capabilities_ack() {
    let mut negotiator = SchemaNegotiator::v0_5();
    negotiator.initiate().unwrap();

    let msg = NegotiationMessage::CapabilitiesAck {
        version: 0x05,
        features: FeatureFlags::v0_5_full(),
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_ok());
    assert_eq!(negotiator.state(), &NegotiationState::SchemaSelected);

    match result.unwrap() {
        NegotiationResponse::SendMessage(NegotiationMessage::SelectSchema { .. }) => {}
        _ => panic!("Expected SendMessage with SelectSchema"),
    }
}

#[test]
fn test_schema_negotiator_handle_select_schema() {
    let mut negotiator = SchemaNegotiator::v0_5();

    // Simulate receiving capabilities first
    negotiator.state = NegotiationState::CapabilitiesReceived;
    negotiator.remote_capabilities = Some(Capabilities::v0_5());

    let mut mappings = HashMap::new();
    mappings.insert(1, "user_id".to_string());

    let msg = NegotiationMessage::SelectSchema {
        schema_id: "test_schema".to_string(),
        fid_mappings: mappings,
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_ok());
    assert_eq!(negotiator.state(), &NegotiationState::SchemaSelected);

    match result.unwrap() {
        NegotiationResponse::SendMessage(NegotiationMessage::Ready { session_id }) => {
            assert_eq!(session_id, 1);
        }
        _ => panic!("Expected SendMessage with Ready"),
    }
}

#[test]
fn test_schema_negotiator_handle_select_schema_fid_conflict() {
    let mut local_mappings = HashMap::new();
    local_mappings.insert(1, "user_id".to_string());

    let mut negotiator = SchemaNegotiator::v0_5().with_fid_mappings(local_mappings);

    // Simulate receiving capabilities first
    negotiator.state = NegotiationState::CapabilitiesReceived;
    negotiator.remote_capabilities = Some(Capabilities::v0_5());

    let mut remote_mappings = HashMap::new();
    remote_mappings.insert(1, "username".to_string()); // Conflict!

    let msg = NegotiationMessage::SelectSchema {
        schema_id: "test_schema".to_string(),
        fid_mappings: remote_mappings,
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_err());
    match result {
        Err(NegotiationError::FidConflict { fid, name1, name2 }) => {
            assert_eq!(fid, 1);
            assert_eq!(name1, "user_id");
            assert_eq!(name2, "username");
        }
        _ => panic!("Expected FidConflict error"),
    }
}

#[test]
fn test_schema_negotiator_handle_ready() {
    let mut negotiator = SchemaNegotiator::v0_5();

    // Simulate schema selected state
    negotiator.state = NegotiationState::SchemaSelected;
    negotiator.remote_capabilities = Some(Capabilities::v0_5());

    let msg = NegotiationMessage::Ready { session_id: 42 };

    let result = negotiator.handle_message(msg);
    assert!(result.is_ok());
    assert_eq!(negotiator.state(), &NegotiationState::Ready);
    assert!(negotiator.is_ready());

    match result.unwrap() {
        NegotiationResponse::Complete(session) => {
            assert_eq!(session.session_id, 42);
        }
        _ => panic!("Expected Complete response"),
    }
}

#[test]
fn test_schema_negotiator_handle_error() {
    let mut negotiator = SchemaNegotiator::v0_5();

    let msg = NegotiationMessage::Error {
        code: ErrorCode::Generic,
        message: "Test error".to_string(),
    };

    let result = negotiator.handle_message(msg);
    assert!(result.is_ok());

    match negotiator.state() {
        NegotiationState::Failed(msg) => {
            assert_eq!(msg, "Test error");
        }
        _ => panic!("Expected Failed state"),
    }

    match result.unwrap() {
        NegotiationResponse::Failed(msg) => {
            assert_eq!(msg, "Test error");
        }
        _ => panic!("Expected Failed response"),
    }
}

#[test]
fn test_negotiation_error_display() {
    let err = NegotiationError::FidConflict {
        fid: 7,
        name1: "field_a".to_string(),
        name2: "field_b".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("FID 7"));
    assert!(msg.contains("field_a"));
    assert!(msg.contains("field_b"));
}

#[test]
fn test_full_negotiation_flow_client_initiated() {
    // Client side
    let mut client = SchemaNegotiator::v0_5();
    let mut client_mappings = HashMap::new();
    client_mappings.insert(1, "user_id".to_string());
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
        }
        _ => panic!("Expected Complete response"),
    }
}
