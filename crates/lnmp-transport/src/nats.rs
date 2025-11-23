//! NATS transport bindings for LNMP.
//!
//! This module provides helpers to map LNMP Envelope metadata to/from NATS message headers,
//! and encode/decode LNMP record payloads.
//!
//! NATS headers are similar to Kafka headers - key-value pairs attached to messages.

use crate::Result;
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use std::collections::HashMap;

/// NATS header name for LNMP timestamp.
pub const HEADER_TIMESTAMP: &str = "lnmp-timestamp";

/// NATS header name for LNMP source identifier.
pub const HEADER_SOURCE: &str = "lnmp-source";

/// NATS header name for LNMP trace ID.
pub const HEADER_TRACE_ID: &str = "lnmp-trace-id";

/// NATS header name for LNMP sequence number.
pub const HEADER_SEQUENCE: &str = "lnmp-sequence";

/// NATS header name prefix for LNMP labels.
pub const HEADER_LABEL_PREFIX: &str = "lnmp-label-";

/// Converts an LNMP Envelope's metadata to NATS headers.
///
/// This function maps envelope metadata fields to standard LNMP NATS headers.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::nats;
/// let headers = nats::envelope_to_nats_headers(&envelope)?;
/// ```
pub fn envelope_to_nats_headers(env: &LnmpEnvelope) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();
    let meta = &env.metadata;

    if let Some(ts) = meta.timestamp {
        headers.insert(HEADER_TIMESTAMP.to_string(), ts.to_string());
    }

    if let Some(src) = &meta.source {
        headers.insert(HEADER_SOURCE.to_string(), src.clone());
    }

    if let Some(trace_id) = &meta.trace_id {
        headers.insert(HEADER_TRACE_ID.to_string(), trace_id.clone());
    }

    if let Some(seq) = meta.sequence {
        headers.insert(HEADER_SEQUENCE.to_string(), seq.to_string());
    }

    for (k, v) in &meta.labels {
        let header_name = format!("{}{}", HEADER_LABEL_PREFIX, k);
        headers.insert(header_name, v.clone());
    }

    Ok(headers)
}

/// Converts NATS headers to LNMP Envelope metadata.
///
/// This function extracts envelope metadata from NATS headers.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::nats;
/// let meta = nats::nats_headers_to_envelope_metadata(&headers)?;
/// ```
pub fn nats_headers_to_envelope_metadata(
    headers: &HashMap<String, String>,
) -> Result<EnvelopeMetadata> {
    let mut meta = EnvelopeMetadata::default();

    if let Some(val) = headers.get(HEADER_TIMESTAMP) {
        meta.timestamp = val.parse().ok();
    }

    if let Some(val) = headers.get(HEADER_SOURCE) {
        meta.source = Some(val.clone());
    }

    if let Some(val) = headers.get(HEADER_TRACE_ID) {
        meta.trace_id = Some(val.clone());
    }

    if let Some(val) = headers.get(HEADER_SEQUENCE) {
        meta.sequence = val.parse().ok();
    }

    for (name, value) in headers {
        if name.starts_with(HEADER_LABEL_PREFIX) {
            let key = name.trim_start_matches(HEADER_LABEL_PREFIX).to_string();
            meta.labels.insert(key, value.clone());
        }
    }

    Ok(meta)
}

/// Encodes an LNMP Envelope to a complete NATS message (payload + headers).
///
/// Returns the encoded message payload and headers as a tuple.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::nats;
/// let (payload, headers) = nats::envelope_to_nats_message(&envelope)?;
/// ```
pub fn envelope_to_nats_message(env: &LnmpEnvelope) -> Result<(Vec<u8>, HashMap<String, String>)> {
    use lnmp_codec::binary::BinaryEncoder;

    let headers = envelope_to_nats_headers(env)?;
    let encoder = BinaryEncoder::new();
    let payload = encoder.encode(&env.record)?;

    Ok((payload, headers))
}

/// Decodes an LNMP Envelope from a NATS message (payload + headers).
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::nats;
/// let envelope = nats::nats_message_to_envelope(&payload, &headers)?;
/// ```
pub fn nats_message_to_envelope(
    payload: &[u8],
    headers: &HashMap<String, String>,
) -> Result<LnmpEnvelope> {
    use lnmp_codec::binary::BinaryDecoder;

    let metadata = nats_headers_to_envelope_metadata(headers)?;
    let decoder = BinaryDecoder::new();
    let record = decoder.decode(payload)?;

    Ok(LnmpEnvelope { metadata, record })
}
