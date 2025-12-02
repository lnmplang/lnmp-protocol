# LNMP-Spatial

Spatial awareness types and hybrid protocol for the LNMP ecosystem, enabling deterministic physical-world interaction in LLM → Machine → Robot → Simulation chains.

## Features

- 🎯 **Core Spatial Types**: Position, Rotation, Velocity, Acceleration, Quaternion, BoundingBox
- 📦 **Binary Codec**: Efficient encoding/decoding (2-3ns latency)
- 🔄 **Delta Encoding**: 99% bandwidth reduction for incremental updates
- 🌊 **Streaming Support**: Continuous telemetry transmission
- 🏗️ **Hybrid Protocol**: Automatic ABS/DELTA mixing for robustness
- 🔮 **Predictive Delta**: Dead reckoning for packet loss resilience  
- 🛡️ **Frame Integrity**: CRC32 checksums and nanosecond timestamps
- ⚡ **High Frequency**: Verified at 1kHz control loops

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lnmp-spatial = { path = "../lnmp-protocol/crates/lnmp-spatial" }
```

### Basic Usage

```rust
use lnmp_spatial::*;

// Create a position
let pos = Position3D { x: 10.0, y: 20.0, z: 30.0 };

// Encode to binary
let mut buffer = Vec::new();
encode_spatial(&SpatialValue::S2(pos), &mut buffer)?;

// Decode from binary
let decoded = decode_spatial(&mut buffer.as_slice())?;
```

### Delta Encoding

```rust
use lnmp_spatial::delta::Delta;

let start = Position3D { x: 10.0, y: 20.0, z: 30.0 };
let end = Position3D { x: 11.0, y: 19.0, z: 32.0 };

// Compute delta (only differences)
let delta = Position3D::compute_delta(&start, &end);
// delta = { dx: 1.0, dy: -1.0, dz: 2.0 }

// Apply delta
let reconstructed = Position3D::apply_delta(&start, &delta);
assert_eq!(reconstructed, end);
```

### Hybrid Protocol

```rust
use lnmp_spatial::protocol::{SpatialStreamer, SpatialStreamerConfig};

let config = SpatialStreamerConfig {
    abs_interval: 100,        // ABS frame every 100 frames
    enable_prediction: true,   // Enable predictive delta
    max_prediction_frames: 3,  // Max 3 predicted frames
};

let mut streamer = SpatialStreamer::with_config(config);

// Sender
let frame = streamer.next_frame(&robot_state, timestamp_ns)?;

// Receiver
let state = streamer.process_frame(&frame)?;
```

## Architecture

### Protocol Stack

```
┌─────────────────────────────────────┐
│  Application (Robot Control)       │
├─────────────────────────────────────┤
│  Hybrid Protocol (SpatialStreamer) │  ← Phase 3
│  - ABS/DELTA mixing                │
│  - Sequence tracking                │
│  - Predictive fallback              │  ← Phase 5
├─────────────────────────────────────┤
│  Frame Layer                        │  ← Phase 4
│  - CRC32 checksum                   │
│  - Nanosecond timestamp             │
├─────────────────────────────────────┤
│  Delta Layer                        │  ← Phase 2
│  - Compute delta                    │
│  - Apply delta                      │
├─────────────────────────────────────┤
│  Binary Codec                       │  ← Phase 1
│  - Encode/Decode                    │
│  - Type system                      │
└─────────────────────────────────────┘
```

### Data Flow

**Normal Operation (No Packet Loss):**
```
Sender                  Receiver
  │                        │
  ├─[Frame 0: ABS]────────>│ ✓ Reset state
  ├─[Frame 1: DELTA]──────>│ ✓ Apply delta
  ├─[Frame 2: DELTA]──────>│ ✓ Apply delta
  ├─[Frame 3: DELTA]──────>│ ✓ Apply delta
  ...
  ├─[Frame 100: ABS]──────>│ ✓ Drift correction
```

**Packet Loss (Predictive Mode):**
```
Sender                  Receiver
  │                        │
  ├─[Frame 97: DELTA]─────>│ ✓ Apply delta
  ├─[Frame 98: DELTA]─────>│ ✓ Apply delta, Predict: 99
  ├─[Frame 99: DELTA]─X    │ ❌ LOST → 🔮 Use prediction
  ├─[Frame 100: ABS]──────>│ ✓ Confirm/correct
```

## Performance

Benchmarks on Apple Silicon M-series:

| Operation             | Latency    | Throughput |
|-----------------------|------------|------------|
| Encode Position3D     | ~2.8 ns    | ~357 M/s   |
| Decode Position3D     | ~2.2 ns    | ~454 M/s   |
| Compute Delta         | ~5 ns      | ~200 M/s   |
| Spatial Transform     | ~7.5 ns    | ~133 M/s   |
| Full Frame (Hybrid)   | ~50 ns     | ~20 M/s    |

**Bandwidth Savings:**
- DELTA vs ABS: **99% reduction** (typical)
- CRC32 overhead: **<1%**

## Examples

### Robot Arm Control
```bash
cargo run --example robot
```

### Telemetry Streaming
```bash
cargo run --example stream
```

### 1kHz Control Loop
```bash
cargo run --example jitter_sim
```

### Prediction vs Non-Prediction
```bash
cargo run --example reflex_sim
```

## Design Philosophy

### Why Hybrid?

> "Robot arm moves with small delta steps, but resets with absolute position every breath."

- **DELTA** for speed and bandwidth efficiency (99% of frames)
- **ABS** for stability and drift correction (1% of frames)
- **Prediction** for packet loss resilience (fallback mechanism)

### Safety-Critical Mode

For applications where prediction is unsafe (e.g., surgery robots):

```rust
let config = SpatialStreamerConfig {
    abs_interval: 10,          // More frequent resets
    enable_prediction: false,  // Disable prediction
    max_prediction_frames: 0,
};
```

## API Reference

See [docs.rs](https://docs.rs/lnmp-spatial) or run:
```bash
cargo doc --open
```

## License

MIT OR Apache-2.0
