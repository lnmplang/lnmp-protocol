//! Semantic equivalence mapping system.
//!
//! This module provides equivalence mapping to recognize synonyms and related terms
//! as semantically equivalent. Mappings are field-specific, allowing different
//! equivalence rules for different fields.
//!
//! # Examples
//!
//! ```
//! use lnmp_codec::EquivalenceMapper;
//!
//! let mut mapper = EquivalenceMapper::new();
//!
//! // Add mapping for field 7 (is_active)
//! mapper.add_mapping(7, "yes".to_string(), "1".to_string());
//! mapper.add_mapping(7, "true".to_string(), "1".to_string());
//! mapper.add_mapping(7, "no".to_string(), "0".to_string());
//! mapper.add_mapping(7, "false".to_string(), "0".to_string());
//!
//! // Map values to canonical form
//! assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
//! assert_eq!(mapper.map(7, "true"), Some("1".to_string()));
//! assert_eq!(mapper.map(7, "no"), Some("0".to_string()));
//! assert_eq!(mapper.map(7, "unmapped"), None);
//! ```

use lnmp_core::FieldId;
use std::collections::HashMap;

/// Equivalence mapper for semantic synonym mapping
///
/// Maps field values to their canonical forms based on field-specific
/// equivalence rules. This enables recognition of synonyms like
/// "admin" → "administrator" or "yes" → "1".
#[derive(Debug, Clone)]
pub struct EquivalenceMapper {
    /// Field-specific mappings: FieldId → (from_value → to_value)
    mappings: HashMap<FieldId, HashMap<String, String>>,
}

impl EquivalenceMapper {
    /// Creates a new empty equivalence mapper
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Creates a mapper with default boolean equivalences
    ///
    /// Provides common boolean synonym mappings that can be applied
    /// to any field by calling `apply_default_bool_mappings(fid)`.
    pub fn with_defaults() -> Self {
        Self::new()
    }

    /// Adds a custom mapping for a specific field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to add the mapping for
    /// * `from` - The source value to map from
    /// * `to` - The canonical value to map to
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::EquivalenceMapper;
    ///
    /// let mut mapper = EquivalenceMapper::new();
    /// mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
    /// mapper.add_mapping(12, "dev".to_string(), "developer".to_string());
    ///
    /// assert_eq!(mapper.map(12, "admin"), Some("administrator".to_string()));
    /// assert_eq!(mapper.map(12, "dev"), Some("developer".to_string()));
    /// ```
    pub fn add_mapping(&mut self, fid: FieldId, from: String, to: String) {
        self.mappings
            .entry(fid)
            .or_default()
            .insert(from, to);
    }

    /// Adds multiple mappings for a specific field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to add mappings for
    /// * `mappings` - Iterator of (from, to) value pairs
    pub fn add_mappings<I>(&mut self, fid: FieldId, mappings: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let field_mappings = self.mappings.entry(fid).or_default();
        for (from, to) in mappings {
            field_mappings.insert(from, to);
        }
    }

    /// Applies default boolean equivalence mappings to a field
    ///
    /// Maps common boolean representations to "1" (true) or "0" (false):
    /// - "yes", "true" → "1"
    /// - "no", "false" → "0"
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::EquivalenceMapper;
    ///
    /// let mut mapper = EquivalenceMapper::new();
    /// mapper.apply_default_bool_mappings(7);
    ///
    /// assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
    /// assert_eq!(mapper.map(7, "true"), Some("1".to_string()));
    /// assert_eq!(mapper.map(7, "no"), Some("0".to_string()));
    /// assert_eq!(mapper.map(7, "false"), Some("0".to_string()));
    /// ```
    pub fn apply_default_bool_mappings(&mut self, fid: FieldId) {
        let bool_mappings = vec![
            ("yes".to_string(), "1".to_string()),
            ("Yes".to_string(), "1".to_string()),
            ("YES".to_string(), "1".to_string()),
            ("true".to_string(), "1".to_string()),
            ("True".to_string(), "1".to_string()),
            ("TRUE".to_string(), "1".to_string()),
            ("no".to_string(), "0".to_string()),
            ("No".to_string(), "0".to_string()),
            ("NO".to_string(), "0".to_string()),
            ("false".to_string(), "0".to_string()),
            ("False".to_string(), "0".to_string()),
            ("FALSE".to_string(), "0".to_string()),
        ];
        self.add_mappings(fid, bool_mappings);
    }

    /// Maps a value to its canonical form for a specific field
    ///
    /// Returns `Some(canonical_value)` if a mapping exists, or `None` if
    /// the value has no mapping for this field.
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to look up mappings for
    /// * `value` - The value to map
    ///
    /// # Examples
    ///
    /// ```
    /// use lnmp_codec::EquivalenceMapper;
    ///
    /// let mut mapper = EquivalenceMapper::new();
    /// mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
    ///
    /// assert_eq!(mapper.map(12, "admin"), Some("administrator".to_string()));
    /// assert_eq!(mapper.map(12, "user"), None);
    /// assert_eq!(mapper.map(99, "admin"), None); // Different field
    /// ```
    pub fn map(&self, fid: FieldId, value: &str) -> Option<String> {
        self.mappings
            .get(&fid)
            .and_then(|field_mappings| field_mappings.get(value))
            .cloned()
    }

    /// Checks if a mapping exists for a specific field and value
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to check
    /// * `value` - The value to check for a mapping
    pub fn has_mapping(&self, fid: FieldId, value: &str) -> bool {
        self.mappings
            .get(&fid)
            .map(|field_mappings| field_mappings.contains_key(value))
            .unwrap_or(false)
    }

    /// Returns the number of fields with mappings
    pub fn field_count(&self) -> usize {
        self.mappings.len()
    }

    /// Returns the number of mappings for a specific field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to count mappings for
    pub fn mapping_count(&self, fid: FieldId) -> usize {
        self.mappings
            .get(&fid)
            .map(|field_mappings| field_mappings.len())
            .unwrap_or(0)
    }

    /// Clears all mappings for a specific field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to clear mappings for
    pub fn clear_field(&mut self, fid: FieldId) {
        self.mappings.remove(&fid);
    }

    /// Clears all mappings
    pub fn clear(&mut self) {
        self.mappings.clear();
    }
}

impl Default for EquivalenceMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mapper_is_empty() {
        let mapper = EquivalenceMapper::new();
        assert_eq!(mapper.field_count(), 0);
    }

    #[test]
    fn test_add_single_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());

        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(mapper.map(12, "user"), None);
        assert_eq!(mapper.field_count(), 1);
        assert_eq!(mapper.mapping_count(12), 1);
    }

    #[test]
    fn test_add_multiple_mappings_same_field() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(12, "dev".to_string(), "developer".to_string());
        mapper.add_mapping(12, "qa".to_string(), "quality_assurance".to_string());

        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(mapper.map(12, "dev"), Some("developer".to_string()));
        assert_eq!(
            mapper.map(12, "qa"),
            Some("quality_assurance".to_string())
        );
        assert_eq!(mapper.field_count(), 1);
        assert_eq!(mapper.mapping_count(12), 3);
    }

    #[test]
    fn test_add_mappings_different_fields() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(7, "yes".to_string(), "1".to_string());

        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
        assert_eq!(mapper.map(12, "yes"), None); // Different field
        assert_eq!(mapper.map(7, "admin"), None); // Different field
        assert_eq!(mapper.field_count(), 2);
    }

    #[test]
    fn test_add_mappings_bulk() {
        let mut mapper = EquivalenceMapper::new();
        let mappings = vec![
            ("admin".to_string(), "administrator".to_string()),
            ("dev".to_string(), "developer".to_string()),
            ("qa".to_string(), "quality_assurance".to_string()),
        ];
        mapper.add_mappings(12, mappings);

        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(mapper.map(12, "dev"), Some("developer".to_string()));
        assert_eq!(
            mapper.map(12, "qa"),
            Some("quality_assurance".to_string())
        );
        assert_eq!(mapper.mapping_count(12), 3);
    }

    #[test]
    fn test_apply_default_bool_mappings() {
        let mut mapper = EquivalenceMapper::new();
        mapper.apply_default_bool_mappings(7);

        // Test true variants
        assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
        assert_eq!(mapper.map(7, "Yes"), Some("1".to_string()));
        assert_eq!(mapper.map(7, "YES"), Some("1".to_string()));
        assert_eq!(mapper.map(7, "true"), Some("1".to_string()));
        assert_eq!(mapper.map(7, "True"), Some("1".to_string()));
        assert_eq!(mapper.map(7, "TRUE"), Some("1".to_string()));

        // Test false variants
        assert_eq!(mapper.map(7, "no"), Some("0".to_string()));
        assert_eq!(mapper.map(7, "No"), Some("0".to_string()));
        assert_eq!(mapper.map(7, "NO"), Some("0".to_string()));
        assert_eq!(mapper.map(7, "false"), Some("0".to_string()));
        assert_eq!(mapper.map(7, "False"), Some("0".to_string()));
        assert_eq!(mapper.map(7, "FALSE"), Some("0".to_string()));

        // Test unmapped values
        assert_eq!(mapper.map(7, "maybe"), None);
        assert_eq!(mapper.map(7, "1"), None);
        assert_eq!(mapper.map(7, "0"), None);
    }

    #[test]
    fn test_has_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());

        assert!(mapper.has_mapping(12, "admin"));
        assert!(!mapper.has_mapping(12, "user"));
        assert!(!mapper.has_mapping(7, "admin"));
    }

    #[test]
    fn test_mapping_count() {
        let mut mapper = EquivalenceMapper::new();
        assert_eq!(mapper.mapping_count(12), 0);

        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        assert_eq!(mapper.mapping_count(12), 1);

        mapper.add_mapping(12, "dev".to_string(), "developer".to_string());
        assert_eq!(mapper.mapping_count(12), 2);

        mapper.add_mapping(7, "yes".to_string(), "1".to_string());
        assert_eq!(mapper.mapping_count(12), 2);
        assert_eq!(mapper.mapping_count(7), 1);
    }

    #[test]
    fn test_field_count() {
        let mut mapper = EquivalenceMapper::new();
        assert_eq!(mapper.field_count(), 0);

        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        assert_eq!(mapper.field_count(), 1);

        mapper.add_mapping(12, "dev".to_string(), "developer".to_string());
        assert_eq!(mapper.field_count(), 1); // Same field

        mapper.add_mapping(7, "yes".to_string(), "1".to_string());
        assert_eq!(mapper.field_count(), 2); // Different field
    }

    #[test]
    fn test_clear_field() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(12, "dev".to_string(), "developer".to_string());
        mapper.add_mapping(7, "yes".to_string(), "1".to_string());

        assert_eq!(mapper.field_count(), 2);
        assert_eq!(mapper.mapping_count(12), 2);

        mapper.clear_field(12);
        assert_eq!(mapper.field_count(), 1);
        assert_eq!(mapper.mapping_count(12), 0);
        assert_eq!(mapper.mapping_count(7), 1);
        assert_eq!(mapper.map(12, "admin"), None);
        assert_eq!(mapper.map(7, "yes"), Some("1".to_string()));
    }

    #[test]
    fn test_clear_all() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(7, "yes".to_string(), "1".to_string());

        assert_eq!(mapper.field_count(), 2);

        mapper.clear();
        assert_eq!(mapper.field_count(), 0);
        assert_eq!(mapper.map(12, "admin"), None);
        assert_eq!(mapper.map(7, "yes"), None);
    }

    #[test]
    fn test_overwrite_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );

        // Overwrite with new mapping
        mapper.add_mapping(12, "admin".to_string(), "superuser".to_string());
        assert_eq!(mapper.map(12, "admin"), Some("superuser".to_string()));
        assert_eq!(mapper.mapping_count(12), 1);
    }

    #[test]
    fn test_case_sensitive_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(12, "Admin".to_string(), "Administrator".to_string());

        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(
            mapper.map(12, "Admin"),
            Some("Administrator".to_string())
        );
        assert_eq!(mapper.map(12, "ADMIN"), None);
    }

    #[test]
    fn test_empty_string_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "".to_string(), "empty".to_string());

        assert_eq!(mapper.map(12, ""), Some("empty".to_string()));
    }

    #[test]
    fn test_with_defaults() {
        let mapper = EquivalenceMapper::with_defaults();
        assert_eq!(mapper.field_count(), 0);
    }

    #[test]
    fn test_default_trait() {
        let mapper = EquivalenceMapper::default();
        assert_eq!(mapper.field_count(), 0);
    }

    #[test]
    fn test_clone() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());

        let cloned = mapper.clone();
        assert_eq!(
            cloned.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(cloned.field_count(), 1);
    }

    #[test]
    fn test_multiple_fields_with_same_mapping_value() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "admin".to_string(), "administrator".to_string());
        mapper.add_mapping(15, "admin".to_string(), "admin_user".to_string());

        // Same source value, different canonical values per field
        assert_eq!(
            mapper.map(12, "admin"),
            Some("administrator".to_string())
        );
        assert_eq!(mapper.map(15, "admin"), Some("admin_user".to_string()));
    }

    #[test]
    fn test_unicode_mapping() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(12, "café".to_string(), "coffee_shop".to_string());
        mapper.add_mapping(12, "日本".to_string(), "japan".to_string());

        assert_eq!(mapper.map(12, "café"), Some("coffee_shop".to_string()));
        assert_eq!(mapper.map(12, "日本"), Some("japan".to_string()));
    }
}
