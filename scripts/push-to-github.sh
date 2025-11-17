#!/usr/bin/env bash
set -euo pipefail

# push-to-github.sh - helper script to init git, add remote, and push local branch

if [ $# -lt 1 ]; then
  echo "Usage: $0 <repo-slug> [branch] -- repo-slug example: lnmplang/lnmp-examples"
  exit 1
fi

REPO=$1
BRANCH=${2:-main}

## Note: gh (GitHub CLI) is NOT required for pushing. The push uses git and
## the remote HTTPS URL. Ensure your local git is configured for authentication.

URL="https://github.com/$REPO.git"

if [ ! -d .git ]; then
  git init
fi

git add .
git commit -m "chore: initial commit" || echo "no changes to commit"
git branch -M "$BRANCH"
git remote add origin "$URL" || git remote set-url origin "$URL"
git push -u origin "$BRANCH"

echo "Pushed to $URL (branch $BRANCH)"
