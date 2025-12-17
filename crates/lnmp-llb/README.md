# lnmp-llb

LNMP-LLM Bridge Layer - Optimization and conversion utilities for LLM-friendly data formats.

> **FID Registry:** All examples use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

## Overview

The LNMP-LLM Bridge Layer (LLB) provides tools for optimizing LNMP data for LLM consumption:

- **ShortForm Encoding**: Ultra-compact format for minimal token usage (7-12× reduction vs JSON)
- **Explain Mode**: Human-readable format with inline comments and type hints
- **Prompt Optimization**: Field value optimization for LLM context efficiency
- **Profile Configuration**: Integration with strict/loose validation profiles
- **Generic Array Support**: Optimized handling for `IntArray`, `FloatArray`, `BoolArray`

## Features

- 🎯 **ShortForm**: Minimal syntax for extreme token reduction (`12=14532 7=1`)
- 📖 **Explain Mode**: Self-documenting format with inline comments (`F12:i=14532  # user_id`)
- 🔄 **Bidirectional Conversion**: Binary ↔ ShortForm ↔ FullText
- ⚡ **High Performance**: Sub-microsecond conversions (see `PERFORMANCE.md`)
- 🔒 **Profile Support**: Strict validation and canonical enforcement
- 📊 **Array Optimization**: Specialized handling for numeric arrays

## Quick Start

```rust
use lnmp_llb::{LlbConverter, LlbConfig};

// Create converter with default config
let converter = LlbConverter::new(LlbConfig::default());

// ShortForm encoding (minimal tokens)
let shortform = "12=14532 7=1 23=[admin,dev]";
let binary = converter.shortform_to_binary(shortform)?;

// Explain mode (human-readable)
let explain = converter.binary_to_explain(&binary, "user_id", "is_active", "roles")?;
// Output: F12:i=14532  # user_id
//         F7:b=1       # is_active
//         F23:sa=[admin,dev]  # roles
```

## ShortForm Encoding

Ultra-compact format that removes all unnecessary syntax:

```rust
use lnmp_llb::LlbConverter;

let converter = LlbConverter::new(LlbConfig::default());

// Standard LNMP: F12=14532;F7=1;F23=[admin,dev]
// ShortForm:     12=14532 7=1 23=[admin,dev]

let record = converter.shortform_to_record("12=14532 7=1")?;
let shortform = converter.record_to_shortform(&record)?;
```

**Features:**
- No `F` prefix on field IDs
- Space-separated instead of semicolons/newlines
- Minimal quotes (only when necessary)
- 7-12× token reduction vs JSON

## Explain Mode

Self-documenting format with inline comments:

```rust
use lnmp_llb::ExplainEncoder;
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
record.add_field(LnmpField { fid: 7, value: LnmpValue::Bool(true) });

let encoder = ExplainEncoder::new();
let explained = encoder.encode_with_explanation(
    &record,
    &["user_id", "is_active"]
)?;

// Output:
// F12:i=14532  # user_id
// F7:b=1       # is_active
```

## Prompt Optimization

Optimize field values for LLM context:

```rust
use lnmp_llb::PromptOptimizer;
use lnmp_core::{LnmpField, LnmpValue};

let optimizer = PromptOptimizer::new();

let field = LnmpField {
    fid: 20,
    value: LnmpValue::String("  Hello World  ".to_string())
};

let optimized = optimizer.optimize_field(&field);
// Result: LnmpValue::String("Hello World") (trimmed)
```

**Optimizations:**
- String trimming and normalization
- Numeric array deduplication
- Boolean canonicalization
- Float precision reduction

## Profile Configuration

Integrate with strictness profiles for validation:

```rust
use lnmp_llb::{LlbConfig, LlbConverter};
use lnmp_core::profile::{LnmpProfile, StrictDeterministicConfig};

// Using predefined profile
let config = LlbConfig::new()
    .with_profile(LnmpProfile::Strict);
let converter = LlbConverter::new(config);

// Custom strict config
let strict_config = StrictDeterministicConfig {
    reject_unsorted_fields: true,
    reject_duplicate_fields: true,
    require_canonical_booleans: true,
    require_canonical_floats: true,
};
let config = LlbConfig::new()
    .with_strict_config(strict_config);
let converter = LlbConverter::new(config);

// Now parsing enforces strict rules
let result = converter.shortform_to_record("7=1 12=100")?; // ✓ Sorted
let error = converter.shortform_to_record("12=100 7=1");   // ✗ Unsorted (error)
```

## Generic Array Support

Optimized handling for numeric arrays:

```rust
use lnmp_core::{LnmpField, LnmpValue, RecordBuilder};
use lnmp_llb::LlbConverter;

// IntArray (F60=int_values from registry)
let record = RecordBuilder::new()
    .add_field(LnmpField {
        fid: 60,  // F60=int_values
        value: LnmpValue::IntArray(vec![1, 2, 3, 4, 5])
    })
    .build();

// FloatArray (F61=float_values from registry)
let record = RecordBuilder::new()
    .add_field(LnmpField {
        fid: 61,  // F61=float_values
        value: LnmpValue::FloatArray(vec![1.1, 2.2, 3.3])
    })
    .build();

// BoolArray (F62=bool_flags from registry)
let record = RecordBuilder::new()
    .add_field(LnmpField {
        fid: 62,  // F62=bool_flags
        value: LnmpValue::BoolArray(vec![true, false, true])
    })
    .build();

let converter = LlbConverter::new(LlbConfig::default());
let shortform = converter.record_to_shortform(&record)?;
// Output: 60=[1,2,3,4,5] 61=[1.1,2.2,3.3] 62=[1,0,1]
```

## Performance

See `PERFORMANCE.md` for detailed benchmarks. Key metrics:

- **ShortForm → Binary**: ~260-460 ns
- **Binary → ShortForm**: ~240-470 ns
- **Explain Encoding**: ~1.4 µs
- **Prompt Optimization**: ~350 ns per record

All operations are sub-microsecond, suitable for high-throughput LLM pipelines.

## Configuration

### LlbConfig

```rust
pub struct LlbConfig {
    pub profile_config: Option<StrictDeterministicConfig>,
}

impl LlbConfig {
    pub fn new() -> Self;
    pub fn with_profile(profile: LnmpProfile) -> Self;
    pub fn with_strict_config(config: StrictDeterministicConfig) -> Self;
}
```

## Examples

See `examples/` for complete examples:

- `profile_config.rs` - Strict validation with custom profile

Run examples:
```bash
cargo run --example profile_config
```

## Testing

```bash
# Run unit tests
cargo test --package lnmp-llb

# Run benchmarks
cargo bench --package lnmp-llb
```

## License

MIT OR Apache-2.0
