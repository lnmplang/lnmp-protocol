use lnmp_core::types::FieldId;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Semantic dictionary that maps field IDs to human-readable names
/// and provides equivalence mappings for semantic normalization
#[derive(Debug, Clone, Default)]
pub struct SemanticDictionary {
    field_names: HashMap<FieldId, String>,
    equivalences: HashMap<FieldId, HashMap<String, String>>,
    normalized_equivalences: HashMap<FieldId, HashMap<String, String>>,
}

fn validate_field_type(fid: u16, field_type: &str) -> Result<(), DictionaryError> {
    // Current accepted types; expand as needed.
    const ALLOWED: &[&str] = &[
        "integer",
        "float",
        "boolean",
        "string",
        "string_array",
        "record",
        "record_array",
    ];
    if ALLOWED.iter().any(|t| t.eq_ignore_ascii_case(field_type)) {
        Ok(())
    } else {
        Err(DictionaryError::InvalidFieldType {
            fid,
            field_type: field_type.to_string(),
        })
    }
}

fn scalar_to_string(val: &serde_yaml::Value) -> Option<String> {
    match val {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn detect_duplicate_field_ids(raw: &str) -> Option<FieldId> {
    let mut in_fields = false;
    let mut seen = std::collections::HashSet::new();
    let mut field_indent: Option<usize> = None;

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("fields:") {
            in_fields = true;
            continue;
        }
        if !in_fields {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if field_indent.is_none()
            && trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            field_indent = Some(indent);
        }
        if let Some(target_indent) = field_indent {
            if indent < target_indent {
                // out of fields block
                in_fields = false;
                continue;
            }
            if indent == target_indent {
                let mut parts = trimmed.splitn(2, ':');
                if let (Some(id_part), Some(_)) = (parts.next(), parts.next()) {
                    if let Ok(num) = id_part.trim().parse::<u16>() {
                        if !seen.insert(num) {
                            return Some(num);
                        }
                    }
                }
            }
        }
    }
    None
}

fn normalize_key(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

fn normalize_equivalences(map: &HashMap<String, String>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (k, v) in map {
        out.insert(normalize_key(k), v.clone());
    }
    out
}

impl SemanticDictionary {
    /// Creates a new empty semantic dictionary
    pub fn new() -> Self {
        Self {
            field_names: HashMap::new(),
            equivalences: HashMap::new(),
            normalized_equivalences: HashMap::new(),
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

        if let Some(dup) = detect_duplicate_field_ids(&content) {
            return Err(DictionaryError::DuplicateFieldId(dup));
        }

        let root: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| DictionaryError::ParseError(e.to_string()))?;

        let fields_mapping = root
            .get("fields")
            .and_then(|v| v.as_mapping())
            .ok_or_else(|| DictionaryError::ParseError("missing 'fields' map".to_string()))?;

        let mut dictionary = Self::new();
        let mut seen = std::collections::HashSet::new();

        for (key, value) in fields_mapping {
            let fid_num = key
                .as_u64()
                .ok_or_else(|| DictionaryError::ParseError("field id must be an integer".into()))?;
            if fid_num > u16::MAX as u64 {
                return Err(DictionaryError::ParseError(
                    "field id out of range (u16)".into(),
                ));
            }
            let fid = fid_num as u16;

            if !seen.insert(fid) {
                return Err(DictionaryError::DuplicateFieldId(fid));
            }

            let _field_map = value.as_mapping().ok_or_else(|| {
                DictionaryError::ParseError("field entry must be a mapping".into())
            })?;

            let name_val = value
                .get("name")
                .ok_or_else(|| DictionaryError::ParseError("field entry missing 'name'".into()))?;
            let name = scalar_to_string(name_val).ok_or_else(|| {
                DictionaryError::ParseError("field 'name' must be a scalar".into())
            })?;

            let field_type = value.get("type").and_then(scalar_to_string);
            if let Some(ref kind) = field_type {
                validate_field_type(fid, kind)?;
            }

            let mut equivalences_map: HashMap<String, String> = HashMap::new();
            if let Some(eq_val) = value.get("equivalences") {
                if let Some(mapping) = eq_val.as_mapping() {
                    for (k, v) in mapping {
                        let from = scalar_to_string(k).ok_or_else(|| {
                            DictionaryError::ParseError("equivalence key must be scalar".into())
                        })?;
                        let to = scalar_to_string(v).ok_or_else(|| {
                            DictionaryError::ParseError("equivalence value must be scalar".into())
                        })?;
                        equivalences_map.insert(from, to);
                    }
                } else {
                    return Err(DictionaryError::ParseError(
                        "equivalences must be a mapping".into(),
                    ));
                }
            }

            dictionary.field_names.insert(fid, name);

            if !equivalences_map.is_empty() {
                dictionary
                    .normalized_equivalences
                    .insert(fid, normalize_equivalences(&equivalences_map));
                dictionary.equivalences.insert(fid, equivalences_map);
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

    /// Normalized lookup (trim + lowercase) for lenient matching of user/LLM inputs.
    pub fn get_equivalence_normalized(&self, fid: FieldId, value: &str) -> Option<&str> {
        let key = normalize_key(value);
        self.normalized_equivalences
            .get(&fid)
            .and_then(|mappings| mappings.get(&key))
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
            .insert(from.clone(), to.clone());
        self.normalized_equivalences
            .entry(fid)
            .or_default()
            .insert(normalize_key(&from), to);
    }

    /// Returns the number of fields defined in the dictionary
    pub fn field_count(&self) -> usize {
        self.field_names.len()
    }

    /// Returns the number of fields with equivalence mappings
    pub fn equivalence_count(&self) -> usize {
        self.equivalences.len()
    }

    /// Iterator over (fid, field_name) pairs.
    pub fn field_name_entries(&self) -> impl Iterator<Item = (FieldId, &str)> {
        self.field_names
            .iter()
            .map(|(fid, name)| (*fid, name.as_str()))
    }
}

/// Errors that can occur when working with semantic dictionaries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictionaryError {
    /// I/O error reading the dictionary file
    IoError(String),
    /// Error parsing the YAML content
    ParseError(String),
    /// Duplicate field ID encountered
    DuplicateFieldId(FieldId),
    /// Invalid or unsupported field type in YAML
    InvalidFieldType { fid: FieldId, field_type: String },
}

impl std::fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictionaryError::IoError(msg) => write!(f, "I/O error: {}", msg),
            DictionaryError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DictionaryError::DuplicateFieldId(fid) => {
                write!(f, "Duplicate field ID in dictionary: {}", fid)
            }
            DictionaryError::InvalidFieldType { fid, field_type } => {
                write!(
                    f,
                    "Invalid field type '{}' for field ID {} (expected one of integer/float/boolean/string/string_array/record/record_array)",
                    field_type, fid
                )
            }
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

    #[test]
    fn test_load_duplicate_field_id_rejected() {
        let yaml_content = r#"
fields:
  1:
    name: first
  1:
    name: second
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = SemanticDictionary::load_from_file(temp_file.path());
        match result {
            Err(DictionaryError::DuplicateFieldId(fid)) => assert_eq!(fid, 1),
            _ => panic!("Expected DuplicateFieldId error"),
        }
    }

    #[test]
    fn test_load_invalid_field_type_rejected() {
        let yaml_content = r#"
fields:
  5:
    name: bad_type
    type: made_up
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = SemanticDictionary::load_from_file(temp_file.path());
        match result {
            Err(DictionaryError::InvalidFieldType { fid, .. }) => assert_eq!(fid, 5),
            _ => panic!("Expected InvalidFieldType error"),
        }
    }

    #[test]
    fn test_get_equivalence_normalized() {
        let mut dict = SemanticDictionary::new();
        dict.add_equivalence(7, "Yes".to_string(), "1".to_string());
        dict.add_equivalence(7, "No".to_string(), "0".to_string());

        assert_eq!(dict.get_equivalence_normalized(7, " yes "), Some("1"));
        assert_eq!(dict.get_equivalence_normalized(7, "NO"), Some("0"));
        assert_eq!(dict.get_equivalence_normalized(7, "maybe"), None);
    }
}
