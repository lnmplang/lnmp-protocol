#!/usr/bin/env bash
set -euo pipefail

# Thin wrapper for `push-to-github.sh` at the top-level repo; it defaults to
# pushing the `lnmplang/lnmp-examples` repo if not provided a slug.

if [ -f "../lnmp-protocol/scripts/push-to-github.sh" ]; then
  TOP_SCRIPT="../lnmp-protocol/scripts/push-to-github.sh"
elif [ -f "../lnpm-protocol/scripts/push-to-github.sh" ]; then
  TOP_SCRIPT="../lnpm-protocol/scripts/push-to-github.sh"
else
  echo "Top-level repo script scripts/push-to-github.sh not found in ../lnmp-protocol or ../lnpm-protocol"
  exit 1
fi

${TOP_SCRIPT} "lnmplang/lnmp-examples" "$@"
