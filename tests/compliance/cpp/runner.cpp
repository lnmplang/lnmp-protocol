/**
 * LNMP v0.3 C++ Compliance Test Runner - Main Entry Point
 * 
 * This executable loads test cases from test-cases.yaml and runs them
 * against the C++ LNMP implementation.
 */

#include "test_runner.h"
#include <iostream>
#include <cstdlib>
#include <filesystem>
#include <gtest/gtest.h>

namespace fs = std::filesystem;

// Global test runner instance
static std::unique_ptr<lnmp::compliance::TestRunner> g_runner;

/**
 * Google Test fixture for LNMP compliance tests
 */
class LnmpComplianceTest : public ::testing::TestWithParam<lnmp::compliance::TestCase> {
protected:
    void SetUp() override {
        // Setup code if needed
    }

    void TearDown() override {
        // Cleanup code if needed
    }
};

/**
 * Parameterized test that runs each test case
 */
TEST_P(LnmpComplianceTest, RunTestCase) {
    const auto& test_case = GetParam();
    
    if (!g_runner) {
        FAIL() << "Test runner not initialized";
    }
    
    auto result = g_runner->run_test(test_case);
    
    switch (result.result) {
        case lnmp::compliance::TestResult::Pass:
            SUCCEED();
            break;
        case lnmp::compliance::TestResult::Fail:
            FAIL() << (result.reason.has_value() ? result.reason.value() : "Test failed");
            break;
        case lnmp::compliance::TestResult::Skip:
            GTEST_SKIP() << (result.reason.has_value() ? result.reason.value() : "Test skipped");
            break;
    }
}

/**
 * Find the test-cases.yaml file
 */
std::string find_test_cases_file() {
    // Try relative to current directory
    std::vector<std::string> possible_paths = {
        "../test-cases.yaml",
        "../../test-cases.yaml",
        "../../../test-cases.yaml",
        "tests/compliance/test-cases.yaml",
        "./test-cases.yaml"
    };
    
    for (const auto& path : possible_paths) {
        if (fs::exists(path)) {
            return path;
        }
    }
    
    throw std::runtime_error("Could not find test-cases.yaml file");
}

/**
 * Main entry point
 */
int main(int argc, char** argv) {
    std::cout << "LNMP v0.3 C++ Compliance Test Runner\n";
    std::cout << "=====================================\n\n";
    
    // Parse command line arguments
    bool verbose = false;
    std::string category;
    std::string test_file;
    
    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "-v" || arg == "--verbose") {
            verbose = true;
        } else if (arg == "-c" || arg == "--category") {
            if (i + 1 < argc) {
                category = argv[++i];
            }
        } else if (arg == "-f" || arg == "--file") {
            if (i + 1 < argc) {
                test_file = argv[++i];
            }
        } else if (arg == "-h" || arg == "--help") {
            std::cout << "Usage: " << argv[0] << " [options]\n";
            std::cout << "Options:\n";
            std::cout << "  -v, --verbose          Print detailed test results\n";
            std::cout << "  -c, --category <name>  Run only tests in specified category\n";
            std::cout << "                         (structural, semantic, error-handling, round-trip)\n";
            std::cout << "  -f, --file <path>      Path to test-cases.yaml file\n";
            std::cout << "  -h, --help             Show this help message\n";
            return 0;
        }
    }
    
    try {
        // Find test cases file
        if (test_file.empty()) {
            test_file = find_test_cases_file();
        }
        
        std::cout << "Loading test cases from: " << test_file << "\n\n";
        
        // Load test suite
        auto suite = lnmp::compliance::TestSuite::load_from_file(test_file);
        std::cout << "LNMP v" << suite.version << " Compliance Test Suite\n\n";
        
        // Create test runner
        g_runner = std::make_unique<lnmp::compliance::TestRunner>(suite);
        
        // Get tests to run
        std::vector<lnmp::compliance::TestCase> tests;
        if (!category.empty()) {
            std::cout << "Running tests in category: " << category << "\n\n";
            if (category == "structural") {
                tests = suite.structural_tests;
            } else if (category == "semantic") {
                tests = suite.semantic_tests;
            } else if (category == "error-handling") {
                tests = suite.error_handling_tests;
            } else if (category == "round-trip") {
                tests = suite.round_trip_tests;
            } else {
                std::cerr << "Unknown category: " << category << "\n";
                return 1;
            }
        } else {
            std::cout << "Running all tests...\n\n";
            tests = suite.all_tests();
        }
        
        // If using Google Test framework
        if (argc > 1 && std::string(argv[1]).find("--gtest") != std::string::npos) {
            ::testing::InitGoogleTest(&argc, argv);
            
            // Register parameterized tests
            ::testing::RegisterTest(
                "LnmpCompliance",
                "AllTests",
                nullptr,
                nullptr,
                __FILE__,
                __LINE__,
                [&tests]() -> LnmpComplianceTest* {
                    return new LnmpComplianceTest();
                }
            );
            
            return RUN_ALL_TESTS();
        }
        
        // Otherwise, run tests directly
        for (const auto& test : tests) {
            g_runner->run_test(test);
        }
        
        // Print results
        if (verbose) {
            g_runner->print_detailed();
        } else {
            g_runner->print_summary();
        }
        
        // Return exit code based on results
        const auto& results = g_runner->get_results();
        size_t failed = std::count_if(results.begin(), results.end(),
            [](const auto& r) { return r.result == lnmp::compliance::TestResult::Fail; });
        
        return failed > 0 ? 1 : 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
