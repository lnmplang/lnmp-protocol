use lnmp_core::types::FieldId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Semantic dictionary that maps field IDs to human-readable names
/// and provides equivalence mappings for semantic normalization
#[derive(Debug, Clone, Default)]
pub struct SemanticDictionary {
    field_names: HashMap<FieldId, String>,
    equivalences: HashMap<FieldId, HashMap<String, String>>,
}

/// Internal structure for deserializing YAML dictionary files
#[derive(Debug, Deserialize, Serialize)]
struct DictionaryFile {
    fields: HashMap<u16, FieldDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FieldDefinition {
    name: String,
    #[serde(rename = "type")]
    field_type: Option<String>,
    #[serde(default)]
    equivalences: HashMap<String, String>,
}

impl SemanticDictionary {
    /// Creates a new empty semantic dictionary
    pub fn new() -> Self {
        Self {
            field_names: HashMap::new(),
            equivalences: HashMap::new(),
        }
    }

    /// Loads a semantic dictionary from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML dictionary file
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the loaded dictionary or an error
    ///
    /// # Example YAML Format
    ///
    /// ```yaml
    /// fields:
    ///   12:
    ///     name: user_id
    ///     type: integer
    ///   7:
    ///     name: is_active
    ///     type: boolean
    ///     equivalences:
    ///       yes: "1"
    ///       true: "1"
    ///       no: "0"
    ///       false: "0"
    ///   23:
    ///     name: roles
    ///     type: string_array
    ///     equivalences:
    ///       admin: administrator
    ///       dev: developer
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DictionaryError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| DictionaryError::IoError(e.to_string()))?;

        let dict_file: DictionaryFile = serde_yaml::from_str(&content)
            .map_err(|e| DictionaryError::ParseError(e.to_string()))?;

        let mut dictionary = Self::new();

        for (fid, field_def) in dict_file.fields {
            dictionary.field_names.insert(fid, field_def.name);

            if !field_def.equivalences.is_empty() {
                dictionary
                    .equivalences
                    .insert(fid, field_def.equivalences);
            }
        }

        Ok(dictionary)
    }

    /// Gets the human-readable name for a field ID
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(&str)` with the field name if found, or `None` if not found
    pub fn get_field_name(&self, fid: FieldId) -> Option<&str> {
        self.field_names.get(&fid).map(|s| s.as_str())
    }

    /// Gets the canonical equivalent value for a field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID
    /// * `value` - The value to map to its canonical form
    ///
    /// # Returns
    ///
    /// Returns `Some(&str)` with the canonical value if a mapping exists,
    /// or `None` if no mapping is defined
    pub fn get_equivalence(&self, fid: FieldId, value: &str) -> Option<&str> {
        self.equivalences
            .get(&fid)
            .and_then(|mappings| mappings.get(value))
            .map(|s| s.as_str())
    }

    /// Adds a field name mapping
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID
    /// * `name` - The human-readable name for the field
    pub fn add_field_name(&mut self, fid: FieldId, name: String) {
        self.field_names.insert(fid, name);
    }

    /// Adds an equivalence mapping for a field
    ///
    /// # Arguments
    ///
    /// * `fid` - The field ID
    /// * `from` - The value to map from
    /// * `to` - The canonical value to map to
    pub fn add_equivalence(&mut self, fid: FieldId, from: String, to: String) {
        self.equivalences
            .entry(fid)
            .or_default()
            .insert(from, to);
    }

    /// Returns the number of fields defined in the dictionary
    pub fn field_count(&self) -> usize {
        self.field_names.len()
    }

    /// Returns the number of fields with equivalence mappings
    pub fn equivalence_count(&self) -> usize {
        self.equivalences.len()
    }
}

/// Errors that can occur when working with semantic dictionaries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictionaryError {
    /// I/O error reading the dictionary file
    IoError(String),
    /// Error parsing the YAML content
    ParseError(String),
}

impl std::fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictionaryError::IoError(msg) => write!(f, "I/O error: {}", msg),
            DictionaryError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for DictionaryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_dictionary() {
        let dict = SemanticDictionary::new();
        assert_eq!(dict.field_count(), 0);
        assert_eq!(dict.equivalence_count(), 0);
    }

    #[test]
    fn test_add_field_name() {
        let mut dict = SemanticDictionary::new();
        dict.add_field_name(12, "user_id".to_string());
        assert_eq!(dict.get_field_name(12), Some("user_id"));
        assert_eq!(dict.get_field_name(99), None);
    }

    #[test]
    fn test_add_equivalence() {
        let mut dict = SemanticDictionary::new();
        dict.add_equivalence(7, "yes".to_string(), "1".to_string());
        dict.add_equivalence(7, "true".to_string(), "1".to_string());
        
        assert_eq!(dict.get_equivalence(7, "yes"), Some("1"));
        assert_eq!(dict.get_equivalence(7, "true"), Some("1"));
        assert_eq!(dict.get_equivalence(7, "no"), None);
        assert_eq!(dict.get_equivalence(99, "yes"), None);
    }

    #[test]
    fn test_load_from_file() {
        let yaml_content = r#"
fields:
  12:
    name: user_id
    type: integer
  7:
    name: is_active
    type: boolean
    equivalences:
      yes: "1"
      true: "1"
      no: "0"
      false: "0"
  23:
    name: roles
    type: string_array
    equivalences:
      admin: administrator
      dev: developer
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let dict = SemanticDictionary::load_from_file(temp_file.path()).unwrap();

        // Check field names
        assert_eq!(dict.get_field_name(12), Some("user_id"));
        assert_eq!(dict.get_field_name(7), Some("is_active"));
        assert_eq!(dict.get_field_name(23), Some("roles"));
        assert_eq!(dict.field_count(), 3);

        // Check equivalences
        assert_eq!(dict.get_equivalence(7, "yes"), Some("1"));
        assert_eq!(dict.get_equivalence(7, "true"), Some("1"));
        assert_eq!(dict.get_equivalence(7, "no"), Some("0"));
        assert_eq!(dict.get_equivalence(7, "false"), Some("0"));
        assert_eq!(dict.get_equivalence(23, "admin"), Some("administrator"));
        assert_eq!(dict.get_equivalence(23, "dev"), Some("developer"));
        assert_eq!(dict.equivalence_count(), 2);
    }

    #[test]
    fn test_load_from_file_minimal() {
        let yaml_content = r#"
fields:
  1:
    name: simple_field
    type: string
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let dict = SemanticDictionary::load_from_file(temp_file.path()).unwrap();

        assert_eq!(dict.get_field_name(1), Some("simple_field"));
        assert_eq!(dict.field_count(), 1);
        assert_eq!(dict.equivalence_count(), 0);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = SemanticDictionary::load_from_file("/nonexistent/path/file.yaml");
        assert!(result.is_err());
        match result {
            Err(DictionaryError::IoError(_)) => {}
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_load_from_invalid_yaml() {
        let yaml_content = "invalid: yaml: content: [[[";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = SemanticDictionary::load_from_file(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(DictionaryError::ParseError(_)) => {}
            _ => panic!("Expected ParseError"),
        }
    }
}
