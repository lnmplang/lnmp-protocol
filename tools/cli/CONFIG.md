# LNMP CLI Configuration

## Environment Variables

The LNMP CLI supports configuration through environment variables:

### Log Level
```bash
export LNMP_LOG_LEVEL=debug    # error, warn, info, debug, trace
```

### Default Output Format
```bash
export LNMP_FORMAT=json        # text, json, binary, compact
```

### Verbose/Quiet Mode
```bash
export LNMP_VERBOSE=1          # Enable verbose output
export LNMP_QUIET=1            # Enable quiet mode
```

### Color Output
```bash
export LNMP_COLOR=always       # auto, always, never
export NO_COLOR=1              # Standard env var to disable colors
```

### Validation Profile
```bash
export LNMP_PROFILE=strict     # loose, standard, strict
```

### Config File
```bash
export LNMP_CONFIG=~/.lnmprc   # Path to config file (future)
```

## Usage in Code

```rust
use lnmp_cli::Config;

fn main() {
    let config = Config::from_env();
    
    if config.is_verbose() {
        println!("Verbose mode enabled");
    }
    
    if config.use_colors() {
        println!("\x1b[32m✓\x1b[0m Green checkmark");
    }
}
```

## Examples

### Debug Mode with JSON Output
```bash
LNMP_LOG_LEVEL=debug LNMP_FORMAT=json lnmp-cli info version
```

### Quiet Mode with No Colors
```bash
LNMP_QUIET=1 NO_COLOR=1 lnmp-cli container inspect file.lnmp
```

### Strict Validation
```bash
LNMP_PROFILE=strict lnmp-cli validate check input.lnmp
```

## Future Enhancements

- [ ] Config file support (~/.lnmprc, .lnmp.toml)
- [ ] Per-project config (.lnmp/config.toml)
- [ ] Config override with CLI flags (--verbose, --format, etc.)
- [ ] Config validation and defaults
