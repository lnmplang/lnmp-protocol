# LNMP Python SDK Performance Benchmarks

**Date:** 2025-11-30
**Platform:** darwin

| Operation | Avg Time (µs) | Ops/Sec | Min (µs) | Max (µs) |
|-----------|---------------|---------|----------|----------|
| Core: Parse (Small) | 0.84 | 1,190,414 | 0.68 | 1.19 |
| Core: Encode Text (Small) | 0.50 | 1,984,915 | 0.48 | 0.56 |
| Core: Encode Binary (Small) | 0.44 | 2,286,544 | 0.43 | 0.44 |
| Core: Decode Binary (Small) | 0.42 | 2,396,607 | 0.40 | 0.43 |
| Spatial: Encode Position3D | 0.21 | 4,818,029 | 0.20 | 0.22 |
| Spatial: Decode Position3D | 0.16 | 6,235,431 | 0.16 | 0.16 |
| Embedding: Delta (128-dim) | 1.82 | 550,658 | 1.78 | 1.84 |
