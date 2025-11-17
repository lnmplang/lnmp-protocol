# Scripts

This folder contains helper scripts to manage repo creation, pushing, and workspace bootstrapping. Use the scripts only after you have set up your local environment and authenticated with GitHub CLI (`gh`).

Available scripts:
- `create-github-repos.sh` - uses `gh repo create` to create the `lnmplang` repos.
- `push-to-github.sh` - initialize local git repo, commit, and push to provided slug.
- `bootstrap-workspace.sh` - run quick local build/test across sibling repos.
 - `subtree-sync.sh` - helper to add/pull/push external subtrees and simplify subtree sync workflow.

Examples:

```bash
cd scripts
./create-github-repos.sh
./push-to-github.sh lnmplang/lnmp-examples
```
