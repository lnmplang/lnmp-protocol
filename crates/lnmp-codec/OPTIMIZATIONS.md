# LNMP v0.5 Optimizations Applied

This document tracks the performance optimizations implemented for LNMP v0.5.

## Latest Benchmarks

Use this section to capture the most recent `cargo bench -p lnmp-codec` summary (see `docs/compat-reporting-guide.md`).

## Completed Optimizations

### 1. VarInt Fast Paths (✅ Completed)

**File**: `crates/lnmp-codec/src/binary/varint.rs`

**Changes**:
- Added fast-path encoding for single-byte values (range: -64 to 63)
- Added fast-path encoding for two-byte values (range: -8192 to 8191)
- Added fast-path decoding for single-byte values
- Added fast-path decoding for two-byte values
- Added `#[inline]` attributes to hot functions
- Pre-allocated Vec capacity in general encoding path

**Impact**:
- Most field IDs (FIDs) are small integers that fit in 1-2 bytes
- Typical records have FIDs < 1000, which benefit from two-byte fast path
- Reduces allocation overhead for common cases

**Benchmark**: Run `cargo bench --bench v05_performance -- nested` to measure impact

### 2. Performance Documentation (✅ Completed)

**Files**:
- `crates/lnmp-codec/PERFORMANCE.md` - Comprehensive performance guide
- `crates/lnmp-codec/scripts/profile.sh` - Profiling helper script
- `crates/lnmp-codec/OPTIMIZATIONS.md` - This file

**Content**:
- Critical path analysis for all v0.5 features
- Profiling tool recommendations
- Optimization strategies for each subsystem
- Performance targets and measurement guidelines

## Recommended Future Optimizations

### 1. Buffer Pooling for Encoding

**Priority**: High
**Complexity**: Medium

**Description**:
Implement a buffer pool to reuse Vec<u8> allocations across multiple encoding operations.

**Implementation**:
```rust
pub struct BufferPool {
    buffers: Vec<Vec<u8>>,
    max_size: usize,
}

impl BufferPool {
    pub fn acquire(&mut self) -> Vec<u8> {
        self.buffers.pop().unwrap_or_else(Vec::new)
    }
    
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        if self.buffers.len() < self.max_size {
            buffer.clear();
            self.buffers.push(buffer);
        }
    }
}
```

**Expected Impact**: 10-20% reduction in allocation overhead

### 2. Merge-Join Algorithm for Delta Computation

**Priority**: High
**Complexity**: Medium

**Description**:
Replace HashMap-based field comparison with a merge-join algorithm that takes advantage of sorted fields.

**Current**: O(n) HashMap lookups for each field
**Optimized**: O(n) single-pass merge of sorted field lists

**Expected Impact**: 30-50% faster delta computation

### 3. SIMD Checksum Computation

**Priority**: Medium
**Complexity**: High

**Description**:
Use SIMD instructions (SSE2/AVX2) for XOR checksum computation in streaming layer.

**Implementation**:
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
unsafe fn compute_xor_checksum_simd(data: &[u8]) -> u32 {
    // Use 128-bit XOR operations
    let mut acc = _mm_setzero_si128();
    
    for chunk in data.chunks_exact(16) {
        let v = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
        acc = _mm_xor_si128(acc, v);
    }
    
    // Reduce 128-bit to 32-bit
    // ... reduction logic
}
```

**Expected Impact**: 2-4x faster checksum computation for large payloads

### 4. Zero-Copy Streaming

**Priority**: Medium
**Complexity**: High

**Description**:
Modify streaming layer to use buffer slices instead of copying data.

**Current**: Data is copied into frame buffers
**Optimized**: Frames reference source data via slices

**Expected Impact**: Eliminate memory copies, reduce memory usage by 50%

### 5. SmallVec for Records

**Priority**: Low
**Complexity**: Low

**Description**:
Use SmallVec to inline small records (up to 8 fields) without heap allocation.

**Implementation**:
```rust
use smallvec::SmallVec;

pub struct LnmpRecord {
    fields: SmallVec<[LnmpField; 8]>,
}
```

**Expected Impact**: 15-25% faster encoding/decoding for small records

### 6. Lazy Field Sorting

**Priority**: Low
**Complexity**: Low

**Description**:
Skip sorting if fields are already in canonical order.

**Implementation**:
```rust
fn encode(&self, record: &LnmpRecord) -> Result<Vec<u8>, BinaryError> {
    if !self.is_sorted(record) {
        record.sort_fields();
    }
    // ... encoding logic
}

fn is_sorted(&self, record: &LnmpRecord) -> bool {
    record.fields().windows(2).all(|w| w[0].fid <= w[1].fid)
}
```

**Expected Impact**: 5-10% faster encoding for pre-sorted records

## Benchmarking Results

### Baseline (Before Optimizations)

Run benchmarks to establish baseline:
```bash
cargo bench --bench v05_performance --save-baseline before
```

### After VarInt Optimizations

Run benchmarks after VarInt optimizations:
```bash
cargo bench --bench v05_performance --save-baseline after-varint
```

Compare results:
```bash
critcmp before after-varint
```

### Target Metrics

Based on Requirement 14, the following targets should be achieved:

| Metric | Target | Status |
|--------|--------|--------|
| Encoding speed per field | < 2μs | ⏳ Pending measurement |
| Decoding speed per field | < 2μs | ⏳ Pending measurement |
| Streaming memory overhead | < 10% | ⏳ Pending measurement |
| Delta encoding savings | > 50% | ⏳ Pending measurement |
| GRPC comparison | Comparable | ⏳ Pending measurement |

## Profiling Workflow

1. **Establish baseline**:
   ```bash
   ./crates/lnmp-codec/scripts/profile.sh bench
   ```

2. **Generate flamegraph**:
   ```bash
   ./crates/lnmp-codec/scripts/profile.sh flame
   ```

3. **Identify hot paths**:
   - Open `flamegraph.svg` in a browser
   - Look for wide bars (high CPU time)
   - Focus on functions taking > 5% of total time

4. **Implement optimization**:
   - Make targeted changes to hot paths
   - Add inline hints where appropriate
   - Reduce allocations in tight loops

5. **Measure improvement**:
   ```bash
   cargo bench --bench v05_performance --save-baseline after-opt
   critcmp before after-opt
   ```

6. **Verify correctness**:
   ```bash
   cargo test --manifest-path crates/lnmp-codec/Cargo.toml
   ```

## Optimization Guidelines

### Do's
- ✅ Profile before optimizing
- ✅ Focus on hot paths (> 5% CPU time)
- ✅ Measure impact of each optimization
- ✅ Add benchmarks for optimized code
- ✅ Verify correctness with tests
- ✅ Document optimization rationale

### Don'ts
- ❌ Optimize without profiling
- ❌ Sacrifice correctness for speed
- ❌ Make code unreadable for minor gains
- ❌ Optimize cold paths
- ❌ Skip benchmarking
- ❌ Break backward compatibility

## Next Steps

1. Run baseline benchmarks and record results
2. Profile with flamegraph to identify actual hot paths
3. Implement high-priority optimizations based on profiling data
4. Measure and document performance improvements
5. Compare with GRPC for equivalent message sizes
6. Iterate on remaining optimizations as needed

## References

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [LNMP v0.5 Design Document](../../.kiro/specs/lnmp-v0.5-advanced-protocol/design.md)
