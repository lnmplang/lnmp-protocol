/**
 * LNMP v0.3 C++ Compliance Test Runner
 * 
 * This module loads test cases from test-cases.yaml and executes them
 * against a C++ LNMP implementation, reporting pass/fail with detailed
 * error messages.
 */

#ifndef LNMP_CPP_TEST_RUNNER_H
#define LNMP_CPP_TEST_RUNNER_H

#include <string>
#include <vector>
#include <map>
#include <optional>
#include <memory>
#include <yaml-cpp/yaml.h>

namespace lnmp {
namespace compliance {

/**
 * Test result status
 */
enum class TestResult {
    Pass,
    Fail,
    Skip
};

/**
 * Test case configuration options
 */
struct TestConfig {
    bool normalize_values = false;
    bool validate_checksums = false;
    bool strict_mode = false;
    bool preserve_checksums = false;
    std::optional<size_t> max_nesting_depth;
    std::map<uint16_t, std::map<std::string, std::string>> equivalence_mapping;
};

/**
 * Expected field structure in test cases
 */
struct ExpectedField {
    uint16_t fid;
    std::string type_name;
    YAML::Node value;
    std::optional<std::string> checksum;
};

/**
 * Expected error structure
 */
struct ExpectedError {
    std::string error;
    std::string message;
    std::optional<uint16_t> field_id;
    std::optional<size_t> line;
    std::optional<size_t> column;
    std::optional<size_t> max_depth;
};

/**
 * Expected output - either fields or an error
 */
struct ExpectedOutput {
    bool is_error;
    std::vector<ExpectedField> fields;
    std::optional<ExpectedError> error;
};

/**
 * A single test case
 */
struct TestCase {
    std::string name;
    std::string category;
    std::string description;
    std::string input;
    std::optional<ExpectedOutput> expected;
    TestConfig config;
    std::optional<std::string> expected_canonical;
};

/**
 * Test execution result
 */
struct TestExecutionResult {
    std::string name;
    TestResult result;
    std::optional<std::string> reason;
};

/**
 * Test suite containing all test cases
 */
class TestSuite {
public:
    std::string version;
    std::vector<TestCase> structural_tests;
    std::vector<TestCase> semantic_tests;
    std::vector<TestCase> error_handling_tests;
    std::vector<TestCase> round_trip_tests;

    /**
     * Load test suite from YAML file
     */
    static TestSuite load_from_file(const std::string& path);

    /**
     * Get all test cases from all categories
     */
    std::vector<TestCase> all_tests() const;

private:
    static std::vector<TestCase> load_tests(const YAML::Node& node);
    static TestConfig load_config(const YAML::Node& node);
    static ExpectedOutput load_expected(const YAML::Node& node);
    static std::vector<ExpectedField> load_expected_fields(const YAML::Node& node);
    static ExpectedError load_expected_error(const YAML::Node& node);
};

/**
 * Test runner for executing compliance tests
 */
class TestRunner {
public:
    explicit TestRunner(const TestSuite& suite);

    /**
     * Run all tests in the suite
     */
    void run_all();

    /**
     * Run tests in a specific category
     */
    void run_category(const std::string& category);

    /**
     * Run a single test case
     */
    TestExecutionResult run_test(const TestCase& test);

    /**
     * Get test results
     */
    const std::vector<TestExecutionResult>& get_results() const;

    /**
     * Print test results summary
     */
    void print_summary() const;

    /**
     * Print detailed results for all tests
     */
    void print_detailed() const;

private:
    TestSuite suite_;
    std::vector<TestExecutionResult> results_;

    /**
     * Run a test that expects successful parsing
     */
    TestExecutionResult run_success_test(const TestCase& test, 
                                         const std::vector<ExpectedField>& expected_fields);

    /**
     * Run a test that expects an error
     */
    TestExecutionResult run_error_test(const TestCase& test, 
                                       const ExpectedError& expected_error);

    /**
     * Run a round-trip test (parse -> encode -> compare)
     */
    TestExecutionResult run_round_trip_test(const TestCase& test);

    /**
     * Validate that a parsed record matches expected fields
     */
    TestExecutionResult validate_record(const std::string& test_name,
                                        const void* record,
                                        const std::vector<ExpectedField>& expected_fields);

    /**
     * Validate that a value matches the expected value
     */
    std::optional<std::string> validate_value(const void* actual,
                                              const YAML::Node& expected,
                                              const std::string& expected_type);

    /**
     * Validate that an error matches the expected error
     */
    TestExecutionResult validate_error(const std::string& test_name,
                                       const std::string& actual_error,
                                       const ExpectedError& expected);
};

} // namespace compliance
} // namespace lnmp

#endif // LNMP_CPP_TEST_RUNNER_H
