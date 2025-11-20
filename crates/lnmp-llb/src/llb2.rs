//! LNMP LLM Optimization Layer v2 (LLB2)
//! This module provides advanced optimization strategies for LLM contexts in LNMP v0.5,
//! including format conversions, nested structure flattening, semantic hints, and
//! collision-safe ID generation.

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder, BinaryError};
use lnmp_codec::{LnmpError, Parser};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use std::collections::HashMap;

/// Configuration for LLB2 optimization features
#[derive(Debug, Clone, Default)]
pub struct LlbConfig {
    /// Enable nested structure flattening using dot notation
    pub enable_flattening: bool,
    /// Enable semantic hint embedding in field representations
    pub enable_semantic_hints: bool,
    /// Generate collision-safe short IDs for field names
    pub collision_safe_ids: bool,
}

impl LlbConfig {
    /// Creates a new LlbConfig with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable nested structure flattening
    pub fn with_flattening(mut self, enable: bool) -> Self {
        self.enable_flattening = enable;
        self
    }

    /// Enable or disable semantic hint embedding
    pub fn with_semantic_hints(mut self, enable: bool) -> Self {
        self.enable_semantic_hints = enable;
        self
    }

    /// Enable or disable collision-safe ID generation
    pub fn with_collision_safe_ids(mut self, enable: bool) -> Self {
        self.collision_safe_ids = enable;
        self
    }
}

// Default is provided by the derive attribute above.

/// Error types for LLB2 operations
#[derive(Debug, Clone, PartialEq)]
pub enum LlbError {
    /// Binary encoding/decoding error
    BinaryError(String),
    /// Text parsing error
    ParseError(String),
    /// Invalid flattening operation
    FlatteningError(String),
    /// Invalid field structure
    InvalidStructure(String),
    /// Collision detected in ID generation
    IdCollision { id: String, names: Vec<String> },
}

impl std::fmt::Display for LlbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlbError::BinaryError(msg) => write!(f, "Binary error: {}", msg),
            LlbError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlbError::FlatteningError(msg) => write!(f, "Flattening error: {}", msg),
            LlbError::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            LlbError::IdCollision { id, names } => {
                write!(f, "ID collision: '{}' maps to {:?}", id, names)
            }
        }
    }
}

impl std::error::Error for LlbError {}

impl From<BinaryError> for LlbError {
    fn from(err: BinaryError) -> Self {
        LlbError::BinaryError(err.to_string())
    }
}

impl From<LnmpError> for LlbError {
    fn from(err: LnmpError) -> Self {
        LlbError::ParseError(err.to_string())
    }
}

/// LLB2 converter for format conversions and optimizations
pub struct LlbConverter {
    config: LlbConfig,
}

impl LlbConverter {
    /// Creates a new LlbConverter with the given configuration
    pub fn new(config: LlbConfig) -> Self {
        Self { config }
    }

    /// Creates a new LlbConverter with default configuration
    /// This is available through the `Default` trait implementation.
    /// Converts binary format to ShortForm text representation
    ///
    /// ShortForm omits the 'F' prefix for extreme token reduction:
    /// Binary → Record → "12=14532;7=1;23=[admin,dev]"
    pub fn binary_to_shortform(&self, binary: &[u8]) -> Result<String, LlbError> {
        let decoder = BinaryDecoder::new();
        let record = decoder.decode(binary)?;
        Ok(self.record_to_shortform(&record))
    }

    /// Converts ShortForm text to binary format
    ///
    /// ShortForm → Record → Binary
    pub fn shortform_to_binary(&self, shortform: &str) -> Result<Vec<u8>, LlbError> {
        let record = self.shortform_to_record(shortform)?;
        let encoder = BinaryEncoder::new();
        Ok(encoder.encode(&record)?)
    }

    /// Converts binary format to FullText (canonical LNMP text) representation
    ///
    /// Binary → Record → "F12=14532\nF7=1\nF23=[admin,dev]"
    pub fn binary_to_fulltext(&self, binary: &[u8]) -> Result<String, LlbError> {
        let decoder = BinaryDecoder::new();
        Ok(decoder.decode_to_text(binary)?)
    }

    /// Converts FullText (canonical LNMP text) to binary format
    ///
    /// FullText → Record → Binary
    pub fn fulltext_to_binary(&self, fulltext: &str) -> Result<Vec<u8>, LlbError> {
        let encoder = BinaryEncoder::new();
        Ok(encoder.encode_text(fulltext)?)
    }

    /// Converts a record to ShortForm representation
    fn record_to_shortform(&self, record: &LnmpRecord) -> String {
        let fields: Vec<String> = record
            .sorted_fields()
            .iter()
            .map(|field| self.field_to_shortform(field))
            .collect();
        fields.join(";")
    }

    /// Converts a field to ShortForm representation (without 'F' prefix)
    fn field_to_shortform(&self, field: &LnmpField) -> String {
        format!("{}={}", field.fid, self.value_to_shortform(&field.value))
    }

    /// Converts a value to ShortForm representation
    fn value_to_shortform(&self, value: &LnmpValue) -> String {
        match value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Float(f) => f.to_string(),
            LnmpValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            LnmpValue::String(s) => self.encode_string_shortform(s),
            LnmpValue::StringArray(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|s| self.encode_string_shortform(s))
                    .collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::NestedRecord(record) => {
                let fields: Vec<String> = record
                    .sorted_fields()
                    .iter()
                    .map(|field| self.field_to_shortform(field))
                    .collect();
                format!("{{{}}}", fields.join(";"))
            }
            LnmpValue::NestedArray(records) => {
                let encoded: Vec<String> = records
                    .iter()
                    .map(|r| {
                        let fields: Vec<String> = r
                            .sorted_fields()
                            .iter()
                            .map(|field| self.field_to_shortform(field))
                            .collect();
                        format!("{{{}}}", fields.join(";"))
                    })
                    .collect();
                format!("[{}]", encoded.join(","))
            }
            LnmpValue::Embedding(_) => String::new(),
            LnmpValue::EmbeddingDelta(_) => String::new(),
        }
    }

    /// Encodes a string for ShortForm (adds quotes if needed)
    fn encode_string_shortform(&self, s: &str) -> String {
        if self.needs_quoting_shortform(s) {
            format!("\"{}\"", self.escape_string(s))
        } else {
            s.to_string()
        }
    }

    /// Checks if a string needs quoting in ShortForm
    fn needs_quoting_shortform(&self, s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        for ch in s.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
                return true;
            }
        }
        false
    }

    /// Escapes special characters in strings
    fn escape_string(&self, s: &str) -> String {
        let mut result = String::new();
        for ch in s.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(ch),
            }
        }
        result
    }

    /// Converts ShortForm text to a record
    fn shortform_to_record(&self, shortform: &str) -> Result<LnmpRecord, LlbError> {
        // Convert ShortForm to FullText by adding 'F' prefixes
        let fulltext = self.shortform_to_fulltext(shortform);

        // Parse using standard LNMP parser
        let mut parser = Parser::new(&fulltext)?;
        Ok(parser.parse_record()?)
    }

    /// Converts ShortForm to FullText by adding 'F' prefixes
    fn shortform_to_fulltext(&self, shortform: &str) -> String {
        // Context-aware scan:
        // - Only prefix 'F' when we are at the start of a field (not inside quotes, not after '=')
        // - Field starts after '{', ';', '\n' or at beginning of string.
        let mut out = String::with_capacity(shortform.len() + 8);
        let mut in_string = false;
        let mut escape = false;
        let mut at_field_start = true;

        for ch in shortform.chars() {
            if in_string {
                out.push(ch);
                if escape {
                    escape = false;
                } else if ch == '\\' {
                    escape = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => {
                    in_string = true;
                    out.push(ch);
                    at_field_start = false;
                }
                ';' | '\n' => {
                    out.push(ch);
                    at_field_start = true;
                }
                '{' => {
                    out.push(ch);
                    at_field_start = true;
                }
                '=' | '[' | ']' | '}' | ',' => {
                    out.push(ch);
                    at_field_start = false;
                }
                digit if at_field_start && digit.is_ascii_digit() => {
                    out.push('F');
                    out.push(digit);
                    at_field_start = false;
                }
                other => {
                    out.push(other);
                    at_field_start = false;
                }
            }
        }

        out
    }

    /// Flattens nested structures using dot notation
    ///
    /// Converts nested records like:
    /// F10:r={F1=42,F2="test"}
    ///
    /// To flattened form:
    /// F10.1=42
    /// F10.2="test"
    ///
    /// This preserves semantic relationships while making the structure
    /// more suitable for LLM consumption.
    pub fn flatten_nested(&self, record: &LnmpRecord) -> Result<LnmpRecord, LlbError> {
        if !self.config.enable_flattening {
            // If flattening is disabled, return the record as-is
            return Ok(record.clone());
        }

        let mut flattened = LnmpRecord::new();
        let mut seen_paths = std::collections::HashMap::new();

        for field in record.fields() {
            self.flatten_field(
                &mut flattened,
                field.fid,
                &field.value,
                vec![],
                &mut seen_paths,
            )?;
        }

        Ok(flattened)
    }

    /// Recursively flattens a field value
    fn flatten_field(
        &self,
        target: &mut LnmpRecord,
        base_fid: u16,
        value: &LnmpValue,
        path: Vec<u16>,
        seen_paths: &mut std::collections::HashMap<u16, Vec<u16>>,
    ) -> Result<(), LlbError> {
        match value {
            // Primitive values are added directly
            LnmpValue::Int(_)
            | LnmpValue::Float(_)
            | LnmpValue::Bool(_)
            | LnmpValue::String(_)
            | LnmpValue::StringArray(_) => {
                let fid = if path.is_empty() {
                    base_fid
                } else {
                    // Encode path deterministically, error if collision occurs.
                    self.encode_path_to_fid(base_fid, &path, seen_paths)?
                };

                target.add_field(LnmpField {
                    fid,
                    value: value.clone(),
                });
            }

            // Nested records are flattened recursively
            LnmpValue::NestedRecord(nested) => {
                for nested_field in nested.fields() {
                    let mut new_path = path.clone();
                    new_path.push(nested_field.fid);
                    self.flatten_field(
                        target,
                        base_fid,
                        &nested_field.value,
                        new_path,
                        seen_paths,
                    )?;
                }
            }

            // Nested arrays are flattened with index encoding
            LnmpValue::NestedArray(records) => {
                for (idx, record) in records.iter().enumerate() {
                    for nested_field in record.fields() {
                        let mut new_path = path.clone();
                        new_path.push((idx as u16) + 1); // 1-indexed
                        new_path.push(nested_field.fid);
                        self.flatten_field(
                            target,
                            base_fid,
                            &nested_field.value,
                            new_path,
                            seen_paths,
                        )?;
                    }
                }
            }
            LnmpValue::Embedding(_) => {
                // Embeddings are not flattened
            }
            LnmpValue::EmbeddingDelta(_) => {
                // Deltas are not flattened
            }
        }

        Ok(())
    }

    /// Encodes a path into a FID via a stable hash, detecting collisions.
    fn encode_path_to_fid(
        &self,
        base_fid: u16,
        path: &[u16],
        seen_paths: &mut std::collections::HashMap<u16, Vec<u16>>,
    ) -> Result<u16, LlbError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if path.is_empty() {
            return Ok(base_fid);
        }

        let mut hasher = DefaultHasher::new();
        base_fid.hash(&mut hasher);
        path.hash(&mut hasher);
        let encoded = (hasher.finish() & 0xFFFF) as u16;

        if let Some(existing) = seen_paths.get(&encoded) {
            if existing != path {
                return Err(LlbError::FlatteningError(format!(
                    "FID collision while flattening: base {} paths {:?} vs {:?}",
                    base_fid, existing, path
                )));
            }
        } else {
            seen_paths.insert(encoded, path.to_vec());
        }

        Ok(encoded)
    }

    /// Unflattens a flattened record back to its original nested structure
    ///
    /// This is the inverse operation of flatten_nested().
    /// Note: This is a best-effort reconstruction and may not perfectly
    /// restore the original structure if FID encoding is ambiguous.
    pub fn unflatten(&self, flat: &LnmpRecord) -> Result<LnmpRecord, LlbError> {
        // Best-effort reverse: group by base_fid and treat higher bits as path hash.
        // This is lossy but preserves non-flattened fields.
        let mut unflattened = LnmpRecord::new();

        for field in flat.fields() {
            // Use high byte as heuristic for base fid when flattening is enabled.
            if field.fid >= 256 {
                let base_fid = field.fid >> 8;
                let leaf_fid = field.fid & 0x00FF;

                let mut nested = LnmpRecord::new();
                nested.add_field(LnmpField {
                    fid: leaf_fid,
                    value: field.value.clone(),
                });

                unflattened.add_field(LnmpField {
                    fid: base_fid,
                    value: LnmpValue::NestedRecord(Box::new(nested)),
                });
            } else {
                unflattened.add_field(field.clone());
            }
        }

        Ok(unflattened)
    }

    /// Adds semantic hints to field representations
    ///
    /// Embeds human-readable semantic information in field representations
    /// to help LLMs understand the meaning of fields.
    ///
    /// Format: "F7=1 # is_admin@sem"
    ///
    /// # Arguments
    ///
    /// * `record` - The record to add hints to
    /// * `hints` - Map of FID to semantic hint string
    ///
    /// # Returns
    ///
    /// A string representation with embedded semantic hints
    pub fn add_semantic_hints(&self, record: &LnmpRecord, hints: &HashMap<u16, String>) -> String {
        if !self.config.enable_semantic_hints {
            // If semantic hints are disabled, return basic format
            let encoder = lnmp_codec::Encoder::new();
            return encoder.encode(record);
        }

        let mut lines = Vec::new();

        for field in record.sorted_fields() {
            let field_str = self.format_field_with_hint(field.fid, &field.value, hints);
            lines.push(field_str);
        }

        lines.join("\n")
    }

    /// Formats a single field with optional semantic hint
    fn format_field_with_hint(
        &self,
        fid: u16,
        value: &LnmpValue,
        hints: &HashMap<u16, String>,
    ) -> String {
        let base = format!("F{}={}", fid, self.format_value_for_hint(value));

        if let Some(hint) = hints.get(&fid) {
            format!("{} # {}@sem", base, hint)
        } else {
            base
        }
    }

    /// Formats a value for semantic hint display
    fn format_value_for_hint(&self, value: &LnmpValue) -> String {
        match value {
            LnmpValue::Int(i) => i.to_string(),
            LnmpValue::Float(f) => f.to_string(),
            LnmpValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            LnmpValue::String(s) => {
                if self.needs_quoting_shortform(s) {
                    format!("\"{}\"", self.escape_string(s))
                } else {
                    s.clone()
                }
            }
            LnmpValue::StringArray(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|s| {
                        if self.needs_quoting_shortform(s) {
                            format!("\"{}\"", self.escape_string(s))
                        } else {
                            s.clone()
                        }
                    })
                    .collect();
                format!("[{}]", items.join(","))
            }
            LnmpValue::NestedRecord(record) => {
                let fields: Vec<String> = record
                    .sorted_fields()
                    .iter()
                    .map(|f| format!("F{}={}", f.fid, self.format_value_for_hint(&f.value)))
                    .collect();
                format!("{{{}}}", fields.join(";"))
            }
            LnmpValue::NestedArray(records) => {
                let encoded: Vec<String> = records
                    .iter()
                    .map(|r| {
                        let fields: Vec<String> = r
                            .sorted_fields()
                            .iter()
                            .map(|f| format!("F{}={}", f.fid, self.format_value_for_hint(&f.value)))
                            .collect();
                        format!("{{{}}}", fields.join(";"))
                    })
                    .collect();
                format!("[{}]", encoded.join(","))
            }
            LnmpValue::EmbeddingDelta(_) => String::new(),
            LnmpValue::Embedding(_) => String::new(),
        }
    }

    /// Generates collision-safe short IDs for field names
    ///
    /// Creates unique, short identifiers for field names that are:
    /// - Collision-free (no two different names map to the same ID)
    /// - Deterministic (same name always produces same ID)
    /// - Short (optimized for token efficiency)
    ///
    /// # Arguments
    ///
    /// * `field_names` - List of field names to generate IDs for
    ///
    /// # Returns
    ///
    /// A HashMap mapping field names to their short IDs
    ///
    /// # Errors
    ///
    /// Returns `LlbError::IdCollision` if a collision is detected
    pub fn generate_short_ids(
        &self,
        field_names: &[String],
    ) -> Result<HashMap<String, String>, LlbError> {
        if self.config.collision_safe_ids {
            // Use the safe version that handles collisions
            return Ok(self.generate_short_ids_safe(field_names));
        }

        let mut id_map = HashMap::new();
        let mut reverse_map: HashMap<String, Vec<String>> = HashMap::new();

        for name in field_names {
            let short_id = self.compute_short_id(name);

            // Check for collisions
            reverse_map
                .entry(short_id.clone())
                .or_default()
                .push(name.clone());

            id_map.insert(name.clone(), short_id);
        }

        // Detect collisions
        for (id, names) in reverse_map.iter() {
            if names.len() > 1 {
                return Err(LlbError::IdCollision {
                    id: id.clone(),
                    names: names.clone(),
                });
            }
        }

        Ok(id_map)
    }

    /// Computes a short ID for a field name using a deterministic algorithm
    ///
    /// Strategy:
    /// 1. Try first letter + length (e.g., "user_id" -> "u7")
    /// 2. If that's too common, use first 2-3 letters
    /// 3. Add numeric suffix if needed for uniqueness
    fn compute_short_id(&self, name: &str) -> String {
        if name.is_empty() {
            return "x".to_string();
        }

        // Strategy 1: First letter + length
        let first_char = name.chars().next().unwrap().to_lowercase().to_string();
        let len = name.len();

        // For very short names, just use the name itself
        if len <= 3 {
            return name.to_lowercase();
        }

        // For longer names, use first letter + length
        format!("{}{}", first_char, len)
    }

    /// Generates collision-safe short IDs with disambiguation
    ///
    /// This version handles collisions by adding suffixes to make IDs unique.
    pub fn generate_short_ids_safe(&self, field_names: &[String]) -> HashMap<String, String> {
        let mut id_map = HashMap::new();
        let mut id_counts: HashMap<String, usize> = HashMap::new();

        for name in field_names {
            let base_id = self.compute_short_id(name);

            // Check if this ID has been used before
            let count = id_counts.entry(base_id.clone()).or_insert(0);

            let final_id = if *count == 0 {
                base_id.clone()
            } else {
                // Add numeric suffix for disambiguation
                format!("{}{}", base_id, count)
            };

            *count += 1;
            id_map.insert(name.clone(), final_id);
        }

        id_map
    }
}

impl Default for LlbConverter {
    fn default() -> Self {
        Self::new(LlbConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llb_config_default() {
        let config = LlbConfig::default();
        assert!(!config.enable_flattening);
        assert!(!config.enable_semantic_hints);
        assert!(!config.collision_safe_ids);
    }

    #[test]
    fn test_llb_config_new() {
        let config = LlbConfig::new();
        assert!(!config.enable_flattening);
        assert!(!config.enable_semantic_hints);
        assert!(!config.collision_safe_ids);
    }

    #[test]
    fn test_llb_config_with_flattening() {
        let config = LlbConfig::new().with_flattening(true);
        assert!(config.enable_flattening);
        assert!(!config.enable_semantic_hints);
        assert!(!config.collision_safe_ids);
    }

    #[test]
    fn test_llb_config_with_semantic_hints() {
        let config = LlbConfig::new().with_semantic_hints(true);
        assert!(!config.enable_flattening);
        assert!(config.enable_semantic_hints);
        assert!(!config.collision_safe_ids);
    }

    #[test]
    fn test_llb_config_with_collision_safe_ids() {
        let config = LlbConfig::new().with_collision_safe_ids(true);
        assert!(!config.enable_flattening);
        assert!(!config.enable_semantic_hints);
        assert!(config.collision_safe_ids);
    }

    #[test]
    fn test_llb_config_builder_chain() {
        let config = LlbConfig::new()
            .with_flattening(true)
            .with_semantic_hints(true)
            .with_collision_safe_ids(true);

        assert!(config.enable_flattening);
        assert!(config.enable_semantic_hints);
        assert!(config.collision_safe_ids);
    }

    #[test]
    fn test_llb_error_display() {
        let err = LlbError::BinaryError("test error".to_string());
        assert_eq!(err.to_string(), "Binary error: test error");

        let err = LlbError::ParseError("parse failed".to_string());
        assert_eq!(err.to_string(), "Parse error: parse failed");

        let err = LlbError::FlatteningError("flatten failed".to_string());
        assert_eq!(err.to_string(), "Flattening error: flatten failed");

        let err = LlbError::InvalidStructure("bad structure".to_string());
        assert_eq!(err.to_string(), "Invalid structure: bad structure");

        let err = LlbError::IdCollision {
            id: "F1".to_string(),
            names: vec!["field1".to_string(), "field_one".to_string()],
        };
        assert!(err.to_string().contains("ID collision"));
        assert!(err.to_string().contains("F1"));
    }

    #[test]
    fn test_binary_to_shortform_simple() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();
        let shortform = converter.binary_to_shortform(&binary).unwrap();

        // ShortForm omits 'F' prefix and sorts by FID
        assert_eq!(shortform, "7=1;12=14532");
    }

    #[test]
    fn test_shortform_to_binary_simple() {
        let shortform = "7=1;12=14532";

        let converter = LlbConverter::default();
        let binary = converter.shortform_to_binary(shortform).unwrap();

        let decoder = BinaryDecoder::new();
        let record = decoder.decode(&binary).unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_binary_to_fulltext() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();
        let fulltext = converter.binary_to_fulltext(&binary).unwrap();

        // FullText uses canonical LNMP format with 'F' prefix
        assert_eq!(fulltext, "F7=1\nF12=14532");
    }

    #[test]
    fn test_fulltext_to_binary() {
        let fulltext = "F7=1\nF12=14532";

        let converter = LlbConverter::default();
        let binary = converter.fulltext_to_binary(fulltext).unwrap();

        let decoder = BinaryDecoder::new();
        let record = decoder.decode(&binary).unwrap();

        assert_eq!(record.fields().len(), 2);
        assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
        assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    }

    #[test]
    fn test_binary_shortform_fulltext_round_trip() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });
        record.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let encoder = BinaryEncoder::new();
        let binary1 = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();

        // Binary → ShortForm → Binary
        let shortform = converter.binary_to_shortform(&binary1).unwrap();
        let binary2 = converter.shortform_to_binary(&shortform).unwrap();
        assert_eq!(binary1, binary2);

        // Binary → FullText → Binary
        let fulltext = converter.binary_to_fulltext(&binary1).unwrap();
        let binary3 = converter.fulltext_to_binary(&fulltext).unwrap();
        assert_eq!(binary1, binary3);
    }

    #[test]
    fn test_shortform_with_string_array() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 23,
            value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();
        let shortform = converter.binary_to_shortform(&binary).unwrap();

        assert_eq!(shortform, "23=[admin,dev]");
    }

    #[test]
    fn test_shortform_with_multiple_fields() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("test".to_string()),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Float(3.14),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();
        let shortform = converter.binary_to_shortform(&binary).unwrap();

        assert_eq!(shortform, "1=42;2=test;3=3.14");
    }

    #[test]
    fn test_shortform_with_quoted_strings() {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 5,
            value: LnmpValue::String("hello world".to_string()),
        });

        let encoder = BinaryEncoder::new();
        let binary = encoder.encode(&record).unwrap();

        let converter = LlbConverter::default();
        let shortform = converter.binary_to_shortform(&binary).unwrap();

        assert_eq!(shortform, r#"5="hello world""#);
    }

    #[test]
    fn test_shortform_to_fulltext_respects_strings() {
        // String contains digits; should not be prefixed with 'F'
        let converter = LlbConverter::default();
        let fulltext = converter.shortform_to_fulltext(r#"12="1234""#);
        assert_eq!(fulltext, r#"F12="1234""#);
    }

    #[test]
    fn test_shortform_to_fulltext_handles_nested() {
        let converter = LlbConverter::default();
        let fulltext = converter.shortform_to_fulltext("10={1=1;2=\"x\"}");
        assert_eq!(fulltext, "F10={F1=1;F2=\"x\"}");
    }
}

#[test]
fn test_flatten_simple_record() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    let converter = LlbConverter::default();
    let flattened = converter.flatten_nested(&record).unwrap();

    // Simple records should remain unchanged
    assert_eq!(flattened.fields().len(), 2);
    assert_eq!(flattened.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        flattened.get_field(2).unwrap().value,
        LnmpValue::String("test".to_string())
    );
}

#[test]
fn test_flatten_nested_record() {
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    inner.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    let mut outer = LnmpRecord::new();
    outer.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });

    let config = LlbConfig::new().with_flattening(true);
    let converter = LlbConverter::new(config);
    let flattened = converter.flatten_nested(&outer).unwrap();

    // Nested fields should be flattened with encoded FIDs
    assert!(flattened.fields().len() >= 2);

    // Check that flattened fields exist (exact FIDs depend on encoding)
    let has_int_field = flattened
        .fields()
        .iter()
        .any(|f| f.value == LnmpValue::Int(42));
    let has_string_field = flattened
        .fields()
        .iter()
        .any(|f| f.value == LnmpValue::String("test".to_string()));

    assert!(has_int_field);
    assert!(has_string_field);
}

#[test]
fn test_flatten_deeply_nested() {
    let mut level2 = LnmpRecord::new();
    level2.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });

    let mut level1 = LnmpRecord::new();
    level1.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::NestedRecord(Box::new(level2)),
    });

    let mut level0 = LnmpRecord::new();
    level0.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::NestedRecord(Box::new(level1)),
    });

    let config = LlbConfig::new().with_flattening(true);
    let converter = LlbConverter::new(config);
    let flattened = converter.flatten_nested(&level0).unwrap();

    // Should have at least one flattened field
    assert!(!flattened.fields().is_empty());

    // Check that the deeply nested value is present
    let has_value = flattened
        .fields()
        .iter()
        .any(|f| f.value == LnmpValue::Int(100));
    assert!(has_value);
}

#[test]
fn test_unflatten_simple_record() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("test".to_string()),
    });

    let converter = LlbConverter::default();
    let unflattened = converter.unflatten(&record).unwrap();

    // Simple records should remain unchanged
    assert_eq!(unflattened.fields().len(), 2);
    assert_eq!(unflattened.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        unflattened.get_field(2).unwrap().value,
        LnmpValue::String("test".to_string())
    );
}

#[test]
fn test_flatten_unflatten_preserves_primitives() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(42),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Float(3.14),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });

    let converter = LlbConverter::default();
    let flattened = converter.flatten_nested(&record).unwrap();
    let unflattened = converter.unflatten(&flattened).unwrap();

    // Primitive values should be preserved
    assert_eq!(unflattened.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        unflattened.get_field(2).unwrap().value,
        LnmpValue::Float(3.14)
    );
    assert_eq!(
        unflattened.get_field(3).unwrap().value,
        LnmpValue::Bool(true)
    );
}

#[test]
fn test_add_semantic_hints_simple() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    let mut hints = HashMap::new();
    hints.insert(7, "is_admin".to_string());
    hints.insert(12, "user_id".to_string());

    let config = LlbConfig::new().with_semantic_hints(true);
    let converter = LlbConverter::new(config);
    let output = converter.add_semantic_hints(&record, &hints);

    assert!(output.contains("F7=1 # is_admin@sem"));
    assert!(output.contains("F12=14532 # user_id@sem"));
}

#[test]
fn test_add_semantic_hints_partial() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("test".to_string()),
    });

    let mut hints = HashMap::new();
    hints.insert(7, "is_admin".to_string());
    // No hint for field 12
    hints.insert(20, "username".to_string());

    let config = LlbConfig::new().with_semantic_hints(true);
    let converter = LlbConverter::new(config);
    let output = converter.add_semantic_hints(&record, &hints);

    assert!(output.contains("F7=1 # is_admin@sem"));
    assert!(output.contains("F12=14532\n")); // No hint
    assert!(output.contains("F20=test # username@sem"));
}

#[test]
fn test_add_semantic_hints_no_hints() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });

    let hints = HashMap::new();

    let converter = LlbConverter::default();
    let output = converter.add_semantic_hints(&record, &hints);

    // Should just output the field without hints
    assert_eq!(output, "F7=1");
}

#[test]
fn test_add_semantic_hints_with_string_array() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    let mut hints = HashMap::new();
    hints.insert(23, "roles".to_string());

    let config = LlbConfig::new().with_semantic_hints(true);
    let converter = LlbConverter::new(config);
    let output = converter.add_semantic_hints(&record, &hints);

    assert_eq!(output, "F23=[admin,dev] # roles@sem");
}

#[test]
fn test_semantic_hints_sorted_by_fid() {
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 100,
        value: LnmpValue::Int(3),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::Int(1),
    });
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::Int(2),
    });

    let mut hints = HashMap::new();
    hints.insert(5, "first".to_string());
    hints.insert(50, "second".to_string());
    hints.insert(100, "third".to_string());

    let converter = LlbConverter::default();
    let output = converter.add_semantic_hints(&record, &hints);

    // Fields should be sorted by FID
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("F5="));
    assert!(lines[1].starts_with("F50="));
    assert!(lines[2].starts_with("F100="));
}

#[test]
fn test_generate_short_ids_simple() {
    let field_names = vec![
        "user_id".to_string(),
        "username".to_string(),
        "email".to_string(),
    ];

    let converter = LlbConverter::default();
    let ids = converter.generate_short_ids(&field_names).unwrap();

    assert_eq!(ids.len(), 3);
    assert!(ids.contains_key("user_id"));
    assert!(ids.contains_key("username"));
    assert!(ids.contains_key("email"));

    // IDs should be short
    assert!(ids["user_id"].len() <= 4);
    assert!(ids["username"].len() <= 4);
    assert!(ids["email"].len() <= 4);
}

#[test]
fn test_generate_short_ids_deterministic() {
    let field_names = vec!["user_id".to_string(), "username".to_string()];

    let converter = LlbConverter::default();
    let ids1 = converter.generate_short_ids(&field_names).unwrap();
    let ids2 = converter.generate_short_ids(&field_names).unwrap();

    // Same input should produce same output
    assert_eq!(ids1, ids2);
}

#[test]
fn test_generate_short_ids_collision_detection() {
    // These names will collide with the simple algorithm (both start with 'u' and have length 7)
    let field_names = vec!["user_id".to_string(), "user_no".to_string()];

    let converter = LlbConverter::default();
    let result = converter.generate_short_ids(&field_names);

    // Should detect collision
    assert!(result.is_err());
    if let Err(LlbError::IdCollision { id, names }) = result {
        assert_eq!(id, "u7");
        assert_eq!(names.len(), 2);
    } else {
        panic!("Expected IdCollision error");
    }
}

#[test]
fn test_generate_short_ids_safe_handles_collisions() {
    // These names will collide with the simple algorithm
    let field_names = vec![
        "user_id".to_string(),
        "user_no".to_string(),
        "user_pk".to_string(),
    ];

    let converter = LlbConverter::default();
    let ids = converter.generate_short_ids_safe(&field_names);

    assert_eq!(ids.len(), 3);

    // All IDs should be unique
    let mut unique_ids: Vec<&String> = ids.values().collect();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), 3);
}

#[test]
fn test_generate_short_ids_short_names() {
    let field_names = vec!["id".to_string(), "age".to_string(), "zip".to_string()];

    let converter = LlbConverter::default();
    let ids = converter.generate_short_ids(&field_names).unwrap();

    // Short names should use themselves as IDs
    assert_eq!(ids["id"], "id");
    assert_eq!(ids["age"], "age");
    assert_eq!(ids["zip"], "zip");
}

#[test]
fn test_generate_short_ids_empty_name() {
    let field_names = vec!["".to_string()];

    let converter = LlbConverter::default();
    let ids = converter.generate_short_ids(&field_names).unwrap();

    // Empty name should get a default ID
    assert!(ids.contains_key(""));
    assert!(!ids[""].is_empty());
}

#[test]
fn test_compute_short_id_various_lengths() {
    let converter = LlbConverter::default();

    assert_eq!(converter.compute_short_id("id"), "id");
    assert_eq!(converter.compute_short_id("age"), "age");
    assert_eq!(converter.compute_short_id("name"), "n4");
    assert_eq!(converter.compute_short_id("user_id"), "u7");
    assert_eq!(converter.compute_short_id("username"), "u8");
    assert_eq!(converter.compute_short_id("email_address"), "e13");
}
