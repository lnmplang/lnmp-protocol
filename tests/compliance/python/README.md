# LNMP Compliance Test Runner (Python)

This directory mirrors the language-agnostic compliance vectors in `tests/compliance/test-cases.yaml` so that a Python LNMP implementation can validate itself against the modular specification set (`spec/lnmp-*-spec.md`). The runner loads YAML, applies per-test config (strict mode, checksum validation, lenient parsing, etc.), and asserts the expected output/error while emitting the referenced REQ IDs.

## Requirements

- Python 3.8 or higher
- pytest
- PyYAML

## Installation

```bash
pip install pytest pyyaml
```

## Running Tests

Run all compliance tests:
```bash
pytest tests/compliance/python/
```

Verbose:
```bash
pytest tests/compliance/python/ -v
```

Run specific test category:
```bash
pytest tests/compliance/python/ -k structural
pytest tests/compliance/python/ -k semantic
pytest tests/compliance/python/ -k error_handling
pytest tests/compliance/python/ -k round_trip
```

## Structure

- `runner.py`: Loads `../test-cases.yaml`, applies config, validates results
- `test_compliance.py`: Thin pytest wrapper
- `README.md`: This document

## Test Case Format

Vectors come from `../test-cases.yaml` (see parent README). Each case includes a `requirements` array referencing the relevant REQ IDs defined in `spec/lnmp-*-spec.md`.

## Implementation Status

⚠️ Ready, awaiting a Python LNMP parser/encoder. All tests currently skip with a friendly message until an implementation is wired in.

### Integration Checklist

1. Install/import your LNMP Python package.  
2. Update `runner.py` where indicated (`TODO: integrate parser/encoder`).  
3. Ensure your parser supports strict/loose parsing, checksum validation, lenient sanitization, nested structures, etc.  
4. Run the commands above and confirm failures, if any, cite the correct REQ IDs.

## Future Work

- Hook into the official LNMP Python crate once available.
- Mirror the Rust fixture verifier if binary/text fixture checking is needed in this repo.
- Add optional benchmarking/fuzzing helpers once the implementation ships.

## References

- `tests/compliance/README.md` (suite overview & YAML format)
- Modular specs: `spec/lnmp-core-spec.md`, `spec/lnmp-text-format.md`, `spec/lnmp-binary-format.md`, `spec/lnmp-canonicalization.md`, `spec/lnmp-security-compliance.md`, `spec/lnmp-migration-versioning.md`
- `spec/grammar.md`, `spec/error-classes.md`
