# Backward Compatibility Guide

## Overview

The LNMP CLI has been completely redesigned from 5 basic commands to 11 comprehensive command groups with 60+ subcommands. This guide helps users migrate from the old CLI to the new one.

## Quick Migration Table

| Old Command | New Command | Notes |
|-------------|-------------|-------|
| `lnmp-cli inspect file.lnmp` | `lnmp-cli container inspect file.lnmp` | Now under `container` group |
| `lnmp-cli decode file.lnmp` | `lnmp-cli container decode file.lnmp` | Now under `container` group |
| `lnmp-cli encode-text input.txt` | `lnmp-cli container encode --mode text input.txt` | Unified encode command |
| `lnmp-cli encode-binary input.txt` | `lnmp-cli container encode --mode binary input.txt` | Unified encode command |
| `lnmp-cli metadata file.lnmp` | `lnmp-cli container metadata file.lnmp` | Now under `container` group |

## Detailed Migration

### Old: `inspect`
```bash
# Old
lnmp-cli inspect file.lnmp --metadata-out meta.bin --metadata-hex

# New
lnmp-cli container inspect file.lnmp
# Metadata extraction is now separate:
lnmp-cli container metadata file.lnmp
```

### Old: `decode`
```bash
# Old
lnmp-cli decode file.lnmp --metadata-out meta.bin

# New
lnmp-cli container decode file.lnmp
```

### Old: `encode-text`
```bash
# Old
lnmp-cli encode-text input.txt -o output.lnmp

# New
lnmp-cli container encode --mode text input.txt -o output.lnmp
```

### Old: `encode-binary`
```bash
# Old
lnmp-cli encode-binary input.txt -o output.lnmp

# New
lnmp-cli container encode --mode binary input.txt -o output.lnmp
```

### Old: `metadata`
```bash
# Old
lnmp-cli metadata file.lnmp --dump meta.bin --raw

# New
lnmp-cli container metadata file.lnmp
```

## Shell Aliases for Compatibility

For users who want to keep using the old command names, add these aliases to your shell configuration:

**Bash/Zsh (~/.bashrc or ~/.zshrc):**
```bash
# LNMP CLI backward compatibility aliases
alias lnmp-inspect='lnmp-cli container inspect'
alias lnmp-decode='lnmp-cli container decode'
alias lnmp-encode-text='lnmp-cli container encode --mode text'
alias lnmp-encode-binary='lnmp-cli container encode --mode binary'
alias lnmp-metadata='lnmp-cli container metadata'
```

**Fish (~/.config/fish/config.fish):**
```fish
# LNMP CLI backward compatibility aliases
alias lnmp-inspect 'lnmp-cli container inspect'
alias lnmp-decode 'lnmp-cli container decode'
alias lnmp-encode-text 'lnmp-cli container encode --mode text'
alias lnmp-encode-binary 'lnmp-cli container encode --mode binary'
alias lnmp-metadata 'lnmp-cli container metadata'
```

## New Features Available

The redesigned CLI offers many new capabilities:

### Performance Benchmarking
```bash
# NEW: Benchmark LNMP performance
lnmp-cli perf benchmark codec
lnmp-cli perf compare json
lnmp-cli perf report summary
```

### Format Conversion
```bash
# NEW: Convert between formats
lnmp-cli convert to-json input.lnmp
lnmp-cli convert from-json input.json
lnmp-cli convert to-binary input.lnmp
```

### Vector Operations
```bash
# NEW: Work with embeddings
lnmp-cli embedding encode vector.txt
lnmp-cli embedding delta compute base.bin target.bin
lnmp-cli embedding similarity vec1.bin vec2.bin --metric cosine
```

### Quantization
```bash
# NEW: Compress vectors
lnmp-cli quant quantize vec.bin --scheme qint8
lnmp-cli quant adaptive vec.bin --target high
```

### Validation & Security
```bash
# NEW: Validate and sanitize
lnmp-cli validate sanitize untrusted.lnmp
lnmp-cli validate compliance file.lnmp
```

### Information & Diagnostics
```bash
# NEW: Get information
lnmp-cli info version
lnmp-cli info features
lnmp-cli info stats file.lnmp
```

## Breaking Changes

### 1. Command Structure
- **Old**: Flat command structure (`lnmp-cli <command>`)
- **New**: Grouped command structure (`lnmp-cli <group> <command>`)

### 2. Option Names
Some options have been renamed for consistency:
- `--metadata-out` → Use `container metadata` command instead
- `--metadata-hex` → Metadata is now in separate command
- `-o` / `--output` → Still supported

### 3. Output Format
- Text output is now more structured and formatted
- Use `LNMP_FORMAT=json` environment variable for JSON output
- Use `--quiet` for minimal output

### 4. Error Handling
- Errors are now more descriptive
- Exit codes are consistent across all commands
- Use `LNMP_LOG_LEVEL=debug` for verbose error messages

## Migration Checklist

- [ ] Update scripts to use new command structure
- [ ] Test critical workflows with new commands
- [ ] Add shell aliases if needed for compatibility
- [ ] Update documentation/README files
- [ ] Update CI/CD pipelines
- [ ] Train team members on new commands

## Getting Help

All commands support `--help`:
```bash
lnmp-cli --help                    # List all command groups
lnmp-cli container --help          # Container group help
lnmp-cli container inspect --help  # Specific command help
```

## Support

If you encounter issues during migration:
1. Check this guide for command mappings
2. Use `--help` for command-specific options
3. Review the main [README.md](README.md)
4. Check [ARCHITECTURE.md](ARCHITECTURE.md) for design details

## Timeline

- **Old CLI**: Deprecated as of v0.5.7
- **New CLI**: Available from v0.5.7
- **Support**: Both versions supported until v0.6.0
- **Removal**: Old commands will be removed in v0.6.0

We recommend migrating to the new CLI as soon as possible to take advantage of new features and improvements.
