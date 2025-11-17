#!/usr/bin/env bash
set -euo pipefail

# create-github-repos.sh
# Uses GitHub CLI (gh) to create repos under the `lnmplang` organization and
# push the current local branches. Requires gh to be authenticated with the
# desired account and organization membership.

REPOS=(
  "lnmplang/lnmp-protocol" 
  "lnmplang/lnmp-examples"
  "lnmplang/lnmp-sdk-go"
  "lnmplang/lnmp-sdk-js"
  "lnmplang/lnmp-sdk-rust"
  "lnmplang/lnmp-sdk-python"
  "lnmplang/lnmp-cli"
  "lnmplang/lnmp-mcp"
)

for REPO in "${REPOS[@]}"; do
  ORG=$(echo "$REPO" | cut -d'/' -f1)
  NAME=$(echo "$REPO" | cut -d'/' -f2)
  echo "Create repo ${REPO} (public)"
  gh repo create "$REPO" --public --confirm || echo "repo may already exist"
done

echo "Done. Remember to push local branches and set default branch if needed."
