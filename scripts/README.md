# Scripts

This folder contains helper scripts to manage repo creation, pushing, and workspace bootstrapping. Use the scripts only after you have set up your local environment and authenticated with GitHub CLI (`gh`).

Available scripts:
- `create-github-repos.sh` - uses `gh repo create` to create the `lnmplang` repos.
- `push-to-github.sh` - initialize local git repo, commit, and push to provided slug.
- `bootstrap-workspace.sh` - run quick local build/test across sibling repos.
 - `subtree-sync.sh` - helper to add/pull/push external subtrees and simplify subtree sync workflow.
- `pre-commit.sh` - small pre-commit checker to block build artifacts and legacy repo name typos; **not installed by default**. To install into your local git hooks:

```bash
# From the repo root:
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
``` 
Alternatively, use the `pre-commit` tool to configure this hook if you prefer.

Examples:

```bash
cd scripts
./create-github-repos.sh
./push-to-github.sh lnmplang/lnmp-examples
```
