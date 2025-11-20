#!/usr/bin/env bash
set -euo pipefail

ARTIFACTS_DIR=${1:-artifacts}
FIXTURES_DIR="fixtures"
TARGET="${ARTIFACTS_DIR}/lnmp-fixtures-v1.tar.gz"

if [ ! -d "$FIXTURES_DIR" ]; then
  echo "Missing fixtures directory: $FIXTURES_DIR" >&2
  exit 1
fi

mkdir -p "$ARTIFACTS_DIR"
tar -czf "$TARGET" -C "$FIXTURES_DIR" .
echo "Wrote fixture bundle: $TARGET"
