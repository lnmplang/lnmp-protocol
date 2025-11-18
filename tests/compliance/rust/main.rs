//! Standalone compliance test runner for LNMP v0.3
//!
//! This binary can be run directly to execute the compliance test suite
//! and report results.
//!
//! Usage:
//!   cargo run --bin lnmp-compliance-runner
//!   cargo run --bin lnmp-compliance-runner -- --category structural
//!   cargo run --bin lnmp-compliance-runner -- --verbose

mod runner;

use runner::{TestRunner, TestSuite};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let mut category_filter: Option<String> = None;
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--category" | "-c" => {
                if i + 1 < args.len() {
                    category_filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --category requires a value");
                    print_usage();
                    process::exit(1);
                }
            }
            "--verbose" | "-v" => {
                verbose = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument '{}'", args[i]);
                print_usage();
                process::exit(1);
            }
        }
    }

    // Load test suite
    let test_file = "tests/compliance/test-cases.yaml";
    let suite = match TestSuite::load_from_file(test_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error: Failed to load test suite from '{}': {}",
                test_file, e
            );
            process::exit(1);
        }
    };

    println!("LNMP v{} Compliance Test Runner", suite.version);
    println!();

    // Create runner and execute tests
    let mut runner = TestRunner::new(suite);

    if let Some(category) = category_filter {
        println!("Running tests in category: {}", category);
        run_category(&mut runner, &category);
    } else {
        println!("Running all tests...");
        runner.run_all();
    }

    // Print results
    if verbose {
        runner.print_detailed();
    } else {
        runner.print_summary();
    }

    // Exit with appropriate code
    let failed_count = runner.results().iter().filter(|(_, r)| r.is_fail()).count();
    if failed_count > 0 {
        process::exit(1);
    }
}

fn run_category(runner: &mut TestRunner, category: &str) {
    let tests: Vec<_> = match category {
        "structural" => runner.suite.structural_tests.iter().collect(),
        "semantic" => runner.suite.semantic_tests.iter().collect(),
        "error-handling" | "error_handling" => runner.suite.error_handling_tests.iter().collect(),
        "round-trip" | "round_trip" => runner.suite.round_trip_tests.iter().collect(),
        _ => {
            eprintln!("Error: Unknown category '{}'", category);
            eprintln!("Valid categories: structural, semantic, error-handling, round-trip");
            process::exit(1);
        }
    };

    for test in tests {
        let result = runner.run_test(test);
        runner.results.push((test.name.clone(), result));
    }
}

fn print_usage() {
    println!("LNMP v0.3 Compliance Test Runner");
    println!();
    println!("Usage:");
    println!("  lnmp-compliance-runner [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -c, --category <CATEGORY>  Run only tests in the specified category");
    println!("                             (structural, semantic, error-handling, round-trip)");
    println!("  -v, --verbose              Print detailed test results");
    println!("  -h, --help                 Print this help message");
    println!();
    println!("Examples:");
    println!("  lnmp-compliance-runner");
    println!("  lnmp-compliance-runner --category structural");
    println!("  lnmp-compliance-runner --verbose");
}
