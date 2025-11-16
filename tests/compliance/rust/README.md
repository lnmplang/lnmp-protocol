# LNMP v0.3 Rust Compliance Test Runner

This directory contains the Rust implementation of the LNMP v0.3 compliance test suite. The test runner loads language-agnostic test cases from `test-cases.yaml` and validates the Rust LNMP implementation against them.

## Features

- **Language-agnostic test cases**: Tests are defined in YAML format and can be shared across implementations
- **Comprehensive coverage**: Tests cover structural, semantic, error-handling, and round-trip scenarios
- **Detailed error reporting**: Failed tests include detailed error messages with context
- **Category filtering**: Run specific test categories independently
- **Multiple execution modes**: Run as Cargo tests or as a standalone binary

## Running Tests

### As Cargo Tests

Run all compliance tests:
```bash
cargo test --package lnmp-compliance-tests
```

Run specific test categories:
```bash
cargo test --package lnmp-compliance-tests run_structural_tests
cargo test --package lnmp-compliance-tests run_semantic_tests
cargo test --package lnmp-compliance-tests run_error_handling_tests
cargo test --package lnmp-compliance-tests run_round_trip_tests
```

### As Standalone Binary

Run all tests:
```bash
cargo run --bin lnmp-compliance-runner
```

Run with verbose output:
```bash
cargo run --bin lnmp-compliance-runner -- --verbose
```

Run specific category:
```bash
cargo run --bin lnmp-compliance-runner -- --category structural
cargo run --bin lnmp-compliance-runner -- --category semantic
cargo run --bin lnmp-compliance-runner -- --category error-handling
cargo run --bin lnmp-compliance-runner -- --category round-trip
```

## Test Categories

### Structural Tests
Tests for canonicalization, field ordering, nested structures, and whitespace handling.

Examples:
- Basic field ordering
- Nested record parsing
- Nested array parsing
- Deep nesting
- Whitespace normalization

### Semantic Tests
Tests for type fidelity, value normalization, checksums, and semantic equivalence.

Examples:
- Integer/float/boolean type fidelity
- Value normalization (negative zero, trailing zeros)
- Checksum validation
- Semantic equivalence mapping

### Error Handling Tests
Tests for proper error detection and reporting.

Examples:
- Invalid field IDs
- Unterminated strings
- Type hint mismatches
- Unclosed nested structures
- Nesting depth limits

### Round-Trip Tests
Tests for parse → encode → parse consistency.

Examples:
- Basic field round-trip
- Nested structure round-trip
- Checksum preservation
- Complex structure round-trip

## Test Case Format

Test cases are defined in `tests/compliance/test-cases.yaml` using the following structure:

```yaml
- name: "Test name"
  category: "structural"
  description: "What this test validates"
  input: "F12=1;F7=2"
  expected:
    fields:
      - fid: 7
        type: int
        value: 2
      - fid: 12
        type: int
        value: 1
  config:
    strict_mode: false
    validate_checksums: false
```

For error tests:
```yaml
- name: "Error test name"
  category: "error-handling"
  description: "What error this test validates"
  input: "FXX=1"
  expected:
    error: "InvalidFieldId"
    message: "Field ID must be numeric"
```

## Implementation Details

### Test Runner Architecture

```
tests/compliance/rust/
├── mod.rs          # Integration test entry point
├── main.rs         # Standalone binary runner
├── runner.rs       # Core test runner implementation
├── Cargo.toml      # Package configuration
└── README.md       # This file
```

### Key Components

- **TestSuite**: Loads and parses test cases from YAML
- **TestRunner**: Executes test cases and collects results
- **TestCase**: Represents a single test case with input, expected output, and config
- **TestResult**: Represents the result of a test (Pass, Fail, Skip)

### Validation Logic

1. **Success Tests**: Parse input, validate against expected fields
2. **Error Tests**: Parse input, validate error type and message
3. **Round-Trip Tests**: Parse input, encode, compare with expected canonical form

## Adding New Tests

To add new test cases:

1. Edit `tests/compliance/test-cases.yaml`
2. Add test case to appropriate category
3. Run tests to validate

Example:
```yaml
structural_tests:
  - name: "My new test"
    category: "structural"
    description: "Tests my new feature"
    input: "F100={F1=test}"
    expected:
      fields:
        - fid: 100
          type: nested_record
          value:
            fields:
              - fid: 1
                type: string
                value: "test"
```

## Requirements Coverage

This test runner implements:
- **Requirement 11.2**: Language-agnostic test suite in portable YAML format
- **Requirement 11.3**: Reference test runner for Rust implementation
- **Requirement 11.4**: Negative test cases for all error classes
- **Requirement 11.5**: Test categories for canonicalization, type fidelity, nested structures, error classes, and semantic checksums

## Exit Codes

- `0`: All tests passed
- `1`: One or more tests failed or error occurred

## Dependencies

- `lnmp-core`: Core LNMP types
- `lnmp-codec`: Parser and encoder
- `serde`: Serialization framework
- `serde_yaml`: YAML parsing

## Future Enhancements

- [ ] Parallel test execution
- [ ] Test filtering by name pattern
- [ ] JSON output format for CI integration
- [ ] Performance benchmarking mode
- [ ] Test coverage reporting
