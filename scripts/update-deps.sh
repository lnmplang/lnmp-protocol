#!/usr/bin/env bash
set -euo pipefail

# update-deps.sh - Replace branch with a tag in examples/Cargo.toml
# Usage: ./update-deps.sh v0.5.0

if [ $# -lt 1 ]; then
  echo "Usage: $0 <tag>
Example: $0 v0.5.0"
  exit 1
fi

TAG=$1
CONFIG_FILE="examples/Cargo.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Could not find $CONFIG_FILE"
  exit 1
fi

sed -i.bak -E "s/(lnmp-core = \{ git = \"https:\/\/github.com\/lnmplang\/lnmp.git\", )branch = \"main\"/\1tag = \"$TAG\"/g" $CONFIG_FILE
sed -i.bak -E "s/(lnmp-codec = \{ git = \"https:\/\/github.com\/lnmplang\/lnmp.git\", )branch = \"main\"/\1tag = \"$TAG\"/g" $CONFIG_FILE
sed -i.bak -E "s/(lnmp-llb = \{ git = \"https:\/\/github.com\/lnmplang\/lnmp.git\", )branch = \"main\"/\1tag = \"$TAG\"/g" $CONFIG_FILE

echo "Updated $CONFIG_FILE with tag $TAG. A backup file is saved as $CONFIG_FILE.bak"
