# Contributing to LNMP Protocol

Thank you for your interest in contributing to LNMP! This guide will help you get started.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Contribution Types](#contribution-types)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation Guidelines](#documentation-guidelines)
- [Release Process](#release-process)

---

## 🤝 Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- **Be respectful** of differing viewpoints and experiences
- **Accept constructive criticism** gracefully
- **Focus on what is best** for the community
- **Show empathy** towards other community members

---

## 🚀 Getting Started

### Prerequisites

**For Rust Development:**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt clippy
```

**For TypeScript SDK:**
```bash
# Install Node.js 20+
# Install dependencies
cd sdk/js && npm install
```

**For Python SDK:**
```bash
# Install Poetry
curl -sSL https://install.python-poetry.org | python3 -

# Install dependencies
cd sdk/python && poetry install
```

### Fork and Clone

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/lnmp-protocol.git
   cd lnmp-protocol
   ```
3. **Add upstream** remote:
   ```bash
   git remote add upstream https://github.com/lnmplang/lnmp-protocol.git
   ```

---

## 🔄 Development Workflow

Refer to [`docs/branching-strategy.md`](docs/branching-strategy.md) for our official branch policy (protected `main`, release branches, and naming conventions).

### 1. Create a Branch

```bash
# Update your local main
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements
- `ci/` - CI/CD changes

### 2. Make Changes

Follow the [Coding Standards](#coding-standards) and ensure:
- Code compiles without warnings
- Tests pass
- Documentation is updated
- Commits are well-formatted

### 3. Commit Your Changes

Use conventional commit format:

```bash
git commit -m "feat(core): add support for generic arrays"
git commit -m "fix(codec): resolve nested structure parsing bug"
git commit -m "docs(api): update v0.5 migration guide"
```

**Commit types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Code style (formatting)
- `refactor` - Code refactoring
- `test` - Test changes
- `chore` - Build/tooling changes
- `ci` - CI/CD changes

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

---

## 🎯 Contribution Types

### Core Protocol (Rust)

**Location**: `crates/`

**What to contribute:**
- New data types or features
- Parser/encoder improvements
- Performance optimizations
- Bug fixes

**Requirements:**
- Must pass `cargo test --workspace --all-features`
- Must pass `cargo clippy --workspace --all-features -- -D warnings`
- Must pass `cargo fmt --all -- --check`
- Add appropriate tests
- Update API documentation

### SDKs and Tools

**SDKs and Tools are now maintained in separate repositories.**
If you wish to contribute to an SDK or Tool, please visit its repository:

- **Python SDK**: [lnmp-sdk-python](https://github.com/lnmplang/lnmp-sdk-python)
- **JS/TS SDK**: [lnmp-sdk-js](https://github.com/lnmplang/lnmp-sdk-js)
- **Rust SDK**: [lnmp-sdk-rust](https://github.com/lnmplang/lnmp-sdk-rust)
- **Go SDK**: [lnmp-sdk-go](https://github.com/lnmplang/lnmp-sdk-go)
- **CLI Tool**: [lnmp-cli](https://github.com/lnmplang/lnmp-cli)
- **MCP Server**: [lnmp-mcp](https://github.com/lnmplang/lnmp-mcp)
- **VSCode Extension**: [lnmp-vscode](https://github.com/lnmplang/lnmp-vscode)

Please check the `CONTRIBUTING.md` in those specific repositories for guidelines.

### Documentation

**Locations**: `docs/`, `semver`, `spec/`

**What to contribute:**
- Typo fixes
- Clarifications
- Protocol specification updates
- Migration guides

### Examples

**Location**: [lnmp-examples](https://github.com/lnmplang/lnmp-examples) or [lnmp-showcase](https://github.com/lnmplang/lnmp-showcase)

Please contribute examples directly to those repositories.

---

## 📥 Pull Request Process

### Before Submitting

- [ ] **Tests pass** locally
- [ ] **Code formatted** (`cargo fmt`, `npm run format`)
- [ ] **Lints pass** (`cargo clippy`, `npm run lint`)
- [ ] **Documentation updated** (if API changed)
- [ ] **CHANGELOG.md updated** (for significant changes)
- [ ] **No merge conflicts** with main branch

### PR Template

When creating a PR, use this template:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] Tests pass
- [ ] Code formatted
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

### Review Process

1. **Automated checks** run (CI/CD)
2. **Maintainer review** (1-2 reviewers required)
3. **Address feedback** if requested
4. **Approval** from maintainers
5. **Merge** (squash merge preferred)

---

## 💻 Coding Standards

### Rust

**Follow Rust conventions:**
```rust
// ✅ Good
pub fn parse_record(input: &str) -> Result<LnmpRecord, Error> {
    // Implementation
}

// ❌ Bad
pub fn ParseRecord(Input: &str) -> Result<LnmpRecord, Error> {
    // Non-idiomatic naming
}
```

**Standards:**
- Use `rustfmt` default configuration
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Prefer `?` operator over `unwrap()`
- Add doc comments for public APIs
- Use `#[derive(Debug)]` where appropriate

### TypeScript

**Follow TypeScript best practices:**
```typescript
// ✅ Good
export interface LnmpRecord {
  fields: LnmpField[];
}

// ❌ Bad
export interface lnmp_record {
  fields: any;
}
```

**Standards:**
- Use strict TypeScript (`strict: true`)
- Prefer interfaces over types for objects
- Use `const` over `let` when possible
- Add JSDoc for exported functions

### Python

**Follow PEP 8:**
```python
# ✅ Good
def parse_record(input_text: str) -> LnmpRecord:
    """Parse LNMP text format."""
    pass

# ❌ Bad
def ParseRecord(InputText):
    pass
```

**Standards:**
- Use type hints
- Follow PEP 8 naming conventions
- Use docstrings for functions/classes
- Lint with `black` and `mypy`

---

## 🧪 Testing Requirements

### Rust Tests

```bash
# Run all tests
cargo test --workspace --all-features

# Run specific crate tests
cargo test -p lnmp-core

# Run with output
cargo test -- --nocapture
```

**Test coverage:**
- Unit tests for core logic
- Integration tests for API surface
- Property tests for parsers/encoders (using `proptest`)
- Doctests for examples in documentation

### TypeScript Tests

```bash
cd sdk/js
npm test                    # Run all tests
npm run test:integration    # Integration tests only
```

### Python Tests

```bash
cd sdk/python
poetry run pytest           # All tests
poetry run pytest --cov     # With coverage
```

### Writing Good Tests

```rust
#[test]
fn test_parse_simple_record() {
    // Arrange
    let input = "F12=14532\nF7=1";
    
    // Act
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();
    
    // Assert
    assert_eq!(record.len(), 2);
    assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
}
```

---

## 📚 Documentation Guidelines

### API Documentation

**Rust:**
```rust
/// Parses LNMP text format into a structured record.
///
/// # Arguments
/// * `input` - The LNMP text to parse
///
/// # Returns
/// A parsed `LnmpRecord` or an error if parsing fails.
///
/// # Example
/// ```
/// use lnmp::Parser;
/// let parser = Parser::new("F12=14532")?;
/// let record = parser.parse_record()?;
/// ```
pub fn parse_record(&mut self) -> Result<LnmpRecord, Error> {
    // ...
}
```

**TypeScript:**
```typescript
/**
 * Parses LNMP text format into a structured record.
 * @param input - The LNMP text to parse
 * @returns Parsed LnmpRecord
 * @throws {ParseError} If parsing fails
 * @example
 * ```typescript
 * const record = Core.parse("F12=14532");
 * ```
 */
export function parse(input: string): LnmpRecord {
    // ...
}
```

### Documentation Structure

Update these files when making changes:

| Change Type | Files to Update |
|-------------|----------------|
| New API | `/docs/api/rust/v0.5.md`, SDK-specific API.md |
| Breaking change | `/docs/migration/`, `CHANGELOG.md` |
| New feature | `README.md`, examples/ |
| Bug fix | `CHANGELOG.md` |

---

## 🚀 Release Process

### For Maintainers

**Rust Crates:**
1. Update versions in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run `cargo build --workspace` to update `Cargo.lock`
4. Run the LNMP compatibility suite (see `docs/compat-reporting-guide.md`) and write the results to `docs/lnmp-compat-matrix.md`.
5. Record the latest codec benchmark summary in `crates/lnmp-codec/OPTIMIZATIONS.md` (even if it’s “unchanged”).
6. Commit: `chore: bump version to 0.5.x`
7. Tag: `git tag v0.5.x`
8. Push: `git push && git push --tags`
9. GitHub Release triggers automatic publish

> These steps verify that the frozen `.lnmp` container rules remain deterministic (fixture table) and that our published performance claims are backed by fresh benchmark data.

For branch management expectations during release prep, see [`docs/branching-strategy.md`](docs/branching-strategy.md).

**NPM Packages:**
1. Run `./scripts/bump-version.sh 0.6.x`
2. Update `CHANGELOG.md`
3. Commit and push
4. Create GitHub Release
5. CI publishes to npm automatically

### Versioning Policy

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (`1.0.0`) - Breaking changes
- **MINOR** (`0.5.0`) - New features (backward compatible)
- **PATCH** (`0.5.1`) - Bug fixes

---

## 🆘 Getting Help

### Questions?

- **[GitHub Discussions](https://github.com/lnmplang/lnmp-protocol/discussions)** - Ask questions, share ideas
- **[GitHub Issues](https://github.com/lnmplang/lnmp-protocol/issues)** - Report bugs, request features
- **[Documentation](./docs/)** - Read the docs

### Stuck?

If you're stuck, don't hesitate to:
1. Ask in GitHub Discussions
2. Tag maintainers in your PR
3. Open a draft PR early for feedback

---

## 🏆 Recognition

Contributors are recognized in:
- `CHANGELOG.md` for their contributions
- GitHub contributors page
- Release notes

Thank you for contributing to LNMP! 🎉

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

See [LICENSE](./LICENSE) for details.
