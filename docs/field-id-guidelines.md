# Field ID (FID) Guidelines

This document provides comprehensive guidelines for working with Field IDs in LNMP.

> **Official Specification:** For the complete governance rules, see [`spec/fid-governance.md`](../spec/fid-governance.md)

## Overview

Field IDs (FIDs) are numeric identifiers (0-65535) that form the core of LNMP's semantic data model. Every field in an LNMP record is identified by a unique FID rather than a string key.

**Example:**
```
F12=14532    # Field 12 = user_id
F7=1         # Field 7 = is_active
F23=[admin]  # Field 23 = roles
```

## FID Range Policy

| Range | Name | Stability | Usage |
|-------|------|-----------|-------|
| 0-255 | **Core** | LOCKED | Protocol-level, SDKs hardcode these |
| 256-16383 | **Standard** | STABLE | Common patterns, may deprecate |
| 16384-32767 | **Extended** | EVOLVING | Domain-specific |
| 32768-65535 | **Private** | UNSTABLE | Application-specific, not registered |

## Official Registry

All official FIDs are registered in [`registry/fids.yaml`](../registry/fids.yaml).

**To use a FID:**
1. Check the registry for existing FIDs covering your concept
2. If exists, use the existing FID
3. If not, follow the "Adding New FIDs" workflow below

## Breaking Change Rules

### What Requires a New FID

| Change Type | Example | Reason |
|-------------|---------|--------|
| **Type change** | `Int` → `String` | Parsers expect specific types |
| **Unit change** | `m/s` → `km/h` | Downstream calculations break |
| **Semantic change** | `user_id` → `session_id` | Meaning drift causes bugs |

### What's Safe (Non-Breaking)

| Change Type | Example | Why Safe |
|-------------|---------|----------|
| Add new field | New `F257` | Backward compatible |
| Add type hint | `F12=123` → `F12:i=123` | Existing parsers work |
| Deprecate field | Mark as DEPRECATED | Decoders still handle it |

### FID Lifecycle

```
PROPOSED → ACTIVE → DEPRECATED → TOMBSTONED
                                      ↓
                              NEVER REUSE
```

## Adding New FIDs

### Workflow

1. **Check registry** - Is the concept already covered?
   ```bash
   grep -i "your_concept" registry/fids.yaml
   ```

2. **Choose range**
   - Core (0-255): Only maintainers
   - Standard (256-16383): Common use cases
   - Extended (16384-32767): Domain-specific

3. **Add to registry**
   ```yaml
   - fid: 300
     name: your_field_name
     type: Int  # or Float, Bool, String, etc.
     unit: null  # or "m", "s", "°C", etc.
     status: PROPOSED
     since: "0.5.14"
     description: "Clear description of purpose"
   ```

4. **Open PR** with title: `fid: Add F257 for your_concept`

5. **CI validates** automatically (schema, uniqueness, range)

6. **After merge**, status becomes `ACTIVE`

## Best Practices

### 1. FIDs Represent Concepts, Not Instances

**Wrong:**
```
F256=[1.0,2.0,3.0]  # position for robot 1
F256=[4.0,5.0,6.0]  # position for robot 2 (same FID!)
```

**Correct:**
```
F1=1;F256=[1.0,2.0,3.0]   # entity_id=1, position
F1=2;F256=[4.0,5.0,6.0]   # entity_id=2, position
```

### 2. Use SchemaNegotiator for Runtime Compatibility

```rust
use lnmp_codec::binary::SchemaNegotiator;

let mut negotiator = SchemaNegotiator::v0_5()
    .with_fid_mappings(local_mappings);

// Detect conflicts before data exchange
let conflicts = SchemaNegotiator::detect_conflicts(
    &local_mappings,
    &remote_mappings
);
```

### 3. Document Units Clearly

```yaml
- fid: 256
  name: position
  type: FloatArray
  unit: "m"  # Always specify SI units
```

## SDK Integration

SDKs should generate constants from the registry:

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

## Related Resources

- [`spec/fid-governance.md`](../spec/fid-governance.md) - Official specification
- [`registry/fids.yaml`](../registry/fids.yaml) - Official FID registry
- [`SchemaNegotiator`](../crates/lnmp-codec/src/binary/negotiation.rs) - Runtime FID conflict detection
