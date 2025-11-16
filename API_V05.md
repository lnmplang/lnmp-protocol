# LNMP v0.5 API Documentation

Complete API reference for LNMP v0.5 Advanced Protocol features.

## Table of Contents

1. [Binary Nested Structures](#binary-nested-structures)
2. [Streaming Frame Layer](#streaming-frame-layer)
3. [Schema Negotiation Layer](#schema-negotiation-layer)
4. [Delta Encoding Layer](#delta-encoding-layer)
5. [LLB2 Optimization Layer](#llb2-optimization-layer)
6. [Configuration Options](#configuration-options)
7. [Error Types](#error-types)

---

## Binary Nested Structures

### BinaryNestedEncoder

Encodes LNMP records with nested structures to binary format.

```rust
pub struct BinaryNestedEncoder {
    config: NestedEncoderConfig,
}

impl BinaryNestedEncoder {
    pub fn new() -> Self
    pub fn with_config(config: NestedEncoderConfig) -> Self
    pub fn encode(&self, record: &LnmpRecord) -> Result<Vec<u8>, BinaryError>
    pub fn encode_nested_record(&self, record: &LnmpRecord) -> Result<Vec<u8>, BinaryError>
    pub fn encode_nested_array(&self, records: &[LnmpRecord]) -> Result<Vec<u8>, BinaryError>
}
```

**Example:**
```rust
use lnmp_codec::binary::{BinaryNestedEncoder, NestedEncoderConfig};

let config = NestedEncoderConfig::new()
    .with_nested_binary(true)
    .with_max_depth(32);

let encoder = BinaryNestedEncoder::with_config(config);
let binary = encoder.encode(&nested_record)?;
```

### BinaryNestedDecoder

Decodes binary format with nested structures back to LNMP records.

```rust
pub struct BinaryNestedDecoder {
    config: NestedDecoderConfig,
}

impl BinaryNestedDecoder {
    pub fn new() -> Self
    pub fn with_config(config: NestedDecoderConfig) -> Self
    pub fn decode(&self, bytes: &[u8]) -> Result<LnmpRecord, BinaryError>
    pub fn decode_nested_record(&self, bytes: &[u8]) -> Result<LnmpRecord, BinaryError>
    pub fn decode_nested_array(&self, bytes: &[u8]) -> Result<Vec<LnmpRecord>, BinaryError>
}
```

**Example:**
```rust
use lnmp_codec::binary::{BinaryNestedDecoder, NestedDecoderConfig};

let config = NestedDecoderConfig::new()
    .with_allow_nested(true)
    .with_max_depth(32);

let decoder = BinaryNestedDecoder::with_config(config);
let record = decoder.decode(&binary)?;
```

### NestedEncoderConfig

Configuration for nested structure encoding.

```rust
pub struct NestedEncoderConfig {
    pub enable_nested_binary: bool,
    pub max_depth: usize,
    pub max_record_size: Option<usize>,
    pub validate_canonical: bool,
}

impl NestedEncoderConfig {
    pub fn new() -> Self
    pub fn with_nested_binary(mut self, enable: bool) -> Self
    pub fn with_max_depth(mut self, depth: usize) -> Self
    pub fn with_max_record_size(mut self, size: Option<usize>) -> Self
    pub fn with_canonical(mut self, canonical: bool) -> Self
}
```

**Defaults:**
- `enable_nested_binary`: `false`
- `max_depth`: `32`
- `max_record_size`: `None` (unlimited)
- `validate_canonical`: `true`

### NestedDecoderConfig

Configuration for nested structure decoding.

```rust
pub struct NestedDecoderConfig {
    pub allow_nested: bool,
    pub validate_nesting: bool,
    pub max_depth: usize,
}

impl NestedDecoderConfig {
    pub fn new() -> Self
    pub fn with_allow_nested(mut self, allow: bool) -> Self
    pub fn with_validate_nesting(mut self, validate: bool) -> Self
    pub fn with_max_depth(mut self, depth: usize) -> Self
}
```

**Defaults:**
- `allow_nested`: `false`
- `validate_nesting`: `true`
- `max_depth`: `32`

---

## Streaming Frame Layer

### StreamingEncoder

Encodes data into streaming frames for chunked transmission.

```rust
pub struct StreamingEncoder {
    config: StreamingConfig,
    state: StreamingState,
}

impl StreamingEncoder {
    pub fn new() -> Self
    pub fn with_config(config: StreamingConfig) -> Self
    pub fn begin_stream(&mut self) -> Result<Vec<u8>, StreamingError>
    pub fn write_chunk(&mut self, data: &[u8]) -> Result<Vec<u8>, StreamingError>
    pub fn end_stream(&mut self) -> Result<Vec<u8>, StreamingError>
    pub fn error_frame(&mut self, error: &str) -> Result<Vec<u8>, StreamingError>
}
```

**Example:**
```rust
use lnmp_codec::binary::{StreamingEncoder, StreamingConfig};

let config = StreamingConfig::new()
    .with_chunk_size(4096)
    .with_checksums(true);

let mut encoder = StreamingEncoder::with_config(config);

let begin = encoder.begin_stream()?;
let chunk = encoder.write_chunk(&data)?;
let end = encoder.end_stream()?;
```

### StreamingDecoder

Decodes streaming frames and reassembles data.

```rust
pub struct StreamingDecoder {
    config: StreamingConfig,
    state: StreamingState,
    buffer: Vec<u8>,
}

impl StreamingDecoder {
    pub fn new() -> Self
    pub fn with_config(config: StreamingConfig) -> Self
    pub fn feed_frame(&mut self, frame: &[u8]) -> Result<StreamingEvent, StreamingError>
    pub fn get_complete_payload(&self) -> Option<&[u8]>
    pub fn reset(&mut self)
}
```

**Example:**
```rust
use lnmp_codec::binary::{StreamingDecoder, StreamingEvent};

let mut decoder = StreamingDecoder::new();

match decoder.feed_frame(&frame)? {
    StreamingEvent::StreamStarted => println!("Stream started"),
    StreamingEvent::ChunkReceived { bytes } => println!("Received {} bytes", bytes),
    StreamingEvent::StreamComplete { total_bytes } => {
        println!("Complete: {} bytes", total_bytes);
        if let Some(payload) = decoder.get_complete_payload() {
            // Process complete payload
        }
    }
    StreamingEvent::StreamError { message } => eprintln!("Error: {}", message),
}
```

### StreamingConfig

Configuration for streaming operations.

```rust
pub struct StreamingConfig {
    pub chunk_size: usize,
    pub enable_compression: bool,
    pub enable_checksums: bool,
}

impl StreamingConfig {
    pub fn new() -> Self
    pub fn with_chunk_size(mut self, size: usize) -> Self
    pub fn with_compression(mut self, enable: bool) -> Self
    pub fn with_checksums(mut self, enable: bool) -> Self
}
```

**Defaults:**
- `chunk_size`: `4096` (4KB)
- `enable_compression`: `false`
- `enable_checksums`: `true`

### BackpressureController

Controls flow to prevent overwhelming receivers.

```rust
pub struct BackpressureController {
    window_size: usize,
    bytes_in_flight: usize,
}

impl BackpressureController {
    pub fn new(window_size: usize) -> Self
    pub fn can_send(&self) -> bool
    pub fn on_chunk_sent(&mut self, size: usize)
    pub fn on_chunk_acked(&mut self, size: usize)
    pub fn bytes_in_flight(&self) -> usize
}
```

**Example:**
```rust
use lnmp_codec::binary::BackpressureController;

let mut controller = BackpressureController::new(8192); // 8KB window

if controller.can_send() {
    send_chunk(&data);
    controller.on_chunk_sent(data.len());
}

// When acknowledgment received
controller.on_chunk_acked(data.len());
```

### Frame Types

```rust
pub enum FrameType {
    Begin = 0xA0,    // Start of stream
    Chunk = 0xA1,    // Data chunk
    End = 0xA2,      // End of stream
    Error = 0xA3,    // Error frame
}
```

### Streaming Events

```rust
pub enum StreamingEvent {
    StreamStarted,
    ChunkReceived { bytes: usize },
    StreamComplete { total_bytes: usize },
    StreamError { message: String },
}
```

---

## Schema Negotiation Layer

### SchemaNegotiator

Manages capability negotiation between peers.

```rust
pub struct SchemaNegotiator {
    local_capabilities: Capabilities,
    remote_capabilities: Option<Capabilities>,
    state: NegotiationState,
}

impl SchemaNegotiator {
    pub fn new(capabilities: Capabilities) -> Self
    pub fn initiate(&mut self) -> Result<Vec<u8>, NegotiationError>
    pub fn handle_message(&mut self, msg: &[u8]) -> Result<NegotiationResponse, NegotiationError>
    pub fn is_ready(&self) -> bool
    pub fn get_session(&self) -> Option<&NegotiationSession>
    pub fn set_fid_mappings(&mut self, mappings: HashMap<u16, String>)
    pub fn set_type_mappings(&mut self, mappings: HashMap<u16, TypeTag>)
}
```

**Example:**
```rust
use lnmp_codec::binary::{SchemaNegotiator, Capabilities, FeatureFlags};

let features = FeatureFlags {
    supports_nested: true,
    supports_streaming: true,
    supports_delta: true,
    supports_llb: true,
    requires_checksums: true,
    requires_canonical: true,
};

let caps = Capabilities {
    version: 5,
    features,
    supported_types: vec![TypeTag::Int, TypeTag::String],
};

let mut negotiator = SchemaNegotiator::new(caps);

// Client initiates
let msg = negotiator.initiate()?;
send_to_peer(msg);

// Handle response
match negotiator.handle_message(&response)? {
    NegotiationResponse::SendMessage(msg) => send_to_peer(msg),
    NegotiationResponse::Complete => println!("Negotiation complete"),
    NegotiationResponse::Failed(err) => eprintln!("Failed: {}", err),
}
```

### Capabilities

Describes protocol capabilities of a peer.

```rust
pub struct Capabilities {
    pub version: u8,
    pub features: FeatureFlags,
    pub supported_types: Vec<TypeTag>,
}
```

### FeatureFlags

Optional protocol features.

```rust
pub struct FeatureFlags {
    pub supports_nested: bool,
    pub supports_streaming: bool,
    pub supports_delta: bool,
    pub supports_llb: bool,
    pub requires_checksums: bool,
    pub requires_canonical: bool,
}

impl FeatureFlags {
    pub fn default() -> Self
    pub fn all() -> Self
    pub fn minimal() -> Self
}
```

### NegotiationState

```rust
pub enum NegotiationState {
    Initial,
    CapabilitiesSent,
    CapabilitiesReceived,
    SchemaSelected,
    Ready,
    Failed(String),
}
```

### NegotiationResponse

```rust
pub enum NegotiationResponse {
    SendMessage(Vec<u8>),
    Complete,
    Failed(String),
}
```

---

## Delta Encoding Layer

### DeltaEncoder

Computes and encodes deltas between records.

```rust
pub struct DeltaEncoder {
    config: DeltaConfig,
}

impl DeltaEncoder {
    pub fn new() -> Self
    pub fn with_config(config: DeltaConfig) -> Self
    pub fn compute_delta(&self, old: &LnmpRecord, new: &LnmpRecord) 
        -> Result<Vec<DeltaOp>, DeltaError>
    pub fn encode_delta(&self, ops: &[DeltaOp]) -> Result<Vec<u8>, DeltaError>
}
```

**Example:**
```rust
use lnmp_codec::binary::{DeltaEncoder, DeltaConfig};

let config = DeltaConfig::new().with_delta_enabled(true);
let encoder = DeltaEncoder::with_config(config);

// Compute delta
let delta_ops = encoder.compute_delta(&old_record, &new_record)?;

// Encode delta
let delta_binary = encoder.encode_delta(&delta_ops)?;
```

### DeltaDecoder

Decodes and applies delta operations.

```rust
pub struct DeltaDecoder {
    config: DeltaConfig,
}

impl DeltaDecoder {
    pub fn new() -> Self
    pub fn with_config(config: DeltaConfig) -> Self
    pub fn decode_delta(&self, bytes: &[u8]) -> Result<Vec<DeltaOp>, DeltaError>
    pub fn apply_delta(&self, base: &mut LnmpRecord, ops: &[DeltaOp]) 
        -> Result<(), DeltaError>
}
```

**Example:**
```rust
use lnmp_codec::binary::DeltaDecoder;

let decoder = DeltaDecoder::new();

// Decode delta
let ops = decoder.decode_delta(&delta_binary)?;

// Apply to base record
let mut current = base_record.clone();
decoder.apply_delta(&mut current, &ops)?;
```

### DeltaConfig

Configuration for delta encoding.

```rust
pub struct DeltaConfig {
    pub enable_delta: bool,
    pub track_changes: bool,
}

impl DeltaConfig {
    pub fn new() -> Self
    pub fn with_delta_enabled(mut self, enable: bool) -> Self
    pub fn with_track_changes(mut self, track: bool) -> Self
}
```

### DeltaOperation

Types of delta operations.

```rust
pub enum DeltaOperation {
    SetField = 0x01,      // Set field value
    DeleteField = 0x02,   // Remove field
    UpdateField = 0x03,   // Modify existing field
    MergeRecord = 0x04,   // Merge nested record
}
```

### DeltaOp

Represents a single delta operation.

```rust
pub struct DeltaOp {
    pub target_fid: u16,
    pub operation: DeltaOperation,
    pub payload: Vec<u8>,
}
```

---

## LLB2 Optimization Layer

### LlbConverter

Converts between formats and optimizes for LLM consumption.

```rust
pub struct LlbConverter {
    config: LlbConfig,
}

impl LlbConverter {
    pub fn new(config: LlbConfig) -> Self
    pub fn default() -> Self
    
    // Format conversions
    pub fn binary_to_shortform(&self, binary: &[u8]) -> Result<String, LlbError>
    pub fn shortform_to_binary(&self, shortform: &str) -> Result<Vec<u8>, LlbError>
    pub fn binary_to_fulltext(&self, binary: &[u8]) -> Result<String, LlbError>
    pub fn fulltext_to_binary(&self, fulltext: &str) -> Result<Vec<u8>, LlbError>
    
    // Flattening
    pub fn flatten_nested(&self, record: &LnmpRecord) -> Result<LnmpRecord, LlbError>
    pub fn unflatten(&self, flat: &LnmpRecord) -> Result<LnmpRecord, LlbError>
    
    // Semantic hints
    pub fn add_semantic_hints(&self, record: &LnmpRecord, hints: &HashMap<u16, String>) 
        -> String
    
    // ID generation
    pub fn generate_short_ids(&self, field_names: &[String]) 
        -> Result<HashMap<String, String>, LlbError>
}
```

**Example:**
```rust
use lnmp_llb::{LlbConverter, LlbConfig};

let config = LlbConfig::new()
    .with_flattening(true)
    .with_semantic_hints(true)
    .with_collision_safe_ids(true);

let converter = LlbConverter::new(config);

// Binary to ShortForm
let shortform = converter.binary_to_shortform(&binary)?;

// Flatten nested
let flattened = converter.flatten_nested(&nested_record)?;

// Add hints
let mut hints = HashMap::new();
hints.insert(12, "user_id".to_string());
let with_hints = converter.add_semantic_hints(&record, &hints);
```

### LlbConfig

Configuration for LLB optimization.

```rust
pub struct LlbConfig {
    pub enable_flattening: bool,
    pub enable_semantic_hints: bool,
    pub collision_safe_ids: bool,
}

impl LlbConfig {
    pub fn new() -> Self
    pub fn with_flattening(mut self, enable: bool) -> Self
    pub fn with_semantic_hints(mut self, enable: bool) -> Self
    pub fn with_collision_safe_ids(mut self, enable: bool) -> Self
}
```

---

## Configuration Options

### Complete EncoderConfig (v0.5)

```rust
pub struct EncoderConfig {
    // v0.4 options
    pub validate_canonical: bool,
    pub sort_fields: bool,
    
    // v0.5 options
    pub enable_nested_binary: bool,
    pub max_depth: usize,
    pub streaming_mode: bool,
    pub delta_mode: bool,
    pub chunk_size: usize,
}
```

### Complete DecoderConfig (v0.5)

```rust
pub struct DecoderConfig {
    // v0.4 options
    pub validate_ordering: bool,
    pub strict_parsing: bool,
    
    // v0.5 options
    pub allow_streaming: bool,
    pub validate_nesting: bool,
    pub allow_delta: bool,
    pub max_depth: usize,
}
```

---

## Error Types

### BinaryError (Extended for v0.5)

```rust
pub enum BinaryError {
    // v0.4 errors
    UnsupportedVersion { found: u8, supported: Vec<u8> },
    InvalidVarInt { reason: String },
    UnexpectedEof { expected: usize, found: usize },
    
    // v0.5 errors
    NestingDepthExceeded { depth: usize, max: usize },
    NestedStructureNotSupported,
    RecordSizeExceeded { size: usize, max: usize },
    InvalidNestedStructure { reason: String },
}
```

### StreamingError

```rust
pub enum StreamingError {
    InvalidFrameType { found: u8 },
    ChecksumMismatch { expected: u32, found: u32 },
    UnexpectedFrame { expected: FrameType, found: FrameType },
    StreamNotStarted,
    StreamAlreadyComplete,
    ChunkSizeExceeded { size: usize, max: usize },
}
```

### NegotiationError

```rust
pub enum NegotiationError {
    FidConflict { fid: u16, name1: String, name2: String },
    TypeMismatch { fid: u16, expected: TypeTag, found: TypeTag },
    UnsupportedFeature { feature: String },
    ProtocolVersionMismatch { local: u8, remote: u8 },
    InvalidState { current: NegotiationState, expected: NegotiationState },
}
```

### DeltaError

```rust
pub enum DeltaError {
    InvalidTargetFid { fid: u16 },
    InvalidOperation { op_code: u8 },
    MergeConflict { fid: u16, reason: String },
    DeltaApplicationFailed { reason: String },
}
```

### LlbError

```rust
pub enum LlbError {
    InvalidFormat { reason: String },
    FlatteningFailed { reason: String },
    UnflatteningFailed { reason: String },
    IdGenerationFailed { reason: String },
}
```

---

## Type Tags

### Complete TypeTag Enum (v0.5)

```rust
pub enum TypeTag {
    // v0.4 types
    Int = 0x01,
    Float = 0x02,
    Bool = 0x03,
    String = 0x04,
    StringArray = 0x05,
    
    // v0.5 types
    NestedRecord = 0x06,
    NestedArray = 0x07,
    
    // Reserved for future
    Reserved08 = 0x08,
    Reserved09 = 0x09,
    Reserved0A = 0x0A,
    Reserved0B = 0x0B,
    Reserved0C = 0x0C,
    Reserved0D = 0x0D,
    Reserved0E = 0x0E,
    Reserved0F = 0x0F,
}
```

---

## Best Practices

### 1. Nested Structures

```rust
// ✓ Good: Set appropriate depth limit
let config = NestedEncoderConfig::new()
    .with_nested_binary(true)
    .with_max_depth(10); // Reasonable limit

// ✗ Bad: Unlimited depth (security risk)
let config = NestedEncoderConfig::new()
    .with_nested_binary(true)
    .with_max_depth(usize::MAX);
```

### 2. Streaming

```rust
// ✓ Good: Use streaming for large records
if record_size > 4096 {
    use_streaming_encoder();
}

// ✗ Bad: Stream tiny records (overhead)
if record_size < 100 {
    use_streaming_encoder(); // Wasteful
}
```

### 3. Delta Encoding

```rust
// ✓ Good: Use delta for frequent updates
if is_update && has_base_record {
    use_delta_encoding();
}

// ✗ Bad: Use delta for first transmission
if is_first_transmission {
    use_delta_encoding(); // No base to delta against
}
```

### 4. Schema Negotiation

```rust
// ✓ Good: Negotiate before sending data
negotiate_capabilities();
send_data();

// ✗ Bad: Assume compatibility
send_data(); // May fail if incompatible
```

---

## Performance Tips

1. **Reuse encoders/decoders** - They're designed for reuse
2. **Batch operations** - Encode multiple records before streaming
3. **Use appropriate chunk sizes** - 4KB default is good for most cases
4. **Enable checksums selectively** - Only when data integrity is critical
5. **Profile before optimizing** - Measure actual performance impact

---

## See Also

- [Migration Guide](MIGRATION_V05.md) - Upgrading from v0.4
- [Examples](examples/) - Working code examples
- [Design Document](.kiro/specs/lnmp-v0.5-advanced-protocol/design.md) - Architecture details
- [Requirements](.kiro/specs/lnmp-v0.5-advanced-protocol/requirements.md) - Feature requirements
