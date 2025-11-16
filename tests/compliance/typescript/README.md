# LNMP v0.3 TypeScript Compliance Test Runner

This directory contains the TypeScript compliance test runner for LNMP v0.3. It loads test cases from the language-agnostic `test-cases.yaml` file and executes them against a TypeScript LNMP implementation.

## Structure

```
typescript/
├── src/
│   ├── types.ts       # TypeScript type definitions for test cases
│   ├── runner.ts      # Main test runner implementation
│   └── cli.ts         # CLI entry point
├── runner.test.ts     # Vitest test file
├── package.json       # Node.js package configuration
├── tsconfig.json      # TypeScript configuration
├── vitest.config.ts   # Vitest configuration
└── README.md          # This file
```

## Installation

Install dependencies:

```bash
npm install
```

## Usage

### Run All Tests

```bash
npm test
```

### Run Tests in a Specific Category

```bash
npm test -- -c structural
npm test -- -c semantic
npm test -- -c error-handling
npm test -- -c round-trip
```

### Run Tests with Verbose Output

```bash
npm test -- -v
```

### Run Tests in Watch Mode (for development)

```bash
npm run test:watch
```

### Run CLI Directly

```bash
node src/cli.ts
node src/cli.ts -c structural -v
```

## Test Categories

The test suite is organized into four categories:

1. **Structural Tests**: Canonicalization, field ordering, nested structures, whitespace handling
2. **Semantic Tests**: Type fidelity, value normalization, semantic checksums, equivalence mapping
3. **Error Handling Tests**: Lexical errors, syntactic errors, semantic errors, structural errors
4. **Round-trip Tests**: Parse → Encode → Parse consistency

## Integration with TypeScript LNMP Implementation

The test runner is designed to integrate with a future TypeScript LNMP implementation. Currently, all tests are skipped with the message "TypeScript LNMP implementation not yet available".

To integrate with your TypeScript LNMP implementation:

1. Install your LNMP package as a dependency
2. Uncomment the integration code in `src/runner.ts`
3. Update the import statements to match your package structure
4. Implement the validation logic for nested structures

Example integration points are marked with `// TODO:` comments in the code.

## Expected TypeScript LNMP API

The test runner expects the following API from the TypeScript LNMP implementation:

```typescript
// Parser
class Parser {
  constructor(input: string, options?: { mode?: ParsingMode });
  parseRecord(): LnmpRecord;
}

enum ParsingMode {
  Strict = 'strict',
  Loose = 'loose',
}

// Encoder
class Encoder {
  constructor(config: EncoderConfig);
  encode(record: LnmpRecord): string;
}

interface EncoderConfig {
  includeTypeHints: boolean;
  canonical: boolean;
  includeChecksums: boolean;
}

// Data structures
interface LnmpRecord {
  fields: LnmpField[];
  sortedFields(): LnmpField[];
}

interface LnmpField {
  fid: number;
  value: LnmpValue;
  typeHint?: TypeHint;
}

type LnmpValue =
  | { type: 'int'; value: number }
  | { type: 'float'; value: number }
  | { type: 'bool'; value: boolean }
  | { type: 'string'; value: string }
  | { type: 'string_array'; value: string[] }
  | { type: 'nested_record'; value: LnmpRecord }
  | { type: 'nested_array'; value: LnmpRecord[] };
```

## Test Results

Test results are reported in three formats:

1. **Summary**: Total, passed, failed, and skipped counts
2. **Failed Tests**: List of failed tests with reasons
3. **Detailed**: All tests with their status and reasons (use `-v` flag)

Example output:

```
================================================================================
LNMP v0.3 Compliance Test Results (TypeScript)
================================================================================
Total:   85
Passed:  0 (0%)
Failed:  0
Skipped: 85
================================================================================
```

## Development

### Running Tests During Development

Use watch mode to automatically re-run tests when files change:

```bash
npm run test:watch
```

### Adding New Test Cases

Test cases are defined in the shared `../test-cases.yaml` file. The TypeScript runner automatically loads and executes all test cases from this file.

### Debugging

To debug the test runner:

1. Add `console.log` statements in `src/runner.ts`
2. Run tests with verbose output: `npm test -- -v`
3. Use Node.js debugging tools: `node --inspect src/cli.ts`

## Requirements

- Node.js 18+ (for ES modules support)
- TypeScript 5.0+
- Vitest 1.0+

## License

This test runner is part of the LNMP v0.3 specification and follows the same license as the main project.
