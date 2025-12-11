# LNMP Compliance Runner (Rust)

This crate (`lnmp-compliance-tests`) is the reference runner for the YAML vectors under `tests/compliance/test-cases.yaml`. It enforces the REQ IDs defined in the modular specs and is wired into CI (see `conformance.yml` and `spec-fixtures.yml`).

## Capabilities

- Loads the YAML suite (structural, semantic, error, round-trip, lenient cases)
- Applies per-test configuration (strict mode, checksum validation, equivalence mappings, lenient parsing, etc.)
- Emits failure messages annotated with REQ IDs (e.g., `[REQ: REQ-CAN-TXT-01]`)
- Supports both library integration tests (`cargo test -p lnmp-compliance-tests`) and standalone execution (`cargo run -p lnmp-compliance-tests --bin lnmp-compliance-runner`)
- Provides the `lnmp-verify-examples` helper used by `spec-fixtures.yml` to validate `spec/examples/`

## Directory Layout

```
tests/compliance/rust/
├── Cargo.toml                  # package + binaries (runner + verify_examples)
├── README.md                   # this file
├── main.rs                     # CLI entry point for lnmp-compliance-runner
├── mod.rs                      # cargo test integration entry point
├── runner.rs                   # core runner/validators
├── test-driver.rs              # CLI wiring + lenient suite merge
├── test-cases-lenient.yaml     # additional lenient-mode vectors
├── verify_examples.rs          # spec fixture verifier
```

## Running the Suite

### As Cargo Tests

```bash
cargo test -p lnmp-compliance-tests
```

### As Standalone Binary

```bash
cargo run -p lnmp-compliance-tests --bin lnmp-compliance-runner
```

Optional flags (passed after `--`):

- `--category <name>` (structural, semantic, error-handling, round-trip, lenient)
- `--verbose`

### Verifying Spec Fixtures

`spec-fixtures.yml` calls the helper binary automatically, but you can run it locally:

```bash
cargo run -p lnmp-compliance-tests --bin lnmp-verify-examples
```

This ensures the canonical text/binary fixtures stored in `spec/examples/` round-trip through the parser/encoder pairings supported by LNMP v0.4.

## Adding/Editing Tests

1. Modify `tests/compliance/test-cases.yaml` (and `test-cases-lenient.yaml` if needed).
2. Include `requirements` arrays referencing the relevant REQ IDs.
3. Run the commands above to verify the Rust harness passes.
4. If fixtures change, update `spec/examples/` and re-run `lnmp-verify-examples`.

## References

- `tests/compliance/README.md` (suite overview)
- `spec/lnmp-*` modular documents (normative references)
- `.github/workflows/conformance.yml`
- `.github/workflows/spec-fixtures.yml`
