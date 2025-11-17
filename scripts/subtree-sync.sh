#!/usr/bin/env bash
set -euo pipefail

OP=${1:-}
REMOTE=${2:-}
PREFIX=${3:-}
BRANCH=${4:-main}
SQUASH=${5:-squash}

if [[ -z "$OP" || -z "$REMOTE" || -z "$PREFIX" ]]; then
  echo "Usage: $0 add|pull|push <remote> <prefix> [branch] [squash|no-squash]"
  exit 2
fi

case "$OP" in
  add)
    git remote add "$REMOTE" "https://github.com/$REMOTE.git" || true
    git fetch "$REMOTE"
    if [[ "$SQUASH" == "no-squash" ]]; then
      git subtree add --prefix="$PREFIX" "$REMOTE" "$BRANCH"
    else
      git subtree add --prefix="$PREFIX" "$REMOTE" "$BRANCH" --squash
    fi
    ;;
  pull)
    git fetch "$REMOTE"
    if [[ "$SQUASH" == "no-squash" ]]; then
      git subtree pull --prefix="$PREFIX" "$REMOTE" "$BRANCH"
    else
      git subtree pull --prefix="$PREFIX" "$REMOTE" "$BRANCH" --squash
    fi
    ;;
  push)
    git subtree push --prefix="$PREFIX" "$REMOTE" "$BRANCH"
    ;;
  *)
    echo "Invalid operation: $OP" >&2
    exit 2
    ;;
esac

echo "Done."
