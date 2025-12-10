## Summary

- Describe the change and why it’s needed
- Link related issues or RFCs

## Testing

- Commands run (e.g., `cargo test`, `cargo test -p lnmp-codec --test container_conformance`, etc.)
- Attach relevant output or screenshots

## Checklist

- [ ] `cargo fmt`
- [ ] `cargo clippy --all-targets --all-features`
- [ ] Module impact reviewed (check all that apply, run linked checklist/tests):
  - [ ] Core (`docs/core-ci-checklist.md`)
  - [ ] Codec (`docs/codec-ci-checklist.md`)
  - [ ] Transport (`docs/transport-ci-checklist.md`)
  - [ ] Envelope (describe tests in “Testing”)
  - [ ] Net (describe tests in “Testing”)
  - [ ] Quant (describe tests in “Testing”)
  - [ ] LLB (describe tests in “Testing”)
  - [ ] Sanitize (describe tests in “Testing”)
  - [ ] Spatial (describe tests in “Testing”)
  - [ ] Other (list modules/tests)
- [ ] Updated compatibility matrix / benchmarks (if protocol behavior changed)
  - [ ] Followed [`docs/compat-reporting-guide.md`](../docs/compat-reporting-guide.md)
  - [ ] Added latest summary to `crates/lnmp-codec/OPTIMIZATIONS.md` (if codec benchmarks were run)
- [ ] Updated docs/spec/CHANGELOG as needed
