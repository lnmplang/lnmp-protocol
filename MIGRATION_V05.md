# LNMP v0.4 to v0.5 Migration Guide

This guide helps you migrate from LNMP v0.4 to v0.5, which introduces production-grade M2M (machine-to-machine) transport capabilities.

## Overview of v0.5 Changes

LNMP v0.5 adds four major subsystems while maintaining full backward compatibility:

1. **Binary Nested Structures (BNS)** - Recursive encoding for hierarchical data
2. **Streaming Frame Layer (SFL)** - Chunked transmission for large payloads
3. **Schema Negotiation Layer (SNL)** - Capability exchange and version negotiation
4. **Delta Encoding & Partial Update Layer (DPL)** - Bandwidth-efficient incremental updates
5. **LLM Optimization Layer v2 (LLB2)** - Enhanced context optimization

## Backward Compatibility

**Good news:** v0.5 is fully backward compatible with v0.4 and v0.3!

- v0.5 decoders can parse v0.4 binary format
- v0.5 decoders can parse v0.3 text format
- v0.5 encoders can produce v0.4-compatible output (when nested features disabled)

## Migration Checklist

### 1. Update Dependencies

```toml
[dependencies]
lnmp-codec = "0.5"
lnmp-core = "0.5"
lnmp-llb = "0.5"  # If using LLB features
```

### 2. Review Breaking Changes

**None!** v0.5 has no breaking changes. All v0.4 code continues to work.

### 3. Opt-in to New Features

New features are opt-in through configuration:

```rust
use lnmp_codec::binary::{BinaryEncoder, EncoderConfig};

// v0.4 style (still works)
let encoder = BinaryEncoder::new();

// v0.5 style with new features
let config = EncoderConfig::new()
    .with_nested_binary(true)      // Enable nested structures
    .with_streaming_mode(true)     // Enable streaming
    .with_delta_mode(true);        // Enable delta encoding

let encoder = BinaryEncoder::with_config(config);
```

## Feature-by-Feature Migration

### Binary Nested Structures

**Before (v0.4):** Nested structures only in text format

```rust
// v0.4: Text format only
let text = "F10={F1=42;F2=test}";
let mut parser = Parser::new(text).unwrap();
let record = parser.parse_record().unwrap();
```

**After (v0.5):** Binary encoding for nested structures

```rust
use lnmp_codec::binary::{BinaryNestedEncoder, NestedEncoderConfig};

// v0.5: Binary nested encoding
let config = NestedEncoderConfig::new()
    .with_nested_binary(true)
    .with_max_depth(32);

let encoder = BinaryNestedEncoder::with_config(config);
let binary = encoder.encode(&record).unwrap();
```

**Migration tip:** Start with text format, then enable binary nested encoding when ready.

### Streaming Frame Layer

**Before (v0.4):** Send complete records

```rust
// v0.4: Send entire record at once
let encoder = BinaryEncoder::new();
let binary = encoder.encode(&large_record).unwrap();
// Send binary over network
```

**After (v0.5):** Stream large payloads in chunks

```rust
use lnmp_codec::binary::{StreamingEncoder, StreamingConfig};

// v0.5: Stream in chunks
let config = StreamingConfig::new()
    .with_chunk_size(4096)
    .with_checksums(true);

let mut streaming_encoder = StreamingEncoder::with_config(config);

// Begin stream
let begin_frame = streaming_encoder.begin_stream().unwrap();
send_frame(begin_frame);

// Send chunks
let chunk_frame = streaming_encoder.write_chunk(&binary).unwrap();
send_frame(chunk_frame);

// End stream
let end_frame = streaming_encoder.end_stream().unwrap();
send_frame(end_frame);
```

**Migration tip:** Use streaming for records > 4KB or when memory is constrained.

### Schema Negotiation

**Before (v0.4):** No capability negotiation

```rust
// v0.4: Assume compatibility
let encoder = BinaryEncoder::new();
let binary = encoder.encode(&record).unwrap();
```

**After (v0.5):** Negotiate capabilities first

```rust
use lnmp_codec::binary::{SchemaNegotiator, Capabilities, FeatureFlags};

// v0.5: Negotiate before sending data
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
    supported_types: vec![/* ... */],
};

let mut negotiator = SchemaNegotiator::new(caps);
let msg = negotiator.initiate().unwrap();
// Send negotiation message to peer
```

**Migration tip:** Add negotiation for new integrations; existing v0.4 peers work without it.

### Delta Encoding

**Before (v0.4):** Send full records for updates

```rust
// v0.4: Send complete updated record
let encoder = BinaryEncoder::new();
let updated_binary = encoder.encode(&updated_record).unwrap();
// Send entire record (wasteful for small changes)
```

**After (v0.5):** Send only changes

```rust
use lnmp_codec::binary::{DeltaEncoder, DeltaConfig};

// v0.5: Send only changed fields
let config = DeltaConfig::new().with_delta_enabled(true);
let delta_encoder = DeltaEncoder::with_config(config);

let delta_ops = delta_encoder.compute_delta(&old_record, &new_record).unwrap();
let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
// Send delta (50%+ bandwidth savings)
```

**Migration tip:** Use delta encoding for frequent updates to reduce bandwidth.

### LLB2 Enhancements

**Before (v0.4):** Basic LLB support

```rust
// v0.4: Basic LLB
use lnmp_llb::LlbConverter;

let converter = LlbConverter::default();
let shortform = converter.to_shortform(&record);
```

**After (v0.5):** Enhanced LLB with binary integration

```rust
use lnmp_llb::{LlbConverter, LlbConfig};

// v0.5: Enhanced LLB with flattening and hints
let config = LlbConfig::new()
    .with_flattening(true)
    .with_semantic_hints(true)
    .with_collision_safe_ids(true);

let converter = LlbConverter::new(config);

// Binary to ShortForm
let shortform = converter.binary_to_shortform(&binary).unwrap();

// Flatten nested structures
let flattened = converter.flatten_nested(&nested_record).unwrap();
```

**Migration tip:** Use flattening for nested structures sent to LLMs.

## Configuration Migration

### Encoder Configuration

**v0.4 EncoderConfig:**
```rust
let config = EncoderConfig::new()
    .with_validate_canonical(true)
    .with_sort_fields(true);
```

**v0.5 EncoderConfig (backward compatible):**
```rust
let config = EncoderConfig::new()
    .with_validate_canonical(true)
    .with_sort_fields(true)
    // New v0.5 options
    .with_nested_binary(true)
    .with_max_depth(32)
    .with_streaming_mode(false)
    .with_delta_mode(false)
    .with_chunk_size(4096);
```

### Decoder Configuration

**v0.4 DecoderConfig:**
```rust
let config = DecoderConfig::new()
    .with_validate_ordering(true)
    .with_strict_parsing(true);
```

**v0.5 DecoderConfig (backward compatible):**
```rust
let config = DecoderConfig::new()
    .with_validate_ordering(true)
    .with_strict_parsing(true)
    // New v0.5 options
    .with_allow_streaming(true)
    .with_validate_nesting(true)
    .with_allow_delta(true)
    .with_max_depth(32);
```

## Common Migration Patterns

### Pattern 1: Add Nested Structure Support

```rust
// Before: Flat records only
let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 1, value: LnmpValue::Int(42) });

// After: Nested records
let mut inner = LnmpRecord::new();
inner.add_field(LnmpField { fid: 1, value: LnmpValue::Int(42) });

let mut outer = LnmpRecord::new();
outer.add_field(LnmpField {
    fid: 10,
    value: LnmpValue::NestedRecord(Box::new(inner)),
});

// Encode with nested support
let config = NestedEncoderConfig::new().with_nested_binary(true);
let encoder = BinaryNestedEncoder::with_config(config);
let binary = encoder.encode(&outer).unwrap();
```

### Pattern 2: Add Streaming for Large Records

```rust
// Before: Memory-intensive for large records
let encoder = BinaryEncoder::new();
let binary = encoder.encode(&large_record).unwrap();

// After: Stream in chunks
let config = StreamingConfig::new().with_chunk_size(4096);
let mut streaming_encoder = StreamingEncoder::with_config(config);

streaming_encoder.begin_stream().unwrap();
streaming_encoder.write_chunk(&binary).unwrap();
streaming_encoder.end_stream().unwrap();
```

### Pattern 3: Add Delta Encoding for Updates

```rust
// Before: Send full record on every update
fn send_update(record: &LnmpRecord) {
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(record).unwrap();
    send_to_peer(binary);
}

// After: Send only changes
fn send_update(old: &LnmpRecord, new: &LnmpRecord) {
    let config = DeltaConfig::new().with_delta_enabled(true);
    let delta_encoder = DeltaEncoder::with_config(config);
    
    let delta_ops = delta_encoder.compute_delta(old, new).unwrap();
    let delta_binary = delta_encoder.encode_delta(&delta_ops).unwrap();
    send_to_peer(delta_binary);
}
```

## Performance Considerations

### When to Use Each Feature

| Feature | Use When | Benefit |
|---------|----------|---------|
| Binary Nested Structures | Hierarchical data | Efficient encoding of complex structures |
| Streaming | Records > 4KB | Reduced memory usage, backpressure control |
| Schema Negotiation | New integrations | Early compatibility detection |
| Delta Encoding | Frequent updates | 50%+ bandwidth savings |
| LLB2 Flattening | Sending to LLMs | Optimal token usage |

### Performance Targets

- **Nested Encoding:** < 2μs per field
- **Streaming Overhead:** < 5% vs non-streaming
- **Delta Savings:** > 50% for typical updates
- **Negotiation Latency:** < 10ms

## Error Handling

### New Error Types in v0.5

```rust
use lnmp_codec::binary::{BinaryError, StreamingError, NegotiationError, DeltaError};

// Nested structure errors
match encoder.encode(&record) {
    Err(BinaryError::NestingDepthExceeded { depth, max }) => {
        eprintln!("Nesting too deep: {} > {}", depth, max);
    }
    Err(BinaryError::RecordSizeExceeded { size, max }) => {
        eprintln!("Record too large: {} > {}", size, max);
    }
    _ => {}
}

// Streaming errors
match decoder.feed_frame(&frame) {
    Err(StreamingError::ChecksumMismatch { expected, found }) => {
        eprintln!("Checksum error: expected {:08x}, found {:08x}", expected, found);
    }
    _ => {}
}

// Negotiation errors
match negotiator.handle_message(&msg) {
    Err(NegotiationError::FidConflict { fid, name1, name2 }) => {
        eprintln!("FID {} conflict: {} vs {}", fid, name1, name2);
    }
    _ => {}
}
```

## Testing Your Migration

### 1. Test Backward Compatibility

```rust
#[test]
fn test_v04_compatibility() {
    // Encode with v0.4 style
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    
    // Decode with v0.5
    let decoder = BinaryDecoder::new();
    let decoded = decoder.decode(&binary).unwrap();
    
    assert_eq!(record, decoded);
}
```

### 2. Test New Features

```rust
#[test]
fn test_nested_binary() {
    let config = NestedEncoderConfig::new().with_nested_binary(true);
    let encoder = BinaryNestedEncoder::with_config(config);
    let binary = encoder.encode(&nested_record).unwrap();
    
    let decoder_config = NestedDecoderConfig::new().with_allow_nested(true);
    let decoder = BinaryNestedDecoder::with_config(decoder_config);
    let decoded = decoder.decode(&binary).unwrap();
    
    assert_eq!(nested_record, decoded);
}
```

### 3. Test Round-Trip Stability

```rust
#[test]
fn test_roundtrip_stability() {
    let binary1 = encoder.encode(&record).unwrap();
    let decoded = decoder.decode(&binary1).unwrap();
    let binary2 = encoder.encode(&decoded).unwrap();
    
    assert_eq!(binary1, binary2);
}
```

## Troubleshooting

### Issue: "NestingDepthExceeded" error

**Solution:** Increase max_depth or flatten your structure

```rust
let config = NestedEncoderConfig::new()
    .with_nested_binary(true)
    .with_max_depth(64); // Increase from default 32
```

### Issue: "ChecksumMismatch" in streaming

**Solution:** Check for data corruption or disable checksums for debugging

```rust
let config = StreamingConfig::new()
    .with_checksums(false); // Disable for debugging
```

### Issue: "FidConflict" during negotiation

**Solution:** Align FID mappings between client and server

```rust
// Ensure both sides use same FID mappings
let mut fid_mappings = HashMap::new();
fid_mappings.insert(12, "user_id".to_string());
negotiator.set_fid_mappings(fid_mappings);
```

## Resources

- **Examples:** See `examples/v05_*.rs` for working code
- **API Docs:** Run `cargo doc --open` for detailed API documentation
- **Spec:** See `.kiro/specs/lnmp-v0.5-advanced-protocol/` for design details
- **Tests:** See `crates/lnmp-codec/tests/v05_*.rs` for test examples

## Getting Help

If you encounter issues during migration:

1. Check the examples in `examples/v05_*.rs`
2. Review the test suite in `crates/lnmp-codec/tests/`
3. Consult the design document in `.kiro/specs/lnmp-v0.5-advanced-protocol/design.md`
4. File an issue on GitHub with your use case

## Summary

v0.5 is a **non-breaking upgrade** that adds powerful new features:

- ✅ All v0.4 code works without changes
- ✅ New features are opt-in via configuration
- ✅ Backward compatible with v0.4 and v0.3
- ✅ Production-grade M2M transport capabilities
- ✅ GRPC-level performance

Start by updating dependencies, then gradually adopt new features as needed!
