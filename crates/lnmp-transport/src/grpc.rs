//! gRPC transport bindings for LNMP.
//!
//! This module provides helpers to map LNMP Envelope metadata to/from gRPC metadata (headers).
//!
//! For gRPC payload handling, you can either:
//! 1. Embed the LNMP binary record inside your Protobuf message as a `bytes` field, or
//! 2. Use LNMP metadata only in the headers and send application data in the Protobuf message.

use crate::{Result, TransportError};
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use std::collections::HashMap;

/// gRPC metadata key for LNMP timestamp.
pub const META_TIMESTAMP: &str = "lnmp-timestamp";

/// gRPC metadata key for LNMP source identifier.
pub const META_SOURCE: &str = "lnmp-source";

/// gRPC metadata key for LNMP trace ID.
pub const META_TRACE_ID: &str = "lnmp-trace-id";

/// gRPC metadata key for LNMP sequence number.
pub const META_SEQUENCE: &str = "lnmp-sequence";

/// gRPC metadata key prefix for LNMP labels.
pub const META_LABEL_PREFIX: &str = "lnmp-label-";

/// Converts an LNMP Envelope's metadata to gRPC metadata.
///
/// This function maps envelope metadata fields to standard LNMP gRPC metadata keys.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::grpc;
/// let metadata = grpc::envelope_to_metadata(&envelope)?;
/// ```
pub fn envelope_to_metadata(env: &LnmpEnvelope) -> Result<HashMap<String, String>> {
    let mut metadata = HashMap::new();
    let meta = &env.metadata;

    if let Some(ts) = meta.timestamp {
        metadata.insert(META_TIMESTAMP.to_string(), ts.to_string());
    }

    if let Some(src) = &meta.source {
        metadata.insert(META_SOURCE.to_string(), src.clone());
    }

    if let Some(trace_id) = &meta.trace_id {
        metadata.insert(META_TRACE_ID.to_string(), trace_id.clone());
    }

    if let Some(seq) = meta.sequence {
        metadata.insert(META_SEQUENCE.to_string(), seq.to_string());
    }

    for (k, v) in &meta.labels {
        let key = format!("{}{}", META_LABEL_PREFIX, k);
        metadata.insert(key, v.clone());
    }

    Ok(metadata)
}

/// Converts gRPC metadata to LNMP Envelope metadata.
///
/// This function extracts envelope metadata from gRPC metadata.
///
/// # Example
///
/// ```rust,ignore
/// use lnmp_transport::grpc;
/// let meta = grpc::metadata_to_envelope_metadata(&metadata)?;
/// ```
pub fn metadata_to_envelope_metadata(map: &HashMap<String, String>) -> Result<EnvelopeMetadata> {
    let mut meta = EnvelopeMetadata::default();

    if let Some(val) = map.get(META_TIMESTAMP) {
        meta.timestamp = Some(val.parse().map_err(|_e| {
            TransportError::InvalidHeaderValue("timestamp".into(), "parse error".into())
        })?);
    }

    if let Some(val) = map.get(META_SOURCE) {
        meta.source = Some(val.clone());
    }

    if let Some(val) = map.get(META_TRACE_ID) {
        meta.trace_id = Some(val.clone());
    }

    if let Some(val) = map.get(META_SEQUENCE) {
        meta.sequence = Some(val.parse().map_err(|_e| {
            TransportError::InvalidHeaderValue("sequence".into(), "parse error".into())
        })?);
    }

    for (name, value) in map {
        if name.starts_with(META_LABEL_PREFIX) {
            let key = name.trim_start_matches(META_LABEL_PREFIX).to_string();
            meta.labels.insert(key, value.clone());
        }
    }

    Ok(meta)
}
