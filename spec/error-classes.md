# LNMP Error Classification

This document provides a comprehensive classification of all error types in LNMP v0.3, organized by category and severity. Each error class includes its purpose, when it occurs, and how implementations should handle it.

## Error Categories

LNMP errors are classified into five primary categories:

1. **Lexical Errors** - Issues during tokenization
2. **Syntactic Errors** - Issues with grammar and structure
3. **Semantic Errors** - Issues with meaning and validation
4. **Structural Errors** - Issues with nested structures and depth
5. **Strict Mode Violations** - Issues with canonical format compliance

## Error Severity Levels

- **Fatal**: Parsing cannot continue, immediate failure required
- **Recoverable**: Parser may attempt recovery or provide partial results
- **Warning**: Non-critical issue that doesn't prevent parsing

---

## 1. Lexical Errors

Errors that occur during the tokenization phase, before syntactic analysis.

### 1.1 InvalidCharacter

**Severity**: Fatal

**Description**: An invalid or unexpected character was encountered in the input stream.

**When it occurs**:
- Non-ASCII characters in field IDs
- Invalid characters in type hints
- Malformed escape sequences

**Example**:
```
F12@=value
    ^ Invalid character '@' in field definition
```

**Error Fields**:
- `char`: The invalid character
- `line`: Line number
- `column`: Column number

---

### 1.2 UnterminatedString

**Severity**: Fatal

**Description**: A quoted string was opened but never closed before EOF or newline.

**When it occurs**:
- Missing closing quote in string value
- Newline inside quoted string without escape

**Example**:
```
F12="unterminated string
                        ^ Missing closing quote
```

**Error Fields**:
- `line`: Line where string started
- `column`: Column where string started

---

### 1.3 InvalidEscapeSequence

**Severity**: Fatal

**Description**: An invalid escape sequence was encountered in a quoted string.

**When it occurs**:
- Unknown escape character (e.g., `\x`, `\u`)
- Incomplete escape sequence at end of string

**Example**:
```
F12="invalid \x escape"
             ^^ Invalid escape sequence
```

**Error Fields**:
- `sequence`: The invalid escape sequence
- `line`: Line number
- `column`: Column number

**Valid escape sequences**: `\\`, `\"`, `\n`, `\r`, `\t`

---

## 2. Syntactic Errors

Errors related to grammar violations and structural syntax.

### 2.1 UnexpectedToken

**Severity**: Fatal

**Description**: A token was encountered that doesn't match the expected grammar at this position.

**When it occurs**:
- Missing equals sign after field ID
- Missing semicolon between fields (in strict mode)
- Unexpected token in nested structure

**Example**:
```
F12:i 14532
      ^ Expected '=', found number
```

**Error Fields**:
- `expected`: Description of expected token(s)
- `found`: The actual token encountered
- `line`: Line number
- `column`: Column number

---

### 2.2 UnexpectedEof

**Severity**: Fatal

**Description**: End of file was reached while expecting more input.

**When it occurs**:
- Incomplete field definition
- Unclosed nested record or array
- Missing value after equals sign

**Example**:
```
F12={F7=1;F8=
            ^ Unexpected end of file
```

**Error Fields**:
- `line`: Line where EOF occurred
- `column`: Column where EOF occurred

---

### 2.3 InvalidFieldId

**Severity**: Fatal

**Description**: Field ID is not a valid u16 integer or is out of range.

**When it occurs**:
- Field ID exceeds 65535
- Field ID contains non-numeric characters
- Field ID is negative

**Example**:
```
F99999=value
 ^^^^^ Field ID exceeds maximum value 65535
```

**Error Fields**:
- `value`: The invalid field ID string
- `line`: Line number
- `column`: Column number

**Valid range**: 0-65535 (u16)

---

### 2.4 InvalidTypeHint

**Severity**: Fatal

**Description**: Type hint is not a recognized type code.

**When it occurs**:
- Unknown type hint character
- Malformed type hint syntax

**Example**:
```
F12:x=value
    ^ Invalid type hint 'x'
```

**Error Fields**:
- `hint`: The invalid type hint
- `line`: Line number
- `column`: Column number

**Valid type hints**: `i`, `f`, `b`, `s`, `sa`, `r`, `ra`

---

## 3. Semantic Errors

Errors related to type validation, checksums, and semantic correctness.

### 3.1 TypeHintMismatch

**Severity**: Fatal

**Description**: The parsed value doesn't match the declared type hint.

**When it occurs**:
- Type hint says integer but value is not numeric
- Type hint says boolean but value is not 0/1
- Type hint says array but value is scalar

**Example**:
```
F12:i=not_a_number
      ^^^^^^^^^^^^ Expected integer, got string
```

**Error Fields**:
- `field_id`: The field ID
- `expected_type`: Type from hint
- `actual_value`: The value that was parsed
- `line`: Line number
- `column`: Column number

---

### 3.2 ChecksumMismatch

**Severity**: Fatal (when validation enabled)

**Description**: The computed semantic checksum doesn't match the provided checksum.

**When it occurs**:
- Value was modified but checksum not updated
- Checksum computed with different normalization rules
- Transmission error corrupted value or checksum

**Example**:
```
F12:i=14532#DEADBEEF
            ^^^^^^^^ Expected 6A93B3F1, found DEADBEEF
```

**Error Fields**:
- `field_id`: The field ID
- `expected`: Computed checksum (hex string)
- `found`: Provided checksum (hex string)
- `line`: Line number
- `column`: Column number

**Note**: Checksum validation is optional and configurable.

---

### 3.3 InvalidValue

**Severity**: Fatal

**Description**: Value is syntactically valid but semantically invalid for the field.

**When it occurs**:
- Integer overflow/underflow
- Float parsing error
- Invalid boolean representation (when strict)
- Empty required field

**Example**:
```
F12:i=999999999999999999999
      ^^^^^^^^^^^^^^^^^^^^^ Integer overflow
```

**Error Fields**:
- `field_id`: The field ID
- `reason`: Description of why value is invalid
- `line`: Line number
- `column`: Column number

---

## 4. Structural Errors

Errors specific to nested structures and depth limits.

### 4.1 NestingTooDeep

**Severity**: Fatal

**Description**: Nested structure exceeds maximum allowed depth.

**When it occurs**:
- Recursive nesting beyond configured limit
- Deeply nested records or arrays

**Example**:
```
F1={F2={F3={F4={F5={F6={F7={F8={F9={F10={F11=1}}}}}}}}}}
                                                    ^ Nesting exceeds maximum depth
```

**Error Fields**:
- `max_depth`: Configured maximum depth
- `actual_depth`: Depth that was reached
- `line`: Line number
- `column`: Column number

**Default limit**: 10 levels

---

### 4.2 InvalidNestedStructure

**Severity**: Fatal

**Description**: Nested structure syntax is malformed or invalid.

**When it occurs**:
- Mismatched braces or brackets
- Invalid separator in nested context
- Empty nested record (when not allowed)
- Mixed array element types

**Example**:
```
F50={F12=1,F7=1}
           ^ Invalid separator ',' in nested record (expected ';')
```

**Error Fields**:
- `reason`: Description of structural issue
- `line`: Line number
- `column`: Column number

---

### 4.3 UnclosedNestedStructure

**Severity**: Fatal

**Description**: Nested record or array was opened but not properly closed.

**When it occurs**:
- Missing closing brace `}`
- Missing closing bracket `]`
- EOF reached inside nested structure

**Example**:
```
F50={F12=1;F7=1
                ^ Missing closing brace
```

**Error Fields**:
- `structure_type`: "record" or "array"
- `opened_at_line`: Line where structure opened
- `opened_at_column`: Column where structure opened
- `line`: Line where error detected
- `column`: Column where error detected

---

## 5. Strict Mode Violations

Errors that occur only when strict mode is enabled, enforcing canonical format.

### 5.1 StrictModeViolation

**Severity**: Fatal (in strict mode), Warning (otherwise)

**Description**: Input doesn't conform to canonical LNMP format.

**When it occurs**:
- Fields not sorted by Field ID
- Inconsistent whitespace
- Missing semicolon separators
- Non-canonical value representation

**Example**:
```
F12=1;F7=1
      ^^^ Fields not sorted by ID (F7 should come before F12)
```

**Error Fields**:
- `reason`: Description of violation
- `line`: Line number
- `column`: Column number

**Common violations**:
- Unsorted fields
- Trailing semicolons
- Extra whitespace
- Unquoted strings that should be quoted

---

## Error Context and Reporting

### ErrorContext Structure

All errors should include rich context for debugging:

```rust
pub struct ErrorContext {
    pub line: usize,
    pub column: usize,
    pub source_snippet: String,  // 3 lines of context
    pub source_file: Option<String>,
}
```

### Rich Error Formatting

Errors should be formatted with source context when available:

```
Error: Checksum mismatch at line 2, column 15
  |
1 | F7:b=1
2 | F12:i=14532#DEADBEEF
  |               ^^^^^^^^ expected 6A93B3F1, found DEADBEEF
3 | F23:sa=[admin,dev]
  |
```

### Error Recovery

Implementations MAY attempt error recovery for:
- Missing semicolons (insert implicit separator)
- Unsorted fields (reorder during parsing)
- Extra whitespace (normalize)

Implementations MUST NOT recover from:
- Checksum mismatches
- Type hint mismatches
- Invalid nested structures
- Nesting depth violations

---

## Implementation Guidelines

### Error Propagation

1. **Fail Fast**: Lexical and syntactic errors should stop parsing immediately
2. **Collect Semantic Errors**: Implementations MAY collect multiple semantic errors before failing
3. **Provide Context**: Always include line/column and source snippet when available
4. **Clear Messages**: Error messages should be actionable and specific

### Error Testing

All error classes must be covered by:
1. Unit tests for error construction and formatting
2. Integration tests for error detection during parsing
3. Compliance tests for cross-language consistency

### Error Codes

Implementations MAY assign numeric error codes for programmatic handling:

| Code | Error Class | Category |
|------|-------------|----------|
| 1000-1099 | Lexical errors | Tokenization |
| 2000-2099 | Syntactic errors | Grammar |
| 3000-3099 | Semantic errors | Validation |
| 4000-4099 | Structural errors | Nesting |
| 5000-5099 | Strict mode violations | Canonicalization |

---

## Version History

- **v0.3**: Added `ChecksumMismatch`, `NestingTooDeep`, `InvalidNestedStructure`, `UnclosedNestedStructure`
- **v0.2**: Added `TypeHintMismatch`, `StrictModeViolation`
- **v0.1**: Initial error classification

---

## See Also

- [LNMP Grammar Specification](grammar.md)
- [LNMP v0.3 Design Document](../.kiro/specs/lnmp-v0.3-semantic-fidelity/design.md)
- [Compliance Test Suite](../tests/compliance/)
