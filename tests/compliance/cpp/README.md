# LNMP v0.3 C++ Compliance Test Runner

This directory contains the C++ compliance test runner for LNMP v0.3. It loads test cases from the language-agnostic `test-cases.yaml` file and executes them against a C++ LNMP implementation.

## Overview

The C++ test runner is designed to:
- Load test cases from YAML format
- Execute tests against a C++ LNMP implementation
- Report pass/fail results with detailed error messages
- Support filtering by test category
- Integrate with Google Test framework

## Dependencies

- **CMake** (>= 3.14): Build system
- **Google Test**: Testing framework
- **yaml-cpp**: YAML parsing library
- **C++17**: Required language standard

## Installation

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install cmake libgtest-dev libyaml-cpp-dev
```

### macOS (Homebrew)

```bash
brew install cmake googletest yaml-cpp
```

### Building from Source

```bash
cd tests/compliance/cpp
mkdir build
cd build
cmake ..
make
```

## Usage

### Run All Tests

```bash
./compliance_runner
```

### Run Tests in a Specific Category

```bash
./compliance_runner --category structural
./compliance_runner --category semantic
./compliance_runner --category error-handling
./compliance_runner --category round-trip
```

### Verbose Output

```bash
./compliance_runner --verbose
```

### Specify Test File Path

```bash
./compliance_runner --file /path/to/test-cases.yaml
```

### Using Google Test

```bash
./compliance_runner --gtest_filter="*"
```

## Project Structure

```
cpp/
├── CMakeLists.txt       # CMake build configuration
├── runner.cpp           # Main entry point
├── test_runner.h        # Test runner interface
├── test_runner.cpp      # Test runner implementation
└── README.md            # This file
```

## Implementation Status

**Current Status**: Test runner framework complete, awaiting C++ LNMP implementation.

The test runner is fully implemented and ready to integrate with a C++ LNMP parser/encoder. Currently, all tests are skipped with the message "C++ LNMP implementation not yet available".

### Integration Points

To integrate with a C++ LNMP implementation, uncomment the integration code in `test_runner.cpp` and implement the following interfaces:

1. **Parser Interface**:
   ```cpp
   namespace lnmp {
       enum class ParsingMode { Strict, Loose };
       
       class Parser {
       public:
           Parser(const std::string& input, ParsingMode mode = ParsingMode::Loose);
           LnmpRecord parse_record();
       };
   }
   ```

2. **Encoder Interface**:
   ```cpp
   namespace lnmp {
       struct EncoderConfig {
           bool include_type_hints = true;
           bool canonical = true;
           bool include_checksums = false;
       };
       
       class Encoder {
       public:
           Encoder(const EncoderConfig& config);
           std::string encode(const LnmpRecord& record);
       };
   }
   ```

3. **Data Structures**:
   ```cpp
   namespace lnmp {
       struct LnmpField {
           uint16_t fid;
           LnmpValue value;
       };
       
       class LnmpRecord {
       public:
           void add_field(const LnmpField& field);
           std::vector<LnmpField> sorted_fields() const;
       };
       
       // LnmpValue enum with variants for different types
   }
   ```

## Test Categories

### Structural Tests
- Field ordering and canonicalization
- Nested record parsing
- Nested array parsing
- Whitespace normalization

### Semantic Tests
- Type fidelity (int, float, bool, string)
- Value normalization
- Semantic checksums (SC32)
- Equivalence mapping

### Error Handling Tests
- Lexical errors (invalid characters, unterminated strings)
- Syntactic errors (missing tokens, unclosed structures)
- Semantic errors (type mismatches, checksum failures)
- Structural errors (nesting depth, duplicate fields)

### Round-Trip Tests
- Parse → Encode → Compare
- Canonical form validation
- Checksum preservation

## Output Format

### Summary Output (Default)

```
================================================================================
LNMP v0.3 Compliance Test Results (C++)
================================================================================
Total:   45
Passed:  42 (93%)
Failed:  3
Skipped: 0
================================================================================

Failed Tests:
--------------------------------------------------------------------------------
❌ Checksum validation - invalid
   Checksum mismatch: expected 6A93B3F1, got DEADBEEF
```

### Detailed Output (--verbose)

```
================================================================================
LNMP v0.3 Compliance Test Results (C++) - Detailed
================================================================================
✅ Basic field ordering
✅ Type hints preserved
✅ String array basic
❌ Checksum validation - invalid
   Checksum mismatch: expected 6A93B3F1, got DEADBEEF
⏭️  Nested record with checksums
   C++ LNMP implementation not yet available
...
```

## Exit Codes

- `0`: All tests passed (or skipped)
- `1`: One or more tests failed or error occurred

## Contributing

When implementing the C++ LNMP library:

1. Implement the parser and encoder interfaces as described above
2. Uncomment the integration code in `test_runner.cpp`
3. Link your LNMP library in `CMakeLists.txt`
4. Run the compliance tests to validate your implementation

## License

This test runner is part of the LNMP project and follows the same license.

## See Also

- [Test Cases Documentation](../test-cases.yaml)
- [Rust Test Runner](../rust/)
- [Python Test Runner](../python/)
- [TypeScript Test Runner](../typescript/)
- [LNMP v0.3 Specification](../../../.kiro/specs/lnmp-v0.3-semantic-fidelity/)
