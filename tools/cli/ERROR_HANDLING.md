# LNMP CLI Error Handling Migration Guide

## Current State

Currently using `anyhow::Result` everywhere for error handling.

## New Error Model

Created `CliError` enum with structured error types:

```rust
pub enum CliError {
    Io(io::Error),              // File I/O errors
    Codec(LnmpError),           // LNMP parsing/encoding errors 
    BinaryCodec(BinaryError),   // Binary format errors
    Embedding(BinaryError),     // Embedding errors
    Quant(QuantError),          // Quantization errors
    Spatial(SpatialError),      // Spatial errors
    Transport(TransportError),  // HTTP/Kafka/gRPC/NATS errors
    Serialization(String),      // JSON/bincode errors
    InvalidInput(String),       // User input validation
    FileNotFound(String),       // Missing files
    Unsupported(String),        // Unsupported operations
    Other(String),              // Generic errors
}
```

## Migration Strategy

### Option 1: Gradual Migration (Recommended)

Keep `anyhow::Result` for now, introduce `CliError` incrementally:

1. Commands can use either `anyhow::Result` or `error::Result`
2. Convert `anyhow` → `CliError` at main.rs boundary
3. Migrate one module at a time

### Option 2: Full Migration

Replace all `anyhow::Result` with `error::Result`:

**Pros:**
- Better error categorization
- Type-safe error handling
- Clear error sources

**Cons:**
- Requires changing all function signatures
- Need `.map_err()` for String errors
- More verbose in some cases

## Usage Examples

### With new CliError

```rust
use crate::error::{CliError, Result};

fn read_lnmp_file(path: &Path) -> Result<LnmpRecord> {
    let text = std::fs::read_to_string(path)?;  // Auto-converts io::Error
    let mut parser = Parser::new(&text)?;        // Auto-converts LnmpError
    Ok(parser.parse_record()?)
}

fn validate_input(value: &str) -> Result<u16> {
    value.parse()
        .map_err(|_| CliError::InvalidInput(format!("Invalid number: {}", value)))
}
```

### Mixed approach

```rust
use anyhow::Result;
use crate::error::CliError;

fn command() -> Result<()> {
    // Use anyhow for simplicity
    let data = read_file(path)?;
    
    // Convert to CliError when needed
    let parsed = Parser::new(&data)
        .map_err(|e| CliError::Codec(e))?;
    
    Ok(())
}
```

## Recommendation

**Keep current `anyhow::Result` for now** because:
1. ✅ Already working and compiling
2. ✅ Simpler error propagation
3. ✅ Good error messages with context
4. ✅ Can add CliError later incrementally

**When to use `CliError`:**
- Library code that others will use
- When you need pattern matching on error types
- For better error categorization in logs
- When building error recovery logic

**Current approach is fine for CLI tool!** `anyhow` is perfect for applications.
