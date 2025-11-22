//! Protocol profiles for controlling determinism and validation requirements
//!
//! The profile system allows configuring LNMP behavior for different use cases:
//! - **Loose**: Maximum compatibility, minimal validation
//! - **Standard**: Balanced mode with canonical output but flexible input
//! - **Strict**: Maximum determinism for LLM drift prevention
//!
//! # Example
//!
//! ```
//! use lnmp_core::profile::{LnmpProfile, StrictDeterministicConfig};
//!
//! // Use strict profile for LLM applications
//! let config = StrictDeterministicConfig::strict();
//! assert!(config.require_type_hints);
//! assert!(config.reject_unsorted_fields);
//!
//! // Or use standard profile for general use
//! let config = StrictDeterministicConfig::standard();
//! assert!(!config.require_type_hints);
//! ```

/// LNMP protocol profile
///
/// Profiles define different levels of strictness for parsing, encoding,
/// and validation of LNMP data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LnmpProfile {
    /// Loose mode: Maximum backward compatibility
    ///
    /// - Accepts unsorted fields
    /// - Type hints optional
    /// - No canonical form enforcement
    /// - Best for: Legacy data, migration scenarios
    Loose,

    /// Standard mode: Balanced approach (default)
    ///
    /// - Canonical output (sorted fields)
    /// - Accepts non-canonical input
    /// - Type hints optional
    /// - Best for: General purpose use
    #[default]
    Standard,

    /// Strict mode: Maximum determinism
    ///
    /// - Enforces canonical form in input/output
    /// - Type hints required
    /// - Field order validation
    /// - Best for: LLM applications, drift prevention
    Strict,
}

impl LnmpProfile {
    /// Returns the string representation of the profile
    pub fn as_str(&self) -> &'static str {
        match self {
            LnmpProfile::Loose => "loose",
            LnmpProfile::Standard => "standard",
            LnmpProfile::Strict => "strict",
        }
    }

    /// Parses a profile from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "loose" => Some(Self::Loose),
            "standard" => Some(Self::Standard),
            "strict" => Some(Self::Strict),
            _ => None,
        }
    }

    /// Returns the config for this profile
    pub fn config(&self) -> StrictDeterministicConfig {
        match self {
            LnmpProfile::Loose => StrictDeterministicConfig::loose(),
            LnmpProfile::Standard => StrictDeterministicConfig::standard(),
            LnmpProfile::Strict => StrictDeterministicConfig::strict(),
        }
    }
}

/// Configuration for strict deterministic behavior
///
/// Controls validation and canonicalization requirements for parsing
/// and encoding LNMP data.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDeterministicConfig {
    /// Reject records with unsorted fields
    ///
    /// When `true`, parsing will fail if fields are not in ascending FID order.
    pub reject_unsorted_fields: bool,

    /// Require type hints on all fields
    ///
    /// When `true`, parsing will fail if any field lacks a type hint.
    pub require_type_hints: bool,

    /// Enforce canonical boolean representation (0/1 only)
    ///
    /// When `true`, parsing will fail for non-canonical boolean values
    /// (e.g., "true", "false", "yes", "no").
    pub canonical_boolean: bool,

    /// Enforce canonical string normalization
    ///
    /// When `true`, strings are normalized (trimmed, case-folded if configured).
    pub canonical_string: bool,

    /// Minimum binary format version
    ///
    /// Reject binary data with version lower than this.
    /// - 0x04: v0.4 (no nested structures)
    /// - 0x05: v0.5 (nested structures supported)
    pub min_binary_version: u8,

    /// Validate field ordering in binary format
    ///
    /// When `true`, binary decoder validates that fields are in ascending FID order.
    pub validate_binary_ordering: bool,
}

impl StrictDeterministicConfig {
    /// Creates config for strict deterministic mode (v0.5-D)
    ///
    /// Maximum strictness for LLM drift prevention:
    /// - All validations enabled
    /// - Type hints required
    /// - Canonical forms enforced
    /// - Minimum binary version 0x05
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::profile::StrictDeterministicConfig;
    ///
    /// let config = StrictDeterministicConfig::strict();
    /// assert!(config.reject_unsorted_fields);
    /// assert!(config.require_type_hints);
    /// assert!(config.canonical_boolean);
    /// assert_eq!(config.min_binary_version, 0x05);
    /// ```
    pub fn strict() -> Self {
        Self {
            reject_unsorted_fields: true,
            require_type_hints: true,
            canonical_boolean: true,
            canonical_string: true,
            min_binary_version: 0x05, // v0.5+ supports nested
            validate_binary_ordering: true,
        }
    }

    /// Creates config for standard mode
    ///
    /// Balanced approach:
    /// - Canonical output but accepts non-canonical input
    /// - Type hints optional
    /// - Binary v0.4 compatible
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::profile::StrictDeterministicConfig;
    ///
    /// let config = StrictDeterministicConfig::standard();
    /// assert!(!config.reject_unsorted_fields);
    /// assert!(!config.require_type_hints);
    /// assert!(config.canonical_boolean); // Still normalize output
    /// ```
    pub fn standard() -> Self {
        Self {
            reject_unsorted_fields: false,
            require_type_hints: false,
            canonical_boolean: true, // Normalize in output
            canonical_string: false,
            min_binary_version: 0x04,
            validate_binary_ordering: false,
        }
    }

    /// Creates config for loose mode
    ///
    /// Maximum backward compatibility:
    /// - All validations disabled
    /// - Accepts any input
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_core::profile::StrictDeterministicConfig;
    ///
    /// let config = StrictDeterministicConfig::loose();
    /// assert!(!config.reject_unsorted_fields);
    /// assert!(!config.require_type_hints);
    /// assert!(!config.canonical_boolean);
    /// ```
    pub fn loose() -> Self {
        Self {
            reject_unsorted_fields: false,
            require_type_hints: false,
            canonical_boolean: false,
            canonical_string: false,
            min_binary_version: 0x04,
            validate_binary_ordering: false,
        }
    }

    /// Returns whether this config enforces strict determinism
    pub fn is_strict(&self) -> bool {
        self.reject_unsorted_fields
            && self.require_type_hints
            && self.canonical_boolean
            && self.validate_binary_ordering
    }

    /// Returns the recommended profile for this config
    pub fn profile(&self) -> LnmpProfile {
        if self.is_strict() {
            LnmpProfile::Strict
        } else if self.reject_unsorted_fields || self.require_type_hints {
            LnmpProfile::Standard
        } else {
            LnmpProfile::Loose
        }
    }
}

impl Default for StrictDeterministicConfig {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_as_str() {
        assert_eq!(LnmpProfile::Loose.as_str(), "loose");
        assert_eq!(LnmpProfile::Standard.as_str(), "standard");
        assert_eq!(LnmpProfile::Strict.as_str(), "strict");
    }

    #[test]
    fn test_profile_parse() {
        assert_eq!(LnmpProfile::parse("loose"), Some(LnmpProfile::Loose));
        assert_eq!(LnmpProfile::parse("standard"), Some(LnmpProfile::Standard));
        assert_eq!(LnmpProfile::parse("strict"), Some(LnmpProfile::Strict));
        assert_eq!(LnmpProfile::parse("STRICT"), Some(LnmpProfile::Strict)); // Case insensitive
        assert_eq!(LnmpProfile::parse("invalid"), None);
    }

    #[test]
    fn test_profile_default() {
        assert_eq!(LnmpProfile::default(), LnmpProfile::Standard);
    }

    #[test]
    fn test_profile_config() {
        let loose_cfg = LnmpProfile::Loose.config();
        assert!(!loose_cfg.reject_unsorted_fields);
        assert!(!loose_cfg.require_type_hints);

        let standard_cfg = LnmpProfile::Standard.config();
        assert!(!standard_cfg.reject_unsorted_fields);
        assert!(!standard_cfg.require_type_hints);
        assert!(standard_cfg.canonical_boolean);

        let strict_cfg = LnmpProfile::Strict.config();
        assert!(strict_cfg.reject_unsorted_fields);
        assert!(strict_cfg.require_type_hints);
        assert!(strict_cfg.canonical_boolean);
    }

    #[test]
    fn test_strict_config() {
        let config = StrictDeterministicConfig::strict();
        assert!(config.reject_unsorted_fields);
        assert!(config.require_type_hints);
        assert!(config.canonical_boolean);
        assert!(config.canonical_string);
        assert_eq!(config.min_binary_version, 0x05);
        assert!(config.validate_binary_ordering);
        assert!(config.is_strict());
        assert_eq!(config.profile(), LnmpProfile::Strict);
    }

    #[test]
    fn test_standard_config() {
        let config = StrictDeterministicConfig::standard();
        assert!(!config.reject_unsorted_fields);
        assert!(!config.require_type_hints);
        assert!(config.canonical_boolean);
        assert!(!config.canonical_string);
        assert_eq!(config.min_binary_version, 0x04);
        assert!(!config.is_strict());
    }

    #[test]
    fn test_loose_config() {
        let config = StrictDeterministicConfig::loose();
        assert!(!config.reject_unsorted_fields);
        assert!(!config.require_type_hints);
        assert!(!config.canonical_boolean);
        assert!(!config.canonical_string);
        assert!(!config.is_strict());
        assert_eq!(config.profile(), LnmpProfile::Loose);
    }

    #[test]
    fn test_config_default() {
        let config = StrictDeterministicConfig::default();
        assert_eq!(config, StrictDeterministicConfig::standard());
    }

    #[test]
    fn test_is_strict() {
        let mut config = StrictDeterministicConfig::strict();
        assert!(config.is_strict());

        config.require_type_hints = false;
        assert!(!config.is_strict());
    }
}
