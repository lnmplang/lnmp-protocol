# Contributing to lnmp-protocol

Thanks for your interest in contributing! This document highlights the
important workflows and links to migration notes if you are upgrading from an
older local workspace or have a copy with older folder names.

Key links
- `MIGRATION_NOTICE.md` — migration instructions and re-clone steps if the
  repository history was rewritten or you have outdated folder names.
- `REPO-STRUCTURE.md` — describes the mono-repo subtree layout and local dev
  tips (how to use `scripts/use-local.sh`, .cargo patch, etc.)

Development
- Use `./scripts/bootstrap-workspace.sh` to run a basic build and test of the
  repository and sibling SDKs.
- To use the new pre-commit hook,
  - install it with `ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit` or configure `pre-commit` if you use that tool.

Subtrees
- Subtrees are maintained independently and imported into this monorepo for
  convenience; maintainers should push releases from the upstream SDKs.
- To update subtrees in this monorepo, use `scripts/subtree-sync.sh` or the
  `Subtree Sync` GitHub Action which can be run manually.

If you run into any problems, open an issue and include the steps you took and
the error messages.
# Contributing to lnmp

Thanks for contributing! Below are the guidelines for contribution and daily
development.

1. For changes to protocol crates, make PRs against `lnmp` repository and keep
   each change scoped to a single crate where possible.
2. Add unit tests and update docs for major changes.
3. Versioning: follow semver for crate versions, update changelog.
4. For cross-language compliance tests, update the `tests/compliance/` where
   necessary.

Local development tips
- To run `lnmp-examples` locally and use the current local `lnmp` checkout,
  use the `.cargo/config.toml` override in the `lnmp-examples` repo that
  replaces `lnmp-core` and `lnmp-codec` with local path versions.
- CI should always reference a `tag` in the `lnmp` repo or published crates.

Release process
- Create a release PR with updated versions and CHANGELOG entries.
- Tag the release with `vX.Y.Z` and merge. Use CI to publish to the
  corresponding package manager (crates.io, npm, PyPI, Go proxy).
