# LNMP Compatibility Reporting Guide

Goal: capture the status of `.lnmp` container fixtures (container/streaming/delta) and associated benchmarks after every CI run or release cut, then surface the results in `docs/lnmp-compat-matrix.md` and module optimization notes.

## Required Commands

```
# Containers
cargo test -p lnmp-codec --test container_conformance

# Streaming layer
cargo test -p lnmp-codec --test streaming_layer_tests

# Delta encoding
cargo test -p lnmp-codec --test delta_encoding_tests

# Optional benchmarks (record summary in OPTIMIZATIONS.md)
cargo bench -p lnmp-codec
```

## CI Workflow Template

1. Run the commands above on every merge to `main` (or nightly). Mark the job as failed if any command fails.
2. Collect the short commit (`git rev-parse --short HEAD`) and the current UTC date.
3. Append/update a row in the table inside `docs/lnmp-compat-matrix.md` with ✅/❌ for Container/Streaming/Delta columns.
4. (Optional) Capture the last few lines of `cargo bench -p lnmp-codec` and paste them into `crates/lnmp-codec/OPTIMIZATIONS.md` under a “Latest Benchmarks” subsection.
5. Commit the doc updates as part of the CI run (or create an automated PR).

### Example GitHub Actions Snippet (conceptual)

```yaml
- name: Run LNMP fixture checks
  run: |
    cargo test -p lnmp-codec --test container_conformance
    cargo test -p lnmp-codec --test streaming_layer_tests
    cargo test -p lnmp-codec --test delta_encoding_tests
    echo "DATE=$(date -u +%F)" >> $GITHUB_ENV
    echo "COMMIT=$(git rev-parse --short HEAD)" >> $GITHUB_ENV

- name: Update compat matrix
  run: |
    python scripts/update_matrix.py "$DATE" "$COMMIT" ✅ ✅ ✅
```

> `update_matrix.py` is an example placeholder; implement using any tooling preferred by your CI environment (Shell, Python, etc.). The key idea is to automate table updates rather than editing manually.

## Manual Workflow

When running locally:
1. Execute the commands above.
2. Open `docs/lnmp-compat-matrix.md` and replace the `_pending automation_` row with the new results (include notes if failures occurred).
3. If benchmarks were run, append the throughput/latency summary to `crates/lnmp-codec/OPTIMIZATIONS.md`.
4. Commit the changes alongside the code update so reviewers can see fixture health at a glance.
