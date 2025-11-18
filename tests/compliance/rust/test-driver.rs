use std::path::Path;

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use serde_yaml;

mod runner;

fn main() {
    let yaml_path = Path::new("tests/compliance/test-cases.yaml");
    let suite = runner::TestSuite::load_from_file(yaml_path).expect("failed to load test cases");

    let lenient_path = Path::new("tests/compliance/rust/test-cases-lenient.yaml");
    let lenient_suite = runner::TestSuite::load_from_file(lenient_path).ok();

    let mut combined = suite;
    if let Some(mut lenient) = lenient_suite {
        combined.lenient_tests.append(&mut lenient.structural_tests);
        combined.lenient_tests.append(&mut lenient.semantic_tests);
        combined.lenient_tests.append(&mut lenient.error_handling_tests);
        combined.lenient_tests.append(&mut lenient.round_trip_tests);
    }

    let mut test_runner = runner::TestRunner::new(combined);
    test_runner.run_all();

    test_runner.report_results();
}
