# LNMP Migration & Versioning Guide

**Status:** Working Draft (modularization of v0.4 RFC §10 + README highlights)  
**Scope:** Version timeline, feature deltas, compatibility requirements, and negotiation guidance grounded in repository artifacts.  
**Audience:** Integrators upgrading between LNMP versions, SDK maintainers, and protocol reviewers.

---

## 1. Version Timeline (Grounded in Repo History)

| Version | Key Features | Evidence |
|---------|--------------|----------|
| v0.3 | Nested structures (text), semantic checksums, ShortForm encoding. | `README.md` (feature list), `crates/lnmp-codec/src/encoder.rs` nested support, `lnmp-core/src/checksum.rs`. |
| v0.4 | Binary format (text↔binary round-trip), VarInt, canonical guarantees. | `spec/lnmp-v0.4-rfc.md`, `crates/lnmp-codec/src/binary/*`. |
| v0.5.x | Binary nested structures, schema negotiation, transport bindings, sanitization, spatial/quant modules. | `README.md`, `crates/lnmp-codec/tests/v05_integration_tests.rs`, `crates/lnmp-core/examples/v05_schema_negotiation.rs`. |

---

## 2. Migration Requirements

- **REQ-MIG-01:** Upgrading from v0.3 → v0.4 MUST preserve text canonicalization semantics; binary encoding is additive. Implementations MUST continue to accept v0.3 text and produce v0.4 canonical output.  
  - Evidence: `tests/backward_compatibility_tests.rs:145-417`.

- **REQ-MIG-02:** v0.5 encoders MUST negotiate binary version (`0x04` vs `0x05`) with peers; falling back to `0x04` when nested data absent is allowed.  
  - Evidence: Schema negotiation example/test; `lnmp-core/examples/v05_schema_negotiation.rs`.

- **REQ-MIG-03:** Feature flags (checksums required, canonical-only, delta encoding) MUST be advertised before sending payloads needing them.  
  - Evidence: `schema_negotiation_tests.rs:170-325`.

---

## 3. Compatibility Matrix

| Capability | v0.3 | v0.4 | v0.5 |
|------------|------|------|------|
| Text canonicalization | ✅ | ✅ | ✅ |
| Binary encoding | ❌ | ✅ (flat) | ✅ (nested) |
| Semantic checksums | ✅ | ✅ | ✅ |
| Schema negotiation | ❌ | ❌ | ✅ |
| Sanitizer module | Experimental | Experimental | ✅ (`lnmp-sanitize`) |

(Data taken from README + crate features.)

---

## 4. Best Practices

1. **Dual-stack support:** Maintain both text and binary encoders until all peers negotiate `0x05`.  
2. **Test vectors:** Use `tests/binary_roundtrip.rs` fixtures for regression detection when upgrading libraries.  
3. **Change logging:** Mirror spec updates in `CHANGELOG.md` to match release artifacts.

---

## 5. Next Steps

- Automate generation of this matrix from `Cargo.toml` feature flags.  
- Provide sample negotiation transcripts in `docs/` to aid SDK implementers.
