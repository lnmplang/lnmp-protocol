# LNMP CLI Module Structure

## Core Modules

### `main.rs`
Entry point, command dispatch

### `cli.rs`
Clap command definitions (10 command groups)

### `commands/`
Command implementations:
- `container.rs` - Container operations
- `codec.rs` - Text codec
- `embedding.rs` - Vector operations
- `spatial.rs` - Spatial data
- `quant.rs` - Quantization
- `transport.rs` - Protocol bindings
- `envelope.rs` - Metadata envelope
- `convert.rs` - Format conversion
- `info.rs` - Information/diagnostics
- `validate.rs` - Validation/security

## Support Modules

### `config.rs`
Global CLI configuration via environment variables:
- Log level
- Output format
- Verbose/quiet modes
- Color support
- Validation profile

### `error.rs`
Structured error types:
- `CliError` enum with variants for Io, Codec, Transport, InvalidInput, etc.
- Auto-conversion from underlying error types

### `io.rs`
I/O helpers:
- File read/write (text & binary)
- stdin/stdout support (`"-"` marker)
- File existence checks
- Parent directory creation

### `print.rs`
Output formatters:
- `Printer` - Color-aware output (success/error/warning/info)
- JSON/text/hex formatting
- Table formatting
- Hex dump (simple & detailed)
- Byte size formatting

### `utils.rs`
General utilities:
- Hex preview/dump
- JSON pretty print
- Byte formatting
- Float list parsing
- String list parsing

## Usage Example

```rust
use lnmp_cli::{Config, Printer, read_input, write_output};

fn main() {
    let config = Config::from_env();
    let printer = Printer::new(config);
    
    printer.info("Starting processing...");
    
    match read_input("input.lnmp") {
        Ok(data) => {
            printer.success("File read successfully");
            // Process data...
        }
        Err(e) => {
            printer.error(&format!("Failed: {}", e));
        }
    }
}
```

## Architecture Benefits

1. **Separation of Concerns**: Each module has clear responsibility
2. **Reusability**: Common functions in shared modules
3. **Configurability**: Environment-based config
4. **Testability**: Isolated, testable modules
5. **Maintainability**: Easy to find and modify code
