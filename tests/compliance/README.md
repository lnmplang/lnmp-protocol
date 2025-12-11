# LNMP Compliance Suite (v0.4+)

This directory houses the language-agnostic compliance suite used by every LNMP implementation (Rust, Python, TypeScript, C++, …) to verify conformance with the modular specifications:

- `spec/lnmp-core-spec.md`
- `spec/lnmp-text-format.md`
- `spec/lnmp-binary-format.md`
- `spec/lnmp-canonicalization.md`
- `spec/lnmp-security-compliance.md`
- `spec/lnmp-migration-versioning.md`
- `spec/grammar.md` & `spec/error-classes.md`

Each YAML test vector includes requirement IDs (`REQ-*`) referencing those documents so failures map directly to spec clauses. CI runs the Rust harness by default, but every SDK is expected to consume the same YAML.

## Test Case Format

Vectors live in `tests/compliance/test-cases.yaml`. Key shapes:

### Successful Parse

```yaml
- name: "Human-readable test name"
  category: "structural|semantic|error-handling|round-trip"
  description: "Brief explanation"
  input: "LNMP text"
  config:
    strict_mode: false
    validate_checksums: false
  expected:
    fields:
      - fid: 7
        type: int
        value: 2
      - fid: 12
        type: int
        value: 1
  requirements:
    - "REQ-TXT-07"
    - "REQ-CAN-TXT-01"
```

### Round-Trip

```yaml
- name: "Round-trip test name"
  category: "round-trip"
  description: "Parse → Encode → Parse"
  input: "F23=admin;F12=1;F7=2"
  expected_canonical: "F7=2
F12=1
F23=admin"
  requirements:
    - "REQ-CAN-RT-01"
```

### Error

```yaml
- name: "Error test name"
  category: "error-handling"
  description: "Which error should fire"
  input: "FXX=1"
  expected:
    error: "InvalidFieldId"
    message: "Field ID must be numeric"
    field_id: 0
  requirements:
    - "REQ-ERR-01"
```

## Field Types

`int`, `float`, `bool`, `string`, `string_array`, `nested_record`, `nested_array` (alias: `record_array`).

## Config Flags

`strict_mode`, `validate_checksums`, `normalize_values`, `max_nesting_depth`, `preserve_checksums`, `equivalence_mapping`, `lenient_mode`, and future semantic dictionary hooks.

## Categories

- **Structural:** canonicalization, nested structures, whitespace, deep nesting.
- **Semantic:** type fidelity, normalization, semantic checksums, equivalence mappings.
- **Error Handling:** lexical/syntactic/semantic/structural edge cases.
- **Round-Trip:** parse→encode→parse stability (text↔binary).

## Error Classes

See `spec/error-classes.md` for canonical names; YAML uses the same identifiers.

## Implementing a Runner

1. Load `test-cases.yaml`.
2. Filter by category (optional).
3. For each case:
   - Parse `input` with the requested `config`.
   - Compare results to `expected` (or `expected_canonical`).
   - Validate error class/message for error cases.
   - Emit REQ IDs when reporting failures.

A typical directory layout:

```
tests/compliance/<language>/
├── runner.{ext}
├── README.md
└── helpers/
```

## Language Runners

| Language | Path | Status |
|----------|------|--------|
| Rust | `tests/compliance/rust/` | ✅ Reference implementation (used in CI) |
| Python | `tests/compliance/python/` | ✅ Awaiting LNMP Python binding |
| TypeScript | `tests/compliance/typescript/` | ⏳ Planned |
| C++ | `tests/compliance/cpp/` | ⏳ Planned |

## Adding New Tests

1. Choose category & REQ IDs.
2. Append to `test-cases.yaml` with `requirements` filled.
3. Run `cargo test -p lnmp-compliance-tests` + other harnesses.

## Validation Checklist

- YAML syntax valid.
- Fields/use of type names valid.
- Error classes exist in `spec/error-classes.md`.
- Expected canonical text/binary matches parser/encoder output.

## References

- `spec/lnmp-core-spec.md`
- `spec/lnmp-text-format.md`
- `spec/lnmp-binary-format.md`
- `spec/lnmp-canonicalization.md`
- `spec/lnmp-security-compliance.md`
- `spec/lnmp-migration-versioning.md`
- `spec/grammar.md`
- `spec/error-classes.md`
- `.github/workflows/spec-fixtures.yml`
