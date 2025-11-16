/**
 * LNMP v0.3 TypeScript Compliance Test Runner
 *
 * This module loads test cases from test-cases.yaml and executes them
 * against a TypeScript LNMP implementation, reporting pass/fail with detailed
 * error messages.
 */

import { readFileSync } from 'fs';
import { parse as parseYaml } from 'yaml';
import {
  TestCase,
  TestConfig,
  TestSuite,
  TestResult,
  TestExecutionResult,
  ExpectedField,
  ExpectedError,
  ExpectedOutput,
} from './types.js';

/**
 * Test runner for executing compliance tests
 */
export class TestRunner {
  private suite: TestSuite;
  private results: TestExecutionResult[] = [];

  constructor(suite: TestSuite) {
    this.suite = suite;
  }

  /**
   * Load test suite from YAML file
   */
  static loadFromFile(path: string): TestRunner {
    const content = readFileSync(path, 'utf-8');
    const suite = parseYaml(content) as TestSuite;
    return new TestRunner(suite);
  }

  /**
   * Get all test cases from all categories
   */
  getAllTests(): TestCase[] {
    return [
      ...(this.suite.structural_tests || []),
      ...(this.suite.semantic_tests || []),
      ...(this.suite.error_handling_tests || []),
      ...(this.suite.round_trip_tests || []),
    ];
  }

  /**
   * Run all tests in the suite
   */
  runAll(): void {
    const tests = this.getAllTests();
    for (const test of tests) {
      const result = this.runTest(test);
      this.results.push(result);
    }
  }

  /**
   * Run tests in a specific category
   */
  runCategory(category: 'structural' | 'semantic' | 'error-handling' | 'round-trip'): void {
    const categoryMap = {
      structural: this.suite.structural_tests || [],
      semantic: this.suite.semantic_tests || [],
      'error-handling': this.suite.error_handling_tests || [],
      'round-trip': this.suite.round_trip_tests || [],
    };

    const tests = categoryMap[category];
    for (const test of tests) {
      const result = this.runTest(test);
      this.results.push(result);
    }
  }

  /**
   * Run a single test case
   */
  runTest(test: TestCase): TestExecutionResult {
    // Handle round-trip tests separately
    if (test.expected_canonical !== undefined) {
      return this.runRoundTripTest(test);
    }

    if (!test.expected) {
      return {
        name: test.name,
        result: TestResult.Fail,
        reason: "Test case has neither 'expected' nor 'expected_canonical' field",
      };
    }

    // Check if this is an error test
    if (this.isExpectedError(test.expected)) {
      return this.runErrorTest(test, test.expected);
    } else {
      return this.runSuccessTest(test, test.expected.fields);
    }
  }

  /**
   * Type guard to check if expected output is an error
   */
  private isExpectedError(expected: ExpectedOutput): expected is ExpectedError {
    return 'error' in expected;
  }

  /**
   * Run a test that expects successful parsing
   */
  private runSuccessTest(test: TestCase, expectedFields: ExpectedField[]): TestExecutionResult {
    // TODO: Integrate with TypeScript LNMP implementation
    // For now, skip tests until implementation is available
    return {
      name: test.name,
      result: TestResult.Skip,
      reason: 'TypeScript LNMP implementation not yet available',
    };

    // Example integration (uncomment when implementation is ready):
    /*
    try {
      const { Parser, ParsingMode } = await import('lnmp');
      
      const mode = test.config?.strict_mode ? ParsingMode.Strict : ParsingMode.Loose;
      const parser = new Parser(test.input, { mode });
      const record = parser.parseRecord();
      
      return this.validateRecord(record, expectedFields);
    } catch (error) {
      return {
        name: test.name,
        result: TestResult.Fail,
        reason: `Failed to parse: ${error}`,
      };
    }
    */
  }

  /**
   * Run a test that expects an error
   */
  private runErrorTest(test: TestCase, expectedError: ExpectedError): TestExecutionResult {
    // TODO: Integrate with TypeScript LNMP implementation
    return {
      name: test.name,
      result: TestResult.Skip,
      reason: 'TypeScript LNMP implementation not yet available',
    };

    // Example integration (uncomment when implementation is ready):
    /*
    try {
      const { Parser, ParsingMode } = await import('lnmp');
      
      const mode = test.config?.strict_mode ? ParsingMode.Strict : ParsingMode.Loose;
      const parser = new Parser(test.input, { mode });
      const record = parser.parseRecord();
      
      return {
        name: test.name,
        result: TestResult.Fail,
        reason: `Expected error '${expectedError.error}' but parsing succeeded`,
      };
    } catch (error) {
      return this.validateError(String(error), expectedError);
    }
    */
  }

  /**
   * Run a round-trip test (parse -> encode -> compare)
   */
  private runRoundTripTest(test: TestCase): TestExecutionResult {
    // TODO: Integrate with TypeScript LNMP implementation
    return {
      name: test.name,
      result: TestResult.Skip,
      reason: 'TypeScript LNMP implementation not yet available',
    };

    // Example integration (uncomment when implementation is ready):
    /*
    try {
      const { Parser, Encoder, EncoderConfig } = await import('lnmp');
      
      const parser = new Parser(test.input);
      const record = parser.parseRecord();
      
      const config: EncoderConfig = {
        includeTypeHints: true,
        canonical: true,
        includeChecksums: test.config?.preserve_checksums || false,
      };
      const encoder = new Encoder(config);
      const encoded = encoder.encode(record);
      
      if (encoded.trim() === test.expected_canonical!.trim()) {
        return {
          name: test.name,
          result: TestResult.Pass,
        };
      } else {
        return {
          name: test.name,
          result: TestResult.Fail,
          reason: `Round-trip mismatch:\nExpected: ${test.expected_canonical}\nGot: ${encoded}`,
        };
      }
    } catch (error) {
      return {
        name: test.name,
        result: TestResult.Fail,
        reason: `Failed to parse: ${error}`,
      };
    }
    */
  }

  /**
   * Validate that a parsed record matches expected fields
   */
  private validateRecord(record: any, expectedFields: ExpectedField[]): TestExecutionResult {
    // TODO: Implement validation logic
    // This will depend on the structure of the TypeScript LNMP implementation
    return {
      name: 'validation',
      result: TestResult.Skip,
      reason: 'Validation not yet implemented',
    };
  }

  /**
   * Validate that a value matches the expected value
   */
  private validateValue(
    actual: any,
    expected: any,
    expectedType: string
  ): { valid: boolean; reason?: string } {
    switch (expectedType) {
      case 'int':
        if (typeof actual !== 'number' || !Number.isInteger(actual)) {
          return { valid: false, reason: `expected int, got ${typeof actual}` };
        }
        if (actual !== expected) {
          return { valid: false, reason: `expected ${expected}, got ${actual}` };
        }
        return { valid: true };

      case 'float':
        if (typeof actual !== 'number') {
          return { valid: false, reason: `expected float, got ${typeof actual}` };
        }
        if (Math.abs(actual - expected) > 1e-10) {
          return { valid: false, reason: `expected ${expected}, got ${actual}` };
        }
        return { valid: true };

      case 'bool':
        if (typeof actual !== 'boolean') {
          return { valid: false, reason: `expected bool, got ${typeof actual}` };
        }
        if (actual !== expected) {
          return { valid: false, reason: `expected ${expected}, got ${actual}` };
        }
        return { valid: true };

      case 'string':
        if (typeof actual !== 'string') {
          return { valid: false, reason: `expected string, got ${typeof actual}` };
        }
        if (actual !== expected) {
          return { valid: false, reason: `expected '${expected}', got '${actual}'` };
        }
        return { valid: true };

      case 'string_array':
        if (!Array.isArray(actual)) {
          return { valid: false, reason: `expected array, got ${typeof actual}` };
        }
        if (actual.length !== expected.length) {
          return {
            valid: false,
            reason: `array length mismatch: expected ${expected.length}, got ${actual.length}`,
          };
        }
        for (let i = 0; i < actual.length; i++) {
          if (actual[i] !== expected[i]) {
            return {
              valid: false,
              reason: `array element ${i} mismatch: expected '${expected[i]}', got '${actual[i]}'`,
            };
          }
        }
        return { valid: true };

      case 'nested_record':
        // TODO: Implement nested record validation
        return { valid: false, reason: 'nested_record validation not yet implemented' };

      case 'nested_array':
        // TODO: Implement nested array validation
        return { valid: false, reason: 'nested_array validation not yet implemented' };

      default:
        return { valid: false, reason: `Unknown expected type: ${expectedType}` };
    }
  }

  /**
   * Validate that an error matches the expected error
   */
  private validateError(actualError: string, expected: ExpectedError): TestExecutionResult {
    const actualLower = actualError.toLowerCase();
    const expectedLower = expected.error.toLowerCase();

    if (!actualLower.includes(expectedLower)) {
      return {
        name: 'error validation',
        result: TestResult.Fail,
        reason: `Error type mismatch: expected '${expected.error}', got '${actualError}'`,
      };
    }

    const expectedMsgLower = expected.message.toLowerCase();
    if (!actualLower.includes(expectedMsgLower)) {
      return {
        name: 'error validation',
        result: TestResult.Fail,
        reason: `Error message mismatch: expected to contain '${expected.message}', got '${actualError}'`,
      };
    }

    return {
      name: 'error validation',
      result: TestResult.Pass,
    };
  }

  /**
   * Get test results
   */
  getResults(): TestExecutionResult[] {
    return this.results;
  }

  /**
   * Get test suite version
   */
  getVersion(): string {
    return this.suite.version;
  }

  /**
   * Print test results summary
   */
  printSummary(): void {
    const total = this.results.length;
    const passed = this.results.filter((r) => r.result === TestResult.Pass).length;
    const failed = this.results.filter((r) => r.result === TestResult.Fail).length;
    const skipped = this.results.filter((r) => r.result === TestResult.Skip).length;

    console.log('\n' + '='.repeat(80));
    console.log('LNMP v0.3 Compliance Test Results (TypeScript)');
    console.log('='.repeat(80));
    console.log(`Total:   ${total}`);
    console.log(`Passed:  ${passed} (${Math.floor((passed * 100) / Math.max(total, 1))}%)`);
    console.log(`Failed:  ${failed}`);
    console.log(`Skipped: ${skipped}`);
    console.log('='.repeat(80));

    if (failed > 0) {
      console.log('\nFailed Tests:');
      console.log('-'.repeat(80));
      for (const { name, result, reason } of this.results) {
        if (result === TestResult.Fail) {
          console.log(`❌ ${name}`);
          if (reason) {
            console.log(`   ${reason}`);
          }
          console.log();
        }
      }
    }
  }

  /**
   * Print detailed results for all tests
   */
  printDetailed(): void {
    console.log('\n' + '='.repeat(80));
    console.log('LNMP v0.3 Compliance Test Results (TypeScript) - Detailed');
    console.log('='.repeat(80));

    for (const { name, result, reason } of this.results) {
      switch (result) {
        case TestResult.Pass:
          console.log(`✅ ${name}`);
          break;
        case TestResult.Fail:
          console.log(`❌ ${name}`);
          if (reason) {
            console.log(`   ${reason}`);
          }
          break;
        case TestResult.Skip:
          console.log(`⏭️  ${name}`);
          if (reason) {
            console.log(`   ${reason}`);
          }
          break;
      }
    }

    this.printSummary();
  }
}
