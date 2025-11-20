#!/usr/bin/env bash
set -euo pipefail

echo "Running LNMP conformance targets..."
cargo test -p lnmp-codec --test container_conformance
cargo test -p lnmp-codec --test streaming_layer_tests
cargo test -p lnmp-codec --test delta_encoding_tests

echo "Running full test suite..."
cargo test --all --verbose

echo "Done."
