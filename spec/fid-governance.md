# LNMP Field ID Governance Specification

**Status:** Official  
**Version:** 1.0.0  
**Date:** 2025-12-17  
**Scope:** Field ID allocation, lifecycle management, backward compatibility

---

## Abstract

This specification defines the governance rules for LNMP Field IDs (FIDs). It establishes allocation policies, lifecycle management, breaking change rules, and compatibility requirements for all LNMP implementations including SDKs.

## 1. Introduction

### 1.1 Purpose

Field IDs are the semantic core of LNMP. Unlike string keys, FIDs provide:
- **Tokenizer efficiency** for LLM consumption
- **Compact binary encoding**
- **Deterministic ordering**

Without governance, FID conflicts and semantic drift would undermine these benefits at scale.

### 1.2 Scope

This specification applies to:
- LNMP Core Protocol (`lnmp-*` crates)
- Official SDKs (Python, TypeScript, Rust, Go)
- Third-party implementations claiming LNMP compliance

### 1.3 Design Principles

1. **Backward Compatibility First** - Same FID maintains same meaning forever
2. **SDK Reliability** - Core FIDs are stable; SDKs can hardcode them
3. **Minimal Overhead** - No runtime registry service required
4. **Single Source of Truth** - `registry/fids.yaml` is canonical

---

## 2. FID Space and Allocation

### 2.1 Numeric Range

LNMP uses 16-bit unsigned integers for FIDs (0-65535).

### 2.2 Range Policy

| Range | Name | Owner | Stability | Usage |
|-------|------|-------|-----------|-------|
| 0-255 | **Core** | Protocol Maintainers | LOCKED | Protocol-level fields, SDK constants |
| 256-16383 | **Standard** | Protocol Maintainers | STABLE | Common use cases, may deprecate |
| 16384-32767 | **Extended** | Contributors | EVOLVING | Domain-specific, PR required |
| 32768-65535 | **Private** | User-defined | UNSTABLE | Application-specific, no registry |

### 2.3 Range Details

#### Core (0-255)
- **LOCKED**: Once published, these FIDs NEVER change
- SDKs MAY hardcode these as constants
- Reserved for fundamental protocol concepts
- Examples: `entity_id`, `timestamp`, `version`

#### Standard (256-16383)  
- **STABLE**: Changes require deprecation period (2 minor versions)
- Common patterns: position, rotation, velocity, sensor data
- Must be registered in `registry/fids.yaml`

#### Extended (16384-32767)
- **EVOLVING**: Can be deprecated without extended period
- Domain-specific: robotics, IoT, finance, etc.
- Requires PR with justification

#### Private (32768-65535)
- **UNSTABLE**: No guarantees
- Not registered centrally
- Users manage conflicts themselves
- NOT recommended for public libraries

---

## 3. FID Lifecycle

```
┌──────────┐     ┌────────┐     ┌────────────┐     ┌─────────────┐
│ PROPOSED │ ──> │ ACTIVE │ ──> │ DEPRECATED │ ──> │ TOMBSTONED  │
└──────────┘     └────────┘     └────────────┘     └─────────────┘
     │                                                    │
     │              NEVER REUSE THIS FID                  │
     └────────────────────────────────────────────────────┘
```

### 3.1 States

| State | Description | Encode | Decode |
|-------|-------------|--------|--------|
| **PROPOSED** | Under review, not yet official | ❌ | ❌ |
| **ACTIVE** | Standard usage | ✅ | ✅ |
| **DEPRECATED** | No new usage, still supported | ⚠️ Warning | ✅ |
| **TOMBSTONED** | Historical only, never reuse | ❌ Error | ✅ Legacy |

### 3.2 Transition Rules

1. **PROPOSED → ACTIVE**: Merge to main after review
2. **ACTIVE → DEPRECATED**: Requires `deprecated_since` version
3. **DEPRECATED → TOMBSTONED**: After 2 minor versions minimum
4. **TOMBSTONED → ACTIVE**: FORBIDDEN (create new FID instead)

---

## 4. Breaking Change Rules

### 4.1 What Requires a New FID

| Change Type | Example | Action |
|-------------|---------|--------|
| Type change | `Int` → `String` | **New FID**, tombstone old |
| Unit change | `m/s` → `km/h` | **New FID**, tombstone old |
| Semantic change | `user_id` → `session_id` | **New FID**, tombstone old |
| Array type change | `IntArray` → `FloatArray` | **New FID**, tombstone old |

### 4.2 What's Safe (Non-Breaking)

| Change Type | Example | Action |
|-------------|---------|--------|
| Add new field | New `fid: 300` | Just add |
| Add optional hint | `:i` type hint | Just add |
| Deprecate field | Stop writing | Mark DEPRECATED |
| Update description | Clarify docs | Just update |

### 4.3 Golden Rule

> **"Same FID, Same Meaning, Forever"**
>
> If you need different meaning, create new FID. Period.

---

## 5. Registry Format

### 5.1 Location

```
lnmp-protocol/
├── registry/
│   ├── fids.yaml      # Official FID registry
│   └── schema.json    # Validation schema
```

### 5.2 YAML Structure

```yaml
metadata:
  version: "1.0.0"
  protocol_version: "0.5.13"

core:
  - fid: 1
    name: entity_id
    type: Int
    unit: null
    status: ACTIVE
    since: "0.1.0"
    description: "Unique entity identifier"

standard:
  - fid: 256
    name: position
    type: FloatArray
    unit: "m"
    status: ACTIVE
    since: "0.5.0"
    description: "[x, y, z] position coordinates"

extended: []
tombstoned: []
```

### 5.3 Field Schema

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `fid` | ✅ | int | 0-65535, unique |
| `name` | ✅ | string | snake_case identifier |
| `type` | ✅ | enum | Int, Float, Bool, String, *Array, Record |
| `unit` | ❌ | string | SI unit or null |
| `status` | ✅ | enum | PROPOSED/ACTIVE/DEPRECATED/TOMBSTONED |
| `since` | ✅ | string | Version when introduced |
| `deprecated_since` | ❌ | string | Version when deprecated |
| `description` | ❌ | string | Human-readable description |

---

## 6. Workflow

### 6.1 Adding a New FID

1. Check `registry/fids.yaml` - is concept already covered?
2. Choose appropriate range (standard: 256-16383, extended: 16384+)
3. Add entry to `registry/fids.yaml` with status `PROPOSED`
4. Open PR with title: `fid: Add F<number> for <concept>`
5. CI validates: uniqueness, range, schema compliance
6. After merge, status becomes `ACTIVE`

### 6.2 Deprecating a FID

1. Add `deprecated_since: "X.Y.Z"` to entry
2. Change status to `DEPRECATED`
3. Update all docs referencing this FID
4. SDK implementations SHOULD emit warning

### 6.3 Tombstoning a FID

1. Wait minimum 2 minor versions after deprecation
2. Change status to `TOMBSTONED`
3. FID number is NEVER reused

---

## 7. SDK Compliance

### 7.1 Requirements

SDKs claiming LNMP compliance MUST:

1. **Support Core FIDs** (0-255) as documented
2. **Decode all ACTIVE FIDs** correctly
3. **Warn on DEPRECATED FIDs** during encode
4. **Accept TOMBSTONED FIDs** for decode (legacy data)

### 7.2 Constants Generation

SDKs SHOULD generate constants from `registry/fids.yaml`:

**Rust:**
```rust
pub const FID_ENTITY_ID: u16 = 1;
pub const FID_POSITION: u16 = 256;
```

**TypeScript:**
```typescript
export const FID = {
  ENTITY_ID: 1,
  POSITION: 256,
} as const;
```

**Python:**
```python
class FID:
    ENTITY_ID = 1
    POSITION = 256
```

---

## 8. CI Enforcement

### 8.1 Validation Checks

CI MUST enforce on registry changes:

| Check | Description |
|-------|-------------|
| Schema compliance | YAML matches `schema.json` |
| FID uniqueness | No duplicate FIDs |
| Range compliance | FIDs in correct range for section |
| Name format | snake_case, alphanumeric |
| No tombstone reuse | TOMBSTONED FIDs not reactivated |

### 8.2 Breaking Change Detection

On PRs modifying existing FIDs, CI MUST:
1. Detect type/unit/semantic changes
2. Block if breaking change on ACTIVE FID
3. Require new FID + tombstone workflow

---

## 9. Compatibility with Existing Mechanisms

### 9.1 SchemaNegotiator Integration

The `SchemaNegotiator` in `lnmp-codec` handles runtime FID conflict detection:

- Registry provides **static** truth (compile-time)
- SchemaNegotiator provides **dynamic** validation (runtime)

Both are complementary, not redundant.

### 9.2 Dynamic FID Discovery (v0.5.14)

The Discovery Protocol enables runtime FID registry exchange:

```rust
// Request peer's FID registry
let request = negotiator.request_registry(None);

// After response
if negotiator.peer_supports_fid(12) {
    // Peer understands user_id field
}
```

**Message Types:**
- `RequestRegistry` - Query peer's FID definitions
- `RegistryResponse` - Full registry response
- `RegistryDelta` - Incremental sync

### 9.3 Registry Sync

`RegistrySync` in `lnmp-core` manages multi-peer version tracking:

```rust
let mut sync = RegistrySync::with_embedded();
sync.register_peer("peer-1".into(), "0.9.0".into());

if sync.is_ahead_of("peer-1") {
    let delta = sync.delta_fids_for("peer-1");
}
```

### 9.4 Envelope Layer

`dictionary_id` in envelope (future extension) CAN reference registry version:

```
#ENVELOPE dictionary_version=1.0.0
F12=14532
```

---

## 10. References

- [Protocol Buffers Language Guide](https://protobuf.dev/programming-guides/proto3/)
- [Confluent Schema Registry](https://docs.confluent.io/platform/current/schema-registry/)
- [LNMP Core Spec](./lnmp-core-spec.md)
- [Field ID Guidelines](../docs/field-id-guidelines.md)

---

## Changelog

- **2025-12-17**: Initial v1.0.0 specification
