# LNMP v0.3 Compliance Test Suite

This directory contains the language-agnostic compliance test suite for LNMP v0.3 implementations. The test suite ensures that all LNMP implementations across different programming languages behave identically and conform to the specification.

## Overview

The compliance test suite is designed to validate:

1. **Structural Correctness**: Canonicalization, field ordering, nested structures
2. **Semantic Fidelity**: Type preservation, value normalization, checksums
3. **Error Handling**: Proper error classification and reporting

## Test Case Format

Test cases are defined in `test-cases.yaml` using a language-agnostic format that can be consumed by test runners in any programming language.

### Successful Parse Test Case

```yaml
- name: "Human-readable test name"
  category: "structural|semantic|error-handling"
  description: "Brief explanation of what is being tested"
  input: "LNMP-formatted string to parse"
  config:  # Optional configuration
    validate_checksums: true
    strict_mode: false
    max_nesting_depth: 10
  expected:
    fields:
      - fid: <field_id>
        type: <type_name>
        value: <value>
        checksum: <optional_checksum_hex>
```

### Error Test Case

```yaml
- name: "Error test name"
  category: "error-handling"
  description: "What error should be triggered"
  input: "Invalid LNMP input"
  expected:
    error: "<ErrorClassName>"
    message: "<error_message_pattern>"
    field_id: <optional_field_id>
    line: <optional_line_number>
    column: <optional_column_number>
```

### Round-Trip Test Case

```yaml
- name: "Round-trip test name"
  category: "structural"
  description: "Parse and re-encode consistency"
  input: "F23=admin;F12=1;F7=2"
  expected_canonical: "F7=2;F12=1;F23=admin"
```

## Field Types

The following type names are used in test cases:

- `int`: Integer value (i64)
- `float`: Floating-point value (f64)
- `bool`: Boolean value (true/false)
- `string`: String value
- `string_array`: Array of strings
- `nested_record`: Nested record structure
- `nested_array`: Array of nested records

## Configuration Options

Test cases can specify optional configuration:

- `validate_checksums` (bool): Enable checksum validation
- `strict_mode` (bool): Enable strict parsing mode (canonical format required)
- `normalize_values` (bool): Enable value normalization
- `max_nesting_depth` (int): Maximum allowed nesting depth
- `preserve_checksums` (bool): Preserve checksums during round-trip
- `equivalence_mapping` (map): Field-specific synonym mappings

## Test Categories

### Structural Tests

Tests in the `structural_tests` section validate:

- Field ordering and canonicalization
- Nested record parsing and encoding
- Nested array parsing and encoding
- Whitespace normalization
- Deep nesting support
- String array handling

### Semantic Tests

Tests in the `semantic_tests` section validate:

- Type fidelity (int, float, bool, string)
- Value normalization rules
- Semantic checksum computation and validation
- Equivalence mapping (synonyms)
- Type hint preservation

### Error Handling Tests

Tests in the `error_handling_tests` section validate:

- Lexical errors (invalid characters, unterminated strings)
- Syntactic errors (missing tokens, unclosed structures)
- Semantic errors (type mismatches, invalid checksums)
- Structural errors (nesting depth, duplicate fields)
- Edge cases (field ID boundaries)

### Round-Trip Tests

Tests in the `round_trip_tests` section validate:

- Parse → Encode → Parse consistency
- Canonical form generation
- Checksum preservation
- Structure preservation

## Error Classes

The following error classes should be recognized by implementations:

### Lexical Errors
- `InvalidCharacter`: Invalid character in input
- `UnterminatedString`: String literal not closed
- `InvalidEscapeSequence`: Invalid escape sequence in string

### Syntactic Errors
- `UnexpectedToken`: Unexpected token in input
- `InvalidFieldId`: Field ID is invalid or out of range

### Semantic Errors
- `TypeHintMismatch`: Value doesn't match declared type hint
- `ChecksumMismatch`: Checksum validation failed
- `InvalidChecksum`: Checksum format is invalid

### Structural Errors
- `NestingTooDeep`: Maximum nesting depth exceeded
- `InvalidNestedStructure`: Nested structure is malformed
- `DuplicateFieldId`: Same field ID appears multiple times
- `StrictModeViolation`: Input violates strict mode requirements

## Implementing a Test Runner

To implement a test runner for your language:

1. **Load Test Cases**: Parse `test-cases.yaml` to load all test cases
2. **Filter by Category**: Run tests from specific categories (structural, semantic, error-handling, round-trip)
3. **Execute Tests**: For each test case:
   - Parse the `input` string using your LNMP parser
   - Apply any `config` options specified
   - Compare the result against `expected`
   - For error cases, verify the error class and message pattern
4. **Report Results**: Generate a report showing pass/fail for each test

### Example Test Runner Structure

```
tests/compliance/<language>/
├── runner.{ext}           # Main test runner
├── test_structural.{ext}  # Structural tests
├── test_semantic.{ext}    # Semantic tests
├── test_errors.{ext}      # Error handling tests
└── README.md              # Language-specific instructions
```

## Language-Specific Test Runners

### Rust
Location: `tests/compliance/rust/`
Run: `cargo test --package lnmp-compliance`
Status: ✅ Implemented

### Python
Location: `tests/compliance/python/`
Run: `pytest tests/compliance/python/` or `python3 tests/compliance/python/runner.py`
Status: ✅ Implemented (awaiting Python LNMP implementation)

The Python test runner is fully functional and ready to validate a Python LNMP implementation. All tests currently skip until a Python implementation is integrated. See `tests/compliance/python/README.md` for details.

### TypeScript
Location: `tests/compliance/typescript/`
Run: `npm test -- compliance`
Status: ⏳ Planned

### C++
Location: `tests/compliance/cpp/`
Run: `ctest -R compliance`
Status: ⏳ Planned

## Adding New Test Cases

To add new test cases:

1. Identify the category (structural, semantic, error-handling, round-trip)
2. Add the test case to the appropriate section in `test-cases.yaml`
3. Follow the format documented above
4. Ensure the test case has:
   - A descriptive name
   - Clear description
   - Valid input
   - Complete expected output
5. Run all language test runners to verify the new test case

## Validation

All test cases should be validated to ensure:

- YAML syntax is correct
- All required fields are present
- Field IDs are valid (0-65535)
- Type names are recognized
- Error classes are defined
- Expected values match input types

## Version

This test suite is for LNMP v0.3.0 - Semantic Fidelity & Structural Extensibility.

## References

- [LNMP v0.3 Requirements](../../.kiro/specs/lnmp-v0.3-semantic-fidelity/requirements.md)
- [LNMP v0.3 Design](../../.kiro/specs/lnmp-v0.3-semantic-fidelity/design.md)
- [Grammar Specification](../../spec/grammar.md)
- [Error Classes](../../spec/error-classes.md)
