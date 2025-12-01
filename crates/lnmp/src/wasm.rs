use wasm_bindgen::prelude::*;
use crate::core::{LnmpRecord, LnmpValue, FieldId};
use crate::codec;
use serde::Serialize;

// Initialize panic hook for better error messages
#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// --- CORE & CODEC ---

#[wasm_bindgen]
pub fn parse_lnmp(text: &str) -> Result<JsValue, JsValue> {
    let mut parser = codec::Parser::new(text)
        .map_err(|e| JsValue::from_str(&format!("Parser init error: {}", e)))?;
    
    let record = parser.parse_record()
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
    serde_wasm_bindgen::to_value(&record)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn encode_lnmp(record_js: JsValue, canonical: bool, type_hints: bool) -> Result<String, JsValue> {
    // Deserialize record
    let record: LnmpRecord = serde_wasm_bindgen::from_value(record_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
    
    // Configure encoder
    let config = crate::codec::EncoderConfig {
        canonical,
        include_type_hints: type_hints,
        ..Default::default()
    };
    
    // Encode
    let encoder = crate::codec::Encoder::with_config(config);
    Ok(encoder.encode(&record))
}

#[wasm_bindgen]
pub fn compute_checksum(fid: FieldId, type_hint: Option<String>, value_js: JsValue) -> Result<u32, JsValue> {
    // Parse type hint
    let type_hint_enum = match type_hint {
        Some(s) => Some(crate::core::TypeHint::parse(&s).ok_or("Invalid type hint")?),
        None => None,
    };
    
    // Deserialize value
    let value: LnmpValue = serde_wasm_bindgen::from_value(value_js)
        .map_err(|e| JsValue::from_str(&format!("Value deserialization error: {}", e)))?;
    
    // Compute checksum
    use crate::core::checksum::SemanticChecksum;
    Ok(SemanticChecksum::compute(fid, type_hint_enum, &value))
}

#[wasm_bindgen]
pub fn format_checksum(checksum: u32) -> String {
    use crate::core::checksum::SemanticChecksum;
    SemanticChecksum::format(checksum)
}

#[wasm_bindgen]
pub fn parse_checksum(hex: &str) -> Result<u32, JsValue> {
    use crate::core::checksum::SemanticChecksum;
    SemanticChecksum::parse(hex)
        .ok_or_else(|| JsValue::from_str("Invalid hex checksum"))
}

#[wasm_bindgen]
pub fn lnmp_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// --- ENVELOPE ---

#[wasm_bindgen]
pub fn envelope_wrap(record_js: JsValue, metadata_js: JsValue) -> Result<JsValue, JsValue> {
    let record: LnmpRecord = serde_wasm_bindgen::from_value(record_js)
        .map_err(|e| JsValue::from_str(&format!("Record error: {}", e)))?;
        
    let metadata: crate::envelope::EnvelopeMetadata = serde_wasm_bindgen::from_value(metadata_js)
        .map_err(|e| JsValue::from_str(&format!("Metadata error: {}", e)))?;
        
    let envelope = crate::envelope::LnmpEnvelope {
        record,
        metadata,
    };
    
    serde_wasm_bindgen::to_value(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn envelope_unwrap(envelope_js: JsValue) -> Result<JsValue, JsValue> {
    let envelope: crate::envelope::LnmpEnvelope = serde_wasm_bindgen::from_value(envelope_js)
        .map_err(|e| JsValue::from_str(&format!("Envelope error: {}", e)))?;
        
    serde_wasm_bindgen::to_value(&envelope.record)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// --- TRANSPORT (HTTP) ---

#[wasm_bindgen]
pub fn transport_to_http(envelope_js: JsValue) -> Result<JsValue, JsValue> {
    let envelope: crate::envelope::LnmpEnvelope = serde_wasm_bindgen::from_value(envelope_js)
        .map_err(|e| JsValue::from_str(&format!("Envelope error: {}", e)))?;
        
    let headers_map = crate::transport::http::envelope_to_headers(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Transport error: {}", e)))?;
        
    // Convert HeaderMap to JS Object
    let headers_obj = js_sys::Object::new();
    for (name, value) in headers_map.iter() {
        if let Ok(val_str) = value.to_str() {
            js_sys::Reflect::set(
                &headers_obj, 
                &JsValue::from_str(name.as_str()), 
                &JsValue::from_str(val_str)
            )?;
        }
    }
    
    Ok(headers_obj.into())
}

#[wasm_bindgen]
pub fn transport_from_http(headers_js: JsValue) -> Result<JsValue, JsValue> {
    // Convert JS Object to HeaderMap
    // This is a bit manual because we need to iterate keys
    let mut header_map = http::HeaderMap::new();
    
    let keys = js_sys::Object::keys(&headers_js.clone().into());
    for i in 0..keys.length() {
        let key = keys.get(i);
        let key_str = key.as_string().unwrap_or_default();
        
        let val = js_sys::Reflect::get(&headers_js, &key)?;
        let val_str = val.as_string().unwrap_or_default();
        
        if let (Ok(name), Ok(value)) = (
            http::HeaderName::from_bytes(key_str.as_bytes()),
            http::HeaderValue::from_str(&val_str)
        ) {
            header_map.insert(name, value);
        }
    }
    
    let metadata = crate::transport::http::headers_to_envelope_metadata(&header_map)
        .map_err(|e| JsValue::from_str(&format!("Transport error: {}", e)))?;
        
    serde_wasm_bindgen::to_value(&metadata)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// --- NET ---

#[wasm_bindgen]
pub fn network_decide(message_js: JsValue, now_ms: f64) -> Result<String, JsValue> {
    let message: crate::net::NetMessage = serde_wasm_bindgen::from_value(message_js)
        .map_err(|e| JsValue::from_str(&format!("Message error: {}", e)))?;
        
    let policy = crate::net::RoutingPolicy::default();
    let decision = policy.decide(&message, now_ms as u64)
        .map_err(|e| JsValue::from_str(&format!("Routing error: {}", e)))?;
        
    Ok(format!("{:?}", decision))
}

#[wasm_bindgen]
pub fn network_importance(message_js: JsValue, now_ms: f64) -> Result<f64, JsValue> {
    let message: crate::net::NetMessage = serde_wasm_bindgen::from_value(message_js)
        .map_err(|e| JsValue::from_str(&format!("Message error: {}", e)))?;
        
    let policy = crate::net::RoutingPolicy::default();
    policy.base_importance(&message, now_ms as u64)
        .map_err(|e| JsValue::from_str(&format!("Routing error: {}", e)))
}

// --- EMBEDDING ---

#[wasm_bindgen]
pub fn embedding_delta(base: Vec<f32>, updated: Vec<f32>) -> Result<JsValue, JsValue> {
    let base_vec = crate::embedding::Vector::from_f32(base);
    let updated_vec = crate::embedding::Vector::from_f32(updated);
    
    // Use from_vectors instead of compute, and provide a dummy base_id (0)
    let delta = crate::embedding::VectorDelta::from_vectors(&base_vec, &updated_vec, 0)
        .map_err(|e| JsValue::from_str(&format!("Delta error: {}", e)))?;
        
    serde_wasm_bindgen::to_value(&delta)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn embedding_apply_delta(base: Vec<f32>, delta_js: JsValue) -> Result<Vec<f32>, JsValue> {
    let base_vec = crate::embedding::Vector::from_f32(base);
    let delta: crate::embedding::VectorDelta = serde_wasm_bindgen::from_value(delta_js)
        .map_err(|e| JsValue::from_str(&format!("Delta error: {}", e)))?;
        
    // Fix argument order: apply(self, base) -> updated
    let updated_vec = delta.apply(&base_vec)
        .map_err(|e| JsValue::from_str(&format!("Apply error: {}", e)))?;
        
    // Convert back to f32 vec
    updated_vec.as_f32()
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

// --- SPATIAL ---

#[wasm_bindgen]
pub fn spatial_encode_frame(frame_js: JsValue) -> Result<Vec<u8>, JsValue> {
    let frame: crate::spatial::protocol::SpatialFrame = serde_wasm_bindgen::from_value(frame_js)
        .map_err(|e| JsValue::from_str(&format!("Frame error: {}", e)))?;
        
    // We don't have a direct "encode frame" function exposed in protocol.rs that takes a frame and returns bytes
    // But we can use bincode directly as the protocol does internally
    bincode::serialize(&frame)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn spatial_decode_frame(bytes: &[u8]) -> Result<JsValue, JsValue> {
    let frame: crate::spatial::protocol::SpatialFrame = bincode::deserialize(bytes)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true);
        
    frame.serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// --- QUANTIZATION ---

#[wasm_bindgen]
pub fn quantize_embedding(vector: Vec<f32>, scheme_id: u8) -> Result<JsValue, JsValue> {
    let vec = crate::embedding::Vector::from_f32(vector);
    
    let scheme = match scheme_id {
        0 => crate::quant::QuantScheme::QInt8,
        1 => crate::quant::QuantScheme::QInt4,
        2 => crate::quant::QuantScheme::Binary,
        3 => crate::quant::QuantScheme::FP16Passthrough,
        _ => return Err(JsValue::from_str("Invalid quantization scheme ID")),
    };
    
    let quantized = crate::quant::quantize_embedding(&vec, scheme)
        .map_err(|e| JsValue::from_str(&format!("Quantization error: {}", e)))?;
        
    serde_wasm_bindgen::to_value(&quantized)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn dequantize_embedding(quantized_js: JsValue) -> Result<Vec<f32>, JsValue> {
    let quantized: crate::quant::QuantizedVector = serde_wasm_bindgen::from_value(quantized_js)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
    let restored = crate::quant::dequantize_embedding(&quantized)
        .map_err(|e| JsValue::from_str(&format!("Dequantization error: {}", e)))?;
        
    restored.as_f32()
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

// --- SANITIZATION ---

#[wasm_bindgen]
pub fn sanitize_text(text: &str) -> String {
    crate::sanitize::sanitize_lnmp_text(text, &crate::sanitize::SanitizationConfig::default()).to_string()
}

// --- LLB (Large Language Blocks) ---

#[wasm_bindgen]
pub fn llb_explain(record_js: JsValue, dictionary_js: JsValue) -> Result<String, JsValue> {
    let record: crate::core::LnmpRecord = serde_wasm_bindgen::from_value(record_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid record: {}", e)))?;
    
    let dictionary: crate::llb::SemanticDictionary = serde_wasm_bindgen::from_value(dictionary_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid dictionary: {}", e)))?;

    let encoder = crate::llb::ExplainEncoder::new(dictionary);
    Ok(encoder.encode_with_explanation(&record))
}

// --- SFE (Secure Function Evaluation / Context Scoring) ---

#[wasm_bindgen]
pub fn sfe_score_context(envelope_js: JsValue, now: u64) -> Result<JsValue, JsValue> {
    let envelope: crate::envelope::LnmpEnvelope = serde_wasm_bindgen::from_value(envelope_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid envelope: {}", e)))?;

    let scorer = crate::sfe::context::ContextScorer::new();
    let profile = scorer.score_envelope(&envelope, now);

    Ok(serde_wasm_bindgen::to_value(&profile)?)
}
