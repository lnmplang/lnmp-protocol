import * as wasm from '../wasm/lnmp.js';

// Re-export all WASM functions
export const {
    parse_lnmp,
    encode_lnmp,
    compute_checksum,
    format_checksum,
    parse_checksum,
    lnmp_version,
    // Envelope
    envelope_wrap,
    envelope_unwrap,
    transport_to_http,
    transport_from_http,
    // Net
    network_decide,
    network_importance,
    // Embedding
    embedding_delta,
    embedding_apply_delta,
    // Quantization
    quantize_embedding,
    dequantize_embedding,
    // Sanitization
    sanitize_text,
    // LLB
    llb_explain,
    // SFE
    sfe_score_context,
    // Spatial
    spatial_encode_frame,
    spatial_decode_frame
} = wasm;

// --- Core Types ---

export type FieldId = number;

export interface LnmpValue {
    Int?: number;
    Float?: number;
    Bool?: boolean;
    String?: string;
    Bytes?: number[];
    Array?: LnmpValue[];
    Record?: LnmpRecord;
    // Add other variants as needed
}

export interface LnmpField {
    fid: FieldId;
    value: LnmpValue;
}

export interface LnmpRecord {
    fields: LnmpField[];
}

// --- Envelope Types ---

export interface EnvelopeMetadata {
    timestamp?: number; // u64 as number (safe for < 2^53)
    source?: string;
    trace_id?: string;
    sequence?: number;
    labels: Record<string, string>;
}

export interface LnmpEnvelope {
    record: LnmpRecord;
    metadata: EnvelopeMetadata;
}

// --- Net Types ---

export enum MessageKind {
    Event = "Event",
    State = "State",
    Command = "Command",
    Query = "Query",
    Alert = "Alert"
}

export interface NetMessage {
    envelope: LnmpEnvelope;
    kind: MessageKind;
    priority: number; // u8
    ttl_ms: number; // u32
}

export type RoutingDecision = "SendToLLM" | "ProcessLocally" | "Drop";

// --- Embedding Types ---

export interface VectorDelta {
    base_id: number; // u16
    changes: DeltaChange[];
}

export interface DeltaChange {
    index: number; // u16
    value: number; // f32
}

// --- Quantization Types ---

export enum QuantScheme {
    QInt8 = 0,
    QInt4 = 1,
    Binary = 2,
    FP16 = 3
}

export interface QuantizedVector {
    scheme: number; // u8 (QuantScheme)
    dim: number; // u16
    data: number[]; // Vec<u8>
    min?: number; // f32
    max?: number; // f32
}

// --- SFE Types ---

export interface ContextProfile {
    freshness_score: number; // f64
    importance: number; // u8
    risk_level: string; // Enum string
    confidence: number; // f64
    llm_hints: Record<string, string>;
}

// --- Spatial Types ---

export interface Position3D {
    x: number;
    y: number;
    z: number;
}

export interface Velocity3D {
    vx: number;
    vy: number;
    vz: number;
}

export interface Orientation {
    roll: number;
    pitch: number;
    yaw: number;
}

export interface SpatialState {
    position?: Position3D;
    velocity?: Velocity3D;
    orientation?: Orientation;
}

export enum FrameMode {
    Absolute = "Absolute",
    Delta = "Delta"
}

export interface SpatialFrameHeader {
    mode: FrameMode;
    sequence_id: number;
    timestamp: number;
    checksum: number;
}

export interface SpatialFrame {
    header: SpatialFrameHeader;
    payload: any; // SpatialValue (complex enum, keeping as any for now)
}
