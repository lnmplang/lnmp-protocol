#!/usr/bin/env node
/**
 * CLI entry point for LNMP v0.3 TypeScript Compliance Test Runner
 */

import { resolve } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import { TestRunner } from './runner.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

interface CliArgs {
  category?: 'structural' | 'semantic' | 'error-handling' | 'round-trip';
  verbose?: boolean;
  help?: boolean;
}

function parseArgs(): CliArgs {
  const args: CliArgs = {};
  const argv = process.argv.slice(2);

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    switch (arg) {
      case '-c':
      case '--category':
        args.category = argv[++i] as any;
        break;
      case '-v':
      case '--verbose':
        args.verbose = true;
        break;
      case '-h':
      case '--help':
        args.help = true;
        break;
    }
  }

  return args;
}

function printHelp(): void {
  console.log(`
LNMP v0.3 TypeScript Compliance Test Runner

Usage: npm test [options]

Options:
  -c, --category <category>  Run only tests in the specified category
                             (structural, semantic, error-handling, round-trip)
  -v, --verbose              Print detailed test results
  -h, --help                 Show this help message

Examples:
  npm test                              # Run all tests
  npm test -- -c structural             # Run only structural tests
  npm test -- -v                        # Run all tests with verbose output
  npm test -- -c semantic -v            # Run semantic tests with verbose output
`);
}

async function main(): Promise<void> {
  const args = parseArgs();

  if (args.help) {
    printHelp();
    process.exit(0);
  }

  // Load test suite
  const testFile = resolve(__dirname, '../../test-cases.yaml');
  
  try {
    const runner = TestRunner.loadFromFile(testFile);
    console.log(`LNMP v${runner.getVersion()} Compliance Test Runner (TypeScript)\n`);

    // Run tests
    if (args.category) {
      console.log(`Running tests in category: ${args.category}`);
      runner.runCategory(args.category);
    } else {
      console.log('Running all tests...');
      runner.runAll();
    }

    // Print results
    if (args.verbose) {
      runner.printDetailed();
    } else {
      runner.printSummary();
    }

    // Exit with appropriate code
    const results = runner.getResults();
    const failedCount = results.filter((r) => r.result === 'fail').length;
    process.exit(failedCount > 0 ? 1 : 0);
  } catch (error) {
    console.error(`Error: ${error}`);
    process.exit(1);
  }
}

main();
