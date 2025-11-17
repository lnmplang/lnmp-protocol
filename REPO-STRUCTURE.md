# lnmp (lnmplang) repository structure and guidelines

This file documents the recommended layout for the `lnmp` ecosystem and the
recommended local developer workflows for the core repo, the examples repo,
and the language-specific SDK repos.

Primary guidelines
- `lnmp` is the canonical Rust implementation containing multiple crates in
  `crates/` (eg. `lnmp-core`, `lnmp-codec`, `lnmp-llb`, `lnmp-sfe`).
- Examples should live in separate repositories (`lnmp-examples`) to make
  the core repo smaller and language-agnostic.
- Language SDKs (Go/JS/Python/Rust) and CLI utilities should exist in their
  own repositories: `lnmp-sdk-go`, `lnmp-sdk-js`, `lnmo-sdk-python`, `lnmp-sdk-rust`, `lnmp-cli`.

Development workflows
1. Local development with a sibling workspace: clone `lnmp` and `lnmp-examples`
  side-by-side (same parent directory). For `lnmp-examples`, use the
  provided `scripts/use-local.sh` to switch `examples/Cargo.toml` from git
  dependencies to local path dependencies. Alternatively, enable the local
  path patch by creating `.cargo/config.toml` in `lnmp-examples` that
  redirects git sources to the local path:

```toml
[patch."https://github.com/lnmplang/lnmp.git"]
lnmp-core = { path = "../lnpm-protocol/crates/lnmp-core" }
lnmp-codec = { path = "../lnpm-protocol/crates/lnmp-codec" }
lnmp-llb = { path = "../lnpm-protocol/crates/lnmp-llb" }
```
2. CI should always use git or published crate dependencies. Prefer `tag` for
   reproducibility (eg. `v0.5.0`) instead of branch references.

Updating example’s dependencies
- Use the helper script `lnmp-examples/scripts/update-deps.sh` to update the
  `Cargo.toml` to a new release tag if desired.

Publishing and Releases
- Each SDK and crate should be released from its specific repo. Release
  automation should be set up similarly across languages: tag → release →
  publish (packages or registries).
