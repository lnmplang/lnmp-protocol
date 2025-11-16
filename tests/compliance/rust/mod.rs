//! Rust compliance tests for LNMP v0.3
//!
//! This module provides integration tests that validate the Rust LNMP
//! implementation against the language-agnostic compliance test suite.

mod runner;

use runner::{TestRunner, TestSuite};

#[test]
fn run_compliance_tests() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.parent().unwrap().join("test-cases.yaml");
    
    let suite = TestSuite::load_from_file(&test_file)
        .unwrap_or_else(|e| panic!("Failed to load test suite from {:?}: {}", test_file, e));

    // Run all tests
    let mut runner = TestRunner::new(suite);
    runner.run_all();

    // Print results
    runner.print_detailed();

    // Assert all tests passed
    let results = runner.results();
    let failed_count = results.iter().filter(|(_, r)| r.is_fail()).count();
    
    if failed_count > 0 {
        panic!("{} compliance test(s) failed", failed_count);
    }
}

#[test]
fn run_structural_tests() {
    // Find the test file - CARGO_MANIFEST_DIR is tests/compliance/rust
    // We need to go up to tests/compliance and find test-cases.yaml
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.parent().unwrap().join("test-cases.yaml");
    
    let suite = TestSuite::load_from_file(&test_file)
        .unwrap_or_else(|e| panic!("Failed to load test suite from {:?}: {}", test_file, e));

    let runner = TestRunner::new(suite);
    
    // Run only structural tests
    for test in runner.suite.structural_tests.iter() {
        let result = runner.run_test(test);
        println!("{}: {:?}", test.name, result);
        
        if result.is_fail() {
            panic!("Structural test '{}' failed", test.name);
        }
    }
}

#[test]
fn run_semantic_tests() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.parent().unwrap().join("test-cases.yaml");
    
    let suite = TestSuite::load_from_file(&test_file)
        .unwrap_or_else(|e| panic!("Failed to load test suite from {:?}: {}", test_file, e));

    let runner = TestRunner::new(suite);
    
    // Run only semantic tests
    for test in runner.suite.semantic_tests.iter() {
        let result = runner.run_test(test);
        println!("{}: {:?}", test.name, result);
        
        if result.is_fail() {
            panic!("Semantic test '{}' failed", test.name);
        }
    }
}

#[test]
fn run_error_handling_tests() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.parent().unwrap().join("test-cases.yaml");
    
    let suite = TestSuite::load_from_file(&test_file)
        .unwrap_or_else(|e| panic!("Failed to load test suite from {:?}: {}", test_file, e));

    let runner = TestRunner::new(suite);
    
    // Run only error handling tests
    for test in runner.suite.error_handling_tests.iter() {
        let result = runner.run_test(test);
        println!("{}: {:?}", test.name, result);
        
        if result.is_fail() {
            panic!("Error handling test '{}' failed", test.name);
        }
    }
}

#[test]
fn run_round_trip_tests() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.parent().unwrap().join("test-cases.yaml");
    
    let suite = TestSuite::load_from_file(&test_file)
        .unwrap_or_else(|e| panic!("Failed to load test suite from {:?}: {}", test_file, e));

    let runner = TestRunner::new(suite);
    
    // Run only round-trip tests
    for test in runner.suite.round_trip_tests.iter() {
        let result = runner.run_test(test);
        println!("{}: {:?}", test.name, result);
        
        if result.is_fail() {
            panic!("Round-trip test '{}' failed", test.name);
        }
    }
}
