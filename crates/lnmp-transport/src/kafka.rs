//! Kafka transport bindings for LNMP.
//!
//! This module provides helpers to map LNMP Envelope metadata to/from Kafka record headers,
//! and encode/decode LNMP record values.

use crate::{Result, TransportError};
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use std::collections::HashMap;

/// Kafka header name for LNMP timestamp.
pub const HEADER_TIMESTAMP: &str = "lnmp.timestamp";

/// Kafka header name for LNMP source identifier.
pub const HEADER_SOURCE: &str = "lnmp.source";

/// Kafka header name for LNMP trace ID.
pub const HEADER_TRACE_ID: &str = "lnmp.trace_id";

/// Kafka header name for LNMP sequence number.
pub const HEADER_SEQUENCE: &str = "lnmp.sequence";

/// Kafka header name prefix for LNMP labels.
pub const HEADER_LABEL_PREFIX: &str = "lnmp.label.";

/// Type alias for Kafka headers (key-value pairs as bytes).
pub type KafkaHeaders = HashMap<String, Vec<u8>>;

/// Converts an LNMP Envelope's metadata to Kafka headers.
///
/// This function maps envelope metadata fields to standard LNMP Kafka headers as byte arrays.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::kafka;
/// let headers = kafka::envelope_to_kafka_headers(&envelope)?;
/// ```
pub fn envelope_to_kafka_headers(env: &LnmpEnvelope) -> Result<KafkaHeaders> {
    let mut headers = HashMap::new();
    let meta = &env.metadata;

    if let Some(ts) = meta.timestamp {
        headers.insert(HEADER_TIMESTAMP.to_string(), ts.to_string().into_bytes());
    }

    if let Some(src) = &meta.source {
        headers.insert(HEADER_SOURCE.to_string(), src.as_bytes().to_vec());
    }

    if let Some(trace_id) = &meta.trace_id {
        headers.insert(HEADER_TRACE_ID.to_string(), trace_id.as_bytes().to_vec());
    }

    if let Some(seq) = meta.sequence {
        headers.insert(HEADER_SEQUENCE.to_string(), seq.to_string().into_bytes());
    }

    for (k, v) in &meta.labels {
        let header_name = format!("{}{}", HEADER_LABEL_PREFIX, k);
        headers.insert(header_name, v.as_bytes().to_vec());
    }

    Ok(headers)
}

/// Converts Kafka headers to LNMP Envelope metadata.
///
/// This function extracts envelope metadata from Kafka headers.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::kafka;
/// let meta = kafka::kafka_headers_to_envelope_metadata(&headers)?;
/// ```
pub fn kafka_headers_to_envelope_metadata(headers: &KafkaHeaders) -> Result<EnvelopeMetadata> {
    let mut meta = EnvelopeMetadata::default();

    if let Some(val) = headers.get(HEADER_TIMESTAMP) {
        let s = String::from_utf8(val.clone()).map_err(|_| {
            TransportError::InvalidHeaderValue("timestamp".into(), "not utf8".into())
        })?;
        meta.timestamp = Some(s.parse().map_err(|_e| {
            TransportError::InvalidHeaderValue("timestamp".into(), "parse error".into())
        })?);
    }

    if let Some(val) = headers.get(HEADER_SOURCE) {
        meta.source =
            Some(String::from_utf8(val.clone()).map_err(|_| {
                TransportError::InvalidHeaderValue("source".into(), "not utf8".into())
            })?);
    }

    if let Some(val) = headers.get(HEADER_TRACE_ID) {
        meta.trace_id = Some(String::from_utf8(val.clone()).map_err(|_| {
            TransportError::InvalidHeaderValue("trace_id".into(), "not utf8".into())
        })?);
    }

    if let Some(val) = headers.get(HEADER_SEQUENCE) {
        let s = String::from_utf8(val.clone()).map_err(|_| {
            TransportError::InvalidHeaderValue("sequence".into(), "not utf8".into())
        })?;
        meta.sequence = Some(s.parse().map_err(|_e| {
            TransportError::InvalidHeaderValue("sequence".into(), "parse error".into())
        })?);
    }

    for (name, value) in headers {
        if name.starts_with(HEADER_LABEL_PREFIX) {
            let key = name.trim_start_matches(HEADER_LABEL_PREFIX).to_string();
            let val_str = String::from_utf8(value.clone()).map_err(|_| {
                TransportError::InvalidHeaderValue(format!("label.{}", key), "not utf8".into())
            })?;
            meta.labels.insert(key, val_str);
        }
    }

    Ok(meta)
}

/// Encodes an LNMP Envelope to a complete Kafka record (value + headers).
///
/// Returns the encoded record value and headers as a tuple.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::kafka;
/// let (value, headers) = kafka::envelope_to_kafka_record(&envelope)?;
/// ```
pub fn envelope_to_kafka_record(env: &LnmpEnvelope) -> Result<(Vec<u8>, KafkaHeaders)> {
    use lnmp_codec::binary::BinaryEncoder;

    let headers = envelope_to_kafka_headers(env)?;
    let encoder = BinaryEncoder::new();
    let value = encoder.encode(&env.record)?;

    Ok((value, headers))
}

/// Decodes an LNMP Envelope from a Kafka record (value + headers).
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::kafka;
/// let envelope = kafka::kafka_record_to_envelope(&value, &headers)?;
/// ```
pub fn kafka_record_to_envelope(value: &[u8], headers: &KafkaHeaders) -> Result<LnmpEnvelope> {
    use lnmp_codec::binary::BinaryDecoder;

    let metadata = kafka_headers_to_envelope_metadata(headers)?;
    let decoder = BinaryDecoder::new();
    let record = decoder.decode(value)?;

    Ok(LnmpEnvelope { metadata, record })
}
