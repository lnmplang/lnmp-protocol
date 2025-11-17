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
  own repositories: `lnmp-sdk-go`, `lnmp-sdk-js`, `lnmp-sdk-python`, `lnmp-sdk-rust`, `lnmp-cli`.
 - The repo imports SDKs/examples/CLI/MCP via `git subtree` for developer
   convenience, so the mono-repo contains them in these locations:
     - `examples/` (lnmp-examples)
     - `sdk/go/` (lnmp-sdk-go)
     - `sdk/js/` (lnmp-sdk-js)
     - `sdk/python/` (lnmp-sdk-python)
     - `sdk/rust/` (lnmp-sdk-rust)
     - `tools/cli/` (lnmp-cli)
     - `tools/mcp/` (lnmp-mcp)

Development workflows
1. Local development with a sibling workspace: clone `lnmp` and `lnmp-examples`
  side-by-side (same parent directory). For `lnmp-examples`, use the
  provided `scripts/use-local.sh` to switch `examples/Cargo.toml` from git
  dependencies to local path dependencies. Alternatively, enable the local
  path patch by creating `.cargo/config.toml` in `lnmp-examples` that
  redirects git sources to the local path:
   If you use this mono-repo, the SDKs and examples are available in `sdk/` and
   `examples/` thanks to subtrees. Keep in mind the official source of
   truth for SDKs is still the independent repo.

  Note: some local folder names used in this workspace historically use the
  `lnpm-` prefix (eg. `lnpm-protocol`) due to earlier typos. For clarity and
  consistency prefer the canonical remote repo name `lnmplang/lnmp-protocol` in
  docs and scripts, but feel free to keep your local folder name as-is — the
  helper scripts in `scripts/` tolerate both local path names.

```toml
[patch."https://github.com/lnmplang/lnmp-protocol.git"]
lnmp-core = { path = "../lnmp-protocol/crates/lnmp-core" }
lnmp-codec = { path = "../lnmp-protocol/crates/lnmp-codec" }
lnmp-llb = { path = "../lnmp-protocol/crates/lnmp-llb" }
# If your local workspace still uses the older folder name, use:
# lnmp-core = { path = "../lnpm-protocol/crates/lnmp-core" }
# lnmp-codec = { path = "../lnpm-protocol/crates/lnmp-codec" }
# lnmp-llb = { path = "../lnpm-protocol/crates/lnmp-llb" }
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
