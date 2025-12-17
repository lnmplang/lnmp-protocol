//! FID Registry for runtime validation
//!
//! This module provides optional runtime validation of Field IDs against the
//! official LNMP FID registry. It allows parsers and encoders to verify that
//! field IDs match expected types.
//!
//! # Example
//!
//! ```ignore
//! use lnmp_core::registry::{FidRegistry, ValidationMode, ValidationResult};
//!
//! // Load registry from YAML
//! let registry = FidRegistry::from_yaml_str(yaml_content)?;
//!
//! // Validate a field
//! let result = registry.validate_field(&field);
//! match result {
//!     ValidationResult::Valid => println!("Field is valid"),
//!     ValidationResult::TypeMismatch { fid, expected, found } => {
//!         println!("F{}: expected {:?}, found {:?}", fid, expected, found);
//!     }
//!     _ => {}
//! }
//! ```

use crate::{LnmpField, LnmpRecord, LnmpValue, TypeHint};
use std::collections::HashMap;

/// FID Registry containing all registered field definitions
#[derive(Debug, Clone)]
pub struct FidRegistry {
    entries: HashMap<u16, FidEntry>,
    version: String,
    protocol_version: String,
}

/// A single FID entry from the registry
#[derive(Debug, Clone)]
pub struct FidEntry {
    /// Field ID (0-65535)
    pub fid: u16,
    /// Human-readable name
    pub name: String,
    /// Expected value type
    pub expected_type: ExpectedType,
    /// FID range category
    pub range: FidRange,
    /// Current status
    pub status: FidStatus,
    /// Version when this FID was introduced
    pub since: String,
    /// Description of the field
    pub description: String,
}

/// Expected type for a FID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType {
    /// Integer value
    Int,
    /// Floating-point value
    Float,
    /// Boolean value
    Bool,
    /// String value
    String,
    /// String array
    StringArray,
    /// Integer array
    IntArray,
    /// Float array
    FloatArray,
    /// Boolean array
    BoolArray,
    /// Nested record
    Record,
    /// Array of records
    RecordArray,
    /// Any type (for private range - no validation)
    Any,
}

/// FID range category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FidRange {
    /// Core range (0-255) - LOCKED, stable forever
    Core,
    /// Standard range (256-16383) - STABLE, may deprecate
    Standard,
    /// Extended range (16384-32767) - EVOLVING
    Extended,
    /// Private range (32768-65535) - Application-specific
    Private,
}

/// FID lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FidStatus {
    /// Field is proposed but not yet active
    Proposed,
    /// Field is active and in use
    Active,
    /// Field is deprecated and should not be used
    Deprecated,
    /// Field is tombstoned and must never be reused
    Tombstoned,
}

/// Validation mode for FID checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// No validation (default, backward compatible)
    #[default]
    None,
    /// Log warnings but continue parsing
    Warn,
    /// Return error on validation failure
    Error,
}

/// Result of FID validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Field is valid
    Valid,
    /// Type mismatch between expected and found
    TypeMismatch {
        /// Field ID
        fid: u16,
        /// Expected type from registry
        expected: ExpectedType,
        /// Found type from value
        found: TypeHint,
    },
    /// FID is not in the registry
    UnknownFid {
        /// Field ID
        fid: u16,
        /// Which range the FID falls into
        range: FidRange,
    },
    /// FID is deprecated
    DeprecatedFid {
        /// Field ID
        fid: u16,
        /// Field name
        name: String,
    },
    /// FID is tombstoned (must not be used)
    TombstonedFid {
        /// Field ID
        fid: u16,
        /// Field name
        name: String,
    },
}

/// Error when loading or parsing the registry
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// Failed to parse YAML
    #[error("Failed to parse registry YAML: {0}")]
    YamlParseError(String),
    /// Invalid FID value
    #[error("Invalid FID value: {0}")]
    InvalidFid(String),
    /// Invalid type
    #[error("Invalid type: {0}")]
    InvalidType(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
}

impl FidRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: String::new(),
            protocol_version: String::new(),
        }
    }

    /// Parse registry from YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, RegistryError> {
        let mut registry = Self::new();

        // Simple YAML parser - parse key sections
        let mut current_section = "";
        let mut current_entry: Option<FidEntryBuilder> = None;

        for line in yaml.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Detect sections
            if trimmed == "core:" {
                current_section = "core";
                continue;
            } else if trimmed == "standard:" {
                current_section = "standard";
                continue;
            } else if trimmed == "extended:" {
                current_section = "extended";
                continue;
            } else if trimmed.starts_with("metadata:") {
                current_section = "metadata";
                continue;
            }

            // Parse metadata
            if current_section == "metadata" {
                if let Some((key, value)) = parse_yaml_kv(trimmed) {
                    match key {
                        "version" => registry.version = value.trim_matches('"').to_string(),
                        "protocol_version" => {
                            registry.protocol_version = value.trim_matches('"').to_string()
                        }
                        _ => {}
                    }
                }
                continue;
            }

            // Parse field entries
            if trimmed.starts_with("- fid:") {
                // Save previous entry
                if let Some(builder) = current_entry.take() {
                    if let Some(entry) = builder.build(current_section) {
                        registry.entries.insert(entry.fid, entry);
                    }
                }

                // Start new entry
                let fid_str = trimmed.trim_start_matches("- fid:").trim();
                if let Ok(fid) = fid_str.parse::<u16>() {
                    current_entry = Some(FidEntryBuilder::new(fid));
                }
            } else if let Some(ref mut builder) = current_entry {
                // Parse entry fields
                if let Some((key, value)) = parse_yaml_kv(trimmed) {
                    let value = value.trim_matches('"').to_string();
                    match key {
                        "name" => builder.name = Some(value),
                        "type" => builder.type_str = Some(value),
                        "status" => builder.status_str = Some(value),
                        "since" => builder.since = Some(value),
                        "description" => builder.description = Some(value),
                        "unit" => builder.unit = Some(value),
                        _ => {}
                    }
                }
            }
        }

        // Don't forget the last entry
        if let Some(builder) = current_entry {
            if let Some(entry) = builder.build(current_section) {
                registry.entries.insert(entry.fid, entry);
            }
        }

        Ok(registry)
    }

    /// Load registry from a YAML file
    pub fn from_yaml_file(path: &std::path::Path) -> Result<Self, RegistryError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| RegistryError::IoError(e.to_string()))?;
        Self::from_yaml_str(&content)
    }

    /// Get a FID entry by ID
    pub fn get(&self, fid: u16) -> Option<&FidEntry> {
        self.entries.get(&fid)
    }

    /// Get the registry version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the protocol version
    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Validate a single field against the registry
    pub fn validate_field(&self, field: &LnmpField) -> ValidationResult {
        let fid = field.fid;
        let range = FidRange::from_fid(fid);

        // Private range - always valid (no registry entries)
        if range == FidRange::Private {
            return ValidationResult::Valid;
        }

        // Check if FID is in registry
        if let Some(entry) = self.entries.get(&fid) {
            // Check status
            match entry.status {
                FidStatus::Tombstoned => {
                    return ValidationResult::TombstonedFid {
                        fid,
                        name: entry.name.clone(),
                    };
                }
                FidStatus::Deprecated => {
                    // Deprecated is a warning, continue with type check
                    // but we'll return this as the result if type matches
                    let type_result = self.check_type(entry, &field.value);
                    if type_result == ValidationResult::Valid {
                        return ValidationResult::DeprecatedFid {
                            fid,
                            name: entry.name.clone(),
                        };
                    }
                    return type_result;
                }
                _ => {}
            }

            // Check type
            self.check_type(entry, &field.value)
        } else {
            // Unknown FID in non-private range
            ValidationResult::UnknownFid { fid, range }
        }
    }

    /// Validate all fields in a record
    pub fn validate_record(&self, record: &LnmpRecord) -> Vec<ValidationResult> {
        record
            .fields()
            .iter()
            .map(|field| self.validate_field(field))
            .filter(|r| *r != ValidationResult::Valid)
            .collect()
    }

    /// Check if a value matches the expected type
    fn check_type(&self, entry: &FidEntry, value: &LnmpValue) -> ValidationResult {
        let matches = match entry.expected_type {
            ExpectedType::Int => matches!(value, LnmpValue::Int(_)),
            ExpectedType::Float => matches!(value, LnmpValue::Float(_)),
            ExpectedType::Bool => matches!(value, LnmpValue::Bool(_)),
            ExpectedType::String => matches!(value, LnmpValue::String(_)),
            ExpectedType::StringArray => matches!(value, LnmpValue::StringArray(_)),
            ExpectedType::IntArray => matches!(value, LnmpValue::IntArray(_)),
            ExpectedType::FloatArray => matches!(value, LnmpValue::FloatArray(_)),
            ExpectedType::BoolArray => matches!(value, LnmpValue::BoolArray(_)),
            ExpectedType::Record => matches!(value, LnmpValue::NestedRecord(_)),
            ExpectedType::RecordArray => matches!(value, LnmpValue::NestedArray(_)),
            ExpectedType::Any => true,
        };

        if matches {
            ValidationResult::Valid
        } else {
            // Determine the actual type hint from the value
            let found = match value {
                LnmpValue::Int(_) => TypeHint::Int,
                LnmpValue::Float(_) => TypeHint::Float,
                LnmpValue::Bool(_) => TypeHint::Bool,
                LnmpValue::String(_) => TypeHint::String,
                LnmpValue::StringArray(_) => TypeHint::StringArray,
                LnmpValue::IntArray(_) => TypeHint::IntArray,
                LnmpValue::FloatArray(_) => TypeHint::FloatArray,
                LnmpValue::BoolArray(_) => TypeHint::BoolArray,
                LnmpValue::NestedRecord(_) => TypeHint::Record,
                LnmpValue::NestedArray(_) => TypeHint::RecordArray,
                LnmpValue::Embedding(_) | LnmpValue::EmbeddingDelta(_) => TypeHint::Embedding,
                #[cfg(feature = "quant")]
                LnmpValue::QuantizedEmbedding(_) => TypeHint::QuantizedEmbedding,
            };

            ValidationResult::TypeMismatch {
                fid: entry.fid,
                expected: entry.expected_type,
                found,
            }
        }
    }

    /// Add an entry to the registry (for testing/programmatic use)
    pub fn add_entry(&mut self, entry: FidEntry) {
        self.entries.insert(entry.fid, entry);
    }
}

impl Default for FidRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FidRange {
    /// Determine the range from a FID value
    pub fn from_fid(fid: u16) -> Self {
        match fid {
            0..=255 => FidRange::Core,
            256..=16383 => FidRange::Standard,
            16384..=32767 => FidRange::Extended,
            _ => FidRange::Private,
        }
    }
}

impl ExpectedType {
    /// Parse type from string
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "int" | "integer" => Some(Self::Int),
            "float" | "double" => Some(Self::Float),
            "bool" | "boolean" => Some(Self::Bool),
            "string" | "str" => Some(Self::String),
            "stringarray" | "string_array" => Some(Self::StringArray),
            "intarray" | "int_array" => Some(Self::IntArray),
            "floatarray" | "float_array" => Some(Self::FloatArray),
            "boolarray" | "bool_array" => Some(Self::BoolArray),
            "record" | "nestedrecord" | "nested_record" => Some(Self::Record),
            "recordarray" | "record_array" | "nestedarray" => Some(Self::RecordArray),
            "any" => Some(Self::Any),
            _ => None,
        }
    }
}

impl FidStatus {
    /// Parse status from string
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PROPOSED" => Some(Self::Proposed),
            "ACTIVE" => Some(Self::Active),
            "DEPRECATED" => Some(Self::Deprecated),
            "TOMBSTONED" => Some(Self::Tombstoned),
            _ => None,
        }
    }
}

// Helper for YAML parsing
fn parse_yaml_kv(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if let Some(pos) = trimmed.find(':') {
        let key = trimmed[..pos].trim();
        let value = trimmed[pos + 1..].trim();
        if !key.is_empty() {
            return Some((key, value));
        }
    }
    None
}

// Builder for FidEntry
struct FidEntryBuilder {
    fid: u16,
    name: Option<String>,
    type_str: Option<String>,
    status_str: Option<String>,
    since: Option<String>,
    description: Option<String>,
    #[allow(dead_code)]
    unit: Option<String>,
}

impl FidEntryBuilder {
    fn new(fid: u16) -> Self {
        Self {
            fid,
            name: None,
            type_str: None,
            status_str: None,
            since: None,
            description: None,
            unit: None,
        }
    }

    fn build(self, section: &str) -> Option<FidEntry> {
        let name = self.name?;
        let type_str = self.type_str.as_deref().unwrap_or("Any");
        let status_str = self.status_str.as_deref().unwrap_or("ACTIVE");

        let expected_type = ExpectedType::parse_str(type_str).unwrap_or(ExpectedType::Any);
        let status = FidStatus::parse_str(status_str).unwrap_or(FidStatus::Active);

        let range = match section {
            "core" => FidRange::Core,
            "standard" => FidRange::Standard,
            "extended" => FidRange::Extended,
            _ => FidRange::from_fid(self.fid),
        };

        Some(FidEntry {
            fid: self.fid,
            name,
            expected_type,
            range,
            status,
            since: self.since.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
        })
    }
}

/// Embedded registry from the official fids.yaml
///
/// This function returns a FidRegistry parsed from the registry file
/// that was embedded at compile time.
///
/// # Example
///
/// ```ignore
/// use lnmp_core::registry::embedded_registry;
///
/// let registry = embedded_registry();
/// println!("Registry version: {}", registry.version());
/// println!("Entries: {}", registry.len());
/// ```
pub fn embedded_registry() -> FidRegistry {
    const YAML: &str = include_str!("../../../registry/fids.yaml");
    FidRegistry::from_yaml_str(YAML).expect("embedded registry should be valid")
}

// =============================================================================
// Phase 4: Registry Synchronization (v0.5.14)
// =============================================================================

/// Registry synchronization manager for multi-peer environments
///
/// Tracks registry versions across peers and computes deltas for sync.
#[derive(Debug, Clone)]
pub struct RegistrySync {
    /// Local registry
    local: FidRegistry,
    /// Known peer registry versions
    peer_versions: HashMap<String, String>,
}

impl RegistrySync {
    /// Create a new sync manager with local registry
    pub fn new(local: FidRegistry) -> Self {
        Self {
            local,
            peer_versions: HashMap::new(),
        }
    }

    /// Create with embedded registry
    pub fn with_embedded() -> Self {
        Self::new(embedded_registry())
    }

    /// Get local registry
    pub fn local(&self) -> &FidRegistry {
        &self.local
    }

    /// Get local registry version
    pub fn local_version(&self) -> &str {
        self.local.version()
    }

    /// Register a peer's registry version
    pub fn register_peer(&mut self, peer_id: String, version: String) {
        self.peer_versions.insert(peer_id, version);
    }

    /// Check if local registry is newer than peer's
    pub fn is_ahead_of(&self, peer_id: &str) -> bool {
        if let Some(peer_version) = self.peer_versions.get(peer_id) {
            self.compare_versions(self.local.version(), peer_version) > 0
        } else {
            true // Unknown peer = assume local is newer
        }
    }

    /// Check if local registry is older than peer's
    pub fn is_behind(&self, peer_id: &str) -> bool {
        if let Some(peer_version) = self.peer_versions.get(peer_id) {
            self.compare_versions(self.local.version(), peer_version) < 0
        } else {
            false
        }
    }

    /// Get FIDs that peer doesn't have (based on version)
    pub fn delta_fids_for(&self, peer_id: &str) -> Vec<u16> {
        // For now, just return all FIDs if peer is behind
        // Full implementation would track FID-by-FID changes
        if self.is_ahead_of(peer_id) {
            self.local.entries.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Get entries for specific FIDs
    pub fn get_entries(&self, fids: &[u16]) -> Vec<&FidEntry> {
        fids.iter().filter_map(|fid| self.local.get(*fid)).collect()
    }

    /// Compare semantic versions (simple comparison)
    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        // Parse version as major.minor.patch
        let parse = |v: &str| -> (u32, u32, u32) {
            let parts: Vec<u32> = v.split('.').filter_map(|s| s.trim().parse().ok()).collect();
            (
                parts.first().copied().unwrap_or(0),
                parts.get(1).copied().unwrap_or(0),
                parts.get(2).copied().unwrap_or(0),
            )
        };

        let (m1, n1, p1) = parse(v1);
        let (m2, n2, p2) = parse(v2);

        if m1 != m2 {
            return (m1 as i32) - (m2 as i32);
        }
        if n1 != n2 {
            return (n1 as i32) - (n2 as i32);
        }
        (p1 as i32) - (p2 as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_YAML: &str = r#"
metadata:
  version: "1.0.0"
  protocol_version: "0.5.13"

core:
  - fid: 1
    name: entity_id
    type: Int
    status: ACTIVE
    since: "0.1.0"
    description: "Entity identifier"

  - fid: 7
    name: is_active
    type: Bool
    status: ACTIVE
    since: "0.1.0"
    description: "Active flag"

  - fid: 12
    name: user_id
    type: Int
    status: ACTIVE
    since: "0.1.0"
    description: "User identifier"

  - fid: 20
    name: name
    type: String
    status: ACTIVE
    since: "0.1.0"
    description: "Name"

  - fid: 99
    name: deprecated_field
    type: Int
    status: DEPRECATED
    since: "0.1.0"
    description: "Deprecated"

standard:
  - fid: 256
    name: position
    type: FloatArray
    status: ACTIVE
    since: "0.5.0"
    description: "Position"
"#;

    #[test]
    fn test_parse_registry() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        assert_eq!(registry.version(), "1.0.0");
        assert_eq!(registry.protocol_version(), "0.5.13");
        assert_eq!(registry.len(), 6);
    }

    #[test]
    fn test_get_entry() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let entry = registry.get(12).unwrap();
        assert_eq!(entry.name, "user_id");
        assert_eq!(entry.expected_type, ExpectedType::Int);
        assert_eq!(entry.range, FidRange::Core);
    }

    #[test]
    fn test_validate_valid_field() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let field = LnmpField {
            fid: 12,
            value: LnmpValue::Int(123),
        };

        assert_eq!(registry.validate_field(&field), ValidationResult::Valid);
    }

    #[test]
    fn test_validate_type_mismatch() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let field = LnmpField {
            fid: 12, // expects Int
            value: LnmpValue::String("hello".to_string()),
        };

        match registry.validate_field(&field) {
            ValidationResult::TypeMismatch { fid, expected, .. } => {
                assert_eq!(fid, 12);
                assert_eq!(expected, ExpectedType::Int);
            }
            _ => panic!("Expected TypeMismatch"),
        }
    }

    #[test]
    fn test_validate_deprecated() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let field = LnmpField {
            fid: 99,
            value: LnmpValue::Int(123),
        };

        match registry.validate_field(&field) {
            ValidationResult::DeprecatedFid { fid, name } => {
                assert_eq!(fid, 99);
                assert_eq!(name, "deprecated_field");
            }
            _ => panic!("Expected DeprecatedFid"),
        }
    }

    #[test]
    fn test_validate_unknown_fid() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let field = LnmpField {
            fid: 50, // not in registry
            value: LnmpValue::Int(123),
        };

        match registry.validate_field(&field) {
            ValidationResult::UnknownFid { fid, range } => {
                assert_eq!(fid, 50);
                assert_eq!(range, FidRange::Core);
            }
            _ => panic!("Expected UnknownFid"),
        }
    }

    #[test]
    fn test_private_range_always_valid() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();

        let field = LnmpField {
            fid: 40000, // Private range
            value: LnmpValue::String("anything".to_string()),
        };

        assert_eq!(registry.validate_field(&field), ValidationResult::Valid);
    }

    #[test]
    fn test_fid_range() {
        assert_eq!(FidRange::from_fid(0), FidRange::Core);
        assert_eq!(FidRange::from_fid(255), FidRange::Core);
        assert_eq!(FidRange::from_fid(256), FidRange::Standard);
        assert_eq!(FidRange::from_fid(16383), FidRange::Standard);
        assert_eq!(FidRange::from_fid(16384), FidRange::Extended);
        assert_eq!(FidRange::from_fid(32767), FidRange::Extended);
        assert_eq!(FidRange::from_fid(32768), FidRange::Private);
        assert_eq!(FidRange::from_fid(65535), FidRange::Private);
    }

    #[test]
    fn test_embedded_registry() {
        let registry = super::embedded_registry();

        // Check that registry is not empty
        assert!(!registry.is_empty());

        // Check version is set
        assert!(!registry.version().is_empty());
        assert!(!registry.protocol_version().is_empty());

        // Check some known FIDs exist
        let entity_id = registry.get(1).expect("F1 should exist");
        assert_eq!(entity_id.name, "entity_id");
        assert_eq!(entity_id.expected_type, ExpectedType::Int);

        let user_id = registry.get(12).expect("F12 should exist");
        assert_eq!(user_id.name, "user_id");
        assert_eq!(user_id.expected_type, ExpectedType::Int);

        // Check spatial FIDs in standard range
        let position = registry.get(256).expect("F256 should exist");
        assert_eq!(position.name, "position");
        assert_eq!(position.range, FidRange::Standard);
    }

    // ==================== RegistrySync Tests (v0.5.14) ====================

    #[test]
    fn test_registry_sync_creation() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let sync = RegistrySync::new(registry);

        assert_eq!(sync.local_version(), "1.0.0");
        assert!(!sync.local().is_empty());
    }

    #[test]
    fn test_registry_sync_with_embedded() {
        let sync = RegistrySync::with_embedded();

        assert!(!sync.local_version().is_empty());
        assert!(!sync.local().is_empty());
    }

    #[test]
    fn test_registry_sync_peer_registration() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let mut sync = RegistrySync::new(registry);

        // Register peer
        sync.register_peer("peer-1".to_string(), "0.9.0".to_string());
        sync.register_peer("peer-2".to_string(), "1.0.0".to_string());
        sync.register_peer("peer-3".to_string(), "1.1.0".to_string());

        // Test is_ahead_of
        assert!(sync.is_ahead_of("peer-1")); // 1.0.0 > 0.9.0
        assert!(!sync.is_ahead_of("peer-2")); // 1.0.0 == 1.0.0
        assert!(!sync.is_ahead_of("peer-3")); // 1.0.0 < 1.1.0

        // Test is_behind
        assert!(!sync.is_behind("peer-1")); // 1.0.0 > 0.9.0
        assert!(!sync.is_behind("peer-2")); // 1.0.0 == 1.0.0
        assert!(sync.is_behind("peer-3")); // 1.0.0 < 1.1.0
    }

    #[test]
    fn test_registry_sync_unknown_peer() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let sync = RegistrySync::new(registry);

        // Unknown peer should be treated as behind
        assert!(sync.is_ahead_of("unknown-peer"));
        assert!(!sync.is_behind("unknown-peer"));
    }

    #[test]
    fn test_registry_sync_delta_fids() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let mut sync = RegistrySync::new(registry);

        // Register old peer
        sync.register_peer("old-peer".to_string(), "0.5.0".to_string());

        // Should return all FIDs for old peer
        let delta = sync.delta_fids_for("old-peer");
        assert!(!delta.is_empty());
        assert!(delta.contains(&1)); // entity_id
        assert!(delta.contains(&12)); // user_id

        // Register current peer
        sync.register_peer("current-peer".to_string(), "1.0.0".to_string());

        // Should return empty for current peer
        let delta = sync.delta_fids_for("current-peer");
        assert!(delta.is_empty());
    }

    #[test]
    fn test_registry_sync_get_entries() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let sync = RegistrySync::new(registry);

        let entries = sync.get_entries(&[1, 12, 999]);

        // Should get 2 entries (1 and 12 exist, 999 doesn't)
        assert_eq!(entries.len(), 2);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"entity_id"));
        assert!(names.contains(&"user_id"));
    }

    #[test]
    fn test_registry_sync_version_comparison() {
        let registry = FidRegistry::from_yaml_str(TEST_YAML).unwrap();
        let mut sync = RegistrySync::new(registry);

        // Test various version comparisons
        sync.register_peer("v1".to_string(), "1.0.0".to_string());
        sync.register_peer("v2".to_string(), "1.0.1".to_string());
        sync.register_peer("v3".to_string(), "1.1.0".to_string());
        sync.register_peer("v4".to_string(), "2.0.0".to_string());
        sync.register_peer("v5".to_string(), "0.9.9".to_string());

        // All should be behind v4 (2.0.0)
        assert!(!sync.is_ahead_of("v4"));
        assert!(sync.is_behind("v4"));

        // All should be ahead of v5 (0.9.9)
        assert!(sync.is_ahead_of("v5"));
        assert!(!sync.is_behind("v5"));
    }
}
