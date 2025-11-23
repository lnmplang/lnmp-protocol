//! Text codec for envelope metadata
//!
//! Encodes envelope metadata as a header comment line before the LNMP record.
//!
//! ## Format
//!
//! ```text
//! #ENVELOPE timestamp=1732373147000 source=auth-service trace_id="abc-123"
//! F12=14532
//! F7=1
//! ```
//!
//! ## Parsing Rules
//!
//! - `#ENVELOPE` keyword must be first line (if present)
//! - Space-separated key=value pairs
//! - Values without spaces unquoted, otherwise double-quoted
//! - Envelope is optional (backward compatible)

use crate::{EnvelopeError, EnvelopeMetadata, Result};

/// Text encoder for envelope metadata
pub struct TextEncoder;

impl TextEncoder {
    /// Encodes metadata as header comment line
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_envelope::{EnvelopeMetadata, text_codec::TextEncoder};
    ///
    /// let mut metadata = EnvelopeMetadata::new();
    /// metadata.timestamp = Some(1732373147000);
    /// metadata.source = Some("auth-service".to_string());
    ///
    /// let header = TextEncoder::encode(&metadata).unwrap();
    /// assert!(header.starts_with("#ENVELOPE"));
    /// ```
    pub fn encode(metadata: &EnvelopeMetadata) -> Result<String> {
        if metadata.is_empty() {
            return Ok(String::new());
        }

        let mut parts = vec!["#ENVELOPE".to_string()];

        // Canonical order: timestamp, source, trace_id, sequence
        if let Some(ts) = metadata.timestamp {
            parts.push(format!("timestamp={}", ts));
        }

        if let Some(ref source) = metadata.source {
            parts.push(format!("source={}", Self::quote_if_needed(source)));
        }

        if let Some(ref trace_id) = metadata.trace_id {
            parts.push(format!("trace_id={}", Self::quote_if_needed(trace_id)));
        }

        if let Some(seq) = metadata.sequence {
            parts.push(format!("sequence={}", seq));
        }

        // Labels (future)
        for (key, value) in &metadata.labels {
            parts.push(format!("{}={}", key, Self::quote_if_needed(value)));
        }

        Ok(parts.join(" "))
    }

    fn quote_if_needed(s: &str) -> String {
        if s.contains(' ') || s.contains('"') || s.contains('=') {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else {
            s.to_string()
        }
    }
}

/// Text decoder for envelope metadata
pub struct TextDecoder;

impl TextDecoder {
    /// Decodes metadata from header comment line
    ///
    /// Returns `None` if no envelope header found (backward compatible).
    ///
    /// # Example
    ///
    /// ```
    /// use lnmp_envelope::text_codec::TextDecoder;
    ///
    /// let header = "#ENVELOPE timestamp=1732373147000 source=test";
    /// let metadata = TextDecoder::decode(header).unwrap();
    ///
    /// assert!(metadata.is_some());
    /// assert_eq!(metadata.unwrap().timestamp, Some(1732373147000));
    /// ```
    pub fn decode(line: &str) -> Result<Option<EnvelopeMetadata>> {
        let line = line.trim();

        if !line.starts_with("#ENVELOPE") {
            return Ok(None);
        }

        let rest = line.strip_prefix("#ENVELOPE").unwrap().trim();
        if rest.is_empty() {
            return Ok(Some(EnvelopeMetadata::new()));
        }

        let mut metadata = EnvelopeMetadata::new();
        let pairs = Self::parse_pairs(rest)?;

        for (key, value) in pairs {
            match key.as_str() {
                "timestamp" => {
                    metadata.timestamp = Some(value.parse().map_err(|_| {
                        EnvelopeError::MalformedHeader(format!("Invalid timestamp: {}", value))
                    })?);
                }
                "source" => {
                    metadata.source = Some(value);
                }
                "trace_id" => {
                    metadata.trace_id = Some(value);
                }
                "sequence" => {
                    metadata.sequence = Some(value.parse().map_err(|_| {
                        EnvelopeError::MalformedHeader(format!("Invalid sequence: {}", value))
                    })?);
                }
                _ => {
                    // Unknown key - store in labels
                    metadata.labels.insert(key, value);
                }
            }
        }

        Ok(Some(metadata))
    }

    fn parse_pairs(input: &str) -> Result<Vec<(String, String)>> {
        let mut pairs = Vec::new();
        let mut chars = input.chars().peekable();

        while chars.peek().is_some() {
            // Skip whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }

            if chars.peek().is_none() {
                break;
            }

            // Parse key
            let mut key = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '=' {
                    chars.next();
                    break;
                } else if ch == ' ' {
                    return Err(EnvelopeError::MalformedHeader(
                        "Expected '=' after key".to_string(),
                    ));
                } else {
                    key.push(ch);
                    chars.next();
                }
            }

            if key.is_empty() {
                return Err(EnvelopeError::MalformedHeader("Empty key".to_string()));
            }

            // Parse value
            let value = if chars.peek() == Some(&'"') {
                // Quoted value
                chars.next(); // Skip opening quote
                let mut val = String::new();
                let mut escaped = false;

                for ch in chars.by_ref() {
                    if escaped {
                        val.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    } else {
                        val.push(ch);
                    }
                }

                val
            } else {
                // Unquoted value
                let mut val = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' {
                        break;
                    }
                    val.push(ch);
                    chars.next();
                }
                val
            };

            pairs.push((key, value));
        }

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty_metadata() {
        let metadata = EnvelopeMetadata::new();
        let encoded = TextEncoder::encode(&metadata).unwrap();
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_encode_timestamp_only() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.timestamp = Some(1732373147000);

        let encoded = TextEncoder::encode(&metadata).unwrap();
        assert_eq!(encoded, "#ENVELOPE timestamp=1732373147000");
    }

    #[test]
    fn test_encode_all_fields() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.timestamp = Some(1732373147000);
        metadata.source = Some("auth-service".to_string());
        metadata.trace_id = Some("abc-123-xyz".to_string());
        metadata.sequence = Some(42);

        let encoded = TextEncoder::encode(&metadata).unwrap();
        assert!(encoded.contains("timestamp=1732373147000"));
        assert!(encoded.contains("source=auth-service"));
        assert!(encoded.contains("trace_id=abc-123-xyz"));
        assert!(encoded.contains("sequence=42"));
    }

    #[test]
    fn test_encode_quotes_spaces() {
        let mut metadata = EnvelopeMetadata::new();
        metadata.source = Some("my service".to_string());

        let encoded = TextEncoder::encode(&metadata).unwrap();
        assert!(encoded.contains("source=\"my service\""));
    }

    #[test]
    fn test_decode_no_envelope() {
        let result = TextDecoder::decode("F12=14532").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_empty_envelope() {
        let result = TextDecoder::decode("#ENVELOPE").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_decode_timestamp_only() {
        let result = TextDecoder::decode("#ENVELOPE timestamp=1732373147000").unwrap();
        let metadata = result.unwrap();
        assert_eq!(metadata.timestamp, Some(1732373147000));
    }

    #[test]
    fn test_decode_all_fields() {
        let input =
            "#ENVELOPE timestamp=1732373147000 source=auth-service trace_id=abc-123 sequence=42";
        let result = TextDecoder::decode(input).unwrap();
        let metadata = result.unwrap();

        assert_eq!(metadata.timestamp, Some(1732373147000));
        assert_eq!(metadata.source, Some("auth-service".to_string()));
        assert_eq!(metadata.trace_id, Some("abc-123".to_string()));
        assert_eq!(metadata.sequence, Some(42));
    }

    #[test]
    fn test_decode_quoted_value() {
        let input = "#ENVELOPE source=\"my service\" trace_id=\"abc-123-xyz\"";
        let result = TextDecoder::decode(input).unwrap();
        let metadata = result.unwrap();

        assert_eq!(metadata.source, Some("my service".to_string()));
        assert_eq!(metadata.trace_id, Some("abc-123-xyz".to_string()));
    }

    #[test]
    fn test_round_trip() {
        let mut original = EnvelopeMetadata::new();
        original.timestamp = Some(1732373147000);
        original.source = Some("test-service".to_string());
        original.trace_id = Some("trace-abc".to_string());
        original.sequence = Some(99);

        let encoded = TextEncoder::encode(&original).unwrap();
        let decoded = TextDecoder::decode(&encoded).unwrap().unwrap();

        assert_eq!(original.timestamp, decoded.timestamp);
        assert_eq!(original.source, decoded.source);
        assert_eq!(original.trace_id, decoded.trace_id);
        assert_eq!(original.sequence, decoded.sequence);
    }

    #[test]
    fn test_decode_unknown_keys_in_labels() {
        let input = "#ENVELOPE timestamp=123 custom_key=custom_value";
        let result = TextDecoder::decode(input).unwrap();
        let metadata = result.unwrap();

        assert_eq!(metadata.timestamp, Some(123));
        assert_eq!(
            metadata.labels.get("custom_key"),
            Some(&"custom_value".to_string())
        );
    }
}
