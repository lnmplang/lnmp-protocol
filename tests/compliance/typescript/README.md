# LNMP Compliance Test Runner (TypeScript)

TypeScript support mirrors the shared YAML vectors under `tests/compliance/test-cases.yaml`. Once a TypeScript LNMP parser/encoder is available, this runner can validate it against the modular specification set (`spec/lnmp-*-spec.md`) and surface REQ IDs in failures.

## Structure

```
typescript/
├── src/
│   ├── types.ts       # TypeScript type definitions for test cases
│   ├── runner.ts      # Main test runner implementation
│   └── cli.ts         # CLI entry point
├── runner.test.ts     # Vitest bridge into the runner
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

> ⚠️ Until a TypeScript LNMP implementation is integrated, tests are skipped with a notice (`implementation not yet available`). The commands below are still useful once the parser/encoder exists.

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

1. **Structural** – canonicalization, field ordering, nested structures, whitespace (REQ-CAN/REQ-TXT)
2. **Semantic** – type fidelity, normalization, semantic checksums, equivalence mapping (REQ-TXT/REQ-SC/REQ-SAN)
3. **Error Handling** – lexical/syntactic/semantic/structural errors (REQ-ERR-*)
4. **Round-trip** – parse ↔ encode stability (REQ-CAN-RT)

## Integration with TypeScript LNMP Implementation

1. Add your LNMP package as a dependency (parser/encoder exported as ES modules).  
2. Wire the parser/encoder into `src/runner.ts` where the TODO markers appear.  
3. Ensure the runner can toggle strict/loose parsing, checksum validation, lenient mode, and nested structures.  
4. Run the commands above and confirm failing cases log the `requirements` array (REQ IDs) from the YAML entry.

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

`npm run test:watch` re-runs tests on file changes. Debug by adding `console.log` entries or running via `node --inspect`.

### Adding New Test Cases

Add vectors to `../test-cases.yaml` (and include `requirements`). The TypeScript runner automatically loads them on the next run.

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

This module is part of the LNMP specification repository and follows the main project license.

## References

- `tests/compliance/README.md`
- Modular specs: `spec/lnmp-core-spec.md`, `spec/lnmp-text-format.md`, `spec/lnmp-binary-format.md`, `spec/lnmp-canonicalization.md`, `spec/lnmp-security-compliance.md`, `spec/lnmp-migration-versioning.md`
- `spec/grammar.md`, `spec/error-classes.md`
