use pyo3::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use lnmp::codec::{Encoder, Parser};
use lnmp::core::LnmpRecord;
use lnmp::embedding::{Vector, VectorDelta};
use lnmp::envelope::{EnvelopeBuilder, LnmpEnvelope};
use lnmp::llb::{ExplainEncoder, SemanticDictionary};
use lnmp::net::{MessageKind, NetMessage, RoutingPolicy};
use lnmp::quant::{quantize_embedding, QuantScheme};
use lnmp::sanitize::sanitize_lnmp_text;
use lnmp::sfe::ContextScorer;

// Core types
#[pyclass]
struct PyLnmpRecord {
    inner: LnmpRecord,
}

#[pyclass]
struct PyLnmpEnvelope {
    inner: LnmpEnvelope,
}

#[pymethods]
impl PyLnmpEnvelope {
    #[getter]
    fn source(&self) -> Option<String> {
        self.inner.metadata.source.clone()
    }

    #[getter]
    fn trace_id(&self) -> Option<String> {
        self.inner.metadata.trace_id.clone()
    }

    #[getter]
    fn timestamp(&self) -> Option<u64> {
        self.inner.metadata.timestamp
    }
}

// Core functions
#[pyfunction]
fn parse(text: &str) -> PyResult<PyLnmpRecord> {
    let mut parser = Parser::new(text)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let record = parser
        .parse_record()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok(PyLnmpRecord { inner: record })
}

#[pyfunction]
fn encode(record: &PyLnmpRecord) -> PyResult<String> {
    let encoder = Encoder::new();
    let text = encoder.encode(&record.inner);
    Ok(text)
}

#[pyfunction]
fn encode_binary(py: Python, record: &PyLnmpRecord) -> PyResult<Py<pyo3::types::PyBytes>> {
    use lnmp::codec::binary::BinaryEncoder;

    let encoder = BinaryEncoder::new();
    let binary = encoder
        .encode(&record.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(pyo3::types::PyBytes::new_bound(py, &binary).into())
}

#[pyfunction]
fn decode_binary(data: &[u8]) -> PyResult<PyLnmpRecord> {
    use lnmp::codec::binary::BinaryDecoder;

    let decoder = BinaryDecoder::new();
    let record = decoder
        .decode(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(PyLnmpRecord { inner: record })
}

// Envelope functions
#[pyfunction]
#[pyo3(signature = (record, source, timestamp_ms=None, trace_id=None))]
fn envelope_wrap(
    record: &PyLnmpRecord,
    source: String,
    timestamp_ms: Option<u64>,
    trace_id: Option<String>,
) -> PyResult<PyLnmpEnvelope> {
    let mut builder = EnvelopeBuilder::new(record.inner.clone()).source(source);

    if let Some(ts) = timestamp_ms {
        builder = builder.timestamp(ts);
    } else {
        // Default to now
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH!")
            .as_millis() as u64;
        builder = builder.timestamp(now);
    }

    if let Some(tid) = trace_id {
        builder = builder.trace_id(tid);
    }

    let envelope = builder.build();
    Ok(PyLnmpEnvelope { inner: envelope })
}

// Network functions
#[pyfunction]
fn routing_decide(envelope: &PyLnmpEnvelope) -> PyResult<String> {
    let policy = RoutingPolicy::default();
    let msg = NetMessage::new(envelope.inner.clone(), MessageKind::Event);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH!")
        .as_millis() as u64;

    let decision = policy
        .decide(&msg, now)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(format!("{:?}", decision))
}

#[pyfunction]
fn context_score(envelope: &PyLnmpEnvelope) -> PyResult<HashMap<String, f64>> {
    let scorer = ContextScorer::default();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH!")
        .as_millis() as u64;

    let profile = scorer.score_envelope(&envelope.inner, now);

    let mut scores = HashMap::new();
    scores.insert("composite".to_string(), profile.composite_score());
    scores.insert("freshness".to_string(), profile.freshness_score);
    scores.insert("importance".to_string(), profile.importance as f64 / 255.0);
    scores.insert("confidence".to_string(), profile.confidence);
    scores.insert("risk".to_string(), profile.risk_level.as_u8() as f64);

    Ok(scores)
}

// Embedding functions
#[pyfunction]
fn embedding_delta(
    py: Python,
    base: Vec<f32>,
    updated: Vec<f32>,
) -> PyResult<(usize, Py<pyo3::types::PyBytes>)> {
    let base_vec = Vector::from_f32(base);
    let updated_vec = Vector::from_f32(updated);

    // Compute delta using from_vectors
    let delta = VectorDelta::from_vectors(&base_vec, &updated_vec, 0)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    let change_count = delta.changes.len();

    // Encode delta
    let encoded = delta
        .encode()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok((
        change_count,
        pyo3::types::PyBytes::new_bound(py, &encoded).into(),
    ))
}

#[pyfunction]
fn embedding_apply_delta(base: Vec<f32>, delta_bytes: &[u8]) -> PyResult<Vec<f32>> {
    use lnmp::embedding::{Vector, VectorDelta};

    let base_vec = Vector::from_f32(base);
    let delta = VectorDelta::decode(delta_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let updated_vec = delta
        .apply(&base_vec)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;

    updated_vec
        .as_f32()
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
}

// Spatial functions
#[pyfunction]
fn spatial_encode_position3d(
    py: Python,
    x: f64,
    y: f64,
    z: f64,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    use lnmp::spatial::{encoder::encode_spatial, Position3D, SpatialValue};

    let position = Position3D {
        x: x as f32,
        y: y as f32,
        z: z as f32,
    };

    let spatial_value = SpatialValue::S2(position);
    let mut buf = Vec::new();
    encode_spatial(&spatial_value, &mut buf)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(pyo3::types::PyBytes::new_bound(py, &buf).into())
}

#[pyfunction]
fn spatial_decode_position3d(data: &[u8]) -> PyResult<(f64, f64, f64)> {
    use lnmp::spatial::decoder::decode_spatial;
    use lnmp::spatial::SpatialValue;

    let mut buf = data;
    let spatial_value = decode_spatial(&mut buf)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    match spatial_value {
        SpatialValue::S2(pos) => Ok((pos.x as f64, pos.y as f64, pos.z as f64)),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Expected Position3D",
        )),
    }
}

// LLB functions
#[pyfunction]
fn debug_explain(text: &str) -> PyResult<String> {
    // Use ExplainEncoder to explain the record
    let mut parser = Parser::new(text)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let record = parser
        .parse_record()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let dict = SemanticDictionary::new(); // Empty dict for now
    let encoder = ExplainEncoder::new(dict);
    let output = encoder.encode_with_explanation(&record);

    Ok(output)
}

// Quantization functions
#[pyfunction]
fn quantize(py: Python, vector: Vec<f32>, scheme: &str) -> PyResult<Py<pyo3::types::PyBytes>> {
    let vec = Vector::from_f32(vector);
    let q_scheme = match scheme {
        "QInt8" => QuantScheme::QInt8,
        "QInt4" => QuantScheme::QInt4,
        "Binary" => QuantScheme::Binary,
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid scheme",
            ))
        }
    };

    let quantized = quantize_embedding(&vec, q_scheme)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Ok(pyo3::types::PyBytes::new_bound(py, &quantized.data).into())
}

// Sanitize functions
#[pyfunction]
fn sanitize(text: &str) -> PyResult<String> {
    use lnmp::sanitize::SanitizationConfig;
    let config = SanitizationConfig::default();
    let sanitized = sanitize_lnmp_text(text, &config);
    Ok(sanitized.to_string())
}

// Transport functions
#[pyfunction]
fn transport_to_http_headers(envelope: &PyLnmpEnvelope) -> PyResult<HashMap<String, String>> {
    use lnmp::transport::http::envelope_to_headers;

    let headers = envelope_to_headers(&envelope.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let mut result = HashMap::new();
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(val_str) = value.to_str() {
                result.insert(name.to_string(), val_str.to_string());
            }
        }
    }

    Ok(result)
}

#[pyfunction]
fn transport_from_http_headers(headers: HashMap<String, String>) -> PyResult<PyLnmpEnvelope> {
    use http::{HeaderMap, HeaderName, HeaderValue};
    use lnmp::transport::http::headers_to_envelope_metadata;
    use std::str::FromStr;

    let mut map = HeaderMap::new();
    for (k, v) in headers {
        if let Ok(name) = HeaderName::from_str(&k) {
            if let Ok(val) = HeaderValue::from_str(&v) {
                map.insert(name, val);
            }
        }
    }

    let metadata = headers_to_envelope_metadata(&map)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    // Create a dummy record for the envelope since we only have metadata
    // In a real scenario, we would parse the body separately
    use lnmp::core::LnmpRecord;
    let record = LnmpRecord::default();

    use lnmp::envelope::LnmpEnvelope;
    let envelope = LnmpEnvelope { record, metadata };

    Ok(PyLnmpEnvelope { inner: envelope })
}

// Schema functions
#[pyfunction]
fn schema_describe() -> PyResult<String> {
    Ok("LNMP Schema Description".to_string())
}

#[pymodule]
fn lnmp_py_core(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLnmpRecord>()?;
    m.add_class::<PyLnmpEnvelope>()?;
    // m.add_class::<PyContextScore>()?; // PyContextScore is not defined in the provided code

    // Core
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(encode_binary, m)?)?;
    m.add_function(wrap_pyfunction!(decode_binary, m)?)?;

    // Envelope
    m.add_function(wrap_pyfunction!(envelope_wrap, m)?)?;

    // Net
    m.add_function(wrap_pyfunction!(routing_decide, m)?)?;
    m.add_function(wrap_pyfunction!(context_score, m)?)?;
    // m.add_function(wrap_pyfunction!(network_importance, m)?)?; // network_importance is not defined
    // m.add_function(wrap_pyfunction!(network_decide, m)?)?; // network_decide is not defined

    // Embedding
    m.add_function(wrap_pyfunction!(embedding_delta, m)?)?;
    m.add_function(wrap_pyfunction!(embedding_apply_delta, m)?)?;

    // Spatial
    m.add_function(wrap_pyfunction!(spatial_encode_position3d, m)?)?;
    m.add_function(wrap_pyfunction!(spatial_decode_position3d, m)?)?;

    // LLB
    m.add_function(wrap_pyfunction!(debug_explain, m)?)?;

    // Quant
    m.add_function(wrap_pyfunction!(quantize, m)?)?;

    // Sanitize
    m.add_function(wrap_pyfunction!(sanitize, m)?)?;

    // Transport
    m.add_function(wrap_pyfunction!(transport_to_http_headers, m)?)?;
    m.add_function(wrap_pyfunction!(transport_from_http_headers, m)?)?;

    // Schema
    m.add_function(wrap_pyfunction!(schema_describe, m)?)?;

    Ok(())
}
