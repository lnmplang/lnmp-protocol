#!/usr/bin/env bash
set -euo pipefail

# Validate repo naming: no references to lnpm- or lnmo- should be present in tracked files.
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # Search committed files, but ignore commented lines and migration docs
  OUT=$(git grep -n --full-name -e "\blnpm-\b" -e "\blnmo-\b" -- ':!*' || true)
  if [ -n "$OUT" ]; then
    # Filter out matches that occur in comments or the migration note
    FILTERED=$(echo "$OUT" | awk '$0 !~ /^[[:space:]]*#/ && $0 !~ /MIGRATION_NOTICE.md/' || true)
    if [ -n "$FILTERED" ]; then
      echo "ERROR: Found legacy repository name references (lnpm- or lnmo-) in files:" >&2
      echo "$FILTERED" >&2
      exit 2
    fi
  fi
fi

exit 0
