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
