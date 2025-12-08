#!/usr/bin/env zsh
set -euo pipefail

# Local reproduction script for .github/workflows/ci.yml
# Run from repository root: ./scripts/run_ci_locally.sh
# This script aims to replicate the CI pipeline steps for local debugging.

echo "=== Running local CI steps ==="

# Optional: detect matrix runner OS on local machine
DEFAULT_OS=$(uname)
if [ "$DEFAULT_OS" = "Darwin" ]; then
  CI_OS="macos-latest"
else
  CI_OS="ubuntu-latest"
fi

# 1) Build
echo "\n--- cargo build --all --verbose ---"
cargo build --all --verbose

# 2) Run tests
echo "\n--- cargo test --all --verbose ---"
cargo test --all --verbose

# 3) Examples (None in pure protocol repo)
echo "\n--- Skipping examples (moved to separate repo) ---"

# 4) Clippy (only on Linux runner in CI but we run it locally for debugging)
# Run with all targets, all features and deny warnings (may require installing extra components)
if command -v cargo-clippy >/dev/null 2>&1 || [ "${1:-}" = "--clippy" ]; then
  echo "\n--- cargo clippy --all-targets --all-features -- -D warnings ---"
  cargo clippy --all-targets --all-features -- -D warnings || true
else
  echo "\n--- Skipping clippy (not installed locally). Use: rustup component add clippy ---"
fi

# 5) Cargo fmt check
if command -v rustfmt >/dev/null 2>&1; then
  echo "\n--- cargo fmt --all -- --check ---"
  cargo fmt --all -- --check
else
  echo "\n--- Skipping cargo fmt check (install rustfmt with rustup component add rustfmt) ---"
fi

# 6) Build docs
echo "\n--- cargo doc --no-deps --all-features ---"
cargo doc --no-deps --all-features

echo "\n=== Local CI steps done ==="
