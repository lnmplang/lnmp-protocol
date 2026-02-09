# LNMP Repository Structure

This document describes the organization of the `lnmp-protocol` repository, which serves as the **Core Kernel** of the LNMP ecosystem.

Official domain: [lnmp.ai](https://lnmp.ai)

## 📁 Repository Structure

```
lnmp-protocol/
├── .github/workflows/          # CI/CD workflows (Core Protocol only)
│   ├── rust-ci.yml            # Rust core testing
│   ├── conformance.yml        # Conformance tests
│   └── release.yml            # Release automation
│
├── crates/                     # 🦀 Rust Core Implementation
│   ├── lnmp/                  # Meta-crate (re-exports all)
│   ├── lnmp-core/             # Core types and record structures
│   ├── lnmp-codec/            # Parser and encoder
│   ├── lnmp-embedding/        # Embedding vector support
│   ├── lnmp-envelope/         # Operational metadata envelope
│   ├── lnmp-llb/              # LLM Bridge (explain mode, optimization)
│   ├── lnmp-net/              # Network message routing
│   ├── lnmp-quant/            # Vector quantization
│   ├── lnmp-sanitize/         # Input sanitization
│   ├── lnmp-sfe/              # Semantic Fidelity Engine
│   ├── lnmp-spatial/          # Spatial protocol support
│   └── lnmp-transport/        # Transport layer mappings
│
├── docs/                       # 📚 Documentation Hub
│   ├── api/                   # API Reference
│   └── migration/             # Migration Guides
│
├── spec/                       # 📋 Protocol Specification
│   ├── grammar.md             # Formal PEG grammar
│   └── error-classes.md       # Error classification
│
├── tests/                      # 🧪 Integration Tests
│   └── compliance/            # Cross-language compliance tests
│
└── scripts/                    # 🔧 Management Scripts
```

---

## 🎯 Core Principles

### 1. **Rust as Source of Truth**
- `lnmp-protocol` (this repo) contains the canonical Rust implementation.
- Published on [crates.io](https://crates.io/crates/lnmp).

### 2. **Decoupled Ecosystem**
- **SDKs** (Python, JS, Go, Rust) live in their own repositories (`lnmp-sdk-*`).
- **Tools** (CLI, MCP, VSCode) live in their own repositories.
- This ensures independent versioning and faster release cycles for downstream tools.

### 3. **Unified Documentation**
- This repository hosts the core **Protocol Specification** (`spec/`).
- SDK-specific documentation resides in the respective SDK repositories.

---

## 🔄 Development Workflow

### Working on Core Protocol
1.  Clone this repository.
2.  Run `cargo test --workspace`.
3.  Submit PRs for protocol improvements or core optimizations.

### Working on SDKs or Tools
Please visit the respective repository:
- [Python SDK](https://github.com/lnmplang/lnmp-sdk-python)
- [JS/TS SDK](https://github.com/lnmplang/lnmp-sdk-js)
- [Rust SDK](https://github.com/lnmplang/lnmp-sdk-rust)
- [Go SDK](https://github.com/lnmplang/lnmp-sdk-go)
- [CLI Tool](https://github.com/lnmplang/lnmp-cli)

---

## 📦 Publishing & Releases

### Rust Crates
1. Update `Cargo.toml` versions (workspace-wide).
2. Update `CHANGELOG.md`.
3. Create a GitHub Release to trigger publication to crates.io.
