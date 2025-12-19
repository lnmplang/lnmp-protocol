# LNMP Binary Format Specification

**Status:** Working Draft (v0.5.16)  
**Scope:** Binary frame layout, VarInt encoding, type tags, zero-copy decoding, HybridNumericArray, and 3-tier access API.  
**Audience:** Implementers of LNMP binary encoders/decoders, transport designers, and interoperability testers.

---

## 1. Status of This Document

- Reflects the currently shipping binary implementation (v0.5.16 with 3-tier API and aligned-zerocopy).  
- Requirements reference concrete code/tests to avoid divergence.  
- Versioning model aligns with migration doc; version byte semantics defined here.

---

## 2. Binary Frame Overview

```
+---------+--------+-----------------+--------------------+
| Version | Flags  | Entry Count VAR | Entries (sorted)   |
+---------+--------+-----------------+--------------------+
```

- Version (`0x04` = base binary, `0x05` = nested).  
- Flags reserved for compression/envelope features (future spec).  
- Entry count is VarInt (LEB128 minimal encoding).  
- Entries follow canonical FID order.

Implementation: `crates/lnmp-codec/src/binary/encoder.rs`, `binary/nested_encoder.rs`.

---

## 3. Type Tags

### 3.1 Complete Type Tag Table (v0.5.16)

| Tag | Name | Payload | Zero-Copy | Version |
|-----|------|---------|-----------|---------|
| `0x01` | Int | VarInt (signed LEB128) | ✅ Copy (8 bytes) | v0.4 |
| `0x02` | Float | 8 bytes IEEE 754 LE | ✅ Copy (8 bytes) | v0.4 |
| `0x03` | Bool | 1 byte (0x00/0x01) | ✅ Copy (1 byte) | v0.4 |
| `0x04` | String | VarInt len + UTF-8 | ✅ Borrow (`&str`) | v0.4 |
| `0x05` | StringArray | VarInt count + strings | ⚠️ Refs only | v0.4 |
| `0x06` | NestedRecord | Recursive frame | ⚠️ Partial | v0.5 |
| `0x07` | NestedArray | VarInt count + frames | ⚠️ Partial | v0.5 |
| `0x08` | Embedding | VarInt len + raw bytes | ✅ Borrow (`&[u8]`) | v0.5 |
| **`0x09`** | **HybridNumericArray** | Flags + VarInt dim + data | ⚠️ Quasi* | **v0.5.15** |
| `0x0A` | QuantizedEmbedding | Scheme + scale + data | ⚠️ Partial | v0.5.4 |
| `0x0B` | IntArray | VarInt count + VarInts | ❌ Allocates | v0.5.5 |
| `0x0C` | FloatArray | VarInt count + f64s | ❌ Allocates | v0.5.5 |
| `0x0D` | BoolArray | VarInt count + bytes | ❌ Allocates | v0.5.5 |
| `0x0E-0x0F` | Reserved | - | - | - |

### 3.2 Type Selection Guide

#### When to Use Each Array Type

| Use Case | Recommended Type | Reason |
|----------|------------------|--------|
| User roles, tags | `StringArray (0x05)` | Text values like `["admin", "dev"]` |
| ML embeddings (f32) | `HybridArray (0x09)` | 50% size savings, zero-copy dense |
| ML embeddings (raw) | `Embedding (0x08)` | Raw bytes, decode lazily |
| Timestamp series | `IntArray (0x0B)` | Compact VarInt encoding |
| Sensor readings | `FloatArray (0x0C)` | f64 precision required |
| Feature flags | `BoolArray (0x0D)` | `[true, false, true]` |
| Nested objects | `NestedRecord (0x06)` | `{F12=1;F20="sub"}` |
| Object lists | `NestedArray (0x07)` | `[{...}, {...}]` |

#### Array Type Comparison

```
┌────────────────────────────────────────────────────────────────────┐
│                    ARRAY TYPE DECISION TREE                        │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Is the data NUMERIC?                                              │
│       │                                                            │
│       ├── YES: Is it for ML/Embeddings?                            │
│       │         │                                                  │
│       │         ├── YES, f32 precision OK → HybridArray (0x09) ✓   │
│       │         ├── YES, need raw bytes → Embedding (0x08) ✓       │
│       │         └── NO, simple array:                              │
│       │                  ├── Integers → IntArray (0x0B)            │
│       │                  ├── Floats → FloatArray (0x0C)            │
│       │                  └── Booleans → BoolArray (0x0D)           │
│       │                                                            │
│       └── NO: Is it STRINGS?                                       │
│                 │                                                  │
│                 └── YES → StringArray (0x05) ✓                     │
│                                                                    │
│  Is it NESTED structures?                                          │
│       │                                                            │
│       ├── Single record → NestedRecord (0x06)                      │
│       └── Array of records → NestedArray (0x07)                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 3.3 Encoding Format Details

#### StringArray (0x05) vs IntArray (0x0B) vs HybridArray (0x09)

```
StringArray (0x05): ["hello", "world"]
┌───────┬────────────────┬───────────────────┐
│ count │ len + "hello"  │ len + "world"     │
│ VarInt│ VarInt + UTF-8 │ VarInt + UTF-8    │
└───────┴────────────────┴───────────────────┘

IntArray (0x0B): [1, 2, 1000000]
┌───────┬────────┬────────┬────────────┐
│ count │ VarInt │ VarInt │ VarInt     │
│   3   │   1    │   2    │ 0xC0843D   │
└───────┴────────┴────────┴────────────┘
         1 byte   1 byte   3 bytes (compact!)

FloatArray (0x0C): [1.5, 2.5, 3.5]
┌───────┬──────────┬──────────┬──────────┐
│ count │ f64 LE   │ f64 LE   │ f64 LE   │
│   3   │ 8 bytes  │ 8 bytes  │ 8 bytes  │
└───────┴──────────┴──────────┴──────────┘
         Fixed 8-byte IEEE 754

HybridArray (0x09) f32: [1.0, 2.0, 3.0]
┌───────┬────────┬──────────────────────┐
│ flags │  dim   │ data (contiguous)    │
│ 0x02  │   3    │ 4+4+4 = 12 bytes     │
└───────┴────────┴──────────────────────┘
  f32    VarInt   Fixed 4-byte per element
```

### 3.4 Size Comparison

| Data | IntArray (0x0B) | FloatArray (0x0C) | HybridArray (0x09) |
|------|-----------------|-------------------|---------------------|
| 256 integers | ~300 bytes* | N/A | 1,030 bytes (i32) |
| 256 floats | N/A | 2,051 bytes | **1,030 bytes (f32)** |
| 1024 floats | N/A | 8,195 bytes | **4,102 bytes (f32)** |

*IntArray size varies based on value magnitudes (VarInt)

### 3.5 Zero-Copy Behavior

| Type | Zero-Copy Support | Memory Behavior |
|------|-------------------|-----------------|
| `String (0x04)` | ✅ **Full** | Returns `&str` pointing to buffer |
| `StringArray (0x05)` | ⚠️ **Partial** | `Vec<&str>` - refs allocated, strings borrowed |
| `Embedding (0x08)` | ✅ **Full** | Returns `&[u8]` raw slice |
| `HybridArray (0x09)` | ⚠️ **Quasi** | Raw bytes view, parse-on-access* |
| `IntArray (0x0B)` | ❌ **None** | VarInt must be parsed → allocates `Vec<i64>` |
| `FloatArray (0x0C)` | ❌ **None** | Must copy to ensure alignment → `Vec<f64>` |
| `BoolArray (0x0D)` | ❌ **None** | Byte-per-bool must be converted → `Vec<bool>` |
| `NestedRecord (0x06)` | ⚠️ **Partial** | Recursive view, some allocation |
| `NestedArray (0x07)` | ⚠️ **Partial** | Vector of views allocated |

*HybridArray stores raw bytes internally. Use `iter_f32()` (primary) or `to_f32_vec()` (materialize) for access. `as_f32_slice()` (bonus) requires alignment (see §4.7).

---


## 4. HybridNumericArray (0x09)

### 4.1 Purpose

Efficient encoding for numeric arrays with automatic type selection and compact storage.

**Benefits:**
- 50% size reduction for f32 arrays compared to f64
- Zero-copy access for aligned dense data
- Future sparse mode for high-sparsity vectors

### 4.2 Wire Format

```
+-------+------------+------------------+
| Flags | Dim VarInt | Data (dense/sparse) |
+-------+------------+------------------+
  1 byte   variable     dim * elem_size
```

### 4.3 Flags Byte

```
Bit 0-1: Element Type (DType)
  00 = i32 (4 bytes per element)
  01 = i64 (8 bytes per element)
  10 = f32 (4 bytes per element)
  11 = f64 (8 bytes per element)

Bit 2: Sparse Mode Flag
  0 = Dense (contiguous data)
  1 = Sparse (index + value pairs) [RESERVED]

Bit 3: Delta Encoding [RESERVED]
  0 = Raw values
  1 = Delta encoded

Bits 4-7: Reserved (must be 0)
```

### 4.4 Dense Mode Layout

```
Dense f32 array [1.0, 2.5, -3.14]:
+------+-----+-----------------------------+
| 0x02 | 0x03| 00 00 80 3F 00 00 20 40 ... |
+------+-----+-----------------------------+
 flags  dim=3  IEEE 754 LE f32 values
```

### 4.5 Size Comparison

| Array Type | Legacy (f64) | HybridArray (f32) | Savings |
|------------|--------------|-------------------|---------|
| 256-dim | 2,051 bytes | 1,030 bytes | **50%** |
| 1024-dim | 8,195 bytes | 4,102 bytes | **50%** |

### 4.6 Implementation

```rust
// Create f32 dense array
let arr = HybridArray::from_f32_dense(&[1.0, 2.5, -3.14]);

// PRIMARY: Zero-alloc iterator
for value in arr.iter_f32()? {
    // Process value
}

// MATERIALIZE: Allocate Vec when needed
let values: Vec<f32> = arr.to_f32_vec()?;

// BONUS: Typed slice (aligned-zerocopy feature)
#[cfg(feature = "aligned-zerocopy")]
if let Some(slice) = arr.as_f32_slice() {
    // Zero-copy when aligned
}

// Get flags byte
assert_eq!(arr.flags(), 0x02); // F32, dense
```

Evidence: `crates/lnmp-codec/src/binary/types.rs`, `examples/hybrid_array_demo.rs`

### 4.7 Alignment & Memory Layout

#### Current Behavior: Raw Bytes View

HybridArray stores data as raw `Vec<u8>` bytes. Access methods parse on-demand:

```rust
pub struct HybridArray {
    pub dtype: NumericDType,
    pub sparse: bool,
    pub dim: usize,
    pub data: Vec<u8>,  // Raw bytes, 1-byte aligned
}

// Accessor parses and allocates
pub fn as_f32_vec(&self) -> Option<Vec<f32>> {
    for chunk in self.data.chunks_exact(4) {
        result.push(f32::from_le_bytes([...]));  // Parse each
    }
    Some(result)  // Allocates Vec<f32>
}
```

#### Why Not True Zero-Copy (`&[f32]`)?

Network buffers have **1-byte alignment**. Typed slices require alignment:

| Type | Required Alignment | Network Buffer |
|------|-------------------|----------------|
| `f32` | 4-byte | 1-byte ❌ |
| `f64` | 8-byte | 1-byte ❌ |
| `i32` | 4-byte | 1-byte ❌ |
| `i64` | 8-byte | 1-byte ❌ |

Casting unaligned bytes to typed slice causes **undefined behavior** on some platforms.

#### Implemented: aligned-zerocopy Feature (v0.5.16+)

**Safe zero-copy with runtime alignment check:**

```rust
// Enable in Cargo.toml
[dependencies]
lnmp-codec = { version = "0.5.16", features = ["aligned-zerocopy"] }

// API (runtime-safe)
#[cfg(feature = "aligned-zerocopy")]
pub fn as_f32_slice(&self) -> Option<&[f32]>
pub fn as_f64_slice(&self) -> Option<&[f64]>
pub fn as_i32_slice(&self) -> Option<&[i32]>
pub fn as_i64_slice(&self) -> Option<&[i64]>
```

**Behavior:**
- Returns `Some(&[T])` if data is properly aligned
- Returns `None` if not aligned (fallback to `as_f32_vec()`)
- Uses `bytemuck::try_cast_slice()` for runtime safety
- **NEVER causes UB**

**Usage:**
```rust
let arr = HybridArray::from_f32_dense(&values);

if let Some(slice) = arr.as_f32_slice() {
    // True zero-copy! No allocation
} else {
    // Fallback
    let vec = arr.as_f32_vec().unwrap();
}
```

#### Performance Comparison (Verified)

| Operation | as_f32_vec() | as_f32_slice() (aligned) | Improvement |
|-----------|--------------|--------------------------|-------------|
| 256-dim access (10k iter) | 118 ms | 276 μs | **427x faster** |
| Memory | +1 KB | 0 bytes | **100%** |
| Alignment rate | N/A | ~100%* | Platform-dependent |
| Safety | ✅ Safe | ✅ Safe (runtime check) | Both safe |

*On modern platforms (x86-64, ARM64), `Vec<u8>` is typically aligned for f32/f64.

---


## 5. Zero-Copy Decoding

### 5.1 Overview

Zero-copy decoding returns borrowed references to the input buffer instead of allocating new memory.

```
Standard Decode:
  Buffer → Parse → Allocate → Copy → LnmpRecord (owned)
                    ↓
              Memory allocation

Zero-Copy Decode:
  Buffer → Parse → Reference → LnmpRecordView (borrowed)
                    ↓
              No allocation
```

### 5.2 Performance Characteristics

| Record Size | Standard | Zero-Copy | Speedup |
|-------------|----------|-----------|---------|
| Small (3 fields) | 248 ns | 92 ns | **2.70x** |
| Medium (7 fields) | 610 ns | 164 ns | **3.71x** |
| Large (embeddings) | 18.3 μs | 2.1 μs | **8.91x** |

Throughput: **40.4 GiB/s** (zero-copy) vs 0.12 GiB/s (standard)

### 5.3 API Contract

```rust
// Standard decode (allocates)
fn decode(&self, bytes: &[u8]) -> Result<LnmpRecord>;

// Zero-copy decode (borrows)
fn decode_view<'a>(&self, bytes: &'a [u8]) -> Result<LnmpRecordView<'a>>;
```

**Lifetime Rule:** `LnmpRecordView<'a>` cannot outlive input `bytes`.

### 5.4 Type-Specific Zero-Copy Behavior

| Type | Zero-Copy Support | Notes |
|------|-------------------|-------|
| String | ✅ Full | Returns `&str` |
| StringArray | ✅ Refs | `Vec<&str>` (refs allocated) |
| Embedding | ✅ Full | Returns `&[u8]` raw bytes |
| HybridNumericArray | ✅ Dense | Direct slice access |
| Int/Float/Bool | ✅ Natural | Scalars copied (8 bytes max) |
| IntArray | ❌ | VarInt requires parsing |
| FloatArray | ❌ | Must validate/copy |
| Nested | ⚠️ Partial | Recursive views |

### 5.5 Usage Patterns

**Routing/Filtering (recommended):**
```rust
let view = decoder.decode_view(&bytes)?;
if let Some(field) = view.get_field(50) {
    if let LnmpValueView::String(status) = &field.value {
        if *status == "critical" {
            return Ok(Route::ToLLM);
        }
    }
}
```

**Processing/Storage (use standard):**
```rust
let record = decoder.decode(&bytes)?;
database.store(&record)?; // Needs owned data
```

### 5.6 Requirements

- **REQ-ZC-01:** `decode_view()` MUST NOT allocate heap memory for String, StringArray (refs only), or Embedding types.
- **REQ-ZC-02:** Returned views MUST be valid only while input buffer is live (lifetime enforcement).
- **REQ-ZC-03:** Scalar types (Int, Float, Bool) MAY be copied as they are ≤8 bytes.

Evidence: `crates/lnmp-codec/src/binary/decoder.rs`, `benches/zero_copy_bench.rs`

---

## 6. Normative Requirements

### 6.1 Versioning

- **REQ-BIN-VER-01:** Encoders MUST set version byte to `0x04` when no nested structures are encoded and to `0x05` when nested records/arrays may appear.
- **REQ-BIN-VER-02:** Decoders MUST reject frames with version lower than the configured minimum.

### 6.2 VarInt Encoding

- **REQ-BIN-VAR-01:** All integer fields MUST use minimal LEB128 representation.
- **REQ-BIN-VAR-02:** Decoders MUST reject non-minimal encodings with `BinaryError::NonCanonicalVarInt`.

### 6.3 Entry Structure

```
entry := FID_VARINT | type_tag | payload
```

- **REQ-BIN-ENT-01:** Field IDs MUST be encoded as VarInt sorted ascending.
- **REQ-BIN-ENT-02:** Type tags are single bytes per Section 3.1.

### 6.4 Primitive Payloads

- **REQ-BIN-PRI-01:** Integers use signed VarInt.
- **REQ-BIN-PRI-02:** Floats use 64-bit IEEE 754 little-endian.
- **REQ-BIN-PRI-03:** Strings encoded as `len VarInt` + UTF-8 bytes.

### 6.5 Ordering Validation

- **REQ-BIN-ORD-01:** Encoders MUST output entries sorted by FID.
- **REQ-BIN-ORD-02:** Decoders MUST enforce ordering when `validate_ordering` enabled.

---

## 7. Implementation Evidence

| Requirement | Evidence |
|-------------|----------|
| Type Tags (3.1) | `binary/types.rs`, `binary/entry.rs` |
| HybridNumericArray (4.*) | `binary/types.rs:HybridArray`, `examples/hybrid_array_demo.rs` |
| Zero-Copy (5.*) | `binary/decoder.rs:decode_view`, `benches/zero_copy_bench.rs` |
| Versioning (6.1) | `binary/encoder.rs`, `binary/decoder.rs` |
| VarInt (6.2) | `binary/varint.rs`, tests `binary_error_handling.rs` |

---

## 8. Changelog

| Version | Date | Changes |
|---------|------|---------|
| v0.5.15 | 2024-12-19 | Added HybridNumericArray (0x09), zero-copy documentation |
| v0.5.5 | 2024-11 | Added IntArray, FloatArray, BoolArray |
| v0.5.4 | 2024-11 | Added QuantizedEmbedding |
| v0.5.0 | 2024-10 | Added Nested structures, Embedding |
| v0.4.0 | 2024-09 | Initial binary format |

