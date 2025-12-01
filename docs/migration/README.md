# LNMP Migration Guides

Step-by-step guides for migrating between LNMP protocol versions.

## 📋 Available Migration Paths

### Production Migrations
- **[v0.4 → v0.5](./v0.4-to-v0.5.md)** ⭐ **Recommended** - Latest stable version
  - Binary nested structures
  - Generic arrays (IntArray, FloatArray, BoolArray)
  - Quantization support
  - Embedding delta mode
  - Spatial protocol

### Legacy Migrations
- **[v0.3 → v0.4](./v0.3-to-v0.4.md)** - Historical reference
  - Basic nested structures
  - Type hints
  - Semantic checksums

### Tool-Specific Migrations
- **[CLI Tool](../../tools/cli/MIGRATION.md)** - Command-line interface updates

---

## 🎯 Quick Start: Migrating to v0.5

### For Rust Users

```toml
# Cargo.toml
[dependencies]
lnmp = "0.5"  # Update from 0.4
```

**Breaking changes:**
1. `StringArray` → Add support for `IntArray`, `FloatArray`, `BoolArray`
2. Nested structures now support binary encoding
3. New `quant` feature for quantization (optional)

### For TypeScript Users

```json
// package.json
{
  "dependencies": {
    "@lnmplang/lnmp": "^0.6.0"  // Aligned with Rust v0.5
  }
}
```

**New features:**
- Quantization APIs
- Spatial encoding
- Embedding delta operations

### For Python Users

```bash
pip install lnmp-python>=0.5.0
```

---

## 📚 Detailed Migration Guides

### Step-by-Step Process

1. **Read the migration guide** for your version
2. **Update dependencies** in your package manager
3. **Review breaking changes** section
4. **Update code** following examples
5. **Run tests** to verify compatibility
6. **Update documentation** if needed

### Migration Checklist

- [ ] Update dependency versions
- [ ] Review breaking changes
- [ ] Update type annotations (if using TypeScript/Python)
- [ ] Update binary encoding logic (if using custom encoders)
- [ ] Run test suite
- [ ] Update integration tests
- [ ] Review performance benchmarks
- [ ] Update API documentation

---

## 🔍 Version Comparison

| Feature | v0.3 | v0.4 | v0.5 |
|---------|------|------|------|
| Nested Records | ❌ | ✅ | ✅ |
| Generic Arrays | ❌ | ❌ | ✅ |
| Quantization | ❌ | ❌ | ✅ |
| Embedding Delta | ❌ | ❌ | ✅ |
| Spatial Protocol | ❌ | ❌ | ✅ |
| Binary Nested | ❌ | Limited | ✅ Full |

---

## 🆘 Migration Support

### Common Issues

**Q: My old binary data won't decode**  
A: v0.5 maintains backward compatibility for v0.4 binary format. Use `BinaryDecoder::with_version()` to specify format version.

**Q: Type errors after updating**  
A: Check if you're using new array types (`IntArray`, etc.). Update type annotations accordingly.

**Q: Performance degradation**  
A: v0.5 has optimizations. Run benchmarks and check if you're using deprecated APIs.

### Getting Help

- **[GitHub Issues](https://github.com/lnmplang/lnmp-protocol/issues)** - Report bugs or ask questions
- **[Discussions](https://github.com/lnmplang/lnmp-protocol/discussions)** - Community support
- **[Examples](../../examples/)** - Reference implementations

---

## 📝 Contributing

Found an issue in a migration guide? Please:
1. Open an issue with details
2. Submit a PR with corrections
3. Help others in discussions

---

## License

MIT License - See [LICENSE](../../LICENSE) for details
