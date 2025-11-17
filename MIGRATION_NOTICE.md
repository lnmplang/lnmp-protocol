# Migration notice & local upgrade instructions

We recently cleaned repository history, removed large build artifacts from the
history, and normalized repository naming (older typos: `lnpm-*` and `lnmo-*` → canonical `lnmp-*`).

Please follow these steps if you have local clones that contain the old
history or folder names:

1) Re-clone or reset local `lnmp-protocol` clone (recommended):

```bash
# Option A: re-clone from scratch
git clone git@github.com:lnmplang/lnmp-protocol.git

# Option B: reset your existing repo (careful: this is destructive locally)
cd lnmp-protocol
git fetch --all
git reset --hard origin/main
git reflog expire --expire=now --all
git gc --prune=now --aggressive
```

2) Normalize local folder names (optional, but recommended):
 - If your local workspace still uses `lnpm-protocol` (older name), rename it to `lnmp-protocol` to match canonical usages and scripts.

```bash
# From the parent directory (one level above your repo folders):
mv lnpm-protocol lnmp-protocol
mv lnpm-mcp lnmp-mcp || true
mv lnmo-sdk-python lnmp-sdk-python || true
```

3) Install the pre-commit hook (recommended):
 - From the repo root run:
```bash
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
```

4) Work the way you normally do: the monorepo contains subtrees with SDKs
and examples, but the official sources remain in their own repos for releases.

If you run into any issues during migration, please create a new issue or
contact a repository maintainer.
