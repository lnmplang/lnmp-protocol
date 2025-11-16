# LNMP v0.3 Python Compliance Test Runner

This directory contains the Python implementation of the LNMP v0.3 compliance test runner.

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

Run tests with verbose output:
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

Run with detailed output:
```bash
pytest tests/compliance/python/ -vv
```

## Structure

- `runner.py`: Core test runner that loads and executes test cases
- `test_compliance.py`: Pytest test suite that uses the runner
- `README.md`: This file

## Test Case Format

Test cases are loaded from `../test-cases.yaml` and executed against a Python LNMP implementation (to be provided).

## Implementation Status

⚠️ **Note**: This test runner is ready to use, but requires a Python LNMP implementation to validate against. The test runner will skip tests until a Python implementation is available.

To integrate with a Python LNMP implementation:

1. Install the Python LNMP package
2. Update the import in `runner.py` to use the actual implementation
3. Implement the parser and encoder interfaces

## Future Work

- Integrate with Python LNMP implementation when available
- Add performance benchmarks
- Add memory usage tests
- Add fuzzing tests
