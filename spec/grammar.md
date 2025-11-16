# LNMP v0.3 Formal Grammar Specification

## Overview

This document provides the formal grammar specification for LNMP (LLM Native Minimal Protocol) version 0.3. The grammar is defined using PEG (Parsing Expression Grammar) notation with Pest syntax, ensuring zero-ambiguity parsing across all implementations.

## Grammar Goals

1. **Zero Ambiguity**: Every valid input has exactly one parse tree
2. **Deterministic**: Parsing behavior is identical across all implementations
3. **Complete**: All valid LNMP v0.3 syntax is covered
4. **Extensible**: Grammar can accommodate future extensions

## PEG Grammar (Pest Syntax)

```pest
// ============================================================================
// Top-Level Rules
// ============================================================================

/// A complete LNMP record
lnmp_record = { SOI ~ field_list? ~ EOI }

/// List of fields separated by semicolons or newlines
field_list = { field ~ (separator ~ field)* ~ separator? }

/// A single field assignment
field = { field_prefix ~ field_id ~ type_hint? ~ equals ~ value ~ checksum? }

// ============================================================================
// Field Components
// ============================================================================

/// Field prefix (always "F")
field_prefix = { "F" }

/// Field identifier (0-65535)
field_id = @{ ASCII_DIGIT+ }

/// Optional type hint
type_hint = { ":" ~ type_code }

/// Type codes
type_code = { 
    "ra"  // Record array (must come before "r" to avoid ambiguity)
  | "sa"  // String array (must come before "s")
  | "i"   // Integer
  | "f"   // Float
  | "b"   // Boolean
  | "s"   // String
  | "r"   // Record
}

/// Assignment operator
equals = { "=" }

// ============================================================================
// Value Types
// ============================================================================

/// Any value type
value = {
    nested_record
  | nested_array
  | string_array
  | boolean
  | number
  | string
}

// ============================================================================
// Nested Structures
// ============================================================================

/// Nested record: {F1=value;F2=value}
nested_record = { "{" ~ nested_field_list? ~ "}" }

/// Field list within nested record (semicolon-separated only)
nested_field_list = { nested_field ~ (";" ~ nested_field)* ~ ";"? }

/// Field within nested record
nested_field = { field_prefix ~ field_id ~ type_hint? ~ equals ~ value ~ checksum? }

/// Nested array of records: [{F1=v},{F2=v}]
nested_array = { "[" ~ record_list? ~ "]" }

/// List of records within array
record_list = { nested_record ~ ("," ~ nested_record)* }

// ============================================================================
// Primitive Types
// ============================================================================

/// String array: [str1,str2,str3]
string_array = { "[" ~ string_list? ~ "]" }

/// List of strings
string_list = { string ~ ("," ~ string)* }

/// String value (quoted or unquoted)
string = { quoted_string | unquoted_string }

/// Quoted string with escape sequences
quoted_string = @{ "\"" ~ quoted_string_inner ~ "\"" }
quoted_string_inner = @{ (escape_sequence | !("\"" | "\\") ~ ANY)* }

/// Unquoted string (alphanumeric, underscore, hyphen, dot)
unquoted_string = @{ (ASCII_ALPHANUMERIC | "_" | "-" | ".")+ }

/// Numeric value (integer or float)
number = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }

/// Boolean value (0 or 1)
boolean = @{ "0" | "1" }

// ============================================================================
// Escape Sequences
// ============================================================================

/// Supported escape sequences in quoted strings
escape_sequence = @{
    "\\\\"   // Backslash
  | "\\\""   // Quote
  | "\\n"    // Newline
  | "\\r"    // Carriage return
  | "\\t"    // Tab
}

// ============================================================================
// Checksum
// ============================================================================

/// Semantic checksum suffix: #XXXXXXXX
checksum = @{ "#" ~ ASCII_HEX_DIGIT{8} }

// ============================================================================
// Separators and Whitespace
// ============================================================================

/// Field separator (semicolon or newline)
separator = _{ ";" | NEWLINE }

/// Whitespace (spaces and tabs, ignored)
WHITESPACE = _{ " " | "\t" }

/// Comments (# followed by text until newline, ignored)
COMMENT = _{ "#" ~ (!NEWLINE ~ ANY)* }
```

## Token Classification

### Terminal Tokens

| Token | Pattern | Description | Example |
|-------|---------|-------------|---------|
| `field_prefix` | `F` | Field prefix marker | `F` |
| `field_id` | `[0-9]+` | Field identifier (0-65535) | `12`, `1024` |
| `type_code` | `i\|f\|b\|s\|sa\|r\|ra` | Type hint code | `:i`, `:sa` |
| `equals` | `=` | Assignment operator | `=` |
| `number` | `-?[0-9]+(\.[0-9]+)?` | Numeric literal | `42`, `-3.14` |
| `boolean` | `0\|1` | Boolean literal | `0`, `1` |
| `unquoted_string` | `[A-Za-z0-9_.-]+` | Unquoted string | `admin`, `user_1` |
| `quoted_string` | `"([^"\\]|\\[\\\"nrt])*"` | Quoted string | `"hello"`, `"line\n"` |
| `checksum` | `#[0-9A-Fa-f]{8}` | Semantic checksum | `#6A93B3F1` |
| `separator` | `;\|\n` | Field separator | `;`, newline |

### Non-Terminal Tokens

| Token | Description |
|-------|-------------|
| `lnmp_record` | Complete LNMP record |
| `field_list` | List of fields |
| `field` | Single field assignment |
| `value` | Any value type |
| `nested_record` | Nested record structure |
| `nested_array` | Array of nested records |
| `string_array` | Array of strings |

## Precedence and Associativity

### Value Type Precedence (Highest to Lowest)

1. **Nested Record** (`{...}`) - Highest priority
2. **Nested Array** (`[{...}]`)
3. **String Array** (`[...]`)
4. **Boolean** (`0` or `1`)
5. **Number** (`-?[0-9]+(\.[0-9]+)?`)
6. **String** (quoted or unquoted) - Lowest priority

### Parsing Strategy

PEG uses **ordered choice**, meaning the first matching alternative is selected. The grammar is structured to avoid ambiguity:

- `nested_record` is tried before `string_array` (both start with brackets)
- `nested_array` contains records, so `[{` is unambiguous
- `boolean` is tried before `number` to match `0` and `1` as booleans
- Type hints disambiguate when needed (e.g., `:b` forces boolean interpretation)

## Nested Structure Syntax

### Nested Records

**Syntax**: `F<fid>={F<fid>=<value>;F<fid>=<value>}`

**Rules**:
1. Opening brace `{` starts nested record context
2. Fields within nested record use semicolon separators (required)
3. Fields are sorted by FID during encoding (canonical form)
4. Closing brace `}` ends nested record context
5. Arbitrary nesting depth supported

**Examples**:
```
F50={F12=1;F7=1}
F100={F1=user;F2={F10=nested;F11=data}}
F200={F1=alice;F2={F10=admin;F11={F20=superuser}}}
```

### Nested Arrays

**Syntax**: `F<fid>=[{F<fid>=<value>},{F<fid>=<value>}]`

**Rules**:
1. Opening bracket `[` starts array context
2. Each element is a complete record in `{...}` format
3. Elements separated by commas
4. Closing bracket `]` ends array context
5. Empty arrays allowed: `F60=[]`

**Examples**:
```
F60=[{F12=1},{F12=2},{F12=3}]
F200=[{F1=alice;F2=admin},{F1=bob;F2=user}]
F300=[]
```

### Mixed Nesting

Nested records can contain nested arrays and vice versa:

```
F100={F1=users;F2=[{F10=alice},{F10=bob}]}
F200=[{F1=dept;F2={F10=eng;F11=dev}}]
```

## Checksum Syntax

### Format

**Syntax**: `#<8-hex-digits>`

**Rules**:
1. Checksum is optional and appears after the value
2. Exactly 8 hexadecimal digits (case-insensitive)
3. Computed from FID + type hint + normalized value using CRC32
4. Validation is configurable (strict mode requires validation)

**Examples**:
```
F12:i=14532#6A93B3F1
F7:b=1#A3F2B1C4
F23:sa=[admin,dev]#D4E5F6A7
```

### Checksum Computation

1. Normalize value using canonical rules
2. Serialize: `{fid}:{type_hint}:{normalized_value}`
3. Compute CRC32 hash
4. Format as 8-character uppercase hex string

**Example**:
```
Field: F12:i=14532
Serialized: "12:i:14532"
CRC32: 0x6A93B3F1
Output: F12:i=14532#6A93B3F1
```

## Escape Sequences

### Supported Escapes in Quoted Strings

| Escape | Character | Description |
|--------|-----------|-------------|
| `\\` | `\` | Backslash |
| `\"` | `"` | Double quote |
| `\n` | LF (0x0A) | Newline |
| `\r` | CR (0x0D) | Carriage return |
| `\t` | TAB (0x09) | Horizontal tab |

**Examples**:
```
F1="hello\nworld"
F2="path\\to\\file"
F3="say \"hello\""
```

## Whitespace and Comments

### Whitespace Rules

1. **Spaces and tabs** are ignored between tokens
2. **Newlines** act as field separators (equivalent to semicolons)
3. **No whitespace** in canonical nested structures

**Examples**:
```
// Valid with whitespace
F12 = 14532 ; F7 = 1

// Canonical (no whitespace)
F12=14532;F7=1
```

### Comment Rules

1. Comments start with `#` and continue to end of line
2. Comments are ignored by parser
3. Checksums take precedence over comments (parsed first)
4. Explain mode uses comments for field annotations

**Examples**:
```
F12=14532  # user_id
F7=1       # is_active
```

**Note**: `#` followed by 8 hex digits is parsed as checksum, not comment.

## Grammar Validation Rules

### Structural Validation

1. **Field ID Range**: 0 ≤ FID ≤ 65535
2. **Type Hint Validity**: Must be one of `i`, `f`, `b`, `s`, `sa`, `r`, `ra`
3. **Nesting Depth**: Recommended maximum 10 levels
4. **Array Size**: Recommended maximum 1000 elements

### Semantic Validation

1. **Type Hint Consistency**: Value must match declared type hint
2. **Checksum Validity**: If present, must match computed checksum
3. **Field Uniqueness**: No duplicate FIDs at same nesting level
4. **Canonical Ordering**: Fields sorted by FID (in strict mode)

## Error Classification

### Lexical Errors

| Error | Description | Example |
|-------|-------------|---------|
| `InvalidCharacter` | Unexpected character in input | `F12=@invalid` |
| `UnterminatedString` | Missing closing quote | `F1="hello` |
| `InvalidEscape` | Unknown escape sequence | `F1="test\x"` |

### Syntactic Errors

| Error | Description | Example |
|-------|-------------|---------|
| `UnexpectedToken` | Token doesn't match grammar | `F12:=14532` |
| `InvalidFieldId` | FID out of range or malformed | `F99999=1` |
| `MissingEquals` | Assignment operator missing | `F12 14532` |
| `UnmatchedBrace` | Unclosed nested structure | `F50={F12=1` |

### Semantic Errors

| Error | Description | Example |
|-------|-------------|---------|
| `TypeHintMismatch` | Value doesn't match type hint | `F12:i=hello` |
| `ChecksumMismatch` | Checksum validation failed | `F12=1#DEADBEEF` |
| `DuplicateField` | Same FID appears twice | `F12=1;F12=2` |

### Structural Errors

| Error | Description | Example |
|-------|-------------|---------|
| `NestingTooDeep` | Exceeds maximum nesting depth | 15 levels deep |
| `InvalidNestedStructure` | Malformed nested syntax | `F50={F12=}` |
| `EmptyField` | Field has no value | `F12=` |

## EBNF Specification

For reference, here is the equivalent EBNF grammar:

```ebnf
lnmp_record       ::= field_list?
field_list        ::= field (separator field)* separator?
field             ::= "F" field_id type_hint? "=" value checksum?

field_id          ::= [0-9]+
type_hint         ::= ":" type_code
type_code         ::= "i" | "f" | "b" | "s" | "sa" | "r" | "ra"

value             ::= nested_record | nested_array | string_array 
                    | boolean | number | string

nested_record     ::= "{" nested_field_list? "}"
nested_field_list ::= nested_field (";" nested_field)* ";"?
nested_field      ::= "F" field_id type_hint? "=" value checksum?

nested_array      ::= "[" record_list? "]"
record_list       ::= nested_record ("," nested_record)*

string_array      ::= "[" string_list? "]"
string_list       ::= string ("," string)*

string            ::= quoted_string | unquoted_string
quoted_string     ::= '"' (escape_sequence | [^"\\])* '"'
unquoted_string   ::= [A-Za-z0-9_.-]+

number            ::= "-"? [0-9]+ ("." [0-9]+)?
boolean           ::= "0" | "1"

escape_sequence   ::= "\\" | "\"" | "\n" | "\r" | "\t"
checksum          ::= "#" [0-9A-Fa-f]{8}
separator         ::= ";" | "\n"
```

## Implementation Notes

### Parser Implementation Strategy

1. **Lexical Analysis**: Tokenize input into terminal tokens
2. **Syntactic Analysis**: Build parse tree using PEG rules
3. **Semantic Analysis**: Validate type hints, checksums, field uniqueness
4. **AST Construction**: Convert parse tree to LnmpRecord structure

### Canonical Form Requirements

For strict mode compliance, encoders must produce canonical output:

1. **Field Ordering**: Sort by FID at every nesting level
2. **Whitespace**: No spaces around operators or separators
3. **Separators**: Use semicolons (not newlines)
4. **Strings**: Use unquoted form when possible
5. **Numbers**: Remove trailing zeros from floats

**Example**:
```
// Non-canonical
F7 = 1
F12 = 14532

// Canonical
F7=1;F12=14532
```

### Grammar Extensions

Future versions may extend the grammar with:

- Additional type codes (e.g., `:d` for dates, `:u` for UUIDs)
- Nested type hints (e.g., `:sa<s>` for string array of strings)
- Compression hints (e.g., `@gzip` suffix)
- Schema references (e.g., `$schema=v0.4`)

The grammar is designed to accommodate these extensions without breaking existing parsers.

## Compliance Testing

### Grammar Validation Tests

Implementations should validate against these test cases:

1. **Basic Fields**: `F12=14532`, `F7:b=1`, `F1:s=hello`
2. **Nested Records**: `F50={F12=1;F7=1}`, `F100={F1=a;F2={F10=b}}`
3. **Nested Arrays**: `F60=[{F12=1},{F12=2}]`, `F200=[]`
4. **Checksums**: `F12:i=14532#6A93B3F1`
5. **Escape Sequences**: `F1="hello\nworld"`, `F2="path\\file"`
6. **Comments**: `F12=1 # comment`, `F7=1#A3F2B1C4`
7. **Whitespace**: `F12 = 1 ; F7 = 0`, `F12=1\nF7=0`

### Error Handling Tests

Implementations should reject these invalid inputs:

1. **Invalid Syntax**: `F12:=1`, `F=12`, `12=1`
2. **Type Mismatches**: `F12:i=hello`, `F7:b=2`
3. **Unterminated Strings**: `F1="hello`, `F2="test\`
4. **Unmatched Braces**: `F50={F12=1`, `F60=[{F12=1}`
5. **Invalid Checksums**: `F12=1#GGGGGGGG`, `F12=1#123`

## References

- **PEG Specification**: [Parsing Expression Grammars](https://en.wikipedia.org/wiki/Parsing_expression_grammar)
- **Pest Parser**: [Pest Book](https://pest.rs/book/)
- **LNMP v0.3 Design**: See `design.md` for architectural details
- **LNMP v0.3 Requirements**: See `requirements.md` for feature requirements

## Version History

- **v0.3.0** (2024): Initial formal grammar specification
  - Added nested record and array syntax
  - Added semantic checksum syntax
  - Defined complete PEG grammar with Pest syntax
  - Added EBNF specification for reference
