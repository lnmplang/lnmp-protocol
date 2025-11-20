# Git subtree guide

This repository imports SDKs, tools, and examples from separate repositories using
`git subtree`. Subtrees embed copies of external repos into this mono-repository
while keeping the original repos active. They are intended for convenience (one
workspace for building, CI, and cross-language integration) while preserving
independent repositories for SDK maintainers.

What changed
- Imported the following external repositories as subtrees:
  - `lnmplang/lnmp-examples` → `examples/` (squashed)
  - `lnmplang/lnmp-sdk-go` → `sdk/go/` (squashed)
  - `lnmplang/lnmp-sdk-js` → `sdk/js/` (squashed)
  - `lnmplang/lnmp-sdk-rust` → `sdk/rust/` (squashed)
  - `lnmplang/lnmp-sdk-python` → `sdk/python/` (squashed)
  - `lnmplang/lnmp-cli` → `tools/cli/` (squashed)
  - `lnmplang/lnmp-mcp` → `tools/mcp/` (squashed)

Notes on squash
- The subtree imports used `--squash` to keep the history compact in this
  repository. If you prefer to keep the full commit history in this monorepo,
  we can re-import without `--squash`, but that increases the repository size.

Common subtree commands

- Add a subtree (example):
```
git remote add lnmp-examples https://github.com/lnmplang/lnmp-examples.git
git fetch lnmp-examples
git subtree add --prefix=examples lnmp-examples main --squash
```

- Pull updates from the subtree remote into this repo:
```
git fetch lnmp-examples
git subtree pull --prefix=examples lnmp-examples main --squash
```

- Quick commands for `tools/mcp` (from repo root):
```
git remote add lnmp-mcp https://github.com/lnmplang/lnmp-mcp.git   # one-time
git subtree pull --prefix=tools/mcp lnmp-mcp main --squash         # pull updates
git subtree push --prefix=tools/mcp lnmp-mcp main                  # publish changes
```

- Push a change from the subtree directory back to its upstream remote:
```
# Make changes under examples/ and commit to this repo; then:
git subtree push --prefix=examples lnmp-examples main
```
Note: pushing back requires that you have write access to the external remote.

Why use subtrees?
- Developers can `git clone` this repository to get a single working tree that
  includes SDKs and examples in the correct structure for building and cross
  language testing.
- SDKs remain in their own repos for independent release/versioning.

Developer Guidance
- If you maintain a subtree and prefer localized development, continue using
  the independent repo (e.g., `lnmp-sdk-go`) and push there normally. To
  synchronize your changes into `lnmp-protocol`, use `git subtree pull` or
  contact the repository owner.
- If you make changes in the monorepo and want them in the source repo, use
  `git subtree push --prefix=<prefix> <remote> main` (or target branch).

- We provide a small helper script at `scripts/subtree-sync.sh` to simplify
  common subtree operations. Example usage:

```
# add
./scripts/subtree-sync.sh add lnmplang/lnmp-examples examples main
# pull
./scripts/subtree-sync.sh pull lnmplang/lnmp-examples examples main
CI automation
- Subtree pulls: `.github/workflows/subtree-sync.yml` (scheduled + manual) pulls updates from external repos into the monorepo.
- Subtree pushes: `.github/workflows/subtree-push.yml` runs automatically on `main` when paths under `examples/`, `sdk/*`, or `tools/{cli,mcp}/` change, and can also be run manually. It pushes monorepo changes back to external repos. Requires a secret `SUBTREE_PUSH_TOKEN` (PAT with `repo` scope and write access to all target repos). For manual runs, pick the target subtree or choose `all`.
