# LNMP-QUANT Performance Analysis

This document details the performance characteristics of the Adaptive and Batch quantization features introduced in Phase 9.

## 1. Adaptive Quantization Overhead

We measured the overhead of the adaptive selection logic compared to direct scheme usage.

| Operation | Time (512-dim) | Overhead |
|-----------|----------------|----------|
| **Direct QInt8** | 1.23 µs | Baseline |
| **Adaptive (High)** | 1.20 µs | ~0% (Negligible) |
| **Adaptive (Maximum)** | 425 ns | N/A (FP16 is faster) |

**Conclusion**: The adaptive selection logic introduces **zero measurable overhead**. The compiler likely inlines the selection match, making it as fast as direct calls.

## 2. Batch Processing Performance

We compared the `quantize_batch` API against a manual sequential loop.

| Batch Size | Sequential Loop | Batch API | Overhead | Per Vector |
|------------|-----------------|-----------|----------|------------|
| **100** | 124 µs | 128 µs | +3.2% | ~1.28 µs |
| **1000** | 1.19 ms | 1.34 ms | +12.6% | ~1.34 µs |

**Analysis**:
- The Batch API introduces a small overhead (~3-13%) compared to a raw loop.
- **Sources of Overhead**:
  1. **Result Collection**: Allocating and populating the `Vec<Result<...>>`.
  2. **Statistics**: Tracking success/failure counts.
  3. **Safety**: Additional checks and wrapping.

**Recommendation**:
- For **maximum raw speed** in tight loops, use `quantize_embedding` directly.
- For **convenience, safety, and reporting**, use `quantize_batch`. The ~150ns per-vector overhead is negligible for most applications.

## 3. Throughput Estimates

Based on single-core benchmarks on M-series silicon:

- **Adaptive FP16**: ~2.3 Million vectors/sec
- **Adaptive QInt8**: ~830,000 vectors/sec
- **Batch QInt8**: ~750,000 vectors/sec

## 4. Memory Usage

- **Adaptive**: No additional memory allocation beyond the quantized vector.
- **Batch**: Allocates a result vector of size `N` (pointers/structs), plus the quantized data.

## 5. Optimization Notes

- **SIMD**: Future SIMD optimizations (Phase 9 Option B) could significantly improve QInt8 performance, potentially doubling throughput.
- **Parallelization**: The Batch API is designed to be easily parallelizable with `rayon` in the future.
