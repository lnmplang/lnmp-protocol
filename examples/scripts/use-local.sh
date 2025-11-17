#!/usr/bin/env bash
set -euo pipefail

# use-local.sh - Replace git dependency spec in examples/Cargo.toml with local path
# Usage: ./use-local.sh

CONFIG_FILE="examples/Cargo.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Could not find $CONFIG_FILE"
  exit 1
fi

sed -i.bak -E "s|lnmp-core = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-core\" \}|lnmp-core = { path = \"../../lnpm-protocol/crates/lnmp-core\" }|g" $CONFIG_FILE
sed -i.bak -E "s|lnmp-codec = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-codec\" \}|lnmp-codec = { path = \"../../lnpm-protocol/crates/lnmp-codec\" }|g" $CONFIG_FILE
sed -i.bak -E "s|lnmp-llb = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-llb\" \}|lnmp-llb = { path = \"../../lnpm-protocol/crates/lnmp-llb\" }|g" $CONFIG_FILE

echo "Replaced git dependency references with local path variants in $CONFIG_FILE. Backup saved as $CONFIG_FILE.bak"
