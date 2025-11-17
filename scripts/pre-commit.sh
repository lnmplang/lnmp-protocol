#!/usr/bin/env bash
set -euo pipefail

# Pre-commit script — should be used by developers to block accidental
# commits that include build artifacts or older repo-name typos.

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)
if [ -z "$STAGED_FILES" ]; then
  exit 0
fi

echo "$STAGED_FILES" | grep -E "(^|/)target(/|$)|(^|/)node_modules(/|$)|(^|/)dist(/|$)|(^|/)build(/|$)|\.egg-info(/|$)|\.rlib$|\.so$" && {
  echo "ERROR: Staged files include build artifacts (node_modules/, target/, build/, dist/ or .egg-info). Please remove them before committing." >&2
  exit 1
}

# Run repository name validation
scripts/checks/validate-repo-names.sh

exit 0
