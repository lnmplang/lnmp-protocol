# LNMP Compliance Test Runner (C++)

This directory hosts a C++ harness for the shared compliance vectors under `tests/compliance/test-cases.yaml`. It exists so a future LNMP C++ parser/encoder can validate itself against the modular specification set (`spec/lnmp-*-spec.md`) and receive REQ-ID annotated failures just like the Rust/Python/TypeScript runners.

## Capabilities

- Loads YAML cases (structural, semantic, error, round-trip, lenient) and applies per-test config.
- Reports pass/fail/skip with REQ IDs to surface the relevant spec clause.
- Integrates with Google Test but can also run as a standalone binary.
- Ready to link against a C++ LNMP implementation once it ships (currently tests are skipped with a friendly notice).

## Dependencies

- CMake ≥ 3.14
- Google Test
- yaml-cpp
- C++17 compiler

## Build & Run

```bash
cd tests/compliance/cpp
mkdir -p build && cd build
cmake .. && make
./compliance_runner              # all categories
./compliance_runner --category semantic
./compliance_runner --verbose
```

Until a parser/encoder is wired in, runs will report the total number of skipped cases.

## Project Layout

```
cpp/
├── CMakeLists.txt
├── runner.cpp         # CLI entry point
├── test_runner.h/.cpp # core runner
└── README.md
```

## Integration Checklist

1. Link your LNMP C++ library in `CMakeLists.txt`.
2. Implement strict/loose parsing, canonical encoding, checksum validation, lenient mode, nested structures, etc., as mandated by the modular specs.
3. Wire the parser/encoder into `test_runner.cpp` where TODO markers are present.
4. Ensure failure output appends the `requirements` array from each YAML case (mirroring the Rust runner’s `[REQ: ...]` suffix).
5. Run the commands above and confirm any failures map cleanly back to the spec.

## Categories

- **Structural:** canonicalization, field ordering, nested records/arrays, whitespace (REQ-CAN-*/REQ-TXT-*)
- **Semantic:** type fidelity, normalization, semantic checksums, equivalence mappings (REQ-TXT/REQ-SC/REQ-SAN)
- **Error Handling:** lexical, syntactic, semantic, structural errors (REQ-ERR-*)
- **Round-Trip:** text ↔ binary ↔ text stability (REQ-CAN-RT-*)

## References

- `tests/compliance/README.md` (suite overview, YAML format)
- Modular specs: `spec/lnmp-core-spec.md`, `spec/lnmp-text-format.md`, `spec/lnmp-binary-format.md`, `spec/lnmp-canonicalization.md`, `spec/lnmp-security-compliance.md`, `spec/lnmp-migration-versioning.md`
- `spec/grammar.md`, `spec/error-classes.md`
- `.github/workflows/conformance.yml` (CI entry point)
