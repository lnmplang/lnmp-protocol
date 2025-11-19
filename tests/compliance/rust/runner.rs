//! Rust compliance test runner for LNMP v0.3
#![allow(dead_code)]
//!
//! This module loads test cases from `test-cases.yaml` and executes them
//! against the Rust LNMP implementation, reporting pass/fail with detailed
//! error messages.

use lnmp_codec::equivalence::EquivalenceMapper;
use lnmp_codec::{
    config::ParserConfig, Encoder, EncoderConfig, Parser, ParsingMode, TextInputMode,
};
use lnmp_core::{LnmpRecord, LnmpValue};
use lnmp_sanitize::SanitizationConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Test case configuration options
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TestConfig {
    #[serde(default)]
    pub normalize_values: bool,
    #[serde(default)]
    pub validate_checksums: bool,
    #[serde(default)]
    pub strict_mode: bool,
    #[serde(default)]
    pub lenient_mode: bool,
    #[serde(default)]
    pub preserve_checksums: bool,
    #[serde(default)]
    pub max_nesting_depth: Option<usize>,
    #[serde(default)]
    pub equivalence_mapping: Option<HashMap<u16, HashMap<String, String>>>,
}

// Default is derived via `Default` trait derived on struct

/// Expected field structure in test cases
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExpectedField {
    pub fid: u16,
    #[serde(rename = "type")]
    pub type_name: String,
    pub value: serde_yaml::Value,
    #[serde(default)]
    pub checksum: Option<String>,
}

/// Expected nested record structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExpectedRecord {
    pub fields: Vec<ExpectedField>,
}

/// Expected error structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExpectedError {
    pub error: String,
    pub message: String,
    #[serde(default)]
    pub field_id: Option<u16>,
    #[serde(default)]
    pub line: Option<usize>,
    #[serde(default)]
    pub column: Option<usize>,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

/// Expected output - either fields or an error
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ExpectedOutput {
    Success { fields: Vec<ExpectedField> },
    Error(ExpectedError),
}

/// A single test case
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    pub name: String,
    pub category: String,
    pub description: String,
    pub input: String,
    #[serde(default)]
    pub expected: Option<ExpectedOutput>,
    #[serde(default)]
    pub config: TestConfig,
    #[serde(default)]
    pub expected_canonical: Option<String>,
}

/// Test suite containing all test cases
#[derive(Debug, Deserialize, Serialize)]
pub struct TestSuite {
    pub version: String,
    #[serde(default)]
    pub structural_tests: Vec<TestCase>,
    #[serde(default)]
    pub semantic_tests: Vec<TestCase>,
    #[serde(default)]
    pub error_handling_tests: Vec<TestCase>,
    #[serde(default)]
    pub round_trip_tests: Vec<TestCase>,
    #[serde(default)]
    pub lenient_tests: Vec<TestCase>,
}

impl TestSuite {
    /// Load test suite from YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let suite: TestSuite = serde_yaml::from_str(&content)?;
        Ok(suite)
    }

    /// Get all test cases from all categories
    pub fn all_tests(&self) -> Vec<&TestCase> {
        let mut tests = Vec::new();
        tests.extend(&self.structural_tests);
        tests.extend(&self.semantic_tests);
        tests.extend(&self.error_handling_tests);
        tests.extend(&self.round_trip_tests);
        tests.extend(&self.lenient_tests);
        tests
    }
}

/// Test result for a single test case
#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail { reason: String },
    Skip { reason: String },
    Info { reason: String },
}

impl TestResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. })
    }
}

/// Test runner for executing compliance tests
pub struct TestRunner {
    pub suite: TestSuite,
    pub results: Vec<(String, TestResult)>,
}

impl TestRunner {
    /// Create a new test runner with the given test suite
    pub fn new(suite: TestSuite) -> Self {
        Self {
            suite,
            results: Vec::new(),
        }
    }

    /// Run all tests in the suite
    pub fn run_all(&mut self) {
        for test in self.suite.all_tests() {
            let result = self.run_test(test);
            self.results.push((test.name.clone(), result));
        }
    }

    /// Run a single test case
    pub fn run_test(&self, test: &TestCase) -> TestResult {
        // Handle round-trip tests separately
        if test.expected_canonical.is_some() {
            return self.run_round_trip_test(test);
        }

        match &test.expected {
            Some(ExpectedOutput::Success { fields }) => self.run_success_test(test, fields),
            Some(ExpectedOutput::Error(expected_error)) => {
                self.run_error_test(test, expected_error)
            }
            None => TestResult::Fail {
                reason: "Test case has neither 'expected' nor 'expected_canonical' field"
                    .to_string(),
            },
        }
    }

    /// Run a test that expects successful parsing
    fn run_success_test(&self, test: &TestCase, expected_fields: &[ExpectedField]) -> TestResult {
        // Parse the input
        let parsing_mode = if test.config.strict_mode {
            ParsingMode::Strict
        } else {
            ParsingMode::Loose
        };

        let parser_config = ParserConfig {
            mode: parsing_mode,
            validate_checksums: test.config.validate_checksums,
            normalize_values: test.config.normalize_values,
            require_checksums: false,
            max_nesting_depth: test.config.max_nesting_depth,
            text_input_mode: if test.config.lenient_mode {
                TextInputMode::Lenient
            } else {
                TextInputMode::Strict
            },
            semantic_dictionary: None,
            structural_limits: None,
        };

        let mut parser = match Parser::with_config(&test.input, parser_config) {
            Ok(p) => p,
            Err(e) => {
                return TestResult::Fail {
                    reason: format!("Failed to create parser: {}", e),
                }
            }
        };

        let mut record = match parser.parse_record() {
            Ok(r) => r,
            Err(e) => {
                return TestResult::Fail {
                    reason: format!("Failed to parse: {}", e),
                }
            }
        };

        // If equivalence mapping is provided, apply to parsed record to canonicalize values
        if let Some(equiv_map) = &test.config.equivalence_mapping {
            let mut mapper = EquivalenceMapper::new();
            for (fid_u16, mappings) in equiv_map.iter() {
                for (from, to) in mappings.iter() {
                    mapper.add_mapping(*fid_u16, from.clone(), to.clone());
                }
            }
            Self::apply_equivalence_mapping_to_record(&mut record, &mapper);
        }

        // Validate the parsed record matches expected fields
        self.validate_record(&record, expected_fields)
    }

    /// Run a test that expects an error
    fn run_error_test(&self, test: &TestCase, expected_error: &ExpectedError) -> TestResult {
        let parsing_mode = if test.config.strict_mode {
            ParsingMode::Strict
        } else {
            ParsingMode::Loose
        };

        let parser_config = ParserConfig {
            mode: parsing_mode,
            validate_checksums: test.config.validate_checksums,
            normalize_values: test.config.normalize_values,
            require_checksums: false,
            max_nesting_depth: test.config.max_nesting_depth,
            text_input_mode: if test.config.lenient_mode {
                TextInputMode::Lenient
            } else {
                TextInputMode::Strict
            },
            semantic_dictionary: None,
            structural_limits: None,
        };

        let mut parser = match Parser::with_config(&test.input, parser_config) {
            Ok(p) => p,
            Err(e) => {
                // Parser creation failed - check if this matches expected error
                return self.validate_error(&e.to_string(), expected_error);
            }
        };

        match parser.parse_record() {
            Ok(_) => TestResult::Fail {
                reason: format!(
                    "Expected error '{}' but parsing succeeded",
                    expected_error.error
                ),
            },
            Err(e) => self.validate_error(&e.to_string(), expected_error),
        }
    }

    /// Run a round-trip test (parse -> encode -> compare)
    fn run_round_trip_test(&self, test: &TestCase) -> TestResult {
        // Parse the input
        let parser_config = ParserConfig {
            mode: ParsingMode::Loose,
            validate_checksums: test.config.validate_checksums,
            normalize_values: test.config.normalize_values,
            require_checksums: false,
            max_nesting_depth: test.config.max_nesting_depth,
            text_input_mode: if test.config.lenient_mode {
                TextInputMode::Lenient
            } else {
                TextInputMode::Strict
            },
            semantic_dictionary: None,
            structural_limits: None,
        };
        let mut parser = match Parser::with_config(&test.input, parser_config) {
            Ok(p) => p,
            Err(e) => {
                return TestResult::Fail {
                    reason: format!("Failed to create parser: {}", e),
                }
            }
        };

        let record = match parser.parse_record() {
            Ok(r) => r,
            Err(e) => {
                return TestResult::Fail {
                    reason: format!("Failed to parse: {}", e),
                }
            }
        };

        // Encode the record
        // Use canonical output. Determine whether to include type hints by
        // checking if expected canonical form contains a type hint pattern.
        let expected = test.expected_canonical.as_ref().unwrap();
        let expected_contains_type_hint = expected.contains(":i")
            || expected.contains(":f")
            || expected.contains(":b")
            || expected.contains(":s")
            || expected.contains(":sa")
            || expected.contains(":r")
            || expected.contains(":ra");
        let config = EncoderConfig {
            include_type_hints: expected_contains_type_hint,
            canonical: true,
            enable_checksums: test.config.preserve_checksums,
            ..Default::default()
        };
        let encoder = Encoder::with_config(config);
        let encoded = encoder.encode(&record);

        // Compare with expected canonical form
        if encoded.trim() == expected.trim() {
            TestResult::Pass
        } else {
            TestResult::Fail {
                reason: format!(
                    "Round-trip mismatch:\nExpected: {}\nGot: {}",
                    expected, encoded
                ),
            }
        }
    }

    /// Validate that a parsed record matches expected fields
    fn validate_record(
        &self,
        record: &LnmpRecord,
        expected_fields: &[ExpectedField],
    ) -> TestResult {
        let actual_fields = record.sorted_fields();
        // Ensure expected fields are compared in canonical (sorted by fid) order
        let mut expected_sorted = expected_fields.to_vec();
        expected_sorted.sort_by_key(|ef| ef.fid);

        // Check field count
        if actual_fields.len() != expected_sorted.len() {
            return TestResult::Fail {
                reason: format!(
                    "Field count mismatch: expected {}, got {}",
                    expected_fields.len(),
                    actual_fields.len()
                ),
            };
        }

        // Check each field
        for (actual, expected) in actual_fields.iter().zip(expected_sorted.iter()) {
            if actual.fid != expected.fid {
                return TestResult::Fail {
                    reason: format!(
                        "Field ID mismatch: expected {}, got {}",
                        expected.fid, actual.fid
                    ),
                };
            }

            // Validate value matches expected
            if let Err(reason) =
                self.validate_value(&actual.value, &expected.value, &expected.type_name)
            {
                return TestResult::Fail {
                    reason: format!("Field {} value mismatch: {}", actual.fid, reason),
                };
            }
        }

        TestResult::Pass
    }

    /// Apply equivalence mappings to a parsed record.
    fn apply_equivalence_mapping_to_record(
        record: &mut lnmp_core::LnmpRecord,
        mapper: &EquivalenceMapper,
    ) {
        // Convert record into owned fields vector so we can mutate values
        let mut fields = record.fields().to_vec();

        for field in fields.iter_mut() {
            match &mut field.value {
                lnmp_core::LnmpValue::String(s) => {
                    if let Some(mapped) = mapper.map(field.fid, s.as_str()) {
                        *s = mapped;
                    }
                }
                lnmp_core::LnmpValue::StringArray(arr) => {
                    for elem in arr.iter_mut() {
                        if let Some(mapped) = mapper.map(field.fid, elem.as_str()) {
                            *elem = mapped;
                        }
                    }
                }
                lnmp_core::LnmpValue::NestedRecord(inner) => {
                    // Recreate nested records from owned boxes
                    let mut nested = *inner.clone();
                    Self::apply_equivalence_mapping_to_record(&mut nested, mapper);
                    *inner = Box::new(nested);
                }
                lnmp_core::LnmpValue::NestedArray(arr) => {
                    for r in arr.iter_mut() {
                        let mut nested = r.clone();
                        Self::apply_equivalence_mapping_to_record(&mut nested, mapper);
                        *r = nested;
                    }
                }
                _ => {}
            }
        }

        *record = lnmp_core::LnmpRecord::from_sorted_fields(fields);
    }

    /// Validate that a value matches the expected value
    fn validate_value(
        &self,
        actual: &LnmpValue,
        expected: &serde_yaml::Value,
        expected_type: &str,
    ) -> Result<(), String> {
        match expected_type {
            "int" => {
                if let LnmpValue::Int(actual_int) = actual {
                    let expected_int = expected
                        .as_i64()
                        .ok_or_else(|| "Expected value is not an integer".to_string())?;
                    if *actual_int == expected_int {
                        Ok(())
                    } else {
                        Err(format!("expected {}, got {}", expected_int, actual_int))
                    }
                } else {
                    Err(format!("expected int, got {:?}", actual))
                }
            }
            "float" => {
                if let LnmpValue::Float(actual_float) = actual {
                    let expected_float = expected
                        .as_f64()
                        .ok_or_else(|| "Expected value is not a float".to_string())?;
                    // Use approximate comparison for floats
                    if (actual_float - expected_float).abs() < 1e-10 {
                        Ok(())
                    } else {
                        Err(format!("expected {}, got {}", expected_float, actual_float))
                    }
                } else {
                    Err(format!("expected float, got {:?}", actual))
                }
            }
            "bool" => {
                if let LnmpValue::Bool(actual_bool) = actual {
                    let expected_bool = expected
                        .as_bool()
                        .ok_or_else(|| "Expected value is not a boolean".to_string())?;
                    if *actual_bool == expected_bool {
                        Ok(())
                    } else {
                        Err(format!("expected {}, got {}", expected_bool, actual_bool))
                    }
                } else {
                    Err(format!("expected bool, got {:?}", actual))
                }
            }
            "string" => {
                if let LnmpValue::String(actual_str) = actual {
                    let expected_str = expected
                        .as_str()
                        .ok_or_else(|| "Expected value is not a string".to_string())?;
                    if actual_str == expected_str {
                        Ok(())
                    } else {
                        Err(format!("expected '{}', got '{}'", expected_str, actual_str))
                    }
                } else {
                    Err(format!("expected string, got {:?}", actual))
                }
            }
            "string_array" => {
                if let LnmpValue::StringArray(actual_arr) = actual {
                    let expected_arr = expected
                        .as_sequence()
                        .ok_or_else(|| "Expected value is not an array".to_string())?;

                    if actual_arr.len() != expected_arr.len() {
                        return Err(format!(
                            "array length mismatch: expected {}, got {}",
                            expected_arr.len(),
                            actual_arr.len()
                        ));
                    }

                    for (i, (actual_elem, expected_elem)) in
                        actual_arr.iter().zip(expected_arr.iter()).enumerate()
                    {
                        let expected_str = expected_elem
                            .as_str()
                            .ok_or_else(|| format!("Array element {} is not a string", i))?;
                        if actual_elem != expected_str {
                            return Err(format!(
                                "array element {} mismatch: expected '{}', got '{}'",
                                i, expected_str, actual_elem
                            ));
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("expected string_array, got {:?}", actual))
                }
            }
            "nested_record" => {
                if let LnmpValue::NestedRecord(actual_record) = actual {
                    let expected_record = expected
                        .as_mapping()
                        .ok_or_else(|| "Expected value is not a record".to_string())?;

                    let expected_fields_value = expected_record
                        .get(serde_yaml::Value::String("fields".to_string()))
                        .ok_or_else(|| "Expected record has no 'fields' key".to_string())?;

                    let expected_fields: Vec<ExpectedField> =
                        serde_yaml::from_value(expected_fields_value.clone())
                            .map_err(|e| format!("Failed to parse expected fields: {}", e))?;

                    match self.validate_record(actual_record, &expected_fields) {
                        TestResult::Pass => Ok(()),
                        TestResult::Fail { reason } => Err(reason),
                        TestResult::Skip { reason } => Err(format!("Skipped: {}", reason)),
                        TestResult::Info { reason } => Err(format!("Info: {}", reason)),
                    }
                } else {
                    Err(format!("expected nested_record, got {:?}", actual))
                }
            }
            "nested_array" => {
                if let LnmpValue::NestedArray(actual_arr) = actual {
                    let expected_arr = expected
                        .as_sequence()
                        .ok_or_else(|| "Expected value is not an array".to_string())?;

                    if actual_arr.len() != expected_arr.len() {
                        return Err(format!(
                            "nested array length mismatch: expected {}, got {}",
                            expected_arr.len(),
                            actual_arr.len()
                        ));
                    }

                    for (i, (actual_record, expected_record)) in
                        actual_arr.iter().zip(expected_arr.iter()).enumerate()
                    {
                        let expected_mapping = expected_record
                            .as_mapping()
                            .ok_or_else(|| format!("Array element {} is not a record", i))?;

                        let expected_fields_value = expected_mapping
                            .get(serde_yaml::Value::String("fields".to_string()))
                            .ok_or_else(|| format!("Array element {} has no 'fields' key", i))?;

                        let expected_fields: Vec<ExpectedField> =
                            serde_yaml::from_value(expected_fields_value.clone()).map_err(|e| {
                                format!("Failed to parse expected fields for element {}: {}", i, e)
                            })?;

                        match self.validate_record(actual_record, &expected_fields) {
                            TestResult::Pass => {}
                            TestResult::Fail { reason } => {
                                return Err(format!(
                                    "nested array element {} mismatch: {}",
                                    i, reason
                                ));
                            }
                            TestResult::Skip { reason } => {
                                return Err(format!(
                                    "nested array element {} skipped: {}",
                                    i, reason
                                ));
                            }
                            TestResult::Info { reason } => {
                                return Err(format!("nested array element {} info: {}", i, reason));
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("expected nested_array, got {:?}", actual))
                }
            }
            _ => Err(format!("Unknown expected type: {}", expected_type)),
        }
    }

    /// Validate that an error matches the expected error
    fn validate_error(&self, actual_error: &str, expected: &ExpectedError) -> TestResult {
        // Check if error type matches (case-insensitive substring match)
        let actual_lower = actual_error.to_lowercase();
        let expected_lower = expected.error.to_lowercase();

        // Also accept camel case variant names such as 'ChecksumMismatch' by
        // checking for a space-separated form (e.g., 'checksum mismatch').
        let expected_spaced = expected
            .error
            .chars()
            .fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push(c);
                acc
            })
            .to_lowercase();

        if !actual_lower.contains(&expected_lower) && !actual_lower.contains(&expected_spaced) {
            return TestResult::Fail {
                reason: format!(
                    "Error type mismatch: expected '{}', got '{}'",
                    expected.error, actual_error
                ),
            };
        }

        // Check if error message contains expected message (case-insensitive)
        let expected_msg_lower = expected.message.to_lowercase();
        if !actual_lower.contains(&expected_msg_lower) {
            return TestResult::Fail {
                reason: format!(
                    "Error message mismatch: expected to contain '{}', got '{}'",
                    expected.message, actual_error
                ),
            };
        }

        TestResult::Pass
    }

    /// Get test results
    pub fn results(&self) -> &[(String, TestResult)] {
        &self.results
    }

    /// Print test results summary
    pub fn print_summary(&self) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|(_, r)| r.is_pass()).count();
        let failed = self.results.iter().filter(|(_, r)| r.is_fail()).count();
        let skipped = total - passed - failed;

        println!("\n{}", "=".repeat(80));
        println!("LNMP v0.3 Compliance Test Results");
        println!("{}", "=".repeat(80));
        println!("Total:   {}", total);
        println!("Passed:  {} ({}%)", passed, (passed * 100) / total.max(1));
        println!("Failed:  {}", failed);
        println!("Skipped: {}", skipped);
        println!("{}", "=".repeat(80));

        if failed > 0 {
            println!("\nFailed Tests:");
            println!("{}", "-".repeat(80));
            for (name, result) in &self.results {
                if let TestResult::Fail { reason } = result {
                    println!("❌ {}", name);
                    println!("   {}", reason);
                    println!();
                }
            }
        }
    }

    /// Print detailed results for all tests
    pub fn print_detailed(&self) {
        println!("\n{}", "=".repeat(80));
        println!("LNMP v0.3 Compliance Test Results (Detailed)");
        println!("{}", "=".repeat(80));

        for (name, result) in &self.results {
            match result {
                TestResult::Pass => println!("✅ {}", name),
                TestResult::Fail { reason } => {
                    println!("❌ {}", name);
                    println!("   {}", reason);
                }
                TestResult::Skip { reason } => {
                    println!("⏭️  {}", name);
                    println!("   {}", reason);
                }
                TestResult::Info { reason } => {
                    println!("ℹ️  {}", name);
                    println!("   {}", reason);
                }
            }
        }

        self.print_summary();
    }
}
