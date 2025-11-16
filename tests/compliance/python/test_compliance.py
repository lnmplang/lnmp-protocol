"""
Pytest test suite for LNMP v0.3 compliance tests

This module integrates the compliance test runner with pytest,
allowing tests to be run using the pytest framework.
"""

import pytest
from pathlib import Path
from runner import TestSuite, TestRunner, TestResult


# Load test suite once for all tests
TEST_FILE = Path(__file__).parent.parent / "test-cases.yaml"
SUITE = TestSuite.load_from_file(TEST_FILE)


class TestStructural:
    """Structural compliance tests"""

    @pytest.fixture(scope="class")
    def runner(self):
        """Create test runner"""
        return TestRunner(SUITE)

    @pytest.mark.parametrize("test_case", SUITE.structural_tests, ids=lambda t: t.name)
    def test_structural(self, runner, test_case):
        """Run structural test case"""
        result, reason = runner.run_test(test_case)

        if result == TestResult.SKIP:
            pytest.skip(reason or "Test skipped")
        elif result == TestResult.FAIL:
            pytest.fail(reason or "Test failed")
        else:
            assert result == TestResult.PASS


class TestSemantic:
    """Semantic compliance tests"""

    @pytest.fixture(scope="class")
    def runner(self):
        """Create test runner"""
        return TestRunner(SUITE)

    @pytest.mark.parametrize("test_case", SUITE.semantic_tests, ids=lambda t: t.name)
    def test_semantic(self, runner, test_case):
        """Run semantic test case"""
        result, reason = runner.run_test(test_case)

        if result == TestResult.SKIP:
            pytest.skip(reason or "Test skipped")
        elif result == TestResult.FAIL:
            pytest.fail(reason or "Test failed")
        else:
            assert result == TestResult.PASS


class TestErrorHandling:
    """Error handling compliance tests"""

    @pytest.fixture(scope="class")
    def runner(self):
        """Create test runner"""
        return TestRunner(SUITE)

    @pytest.mark.parametrize("test_case", SUITE.error_handling_tests, ids=lambda t: t.name)
    def test_error_handling(self, runner, test_case):
        """Run error handling test case"""
        result, reason = runner.run_test(test_case)

        if result == TestResult.SKIP:
            pytest.skip(reason or "Test skipped")
        elif result == TestResult.FAIL:
            pytest.fail(reason or "Test failed")
        else:
            assert result == TestResult.PASS


class TestRoundTrip:
    """Round-trip compliance tests"""

    @pytest.fixture(scope="class")
    def runner(self):
        """Create test runner"""
        return TestRunner(SUITE)

    @pytest.mark.parametrize("test_case", SUITE.round_trip_tests, ids=lambda t: t.name)
    def test_round_trip(self, runner, test_case):
        """Run round-trip test case"""
        result, reason = runner.run_test(test_case)

        if result == TestResult.SKIP:
            pytest.skip(reason or "Test skipped")
        elif result == TestResult.FAIL:
            pytest.fail(reason or "Test failed")
        else:
            assert result == TestResult.PASS


def test_suite_loaded():
    """Verify test suite loaded successfully"""
    assert SUITE is not None
    assert SUITE.version == "0.3.0"
    assert len(SUITE.all_tests()) > 0


def test_structural_tests_exist():
    """Verify structural tests are present"""
    assert len(SUITE.structural_tests) > 0


def test_semantic_tests_exist():
    """Verify semantic tests are present"""
    assert len(SUITE.semantic_tests) > 0


def test_error_handling_tests_exist():
    """Verify error handling tests are present"""
    assert len(SUITE.error_handling_tests) > 0


def test_round_trip_tests_exist():
    """Verify round-trip tests are present"""
    assert len(SUITE.round_trip_tests) > 0
