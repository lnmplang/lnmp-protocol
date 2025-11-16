/**
 * Vitest test file for LNMP v0.3 TypeScript Compliance Tests
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { resolve } from 'path';
import { TestRunner } from './src/runner.js';
import { TestResult } from './src/types.js';

describe('LNMP v0.3 Compliance Tests', () => {
  let runner: TestRunner;

  beforeAll(() => {
    const testFile = resolve(__dirname, '../test-cases.yaml');
    runner = TestRunner.loadFromFile(testFile);
  });

  it('should load test suite successfully', () => {
    expect(runner).toBeDefined();
    expect(runner.getVersion()).toBe('0.3.0');
  });

  it('should have test cases', () => {
    const tests = runner.getAllTests();
    expect(tests.length).toBeGreaterThan(0);
  });

  describe('Structural Tests', () => {
    it('should run all structural tests', () => {
      runner.runCategory('structural');
      const results = runner.getResults();
      expect(results.length).toBeGreaterThan(0);
    });
  });

  describe('Semantic Tests', () => {
    it('should run all semantic tests', () => {
      const freshRunner = TestRunner.loadFromFile(resolve(__dirname, '../test-cases.yaml'));
      freshRunner.runCategory('semantic');
      const results = freshRunner.getResults();
      expect(results.length).toBeGreaterThan(0);
    });
  });

  describe('Error Handling Tests', () => {
    it('should run all error handling tests', () => {
      const freshRunner = TestRunner.loadFromFile(resolve(__dirname, '../test-cases.yaml'));
      freshRunner.runCategory('error-handling');
      const results = freshRunner.getResults();
      expect(results.length).toBeGreaterThan(0);
    });
  });

  describe('Round-trip Tests', () => {
    it('should run all round-trip tests', () => {
      const freshRunner = TestRunner.loadFromFile(resolve(__dirname, '../test-cases.yaml'));
      freshRunner.runCategory('round-trip');
      const results = freshRunner.getResults();
      expect(results.length).toBeGreaterThan(0);
    });
  });

  describe('Test Execution', () => {
    it('should execute all tests', () => {
      const freshRunner = TestRunner.loadFromFile(resolve(__dirname, '../test-cases.yaml'));
      freshRunner.runAll();
      const results = freshRunner.getResults();
      
      expect(results.length).toBeGreaterThan(0);
      
      // Count results by type
      const passed = results.filter(r => r.result === TestResult.Pass).length;
      const failed = results.filter(r => r.result === TestResult.Fail).length;
      const skipped = results.filter(r => r.result === TestResult.Skip).length;
      
      // All tests should be skipped until TypeScript implementation is available
      expect(skipped).toBeGreaterThan(0);
      
      console.log(`\nTest Summary: ${passed} passed, ${failed} failed, ${skipped} skipped`);
    });
  });
});
