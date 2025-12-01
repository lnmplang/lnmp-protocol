# LNMP SDK - JavaScript / TypeScript

This directory contains the official TypeScript implementation of the **LNMP (LLM Native Minimal Protocol)**.

## Architecture (Monorepo)

This project is structured as a monorepo containing the following packages:

- **[`@lnmplang/lnmp`](packages/lnmp)**: The **MAIN** package for users. It provides a unified API for all LNMP modules (Core, Envelope, Net, Embedding, Spatial, LLB, SFE).
- **[`@lnmplang/wasm-bindings`](packages/wasm-bindings)**: Internal package providing high-performance WASM bindings to the Rust `lnmp` meta-crate.

## Installation

Users should install the main package:

```bash
npm install @lnmplang/lnmp
```

## Features

The SDK provides full feature parity with the Rust implementation:

- **Core**: Strict typing, parsing, and encoding of LNMP records.
- **Envelope**: Metadata wrapping and HTTP header interoperability.
- **Network**: AI-driven routing and importance scoring.
- **Embedding**: Vector operations, delta compression, and quantization (QInt8, QInt4, Binary).
- **Spatial**: Efficient spatial frame streaming for hybrid reality.
- **Sanitization**: Input cleaning and validation.
- **LLB (Explain Mode)**: Human-readable explanations for LNMP data.
- **SFE (Context Scoring)**: Context evaluation for RAG and LLM prioritization.
- **WASM**: Powered by Rust for maximum performance.

## Development

To contribute to the SDK:

### 1. Setup

```bash
npm install
```

### 2. Build

Build all packages in the correct order:

```bash
npm run build
```

### 3. Test

Run integration tests:

```bash
cd packages/lnmp
node integration-test.js
```

### 4. Benchmark

Run performance benchmarks:

```bash
cd packages/lnmp
node benchmark.js
```

## License

MIT
