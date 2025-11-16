/**
 * LNMP v0.3 C++ Compliance Test Runner Implementation
 */

#include "test_runner.h"
#include <fstream>
#include <iostream>
#include <algorithm>
#include <cctype>

namespace lnmp {
namespace compliance {

// ============================================================================
// TestSuite Implementation
// ============================================================================

TestSuite TestSuite::load_from_file(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open test file: " + path);
    }

    YAML::Node root = YAML::Load(file);
    TestSuite suite;

    suite.version = root["version"].as<std::string>("unknown");
    
    if (root["structural_tests"]) {
        suite.structural_tests = load_tests(root["structural_tests"]);
    }
    if (root["semantic_tests"]) {
        suite.semantic_tests = load_tests(root["semantic_tests"]);
    }
    if (root["error_handling_tests"]) {
        suite.error_handling_tests = load_tests(root["error_handling_tests"]);
    }
    if (root["round_trip_tests"]) {
        suite.round_trip_tests = load_tests(root["round_trip_tests"]);
    }

    return suite;
}

std::vector<TestCase> TestSuite::all_tests() const {
    std::vector<TestCase> all;
    all.insert(all.end(), structural_tests.begin(), structural_tests.end());
    all.insert(all.end(), semantic_tests.begin(), semantic_tests.end());
    all.insert(all.end(), error_handling_tests.begin(), error_handling_tests.end());
    all.insert(all.end(), round_trip_tests.begin(), round_trip_tests.end());
    return all;
}

std::vector<TestCase> TestSuite::load_tests(const YAML::Node& node) {
    std::vector<TestCase> tests;
    
    for (const auto& item : node) {
        TestCase test;
        test.name = item["name"].as<std::string>();
        test.category = item["category"].as<std::string>();
        test.description = item["description"].as<std::string>();
        test.input = item["input"].as<std::string>();
        
        if (item["config"]) {
            test.config = load_config(item["config"]);
        }
        
        if (item["expected"]) {
            test.expected = load_expected(item["expected"]);
        }
        
        if (item["expected_canonical"]) {
            test.expected_canonical = item["expected_canonical"].as<std::string>();
        }
        
        tests.push_back(test);
    }
    
    return tests;
}

TestConfig TestSuite::load_config(const YAML::Node& node) {
    TestConfig config;
    
    if (node["normalize_values"]) {
        config.normalize_values = node["normalize_values"].as<bool>();
    }
    if (node["validate_checksums"]) {
        config.validate_checksums = node["validate_checksums"].as<bool>();
    }
    if (node["strict_mode"]) {
        config.strict_mode = node["strict_mode"].as<bool>();
    }
    if (node["preserve_checksums"]) {
        config.preserve_checksums = node["preserve_checksums"].as<bool>();
    }
    if (node["max_nesting_depth"]) {
        config.max_nesting_depth = node["max_nesting_depth"].as<size_t>();
    }
    if (node["equivalence_mapping"]) {
        for (const auto& field_mapping : node["equivalence_mapping"]) {
            uint16_t fid = field_mapping.first.as<uint16_t>();
            std::map<std::string, std::string> mappings;
            
            for (const auto& mapping : field_mapping.second) {
                mappings[mapping.first.as<std::string>()] = mapping.second.as<std::string>();
            }
            
            config.equivalence_mapping[fid] = mappings;
        }
    }
    
    return config;
}

ExpectedOutput TestSuite::load_expected(const YAML::Node& node) {
    ExpectedOutput output;
    
    // Check if this is an error expectation
    if (node["error"]) {
        output.is_error = true;
        output.error = load_expected_error(node);
    } else {
        output.is_error = false;
        if (node["fields"]) {
            output.fields = load_expected_fields(node["fields"]);
        }
    }
    
    return output;
}

std::vector<ExpectedField> TestSuite::load_expected_fields(const YAML::Node& node) {
    std::vector<ExpectedField> fields;
    
    for (const auto& field_node : node) {
        ExpectedField field;
        field.fid = field_node["fid"].as<uint16_t>();
        field.type_name = field_node["type"].as<std::string>();
        field.value = field_node["value"];
        
        if (field_node["checksum"]) {
            field.checksum = field_node["checksum"].as<std::string>();
        }
        
        fields.push_back(field);
    }
    
    return fields;
}

ExpectedError TestSuite::load_expected_error(const YAML::Node& node) {
    ExpectedError error;
    error.error = node["error"].as<std::string>();
    error.message = node["message"].as<std::string>();
    
    if (node["field_id"]) {
        error.field_id = node["field_id"].as<uint16_t>();
    }
    if (node["line"]) {
        error.line = node["line"].as<size_t>();
    }
    if (node["column"]) {
        error.column = node["column"].as<size_t>();
    }
    if (node["max_depth"]) {
        error.max_depth = node["max_depth"].as<size_t>();
    }
    
    return error;
}

// ============================================================================
// TestRunner Implementation
// ============================================================================

TestRunner::TestRunner(const TestSuite& suite) : suite_(suite) {}

void TestRunner::run_all() {
    auto tests = suite_.all_tests();
    for (const auto& test : tests) {
        results_.push_back(run_test(test));
    }
}

void TestRunner::run_category(const std::string& category) {
    std::vector<TestCase> tests;
    
    if (category == "structural") {
        tests = suite_.structural_tests;
    } else if (category == "semantic") {
        tests = suite_.semantic_tests;
    } else if (category == "error-handling") {
        tests = suite_.error_handling_tests;
    } else if (category == "round-trip") {
        tests = suite_.round_trip_tests;
    }
    
    for (const auto& test : tests) {
        results_.push_back(run_test(test));
    }
}

TestExecutionResult TestRunner::run_test(const TestCase& test) {
    // Handle round-trip tests separately
    if (test.expected_canonical.has_value()) {
        return run_round_trip_test(test);
    }
    
    if (!test.expected.has_value()) {
        return TestExecutionResult{
            test.name,
            TestResult::Fail,
            "Test case has neither 'expected' nor 'expected_canonical' field"
        };
    }
    
    const auto& expected = test.expected.value();
    
    if (expected.is_error) {
        return run_error_test(test, expected.error.value());
    } else {
        return run_success_test(test, expected.fields);
    }
}

TestExecutionResult TestRunner::run_success_test(
    const TestCase& test,
    const std::vector<ExpectedField>& expected_fields) {
    
    // TODO: Integrate with C++ LNMP implementation
    // For now, skip tests until implementation is available
    return TestExecutionResult{
        test.name,
        TestResult::Skip,
        "C++ LNMP implementation not yet available"
    };
    
    /* Example integration (uncomment when implementation is ready):
    
    try {
        // Create parser with appropriate mode
        lnmp::ParsingMode mode = test.config.strict_mode 
            ? lnmp::ParsingMode::Strict 
            : lnmp::ParsingMode::Loose;
        
        lnmp::Parser parser(test.input, mode);
        auto record = parser.parse_record();
        
        return validate_record(test.name, &record, expected_fields);
    } catch (const std::exception& e) {
        return TestExecutionResult{
            test.name,
            TestResult::Fail,
            std::string("Failed to parse: ") + e.what()
        };
    }
    */
}

TestExecutionResult TestRunner::run_error_test(
    const TestCase& test,
    const ExpectedError& expected_error) {
    
    // TODO: Integrate with C++ LNMP implementation
    return TestExecutionResult{
        test.name,
        TestResult::Skip,
        "C++ LNMP implementation not yet available"
    };
    
    /* Example integration (uncomment when implementation is ready):
    
    try {
        lnmp::ParsingMode mode = test.config.strict_mode 
            ? lnmp::ParsingMode::Strict 
            : lnmp::ParsingMode::Loose;
        
        lnmp::Parser parser(test.input, mode);
        auto record = parser.parse_record();
        
        return TestExecutionResult{
            test.name,
            TestResult::Fail,
            "Expected error '" + expected_error.error + "' but parsing succeeded"
        };
    } catch (const std::exception& e) {
        return validate_error(test.name, e.what(), expected_error);
    }
    */
}

TestExecutionResult TestRunner::run_round_trip_test(const TestCase& test) {
    // TODO: Integrate with C++ LNMP implementation
    return TestExecutionResult{
        test.name,
        TestResult::Skip,
        "C++ LNMP implementation not yet available"
    };
    
    /* Example integration (uncomment when implementation is ready):
    
    try {
        lnmp::Parser parser(test.input);
        auto record = parser.parse_record();
        
        lnmp::EncoderConfig config;
        config.include_type_hints = true;
        config.canonical = true;
        config.include_checksums = test.config.preserve_checksums;
        
        lnmp::Encoder encoder(config);
        std::string encoded = encoder.encode(record);
        
        // Trim whitespace for comparison
        auto trim = [](std::string s) {
            s.erase(s.begin(), std::find_if(s.begin(), s.end(), [](unsigned char ch) {
                return !std::isspace(ch);
            }));
            s.erase(std::find_if(s.rbegin(), s.rend(), [](unsigned char ch) {
                return !std::isspace(ch);
            }).base(), s.end());
            return s;
        };
        
        if (trim(encoded) == trim(test.expected_canonical.value())) {
            return TestExecutionResult{
                test.name,
                TestResult::Pass,
                std::nullopt
            };
        } else {
            return TestExecutionResult{
                test.name,
                TestResult::Fail,
                "Round-trip mismatch:\nExpected: " + test.expected_canonical.value() + 
                "\nGot: " + encoded
            };
        }
    } catch (const std::exception& e) {
        return TestExecutionResult{
            test.name,
            TestResult::Fail,
            std::string("Failed to parse: ") + e.what()
        };
    }
    */
}

TestExecutionResult TestRunner::validate_record(
    const std::string& test_name,
    const void* record,
    const std::vector<ExpectedField>& expected_fields) {
    
    // TODO: Implement validation logic
    // This will depend on the structure of the C++ LNMP implementation
    return TestExecutionResult{
        test_name,
        TestResult::Skip,
        "Validation not yet implemented"
    };
}

std::optional<std::string> TestRunner::validate_value(
    const void* actual,
    const YAML::Node& expected,
    const std::string& expected_type) {
    
    // TODO: Implement value validation logic
    // This will depend on the structure of the C++ LNMP implementation
    return "Validation not yet implemented";
}

TestExecutionResult TestRunner::validate_error(
    const std::string& test_name,
    const std::string& actual_error,
    const ExpectedError& expected) {
    
    // Convert to lowercase for case-insensitive comparison
    auto to_lower = [](std::string s) {
        std::transform(s.begin(), s.end(), s.begin(),
                      [](unsigned char c) { return std::tolower(c); });
        return s;
    };
    
    std::string actual_lower = to_lower(actual_error);
    std::string expected_lower = to_lower(expected.error);
    
    if (actual_lower.find(expected_lower) == std::string::npos) {
        return TestExecutionResult{
            test_name,
            TestResult::Fail,
            "Error type mismatch: expected '" + expected.error + "', got '" + actual_error + "'"
        };
    }
    
    std::string expected_msg_lower = to_lower(expected.message);
    if (actual_lower.find(expected_msg_lower) == std::string::npos) {
        return TestExecutionResult{
            test_name,
            TestResult::Fail,
            "Error message mismatch: expected to contain '" + expected.message + 
            "', got '" + actual_error + "'"
        };
    }
    
    return TestExecutionResult{
        test_name,
        TestResult::Pass,
        std::nullopt
    };
}

const std::vector<TestExecutionResult>& TestRunner::get_results() const {
    return results_;
}

void TestRunner::print_summary() const {
    size_t total = results_.size();
    size_t passed = std::count_if(results_.begin(), results_.end(),
        [](const auto& r) { return r.result == TestResult::Pass; });
    size_t failed = std::count_if(results_.begin(), results_.end(),
        [](const auto& r) { return r.result == TestResult::Fail; });
    size_t skipped = total - passed - failed;
    
    std::cout << "\n" << std::string(80, '=') << "\n";
    std::cout << "LNMP v0.3 Compliance Test Results (C++)\n";
    std::cout << std::string(80, '=') << "\n";
    std::cout << "Total:   " << total << "\n";
    std::cout << "Passed:  " << passed << " (" 
              << (total > 0 ? (passed * 100) / total : 0) << "%)\n";
    std::cout << "Failed:  " << failed << "\n";
    std::cout << "Skipped: " << skipped << "\n";
    std::cout << std::string(80, '=') << "\n";
    
    if (failed > 0) {
        std::cout << "\nFailed Tests:\n";
        std::cout << std::string(80, '-') << "\n";
        for (const auto& result : results_) {
            if (result.result == TestResult::Fail) {
                std::cout << "❌ " << result.name << "\n";
                if (result.reason.has_value()) {
                    std::cout << "   " << result.reason.value() << "\n";
                }
                std::cout << "\n";
            }
        }
    }
}

void TestRunner::print_detailed() const {
    std::cout << "\n" << std::string(80, '=') << "\n";
    std::cout << "LNMP v0.3 Compliance Test Results (C++) - Detailed\n";
    std::cout << std::string(80, '=') << "\n";
    
    for (const auto& result : results_) {
        switch (result.result) {
            case TestResult::Pass:
                std::cout << "✅ " << result.name << "\n";
                break;
            case TestResult::Fail:
                std::cout << "❌ " << result.name << "\n";
                if (result.reason.has_value()) {
                    std::cout << "   " << result.reason.value() << "\n";
                }
                break;
            case TestResult::Skip:
                std::cout << "⏭️  " << result.name << "\n";
                if (result.reason.has_value()) {
                    std::cout << "   " << result.reason.value() << "\n";
                }
                break;
        }
    }
    
    print_summary();
}

} // namespace compliance
} // namespace lnmp
