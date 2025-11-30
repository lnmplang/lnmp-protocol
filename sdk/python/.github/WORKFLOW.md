# Python SDK Workflow Organization

## Overview

The Python SDK has its own self-contained CI/CD workflow located in `sdk/python/.github/workflows/`. This allows the SDK to be managed independently and potentially moved to its own repository in the future.

## Workflow Structure

### 1. **ci.yml** - Main CI/CD Pipeline

**Triggers:**
- Push to `main` or `develop` branches (when SDK files change)
- Pull requests to `main` (when SDK files change)
- Release events

**Jobs:**

#### Quality Check
- Runs `cargo fmt`, `cargo clippy`, and Rust unit tests
- Ensures code quality before proceeding

#### Python Tests
- Matrix strategy across Python 3.9-3.12
- Multi-platform: Linux, macOS, Windows
- Runs pytest with coverage reporting
- Uploads coverage to Codecov

#### Build Wheels
- Only runs on release events
- Builds native wheels for:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, arm64)
  - Windows (x86_64)
- Uploads wheels as artifacts

#### Publish to PyPI
- Only runs for non-prerelease releases
- Downloads all platform wheels
- Publishes to PyPI using trusted publisher (OIDC)

#### Benchmarks
- Runs on pushes to `main` only
- Executes performance benchmarks
- Uploads results as artifacts

## Path Filtering

All jobs use path filtering to only run when SDK-specific files change:
```yaml
paths:
  - 'sdk/python/**'
  - '.github/workflows/python-ci.yml'
```

## Independence from Main Repo

This structure allows the SDK to:
1. Have its own release cycle
2. Be tested independently
3. Maintain separate versioning
4. Eventually move to its own repository with minimal changes

## Future Migration

To move the SDK to a separate repository:
1. Copy `sdk/python/` to new repo root
2. Update path filters in workflow (remove `sdk/python/` prefix)
3. Update checkout paths
4. Set up PyPI trusted publisher for new repo
5. Update documentation links

## Environment Variables

The workflow uses:
- `CARGO_TERM_COLOR=always` - Colored Cargo output
- `RUST_BACKTRACE=1` - Full backtraces on errors

## Secrets Required

For PyPI publishing, configure:
- **Trusted Publisher (Recommended)**: Configure at https://pypi.org/manage/account/publishing/
  - Owner: `lnmplang`
  - Repository: `lnmp-protocol` (or new SDK repo name)
  - Workflow: `ci.yml`
  - Environment: `pypi`

No manual tokens required with trusted publishers!
