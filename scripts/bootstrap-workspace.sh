#!/usr/bin/env bash
set -euo pipefail

echo "Bootstrapping workspace: running minimal builds & tests"

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "Running Rust workspace tests"
cd "$ROOT_DIR" && cargo build --all --verbose

for d in ../lnmp-sdk-rust ../lnmp-cli ../lnmp-examples ../lnmp-sdk-go ../lnmp-sdk-js ../lnmp-sdk-python ../lnmo-sdk-python ../lnpm-mcp ../lnmp-mcp; do
  if [ -d "$d" ]; then
    echo "Checking $d"
    pushd "$d" > /dev/null
    case $(basename "$d") in
      lnmp-sdk-go)
        go test ./... || true
        ;;
      lnmp-sdk-js)
        npm install || true
        npm test || true
        ;;
      lnmp-sdk-python|lnmo-sdk-python)
        python3 -m pip install -e . --user || true
        python3 -m pytest -q || true
        ;;
      lnmp-sdk-rust)
        cargo test --quiet || true
        ;;
      lnmp-cli)
        cargo build --quiet || true
        ;;
      lnmp-examples)
        cargo build --manifest-path examples/Cargo.toml --verbose || true
        ;;
      lnmp-mcp|lnpm-mcp)
        python3 -m pip install -e . --user || true
        python3 -m pytest -q || true
        ;;
      *)
        echo "Unknown directory: $d"
        ;;
    esac
    popd > /dev/null
  fi
done

echo "Workspace bootstrap finished."
