"""
LNMP v0.3 Python Compliance Test Runner

This module loads test cases from test-cases.yaml and executes them
against a Python LNMP implementation, reporting pass/fail with detailed
error messages.
"""

import yaml
from pathlib import Path
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass
from enum import Enum


class TestResult(Enum):
    """Test result status"""
    PASS = "pass"
    FAIL = "fail"
    SKIP = "skip"


@dataclass
class TestConfig:
    """Test case configuration options"""
    normalize_values: bool = False
    validate_checksums: bool = False
    strict_mode: bool = False
    preserve_checksums: bool = False
    max_nesting_depth: Optional[int] = None
    equivalence_mapping: Optional[Dict[int, Dict[str, str]]] = None


@dataclass
class ExpectedField:
    """Expected field structure in test cases"""
    fid: int
    type_name: str
    value: Any
    checksum: Optional[str] = None


@dataclass
class ExpectedError:
    """Expected error structure"""
    error: str
    message: str
    field_id: Optional[int] = None
    line: Optional[int] = None
    column: Optional[int] = None
    max_depth: Optional[int] = None


@dataclass
class TestCase:
    """A single test case"""
    name: str
    category: str
    description: str
    input: str
    expected: Optional[Union[Dict[str, Any], ExpectedError]] = None
    config: TestConfig = None
    expected_canonical: Optional[str] = None

    def __post_init__(self):
        if self.config is None:
            self.config = TestConfig()


class TestSuite:
    """Test suite containing all test cases"""

    def __init__(self, data: Dict[str, Any]):
        self.version = data.get("version", "unknown")
        self.structural_tests = self._load_tests(data.get("structural_tests", []))
        self.semantic_tests = self._load_tests(data.get("semantic_tests", []))
        self.error_handling_tests = self._load_tests(data.get("error_handling_tests", []))
        self.round_trip_tests = self._load_tests(data.get("round_trip_tests", []))

    def _load_tests(self, test_data: List[Dict[str, Any]]) -> List[TestCase]:
        """Load test cases from YAML data"""
        tests = []
        for item in test_data:
            config_data = item.get("config", {})
            config = TestConfig(
                normalize_values=config_data.get("normalize_values", False),
                validate_checksums=config_data.get("validate_checksums", False),
                strict_mode=config_data.get("strict_mode", False),
                preserve_checksums=config_data.get("preserve_checksums", False),
                max_nesting_depth=config_data.get("max_nesting_depth"),
                equivalence_mapping=config_data.get("equivalence_mapping"),
            )

            test = TestCase(
                name=item["name"],
                category=item["category"],
                description=item["description"],
                input=item["input"],
                expected=item.get("expected"),
                config=config,
                expected_canonical=item.get("expected_canonical"),
            )
            tests.append(test)
        return tests

    @classmethod
    def load_from_file(cls, path: Path) -> "TestSuite":
        """Load test suite from YAML file"""
        with open(path, "r") as f:
            data = yaml.safe_load(f)
        return cls(data)

    def all_tests(self) -> List[TestCase]:
        """Get all test cases from all categories"""
        return (
            self.structural_tests
            + self.semantic_tests
            + self.error_handling_tests
            + self.round_trip_tests
        )


class TestRunner:
    """Test runner for executing compliance tests"""

    def __init__(self, suite: TestSuite):
        self.suite = suite
        self.results: List[tuple[str, TestResult, Optional[str]]] = []

    def run_all(self):
        """Run all tests in the suite"""
        for test in self.suite.all_tests():
            result, reason = self.run_test(test)
            self.results.append((test.name, result, reason))

    def run_test(self, test: TestCase) -> tuple[TestResult, Optional[str]]:
        """Run a single test case"""
        # Handle round-trip tests separately
        if test.expected_canonical is not None:
            return self._run_round_trip_test(test)

        if test.expected is None:
            return TestResult.FAIL, "Test case has neither 'expected' nor 'expected_canonical' field"

        # Check if this is an error test
        if isinstance(test.expected, dict) and "error" in test.expected:
            expected_error = ExpectedError(
                error=test.expected["error"],
                message=test.expected["message"],
                field_id=test.expected.get("field_id"),
                line=test.expected.get("line"),
                column=test.expected.get("column"),
                max_depth=test.expected.get("max_depth"),
            )
            return self._run_error_test(test, expected_error)
        else:
            # Success test
            expected_fields = self._parse_expected_fields(test.expected.get("fields", []))
            return self._run_success_test(test, expected_fields)

    def _run_success_test(
        self, test: TestCase, expected_fields: List[ExpectedField]
    ) -> tuple[TestResult, Optional[str]]:
        """Run a test that expects successful parsing"""
        # TODO: Integrate with Python LNMP implementation
        # For now, skip tests until implementation is available
        return TestResult.SKIP, "Python LNMP implementation not yet available"

        # Example integration (uncomment when implementation is ready):
        # try:
        #     from lnmp import Parser, ParsingMode
        #     
        #     mode = ParsingMode.STRICT if test.config.strict_mode else ParsingMode.LOOSE
        #     parser = Parser(test.input, mode=mode)
        #     record = parser.parse_record()
        #     
        #     return self._validate_record(record, expected_fields)
        # except Exception as e:
        #     return TestResult.FAIL, f"Failed to parse: {e}"

    def _run_error_test(
        self, test: TestCase, expected_error: ExpectedError
    ) -> tuple[TestResult, Optional[str]]:
        """Run a test that expects an error"""
        # TODO: Integrate with Python LNMP implementation
        return TestResult.SKIP, "Python LNMP implementation not yet available"

        # Example integration (uncomment when implementation is ready):
        # try:
        #     from lnmp import Parser, ParsingMode
        #     
        #     mode = ParsingMode.STRICT if test.config.strict_mode else ParsingMode.LOOSE
        #     parser = Parser(test.input, mode=mode)
        #     record = parser.parse_record()
        #     
        #     return TestResult.FAIL, f"Expected error '{expected_error.error}' but parsing succeeded"
        # except Exception as e:
        #     return self._validate_error(str(e), expected_error)

    def _run_round_trip_test(self, test: TestCase) -> tuple[TestResult, Optional[str]]:
        """Run a round-trip test (parse -> encode -> compare)"""
        # TODO: Integrate with Python LNMP implementation
        return TestResult.SKIP, "Python LNMP implementation not yet available"

        # Example integration (uncomment when implementation is ready):
        # try:
        #     from lnmp import Parser, Encoder, EncoderConfig
        #     
        #     parser = Parser(test.input)
        #     record = parser.parse_record()
        #     
        #     config = EncoderConfig(
        #         include_type_hints=True,
        #         canonical=True,
        #         include_checksums=test.config.preserve_checksums,
        #     )
        #     encoder = Encoder(config)
        #     encoded = encoder.encode(record)
        #     
        #     if encoded.strip() == test.expected_canonical.strip():
        #         return TestResult.PASS, None
        #     else:
        #         return TestResult.FAIL, f"Round-trip mismatch:\nExpected: {test.expected_canonical}\nGot: {encoded}"
        # except Exception as e:
        #     return TestResult.FAIL, f"Failed to parse: {e}"

    def _validate_record(
        self, record: Any, expected_fields: List[ExpectedField]
    ) -> tuple[TestResult, Optional[str]]:
        """Validate that a parsed record matches expected fields"""
        # TODO: Implement validation logic
        # This will depend on the structure of the Python LNMP implementation
        return TestResult.SKIP, "Validation not yet implemented"

    def _validate_value(
        self, actual: Any, expected: Any, expected_type: str
    ) -> tuple[bool, Optional[str]]:
        """Validate that a value matches the expected value"""
        # TODO: Implement value validation logic
        return False, "Validation not yet implemented"

    def _validate_error(
        self, actual_error: str, expected: ExpectedError
    ) -> tuple[TestResult, Optional[str]]:
        """Validate that an error matches the expected error"""
        actual_lower = actual_error.lower()
        expected_lower = expected.error.lower()

        if expected_lower not in actual_lower:
            return TestResult.FAIL, f"Error type mismatch: expected '{expected.error}', got '{actual_error}'"

        expected_msg_lower = expected.message.lower()
        if expected_msg_lower not in actual_lower:
            return TestResult.FAIL, f"Error message mismatch: expected to contain '{expected.message}', got '{actual_error}'"

        return TestResult.PASS, None

    def _parse_expected_fields(self, fields_data: List[Dict[str, Any]]) -> List[ExpectedField]:
        """Parse expected fields from YAML data"""
        fields = []
        for field_data in fields_data:
            field = ExpectedField(
                fid=field_data["fid"],
                type_name=field_data["type"],
                value=field_data["value"],
                checksum=field_data.get("checksum"),
            )
            fields.append(field)
        return fields

    def get_results(self) -> List[tuple[str, TestResult, Optional[str]]]:
        """Get test results"""
        return self.results

    def print_summary(self):
        """Print test results summary"""
        total = len(self.results)
        passed = sum(1 for _, r, _ in self.results if r == TestResult.PASS)
        failed = sum(1 for _, r, _ in self.results if r == TestResult.FAIL)
        skipped = sum(1 for _, r, _ in self.results if r == TestResult.SKIP)

        print("\n" + "=" * 80)
        print("LNMP v0.3 Compliance Test Results (Python)")
        print("=" * 80)
        print(f"Total:   {total}")
        print(f"Passed:  {passed} ({(passed * 100) // max(total, 1)}%)")
        print(f"Failed:  {failed}")
        print(f"Skipped: {skipped}")
        print("=" * 80)

        if failed > 0:
            print("\nFailed Tests:")
            print("-" * 80)
            for name, result, reason in self.results:
                if result == TestResult.FAIL:
                    print(f"❌ {name}")
                    if reason:
                        print(f"   {reason}")
                    print()

    def print_detailed(self):
        """Print detailed results for all tests"""
        print("\n" + "=" * 80)
        print("LNMP v0.3 Compliance Test Results (Python) - Detailed")
        print("=" * 80)

        for name, result, reason in self.results:
            if result == TestResult.PASS:
                print(f"✅ {name}")
            elif result == TestResult.FAIL:
                print(f"❌ {name}")
                if reason:
                    print(f"   {reason}")
            elif result == TestResult.SKIP:
                print(f"⏭️  {name}")
                if reason:
                    print(f"   {reason}")

        self.print_summary()


def main():
    """Main entry point for standalone execution"""
    import sys
    import argparse

    parser = argparse.ArgumentParser(description="LNMP v0.3 Python Compliance Test Runner")
    parser.add_argument(
        "-c",
        "--category",
        choices=["structural", "semantic", "error-handling", "round-trip"],
        help="Run only tests in the specified category",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Print detailed test results"
    )
    args = parser.parse_args()

    # Load test suite
    test_file = Path(__file__).parent.parent / "test-cases.yaml"
    if not test_file.exists():
        print(f"Error: Test file not found: {test_file}", file=sys.stderr)
        sys.exit(1)

    suite = TestSuite.load_from_file(test_file)
    print(f"LNMP v{suite.version} Compliance Test Runner (Python)")
    print()

    # Create runner and execute tests
    runner = TestRunner(suite)

    if args.category:
        print(f"Running tests in category: {args.category}")
        category_map = {
            "structural": suite.structural_tests,
            "semantic": suite.semantic_tests,
            "error-handling": suite.error_handling_tests,
            "round-trip": suite.round_trip_tests,
        }
        tests = category_map[args.category]
        for test in tests:
            result, reason = runner.run_test(test)
            runner.results.append((test.name, result, reason))
    else:
        print("Running all tests...")
        runner.run_all()

    # Print results
    if args.verbose:
        runner.print_detailed()
    else:
        runner.print_summary()

    # Exit with appropriate code
    failed_count = sum(1 for _, r, _ in runner.results if r == TestResult.FAIL)
    sys.exit(1 if failed_count > 0 else 0)


if __name__ == "__main__":
    main()
