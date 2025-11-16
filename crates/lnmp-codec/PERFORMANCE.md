# LNMP v0.5 Performance Optimization Guide

This document describes the performance characteristics of LNMP v0.5 and optimization strategies for critical paths.

## Running Benchmarks

To run the performance benchmarks:

```bash
cargo bench --bench v05_performance
```

This will generate HTML reports in `target/criterion/` with detailed performance metrics.

## Critical Path Analysis

### 1. VarInt Encoding/Decoding

**Location**: `crates/lnmp-codec/src/binary/varint.rs`

**Current Implementation**: The VarInt encoding uses a loop-based approach that processes one byte at a time.

**Optimization Opportunities**:
- **Inline hints**: The `encode_varint` and `decode_varint` functions are already marked with `#[inline]`, which helps the compiler optimize hot paths
- **Branch prediction**: The current implementation has predictable branches for common cases (small integers)
- **SIMD potential**: For batch encoding of multiple VarInts, SIMD instructions could be used

**Recommended Actions**:
1. Profile with `cargo flamegraph` to identify if VarInt is a bottleneck
2. Consider specialized fast paths for common integer ranges (0-127, 0-16383)
3. For nested structures with many fields, consider batch encoding

### 2. Nested Structure Encoding

**Location**: `crates/lnmp-codec/src/binary/nested_encoder.rs`

**Current Implementation**: Recursive encoding with depth tracking and validation.

**Optimization Opportunities**:
- **Memory allocation**: Each nested level allocates a new Vec<u8> for the encoded data
- **Depth tracking**: Depth validation adds overhead on every recursive call
- **Field sorting**: Canonical ordering requires sorting fields at each level

**Recommended Actions**:
1. **Pre-allocate buffers**: Estimate total size and pre-allocate a single buffer
   ```rust
   // Instead of multiple Vec::new()
   let estimated_size = estimate_encoded_size(record);
   let mut buffer = Vec::with_capacity(estimated_size);
   ```

2. **Optimize depth tracking**: Use a single counter passed by reference instead of creating new contexts
   ```rust
   fn encode_recursive(&self, record: &LnmpRecord, buffer: &mut Vec<u8>, depth: &mut usize) {
       *depth += 1;
       // ... encoding logic
       *depth -= 1;
   }
   ```

3. **Lazy sorting**: If records are already sorted (common case), skip the sort operation
   ```rust
   if !self.is_sorted(record) {
       record.sort_fields();
   }
   ```

### 3. Streaming Layer

**Location**: `crates/lnmp-codec/src/binary/streaming.rs`

**Current Implementation**: Frame-based chunking with checksum computation.

**Optimization Opportunities**:
- **Checksum computation**: XOR checksum is computed byte-by-byte
- **Frame allocation**: Each frame allocates a new Vec<u8>
- **Buffer copying**: Data is copied multiple times during chunking

**Recommended Actions**:
1. **SIMD checksums**: Use SIMD instructions for faster XOR computation
   ```rust
   #[cfg(target_arch = "x86_64")]
   use std::arch::x86_64::*;
   
   fn compute_xor_checksum_simd(data: &[u8]) -> u32 {
       // Use 128-bit XOR operations
   }
   ```

2. **Zero-copy streaming**: Use buffer slices instead of copying
   ```rust
   pub struct StreamingEncoder<'a> {
       source: &'a [u8],
       position: usize,
   }
   ```

3. **Reuse frame buffers**: Pool frame buffers to reduce allocations
   ```rust
   struct FramePool {
       buffers: Vec<Vec<u8>>,
   }
   ```

### 4. Delta Encoding

**Location**: `crates/lnmp-codec/src/binary/delta.rs`

**Current Implementation**: Field-by-field comparison with HashMap lookups.

**Optimization Opportunities**:
- **HashMap overhead**: Multiple HashMap lookups for each field
- **Value comparison**: Deep comparison of nested structures
- **Allocation**: Each delta operation allocates a new DeltaOp

**Recommended Actions**:
1. **Sorted field iteration**: Since fields are sorted by FID, use merge-join algorithm
   ```rust
   fn diff_records_sorted(old: &LnmpRecord, new: &LnmpRecord) -> Vec<DeltaOp> {
       let mut old_iter = old.fields().iter();
       let mut new_iter = new.fields().iter();
       let mut ops = Vec::new();
       
       let mut old_field = old_iter.next();
       let mut new_field = new_iter.next();
       
       loop {
           match (old_field, new_field) {
               (Some(o), Some(n)) if o.fid == n.fid => {
                   if o.value != n.value {
                       ops.push(DeltaOp::update(o.fid, &n.value));
                   }
                   old_field = old_iter.next();
                   new_field = new_iter.next();
               }
               (Some(o), Some(n)) if o.fid < n.fid => {
                   ops.push(DeltaOp::delete(o.fid));
                   old_field = old_iter.next();
               }
               (Some(o), Some(n)) => {
                   ops.push(DeltaOp::set(n.fid, &n.value));
                   new_field = new_iter.next();
               }
               (Some(o), None) => {
                   ops.push(DeltaOp::delete(o.fid));
                   old_field = old_iter.next();
               }
               (None, Some(n)) => {
                   ops.push(DeltaOp::set(n.fid, &n.value));
                   new_field = new_iter.next();
               }
               (None, None) => break,
           }
       }
       
       ops
   }
   ```

2. **Pre-allocate delta ops**: Estimate the number of changes
   ```rust
   let estimated_ops = (old.fields().len() + new.fields().len()) / 2;
   let mut ops = Vec::with_capacity(estimated_ops);
   ```

3. **Shallow comparison first**: Check field count and FIDs before deep value comparison
   ```rust
   if old.fields().len() != new.fields().len() {
       // Definitely different
   } else if old.fids() == new.fids() {
       // Only values might differ
   }
   ```

## Memory Optimization

### 1. Reduce Allocations

**Current hotspots**:
- Vec<u8> allocations for each encoded value
- String allocations for error messages
- HashMap allocations for field lookups

**Strategies**:
1. **Buffer pooling**: Reuse buffers across encoding operations
2. **Arena allocation**: Use a bump allocator for temporary data
3. **Cow strings**: Use `Cow<'static, str>` for error messages

### 2. Optimize Data Structures

**Current structures**:
- `LnmpRecord` uses `Vec<LnmpField>` for fields
- `LnmpValue` is an enum with Box<LnmpRecord> for nested records

**Potential improvements**:
1. **Inline small records**: Use SmallVec for records with few fields
   ```rust
   use smallvec::SmallVec;
   
   pub struct LnmpRecord {
       fields: SmallVec<[LnmpField; 8]>,  // Inline up to 8 fields
   }
   ```

2. **Compact field representation**: Pack FID and type tag into a single u32
   ```rust
   #[repr(C)]
   struct CompactField {
       fid_and_tag: u32,  // 16 bits FID + 8 bits tag + 8 bits flags
       value: CompactValue,
   }
   ```

## Profiling Tools

### 1. Criterion Benchmarks

Run the benchmarks with different configurations:

```bash
# Run all benchmarks
cargo bench --bench v05_performance

# Run specific benchmark group
cargo bench --bench v05_performance -- nested_encoding

# Generate flamegraph
cargo bench --bench v05_performance -- --profile-time=10
```

### 2. Flamegraph

Install and use cargo-flamegraph:

```bash
cargo install flamegraph
cargo flamegraph --bench v05_performance
```

### 3. Perf (Linux only)

```bash
perf record --call-graph dwarf cargo bench --bench v05_performance
perf report
```

### 4. Valgrind/Cachegrind

```bash
valgrind --tool=cachegrind cargo bench --bench v05_performance
```

## Performance Targets

Based on the requirements (Requirement 14), the following targets should be met:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Encoding throughput | Comparable to GRPC | TBD | ⏳ |
| Streaming memory overhead | < 10% | TBD | ⏳ |
| Delta encoding savings | > 50% for typical updates | TBD | ⏳ |
| Encoding speed per field | < 2μs | TBD | ⏳ |
| Decoding speed per field | < 2μs | TBD | ⏳ |

To measure these targets, run:

```bash
cargo bench --bench v05_performance > benchmark_results.txt
```

## Optimization Checklist

- [ ] Profile with flamegraph to identify hot paths
- [ ] Optimize VarInt encoding for common cases
- [ ] Reduce allocations in nested encoding
- [ ] Implement zero-copy streaming where possible
- [ ] Use merge-join algorithm for delta computation
- [ ] Add buffer pooling for frequently allocated structures
- [ ] Consider SIMD for checksum computation
- [ ] Benchmark against GRPC for equivalent messages
- [ ] Measure memory usage during streaming
- [ ] Validate delta encoding bandwidth savings

## Next Steps

1. **Baseline measurements**: Run benchmarks and record current performance
2. **Profile hot paths**: Use flamegraph to identify bottlenecks
3. **Implement optimizations**: Start with highest-impact optimizations
4. **Measure improvements**: Re-run benchmarks after each optimization
5. **Compare with GRPC**: Implement equivalent GRPC messages for comparison

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
