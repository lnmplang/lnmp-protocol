# LNMP Branching Strategy

LNMP follows a Git workflow designed for regulated protocol work:

## Permanent Branches
- `main`: Always green, protected via GitHub branch protection. All changes land through Pull Requests (PRs) with passing CI.
- `release/x.y.z`: (Created as needed) Stabilization branches for new releases. Only hotfixes and release-critical changes allowed.

## Working Branches
- `feature/<short-desc>`: New features or large refactors.
- `fix/<short-desc>`: Bug fixes.
- `docs/<short-desc>`: Documentation-only updates.
- `ci/<short-desc>`: Build/CI tweaks.

Guidelines:
1. Branch from `main` (unless preparing a hotfix on a release branch).
2. Keep branches short-lived; merge via PR as soon as CI passes and reviews complete.
3. Delete local/remote feature branches after merge.

## Release Procedure (summary)
1. Create `release/<version>` from `main`.
2. Run release checklist (tests, compat matrix, benchmarks).
3. Tag the release (`git tag v<version>`), then merge release branch back into `main`.
4. Remove the release branch when no longer needed.

See `CONTRIBUTING.md#🚀-release-process` for detailed steps.
