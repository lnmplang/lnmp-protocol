#!/usr/bin/env bash
set -euo pipefail

# normalize-local-names.sh
# Rename local sibling folders to canonical names if they exist with older names
# Usage: ./normalize-local-names.sh [--yes]

DRY_RUN=1
if [ "${1:-}" = "--yes" ]; then
  DRY_RUN=0
fi

declare -A renames
renames=(
  ["lnpm-protocol"]="lnmp-protocol"
  ["lnpm-mcp"]="lnmp-mcp"
  ["lnmo-sdk-python"]="lnmp-sdk-python"
)

echo "Local rename operations (dry-run=${DRY_RUN}):"
for old in "${!renames[@]}"; do
  new=${renames[$old]}
  if [ -d "../$old" ] && [ ! -d "../$new" ]; then
    echo "Rename: ../$old -> ../$new"
    if [ "$DRY_RUN" -eq 0 ]; then
      mv "../$old" "../$new"
      echo "Renamed ../$old -> ../$new"
    fi
  fi
done

echo "Rename complete (dry-run=${DRY_RUN}). If you ran with --yes, you may need to update your shell / IDE to point to new folders." 
