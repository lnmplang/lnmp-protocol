# LNMP Embedding Delta Mode

## Overview

Delta encoding for embedding vectors enables efficient incremental updates by transmitting only the changed values instead of the full vector. This optimization is critical for AI-native applications with frequent small updates.

## Performance

### Size Reduction

| Change % | Full Size (1536-dim) | Delta Size | Reduction |
|----------|---------------------|------------|-----------|
| 1%       | 6,148 bytes        | 94 bytes   | **98.5%** |
| 5%       | 6,148 bytes        | 460 bytes  | **92.5%** |
| 10%      | 6,148 bytes        | 922 bytes  | **85.0%** |
| 30%      | 6,148 bytes        | 2,764 bytes| **55.0%** |

### Real-World Impact

**Scenario**: 30 AI agents, each sending 10 updates/second

- **Without Delta**: 30 × 10 × 6KB = **1.8 MB/s**
- **With Delta (1% changes)**: 30 × 10 × 94 bytes = **28 KB/s**
- **Bandwidth Savings**: **98.4%**

## Usage

### Basic Delta Operations

```rust
use lnmp_embedding::{Vector, VectorDelta};

// Create base and updated embeddings
let old_embedding = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4]);
let new_embedding = Vector::from_f32(vec![0.1, 0.25, 0.3, 0.35]);

// Compute delta
let delta = VectorDelta::from_vectors(&old_embedding, &new_embedding, 1001)?;

// Encode delta (small payload)
let encoded = delta.encode()?;
println!("Delta size: {} bytes", encoded.len()); // Much smaller than full vector

// On receiving end: decode and apply
let decoded = VectorDelta::decode(&encoded)?;
let reconstructed = decoded.apply(&old_embedding)?;

assert_eq!(new_embedding, reconstructed);
```

### Adaptive Strategy

```rust
use lnmp_embedding::UpdateStrategy;

// Create adaptive strategy with 30% threshold
let strategy = UpdateStrategy::Adaptive { threshold: 30 };

let delta = VectorDelta::from_vectors(&old, &new, 1)?;

if strategy.should_use_delta(&delta, old.dim) {
    // Send delta (efficient for small changes)
    send_delta(delta.encode()?);
} else {
    // Send full vector (more efficient for large changes)
    send_full(Encoder::encode(&new)?);
}
```

### Streaming Updates

```rust
// Agent with evolving context
let mut current_state = Vector::from_f32(initial_embedding);
let base_id = 1001;

loop {
    // Agent updates context slightly
    let new_state = agent.update_context();
    
    // Compute and send delta
    let delta = VectorDelta::from_vectors(&current_state, &new_state, base_id)?;
    stream.send_delta(delta.encode()?).await?;
    
    // Update current state
    current_state = new_state;
}
```

## When to Use Delta

### ✅ Use Delta For:

1. **Streaming Reasoning**: Incremental context updates during agent thinking
2. **Multi-Agent Sync**: Coordinated state updates across many agents
3. **Real-time Systems**: Robotics, gaming, live visualization
4. **Online Learning**: Gradual model weight updates

### ❌ Use Full Vector For:

1. **Initial Transmission**: First embedding in a session
2. **Complete Context Switch**: Major semantic change
3. **Checkpoints**: Guaranteed consistency points
4. **Large Changes**: >30% of values modified (delta not efficient)

## Delta Format

```
┌─────────────────────────────────────┐
│ base_id: u16        (2 bytes)       │
│ change_count: u16   (2 bytes)       │
│ ┌─────────────────────────────────┐ │
│ │ index: u16       (2 bytes)      │ │
│ │ delta: f32       (4 bytes)      │ │
│ └─────────────────────────────────┘ │
│ ... repeated change_count times     │
└─────────────────────────────────────┘

Total size: 4 + (6 × change_count) bytes
```

## API Reference

### VectorDelta

```rust
pub struct VectorDelta {
    pub base_id: u16,
    pub changes: Vec<DeltaChange>,
}

impl VectorDelta {
    // Compute delta between two vectors
    pub fn from_vectors(old: &Vector, new: &Vector, base_id: u16) -> Result<Self, String>;
    
    // Apply delta to base vector
    pub fn apply(&self, base: &Vector) -> Result<Vector, String>;
    
    // Encode/decode delta
    pub fn encode(&self) -> Result<Vec<u8>, std::io::Error>;
    pub fn decode(data: &[u8]) -> Result<Self, std::io::Error>;
    
    // Utility methods
    pub fn encoded_size(&self) -> usize;
    pub fn change_ratio(&self, total_dim: u16) -> f32;
    pub fn is_beneficial(&self, full_vector_size: usize) -> bool;
}
```

### UpdateStrategy

```rust
pub enum UpdateStrategy {
    AlwaysFull,                          // Never use delta
    AlwaysDelta,                         // Always use delta
    Adaptive { threshold: u8 },          // Use delta if change < threshold%
}

impl UpdateStrategy {
    pub fn should_use_delta(&self, delta: &VectorDelta, vector_dim: u16) -> bool;
}
```

## Examples

See [`examples/delta_demo.rs`](../../examples/examples/delta_demo.rs) for comprehensive demonstrations of:
- Basic delta operations
- Size comparisons across dimensions
- Streaming agent updates
- Adaptive strategy usage

## Performance Characteristics

- **Delta Computation**: O(n) where n = vector dimension
- **Delta Encoding**: O(k) where k = number of changes
- **Delta Application**: O(k) where k = number of changes
- **Memory**: Minimal overhead, no temporary allocations

## Integration with LNMP Protocol

Delta embeddings can be transmitted using LNMP Mode 0x04 (Delta) or embedded within existing container modes. The compact binary format ensures efficient wire transmission and minimal parsing overhead.
