/**
 * LNMP v0.3 TypeScript Compliance Test Types
 */

/**
 * Test case configuration options
 */
export interface TestConfig {
  normalize_values?: boolean;
  validate_checksums?: boolean;
  strict_mode?: boolean;
  preserve_checksums?: boolean;
  max_nesting_depth?: number;
  equivalence_mapping?: Record<number, Record<string, string>>;
}

/**
 * Expected field structure in test cases
 */
export interface ExpectedField {
  fid: number;
  type: string;
  value: any;
  checksum?: string;
}

/**
 * Expected nested record structure
 */
export interface ExpectedRecord {
  fields: ExpectedField[];
}

/**
 * Expected error structure
 */
export interface ExpectedError {
  error: string;
  message: string;
  field_id?: number;
  line?: number;
  column?: number;
  max_depth?: number;
}

/**
 * Expected output - either fields or an error
 */
export type ExpectedOutput =
  | { fields: ExpectedField[] }
  | ExpectedError;

/**
 * A single test case
 */
export interface TestCase {
  name: string;
  category: string;
  description: string;
  input: string;
  expected?: ExpectedOutput;
  config?: TestConfig;
  expected_canonical?: string;
}

/**
 * Test suite containing all test cases
 */
export interface TestSuite {
  version: string;
  structural_tests?: TestCase[];
  semantic_tests?: TestCase[];
  error_handling_tests?: TestCase[];
  round_trip_tests?: TestCase[];
}

/**
 * Test result status
 */
export enum TestResult {
  Pass = 'pass',
  Fail = 'fail',
  Skip = 'skip',
}

/**
 * Test execution result
 */
export interface TestExecutionResult {
  name: string;
  result: TestResult;
  reason?: string;
}
