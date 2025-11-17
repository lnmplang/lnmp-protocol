#!/usr/bin/env bash
set -euo pipefail

# Validate repo naming: no references to lnpm- or lnmo- should be present in tracked files.
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # Search committed files
  if git grep -n --full-name -e "\blnpm-\b" -e "\blnmo-\b" -- ':!*' >/dev/null 2>&1; then
    echo "ERROR: Found legacy repository name references (lnpm- or lnmo-) in files:" >&2
    git grep -n --full-name -e "\blnpm-\b" -e "\blnmo-\b" -- ':!*' >&2
    exit 2
  fi
fi

exit 0
