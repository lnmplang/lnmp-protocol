//! HTTP transport bindings for LNMP.
//!
//! This module provides helpers to map LNMP Envelope metadata to/from HTTP headers,
//! encode/decode LNMP record bodies, and integrate with W3C Trace Context for distributed tracing.

use crate::{Result, TransportError};
#[cfg(feature = "http")]
use http::{HeaderMap, HeaderName, HeaderValue};
use lnmp_codec::Parser;
use lnmp_core::LnmpRecord;
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use std::str::FromStr;

/// HTTP header name for LNMP timestamp (Unix epoch milliseconds).
pub const HEADER_TIMESTAMP: &str = "X-LNMP-Timestamp";

/// HTTP header name for LNMP source identifier.
pub const HEADER_SOURCE: &str = "X-LNMP-Source";

/// HTTP header name for LNMP trace ID.
pub const HEADER_TRACE_ID: &str = "X-LNMP-Trace-Id";

/// HTTP header name for LNMP sequence number.
pub const HEADER_SEQUENCE: &str = "X-LNMP-Sequence";

/// HTTP header name prefix for LNMP labels.
pub const HEADER_LABEL_PREFIX: &str = "X-LNMP-Label-";

/// W3C Trace Context traceparent header name.
pub const HEADER_TRACEPARENT: &str = "traceparent";

/// Content-Type for LNMP binary format.
pub const CONTENT_TYPE_LNMP_BINARY: &str = "application/lnmp-binary";

/// Content-Type for LNMP text format.
pub const CONTENT_TYPE_LNMP_TEXT: &str = "application/lnmp-text";

/// Converts an LNMP Envelope's metadata to HTTP headers.
///
/// This function maps envelope metadata fields to standard LNMP HTTP headers:
/// - `timestamp` → `X-LNMP-Timestamp`
/// - `source` → `X-LNMP-Source`
/// - `trace_id` → `X-LNMP-Trace-Id` and `traceparent` (W3C Trace Context)
/// - `sequence` → `X-LNMP-Sequence`
/// - `labels["key"]` → `X-LNMP-Label-key`
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::http;
/// let headers = http::envelope_to_headers(&envelope)?;
/// ```
#[cfg(feature = "http")]
pub fn envelope_to_headers(env: &LnmpEnvelope) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let meta = &env.metadata;

    if let Some(ts) = meta.timestamp {
        headers.insert(
            HeaderName::from_static("x-lnmp-timestamp"),
            HeaderValue::from_str(&ts.to_string()).map_err(|e| {
                TransportError::InvalidHeaderValue("timestamp".into(), e.to_string())
            })?,
        );
    }

    if let Some(src) = &meta.source {
        headers.insert(
            HeaderName::from_static("x-lnmp-source"),
            HeaderValue::from_str(src)
                .map_err(|e| TransportError::InvalidHeaderValue("source".into(), e.to_string()))?,
        );
    }

    if let Some(trace_id) = &meta.trace_id {
        headers.insert(
            HeaderName::from_static("x-lnmp-trace-id"),
            HeaderValue::from_str(trace_id).map_err(|e| {
                TransportError::InvalidHeaderValue("trace_id".into(), e.to_string())
            })?,
        );

        // Generate W3C traceparent header
        let traceparent = trace_id_to_traceparent(trace_id, None, 0x01);
        headers.insert(
            HeaderName::from_static("traceparent"),
            HeaderValue::from_str(&traceparent).map_err(|e| {
                TransportError::InvalidHeaderValue("traceparent".into(), e.to_string())
            })?,
        );
    }

    if let Some(seq) = meta.sequence {
        headers.insert(
            HeaderName::from_static("x-lnmp-sequence"),
            HeaderValue::from_str(&seq.to_string()).map_err(|e| {
                TransportError::InvalidHeaderValue("sequence".into(), e.to_string())
            })?,
        );
    }

    for (k, v) in &meta.labels {
        let header_name = format!("{}{}", HEADER_LABEL_PREFIX, k).to_lowercase();
        if let Ok(name) = HeaderName::from_str(&header_name) {
            if let Ok(val) = HeaderValue::from_str(v) {
                headers.insert(name, val);
            }
        }
    }

    Ok(headers)
}

/// Converts HTTP headers to LNMP Envelope metadata.
///
/// This function extracts envelope metadata from HTTP headers. Missing headers
/// result in `None` values (fail-safe behavior).
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::http;
/// let meta = http::headers_to_envelope_metadata(&headers)?;
/// ```
#[cfg(feature = "http")]
pub fn headers_to_envelope_metadata(headers: &HeaderMap) -> Result<EnvelopeMetadata> {
    let mut meta = EnvelopeMetadata::default();

    // Try to parse all headers, but don't fail on missing ones
    if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-timestamp")) {
        if let Ok(s) = val.to_str() {
            meta.timestamp = s.parse().ok();
        }
    }

    if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-source")) {
        if let Ok(s) = val.to_str() {
            meta.source = Some(s.to_string());
        }
    }

    if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-trace-id")) {
        if let Ok(s) = val.to_str() {
            meta.trace_id = Some(s.to_string());
        }
    } else if let Some(val) = headers.get(HeaderName::from_static("traceparent")) {
        // Fallback to extracting trace_id from W3C traceparent
        if let Ok(s) = val.to_str() {
            if let Ok(trace_id) = traceparent_to_trace_id(s) {
                meta.trace_id = Some(trace_id);
            }
        }
    }

    if let Some(val) = headers.get(HeaderName::from_static("x-lnmp-sequence")) {
        if let Ok(s) = val.to_str() {
            meta.sequence = s.parse().ok();
        }
    }

    for (name, value) in headers {
        let name_str = name.as_str();
        if name_str.starts_with("x-lnmp-label-") {
            let key = name_str.trim_start_matches("x-lnmp-label-").to_string();
            if let Ok(val_str) = value.to_str() {
                meta.labels.insert(key, val_str.to_string());
            }
        }
    }

    Ok(meta)
}

/// Converts an LNMP trace_id to a W3C Trace Context traceparent header value.
///
/// Format: `version-trace_id-parent_id-trace_flags`
/// - version: `00` (fixed)
/// - trace_id: 32 hex digits (padded/truncated from input)
/// - parent_id: 16 hex digits (provided or generated)
/// - trace_flags: 2 hex digits (sampling decision)
///
/// # Example
///
/// ```rust,ignore
/// let traceparent = trace_id_to_traceparent("abc-123-xyz", None, 01);
/// // Result: "00-6162632d3132332d78797a00000000000-0123456789abcdef-01"
/// ```
pub fn trace_id_to_traceparent(trace_id: &str, span_id: Option<&str>, flags: u8) -> String {
    // Normalize trace_id to 32 hex chars
    let normalized_trace_id = normalize_trace_id_for_w3c(trace_id);

    // Use provided span_id or generate a default one
    let span_id_str = span_id.unwrap_or("0123456789abcdef");
    let normalized_span_id = normalize_span_id_for_w3c(span_id_str);

    format!(
        "00-{}-{}-{:02x}",
        normalized_trace_id, normalized_span_id, flags
    )
}

/// Extracts the trace_id from a W3C Trace Context traceparent header.
///
/// # Example
///
/// ```rust,ignore
/// let trace_id = traceparent_to_trace_id("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")?;
/// // Result: "0af7651916cd43dd8448eb211c80319c"
/// ```
pub fn traceparent_to_trace_id(traceparent: &str) -> Result<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() != 4 {
        return Err(TransportError::InvalidHeaderValue(
            "traceparent".into(),
            "invalid format (expected version-trace_id-span_id-flags)".into(),
        ));
    }

    Ok(parts[1].to_string())
}

/// Encodes an LNMP record to binary format for HTTP body.
///
/// Returns the encoded bytes and the appropriate Content-Type header value.
///
/// # Example
///
/// ```rust,ignore
/// let (body, content_type) = record_to_http_body(&record)?;
/// // body: Vec<u8>
/// // content_type: "application/lnmp-binary"
/// ```
pub fn record_to_http_body(record: &LnmpRecord) -> Result<(Vec<u8>, &'static str)> {
    use lnmp_codec::binary::BinaryEncoder;

    let encoder = BinaryEncoder::new();
    let encoded = encoder.encode(record)?;
    Ok((encoded, CONTENT_TYPE_LNMP_BINARY))
}

/// Decodes an LNMP record from HTTP body bytes.
///
/// Supports both binary and text formats based on Content-Type header.
///
/// # Example
///
/// ```rust,ignore
/// let record = http_body_to_record(&body, "application/lnmp-binary")?;
/// ```
pub fn http_body_to_record(body: &[u8], content_type: &str) -> Result<LnmpRecord> {
    if content_type.contains("lnmp-binary") || content_type.contains("octet-stream") {
        // Binary format
        use lnmp_codec::binary::BinaryDecoder;
        let decoder = BinaryDecoder::new();
        Ok(decoder.decode(body)?)
    } else if content_type.contains("lnmp-text") || content_type.contains("text/plain") {
        // Text format
        let text = std::str::from_utf8(body)
            .map_err(|_| TransportError::InvalidHeaderValue("body".into(), "not utf8".into()))?;
        let mut parser = Parser::new(text)?;
        Ok(parser.parse_record()?)
    } else {
        Err(TransportError::InvalidHeaderValue(
            "content-type".into(),
            format!("unsupported: {}", content_type),
        ))
    }
}

// Helper functions

fn normalize_trace_id_for_w3c(trace_id: &str) -> String {
    let hex_only: String = trace_id.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    let mut normalized = hex_only.to_lowercase();

    // Pad or truncate to 32 hex chars
    if normalized.len() < 32 {
        normalized.push_str(&"0".repeat(32 - normalized.len()));
    } else if normalized.len() > 32 {
        normalized.truncate(32);
    }

    normalized
}

fn normalize_span_id_for_w3c(span_id: &str) -> String {
    let hex_only: String = span_id.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    let mut normalized = hex_only.to_lowercase();

    // Pad or truncate to 16 hex chars
    if normalized.len() < 16 {
        normalized.push_str(&"0".repeat(16 - normalized.len()));
    } else if normalized.len() > 16 {
        normalized.truncate(16);
    }

    normalized
}
