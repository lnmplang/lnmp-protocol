# Shell Completion Guide

## Overview

The LNMP CLI supports shell completions for Bash, Zsh, Fish, PowerShell, and Elvish via clap.

## Generating Completions

The CLI can generate completion scripts for your shell:

```bash
# This feature will be available in a future version
# clap supports completion generation with the `completions` subcommand
```

## Manual Setup (Current)

Until automatic completion generation is added, you can use these manual completions:

### Bash

Add to your `~/.bashrc`:

```bash
# LNMP CLI completions
_lnmp_cli_completions() {
    local cur prev commands
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Top-level commands
    commands="container codec embedding spatial quant transport envelope convert info validate perf help"
    
    # Command groups
    case "${COMP_WORDS[1]}" in
        container)
            COMPREPLY=($(compgen -W "inspect decode encode metadata" -- "$cur"))
            ;;
        codec)
            COMPREPLY=($(compgen -W "parse format validate checksum normalize" -- "$cur"))
            ;;
        embedding)
            COMPREPLY=($(compgen -W "encode decode delta similarity" -- "$cur"))
            ;;
        spatial)
            COMPREPLY=($(compgen -W "encode decode delta stream validate" -- "$cur"))
            ;;
        quant)
            COMPREPLY=($(compgen -W "quantize dequantize adaptive batch benchmark" -- "$cur"))
            ;;
        transport)
            COMPREPLY=($(compgen -W "http kafka grpc nats" -- "$cur"))
            ;;
        envelope)
            COMPREPLY=($(compgen -W "create wrap unwrap extract" -- "$cur"))
            ;;
        convert)
            COMPREPLY=($(compgen -W "to-json from-json to-binary from-binary to-shortform from-shortform" -- "$cur"))
            ;;
        info)
            COMPREPLY=($(compgen -W "version features stats profile" -- "$cur"))
            ;;
        validate)
            COMPREPLY=($(compgen -W "sanitize check strict compliance" -- "$cur"))
            ;;
        perf)
            COMPREPLY=($(compgen -W "benchmark compare report" -- "$cur"))
            ;;
        *)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
    esac
}

complete -F _lnmp_cli_completions lnmp-cli
```

### Zsh

Add to your `~/.zshrc`:

```zsh
# LNMP CLI completions
_lnmp_cli() {
    local -a commands
    
    commands=(
        'container:Container file operations'
        'codec:Text codec operations'
        'embedding:Vector embedding operations'
        'spatial:Spatial data operations'
        'quant:Quantization operations'
        'transport:Transport protocol operations'
        'envelope:Envelope metadata operations'
        'convert:Format conversion utilities'
        'info:Information and diagnostics'
        'validate:Validation and security'
        'perf:Performance benchmarking'
        'help:Print help information'
    )
    
    _describe 'command' commands
}

compdef _lnmp_cli lnmp-cli
```

### Fish

Create `~/.config/fish/completions/lnmp-cli.fish`:

```fish
# LNMP CLI completions for Fish

# Top-level commands
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'container' -d 'Container file operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'codec' -d 'Text codec operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'embedding' -d 'Vector embedding operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'spatial' -d 'Spatial data operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'quant' -d 'Quantization operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'transport' -d 'Transport protocol operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'envelope' -d 'Envelope metadata operations'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'convert' -d 'Format conversion utilities'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'info' -d 'Information and diagnostics'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'validate' -d 'Validation and security'
complete -c lnmp-cli -f -n '__fish_use_subcommand' -a 'perf' -d 'Performance benchmarking'

# Container subcommands
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from container' -a 'inspect' -d 'Inspect container'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from container' -a 'decode' -d 'Decode container'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from container' -a 'encode' -d 'Encode container'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from container' -a 'metadata' -d 'Show metadata'

# Codec subcommands
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from codec' -a 'parse' -d 'Parse LNMP text'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from codec' -a 'format' -d 'Format LNMP text'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from codec' -a 'validate' -d 'Validate syntax'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from codec' -a 'checksum' -d 'Compute checksum'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from codec' -a 'normalize' -d 'Normalize format'

# Perf subcommands
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from perf' -a 'benchmark' -d 'Run benchmarks'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from perf' -a 'compare' -d 'Compare with other formats'
complete -c lnmp-cli -f -n '__fish_seen_subcommand_from perf' -a 'report' -d 'Generate reports'

# Global options
complete -c lnmp-cli -s h -l help -d 'Print help information'
complete -c lnmp-cli -s V -l version -d 'Print version information'
```

## Future: Automatic Generation

In a future version, completions will be auto-generated:

```bash
# Future feature
lnmp-cli completions bash > ~/.local/share/bash-completion/completions/lnmp-cli
lnmp-cli completions zsh > ~/.zfunc/_lnmp-cli
lnmp-cli completions fish > ~/.config/fish/completions/lnmp-cli.fish
```

This feature will use clap's built-in completion generation.

## Testing Completions

After setup, test your completions:

```bash
# Type this and press TAB
lnmp-cli <TAB>
# Should show: container, codec, embedding, spatial, etc.

# Type this and press TAB
lnmp-cli perf <TAB>
# Should show: benchmark, compare, report

# Type this and press TAB
lnmp-cli perf benchmark <TAB>
# Should show: codec, embedding, transport, full
```

## Troubleshooting

### Bash: Completions not working
```bash
# Ensure bash-completion is installed
# macOS: brew install bash-completion
# Ubuntu: sudo apt install bash-completion

# Reload your .bashrc
source ~/.bashrc
```

### Zsh: Completions not working
```bash
# Ensure compinit is loaded in .zshrc
autoload -Uz compinit && compinit

# Reload
source ~/.zshrc
```

### Fish: Completions not working
```bash
# Ensure completions directory exists
mkdir -p ~/.config/fish/completions

# Reload Fish
source ~/.config/fish/config.fish
```

## Contributing

To improve completions:
1. Test with your shell
2. Add missing subcommands
3. Add option completions (--help, --format, etc.)
4. Submit updates to the repository

## See Also

- [README.md](README.md) - Main documentation
- [MIGRATION.md](MIGRATION.md) - Migration from old CLI
- [clap documentation](https://docs.rs/clap) - For developers
