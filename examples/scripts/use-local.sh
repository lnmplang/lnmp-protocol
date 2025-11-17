#!/usr/bin/env bash
set -euo pipefail

# use-local.sh - Replace git dependency spec in examples/Cargo.toml with local path
# Usage: ./use-local.sh

CONFIG_FILE="examples/Cargo.toml"

# Determine canonical local repo directory (support older locally-named variants)
if [ -d "../../lnmp-protocol" ]; then
  LNMP_LOCAL_PATH="../../lnmp-protocol"
elif [ -d "../../lnpm-protocol" ]; then
  LNMP_LOCAL_PATH="../../lnpm-protocol"
else
  LNMP_LOCAL_PATH="../../lnmp-protocol"
  echo "Warning: neither ../../lnmp-protocol nor ../../lnpm-protocol exists; defaulting to ../../lnmp-protocol"
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Could not find $CONFIG_FILE"
  exit 1
fi

sed -i.bak -E "s|lnmp-core = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-core\" \}|lnmp-core = { path = \"${LNMP_LOCAL_PATH}/crates/lnmp-core\" }|g" $CONFIG_FILE
sed -i.bak -E "s|lnmp-codec = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-codec\" \}|lnmp-codec = { path = \"${LNMP_LOCAL_PATH}/crates/lnmp-codec\" }|g" $CONFIG_FILE
sed -i.bak -E "s|lnmp-llb = \{ git = \"https://github.com/lnmplang/lnmp-protocol.git\", branch = \"main\", package = \"lnmp-llb\" \}|lnmp-llb = { path = \"${LNMP_LOCAL_PATH}/crates/lnmp-llb\" }|g" $CONFIG_FILE

echo "Replaced git dependency references with local path variants in $CONFIG_FILE. Backup saved as $CONFIG_FILE.bak"
