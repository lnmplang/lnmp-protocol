# LNMP Repository Structure

This document describes the organization and development workflows for the LNMP protocol monorepo.

## 📁 Repository Structure

```
lnmp-protocol/
├── .github/workflows/          # CI/CD workflows
│   ├── rust-ci.yml            # Rust core testing
│   ├── conformance.yml        # Conformance tests
│   ├── release.yml            # Release automation
│   ├── subtree-push.yml       # Subtree management
│   └── subtree-sync.yml       # Subtree sync
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
│   └── lnmp-transport/        # Transport layer (HTTP, gRPC, etc.)
│
├── docs/                       # 📚 Documentation Hub
│   ├── api/                   # API Reference
│   │   ├── README.md          # API index
│   │   └── rust/              # Rust API docs
│   │       ├── v0.5.md        # Latest API
│   │       └── v0.3.md        # Legacy API
│   └── migration/             # Migration Guides
│       ├── README.md          # Migration index
│       ├── v0.4-to-v0.5.md    # Current migration
│       └── v0.3-to-v0.4.md    # Legacy migration
│
├── sdk/                        # 🌐 Language SDKs (Subtrees)
│   ├── go/                    # Go SDK (lnmp-sdk-go)
│   ├── js/                    # TypeScript SDK (lnmp-sdk-js)
│   │   ├── .github/workflows/ # TS SDK CI/CD
│   │   │   ├── wasm-sdk.yml   # WASM build & tests
│   │   │   └── npm-publish.yml # NPM publishing
│   │   └── packages/
│   │       ├── wasm-bindings/ # WASM bindings
│   │       └── lnmp/          # High-level SDK
│   ├── python/                # Python SDK (lnmp-sdk-python)
│   └── rust/                  # Rust SDK (lnmp-sdk-rust)
│
├── tools/                      # 🛠️ Development Tools (Subtrees)
│   ├── cli/                   # Command-line interface (lnmp-cli)
│   │   └── .github/workflows/ # CLI CI/CD
│   │       └── cli-test.yml
│   ├── mcp/                   # Model Context Protocol server (lnmp-mcp)
│   │   └── .github/workflows/ # MCP CI/CD
│   │       └── mcp-test.yml
│   └── vscode-extension/      # VS Code extension
│
├── examples/                   # 📖 Code Examples (Subtree: lnmp-examples)
│   ├── examples/              # Example implementations
│   └── scripts/               # Helper scripts
│
├── tests/                      # 🧪 Integration & Compliance Tests
│   └── compliance/            # Cross-language compliance tests
│
├── spec/                       # 📋 Protocol Specification
│   ├── grammar.md             # Formal PEG grammar
│   └── error-classes.md       # Error classification
│
└── scripts/                    # 🔧 Repository Management Scripts
    └── checks/                # Pre-commit checks

```

---

## 🎯 Core Principles

### 1. **Rust as Source of Truth**
- `lnmp-protocol` (this repo) contains the canonical Rust implementation
- All other SDKs are derived from or validated against Rust core
- Published on [crates.io](https://crates.io/crates/lnmp) as `lnmp = "0.5.x"`

### 2. **Subtree Architecture**
- SDKs, tools, and examples are independent repos
- Imported via `git subtree` for monorepo convenience
- Each subtree has its own CI/CD workflows

### 3. **Modular CI/CD**
- Root CI/CD: Rust core only
- SDK CI/CD: In respective `sdk/*/github/workflows/`
- Tool CI/CD: In respective `tools/*/.github/workflows/`
- Path filters ensure only relevant builds trigger

### 4. **Unified Documentation**
- `/docs/api/` - All API references
- `/docs/migration/` - Version upgrade guides
- SDK-specific docs stay in SDK directories

---

## 🔄 Development Workflows

### Local Development with Subtrees

The monorepo includes SDKs and tools via subtrees for convenience. However, **official source of truth** for each is its independent repo:

| Component | Subtree Path | Independent Repo |
|-----------|--------------|------------------|
| JS/TS SDK | `sdk/js/` | `lnmplang/lnmp-sdk-js` |
| Python SDK | `sdk/python/` | `lnmplang/lnmp-sdk-python` |
| Rust SDK | `sdk/rust/` | `lnmplang/lnmp-sdk-rust` |
| Go SDK | `sdk/go/` | `lnmplang/lnmp-sdk-go` |
| CLI Tool | `tools/cli/` | `lnmplang/lnmp-cli` |
| MCP Server | `tools/mcp/` | `lnmplang/lnmp-mcp` |
| Examples | `examples/` | `lnmplang/lnmp-examples` |

### Working with Local Rust Dependencies

For local development with Rust examples or SDKs:

```toml
# .cargo/config.toml (in example/SDK repo)
[patch."https://github.com/lnmplang/lnmp-protocol.git"]
lnmp-core = { path = "../lnmp-protocol/crates/lnmp-core" }
lnmp-codec = { path = "../lnmp-protocol/crates/lnmp-codec" }
lnmp = { path = "../lnmp-protocol/crates/lnmp" }
```

Or use the provided `scripts/use-local.sh` helper.

### CI Best Practices

- **Always use git or published dependencies in CI**
- Prefer tags for reproducibility: `lnmp = { git = "...", tag = "v0.5.12" }`
- Never use local path dependencies in CI

---

## 📦 Publishing & Releases

### Rust Crates
1. Update `Cargo.toml` versions (workspace-wide)
2. Update `CHANGELOG.md`
3. Tag: `git tag v0.5.x`
4. GitHub Release triggers automatic publish to crates.io

### NPM Packages (TypeScript SDK)
1. Bump version: `./scripts/bump-version.sh 0.6.0`
2. Update `CHANGELOG.md`
3. Create GitHub Release
4. Workflow publishes to npm automatically

### Python Package
1. Update `pyproject.toml` version
2. Create GitHub Release
3. Workflow publishes to PyPI

---

## 🔧 Common Tasks

### Update Subtree from Upstream

```bash
# Update JS SDK
git subtree pull --prefix=sdk/js https://github.com/lnmplang/lnmp-sdk-js.git main --squash

# Update CLI tool
git subtree pull --prefix=tools/cli https://github.com/lnmplang/lnmp-cli.git main --squash
```

### Push Subtree Changes Back

```bash
# Push JS SDK changes
git subtree push --prefix=sdk/js https://github.com/lnmplang/lnmp-sdk-js.git main

# Or use GitHub workflow: subtree-push.yml
```

### Run Full Test Suite

```bash
# Rust core
cargo test --workspace --all-features

# TypeScript SDK
cd sdk/js && npm run build && npm test

# Python SDK
cd sdk/python && poetry run pytest

# CLI tool
cd tools/cli && cargo test
```

---

## 📝 Documentation Guidelines

### Where to Document

- **Protocol spec**: `/spec/`
- **API reference**: `/docs/api/`
- **Migration guides**: `/docs/migration/`
- **SDK usage**: `sdk/*/README.md`
- **Tool usage**: `tools/*/README.md`
- **Examples**: `examples/*/README.md`

### Cross-Referencing

Use relative paths for links:
```markdown
See [API Reference](../../docs/api/) for details.
See [Migration Guide](../../docs/migration/v0.4-to-v0.5.md).
```

---

## 🚀 Quick Start

### For Contributors

1. Clone the repo
2. Install Rust: `rustup`
3. Build core: `cargo build --workspace`
4. Run tests: `cargo test --workspace`
5. Check formatting: `cargo fmt --all -- --check`
6. Run lints: `cargo clippy --workspace --all-features`

### For SDK Developers

1. Navigate to SDK directory: `cd sdk/js`
2. Follow SDK-specific `README.md`
3. SDK CI runs independently

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/lnmplang/lnmp-protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lnmplang/lnmp-protocol/discussions)
- **Documentation**: [docs/](./docs/)

---

## License

MIT License - See [LICENSE](./LICENSE) for details.
