# LNMP Envelope Specification v1.0

**Status:** Draft  
**Version:** 1.0  
**Date:** 2024-11-23

## Abstract

This specification defines the **LNMP Envelope** format, which adds operational context metadata (timestamp, source, tracing, sequencing) to LNMP records without affecting core determinism. The design aligns with industry standards including CloudEvents, Kafka Headers, and W3C Trace Context.

## 1. Introduction

### 1.1 Purpose

LNMP Core provides deterministic, semantic data representation but lacks operational context:
- **What happened?** ✅ (LnmpRecord)
- **When did it happen?** ❌
- **Where did it come from?** ❌
- **How does it fit in a request chain?** ❌

LNMP Envelope fills this gap while preserving core guarantees.

### 1.2 Design Goals

1. **Preserve Core Determinism**: Envelope does NOT affect `SemanticChecksum` or canonical ordering
2. **Zero Overhead**: Unused envelope features have no performance cost
3. **Standards Aligned**: Compatible with CloudEvents, Kafka, OpenTelemetry
4. **Transport Agnostic**: Defined independently, bindings provided separately
5. **Future Proof**: Extensible without breaking changes

### 1.3 Non-Goals

- Application-level LLM scoring algorithms
- Multi-tenant authorization logic
- Automatic metadata injection
- Event sourcing engine

## 2. Terminology

- **Record**: Core LNMP data structure (`LnmpRecord`)
- **Envelope**: Wrapper providing operational metadata
- **Metadata**: Operational context fields (timestamp, source, etc.)
- **Container**: File-level LNMP wrapper (`.lnmp` format)
- **TLV**: Type-Length-Value encoding

## 3. Data Model

### 3.1 Envelope Metadata Fields

| Field       | Type              | Required | Purpose                           |
|-------------|-------------------|----------|-----------------------------------|
| `timestamp` | `u64`             | No       | Event time (Unix epoch ms, UTC)   |
| `source`    | `String`          | No       | Service/device/tenant identifier  |
| `trace_id`  | `String`          | No       | Distributed tracing correlation   |
| `sequence`  | `u64`             | No       | Monotonic version number          |
| `labels`    | `Map<String, String>` | No   | Extensibility (future)            |

**Constraints:**
- `source`: SHOULD be ≤ 64 characters
- `trace_id`: SHOULD be ≤ 128 characters, MAY follow W3C Trace Context format
- `sequence`: MUST be monotonically increasing for given entity
- `labels`: Reserved for future use, implementations MAY ignore

### 3.2 Envelope Structure

An envelope consists of:
1. **Metadata Block** (optional fields)
2. **Record Payload** (mandatory `LnmpRecord`)

**Invariant:** The record payload's `SemanticChecksum` is computed independently of envelope metadata.

## 4. Binary Encoding

### 4.1 Container Integration

Envelope metadata uses the **Metadata Extension Block** mechanism defined in `spec/lnmp-metadata-extension-rfc.md`.

**Container Header:**
```
Offset  Size  Field
0       4     Magic: "LNMP"
4       1     Version: 0x01
5       1     Mode: Binary/Text/Stream/Delta
6       2     Flags (bit 15 = has_ext_metadata)
8       4     metadata_length (big-endian u32)
```

When `flags` bit 15 is set:
- Metadata block contains TLV chain
- First entries MAY be mode-specific (Stream/Delta)
- Envelope entries follow

### 4.2 TLV Encoding

Each metadata entry:
```
Type (1 byte) | Length (2 bytes, BE) | Value (Length bytes)
```

**Type Codes:**

| Type   | Name       | Value Format           |
|--------|------------|------------------------|
| `0x10` | Timestamp  | u64 big-endian         |
| `0x11` | Source     | UTF-8 string           |
| `0x12` | TraceID    | UTF-8 string           |
| `0x13` | Sequence   | u64 big-endian         |
| `0x14` | Label      | (Reserved)             |

**Encoding Rules:**
1. Entries MUST appear in ascending type order
2. Each type MUST appear at most once
3. Unknown types MUST be skipped using length field

**Example:**
```
Type: 0x10, Length: 8, Value: 0x0000018C9A3B2F28  (timestamp)
Type: 0x11, Length: 12, Value: "auth-service"    (source)
Type: 0x12, Length: 11, Value: "trace-abc-1"     (trace_id)
```

### 4.3 Decoding Algorithm

```
while bytes_remaining > 0:
    type = read_u8()
    length = read_u16_be()
    
    if type == 0x10:  # Timestamp
        if length != 8: error
        timestamp = read_u64_be()
    elif type == 0x11:  # Source
        source = read_string(length)
    elif type == 0x12:  # TraceID
        trace_id = read_string(length)
    elif type == 0x13:  # Sequence
        if length != 8: error
        sequence = read_u64_be()
    else:
        skip(length)  # Unknown type, forward compatible
```

## 5. Text Encoding

### 5.1 Header Comment Format

Envelope metadata encoded as first line starting with `#ENVELOPE`:

```
#ENVELOPE timestamp=<value> source=<value> trace_id=<value> sequence=<value>
<LNMP record follows>
```

**Syntax:**
- `#ENVELOPE` keyword required
- Space-separated `key=value` pairs
- Values without spaces unquoted, otherwise double-quoted
- Unknown keys SHOULD be ignored

**Example:**
```
#ENVELOPE timestamp=1732373147000 source=auth-service trace_id="abc-123-xyz" sequence=42
F12=14532
F7=1
```

### 5.2 Parsing Rules

**Mandatory Behavior:**
- Parser MUST accept records without `#ENVELOPE` line (backward compatible)
- If present, `#ENVELOPE` MUST be first line
- Malformed envelope in strict mode: error
- Malformed envelope in loose mode: ignore and parse record

**Optional Fields:**
- All fields optional, any subset valid
- Order of key=value pairs does not matter

### 5.3 Canonical Text Form

For deterministic output, encoder MUST:
1. Place `#ENVELOPE` on first line (if metadata present)
2. Order keys: `timestamp`, `source`, `trace_id`, `sequence`
3. Use minimal quoting (quote only if value contains spaces)

## 6. Determinism Guarantee

### 6.1 Checksum Independence

**Invariant:**
```
SemanticChecksum(Record) = f(Record.fields)
```

Envelope metadata is **NOT** included in checksum computation.

**Verification:**
```rust
let record = /* ... */;
let cs1 = SemanticChecksum::compute_record(&record);

let envelope = EnvelopeBuilder::new(record.clone())
    .timestamp(123456789)
    .source("test")
    .build();

let cs2 = SemanticChecksum::compute_record(&envelope.record);

assert_eq!(cs1, cs2);  // MUST pass
```

### 6.2 Canonical Ordering

Envelope encoding (both binary and text) MUST be deterministic:
- Binary: TLV entries in type order
- Text: Keys in specified order

**Test:**
```rust
let env1 = EnvelopeBuilder::new(record.clone())
    .timestamp(123)
    .source("srv")
    .build();

let env2 = EnvelopeBuilder::new(record.clone())
    .source("srv")    // Note: different construction order
    .timestamp(123)
    .build();

let bin1 = encode_binary(&env1);
let bin2 = encode_binary(&env2);

assert_eq!(bin1, bin2);  // MUST pass
```

## 7. Transport Bindings (Informative)

### 7.1 HTTP

**Request Headers:**
```http
X-LNMP-Timestamp: 1732373147000
X-LNMP-Source: auth-service
X-LNMP-Trace-ID: abc-123
X-LNMP-Sequence: 42
```

**Rationale:** Standard `X-` prefix for custom headers, kebab-case naming

### 7.2 Kafka

**Record Headers:**
```
lnmp.timestamp: "1732373147000"
lnmp.source: "auth-service"
lnmp.trace_id: "abc-123"
lnmp.sequence: "42"
```

**Rationale:** Follows Kafka header conventions, string values for interoperability

### 7.3 gRPC

**Metadata:**
```
lnmp-timestamp: "1732373147000"
lnmp-source: "auth-service"
lnmp-trace-id: "abc-123"
lnmp-sequence: "42"
```

**Rationale:** gRPC metadata keys lowercase, ASCII only

## 8. Standards Alignment

### 8.1 CloudEvents Mapping

| CloudEvents     | LNMP Envelope   | Notes                          |
|-----------------|-----------------|--------------------------------|
| `time`          | `timestamp`     | Both represent event time      |
| `source`        | `source`        | Identifying the origin         |
| `id`            | `trace_id` + `sequence` | Combined unique ID  |
| `type`          | (FID-based)     | LNMP uses field IDs for typing |
| `data`          | `record`        | Event payload                  |

### 8.2 W3C Trace Context

LNMP `trace_id` field CAN hold:
- Full `traceparent` value: `00-<trace-id>-<span-id>-<flags>`
- Or just `<trace-id>` portion

Implementations SHOULD preserve trace context from OpenTelemetry SDK.

### 8.3 Kafka Headers

LNMP envelope aligns with Kafka best practices:
- Metadata separate from payload
- Headers for routing/filtering
- Timestamp semantics match Kafka timestamp

## 9. Security Considerations

### 9.1 Metadata Injection

Applications MUST validate metadata from untrusted sources:
- Timestamp: Check for future dates, unrealistic values
- Source: Validate against known service identifiers
- Trace ID: Sanitize for length, character set

### 9.2 Information Disclosure

Envelope metadata MAY contain sensitive information:
- `source` might reveal internal topology
- `trace_id` could be used for tracking

Implementations SHOULD provide filtering/redaction mechanisms.

## 10. Extensibility

### 10.1 Future Fields

New fields added via:
1. Binary: New TLV type codes (0x15+)
2. Text: New keys in `#ENVELOPE` line

Decoders MUST skip unknown types/keys.

### 10.2 Labels (Reserved)

Type code `0x14` reserved for key-value labels. Future spec will define:
- Encoding format
- Common label keys (tenant, environment, region, priority)
- Label validation rules

## 11. Conformance

An implementation is conformant if:
1. ✅ Preserves core determinism (checksum independence)
2. ✅ Encodes/decodes binary TLV format correctly
3. ✅ Encodes/decodes text header format correctly
4. ✅ Skips unknown types/keys gracefully
5. ✅ Produces deterministic encoding (canonical ordering)
6. ✅ Zero overhead when envelope unused (optional test)

**Test Suite:** `tests/compliance/envelope/`

## 12. References

- [CloudEvents Spec v1.0](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [Kafka Headers](https://kafka.apache.org/documentation/#recordheader)
- [LNMP Core Spec](./lnmp-v0.5-spec.md)
- [LNMP Container Format](./lnmp-container-format.md)
- [LNMP Metadata Extension RFC](./lnmp-metadata-extension-rfc.md)

---

**Appendix A: Complete Binary Example**

```
# Container Header
4C 4E 4D 50           # Magic "LNMP"
01                    # Version 0x01
02                    # Mode: Binary
80 00                 # Flags: bit 15 set (has extension)
00 00 00 1F           # metadata_length = 31 bytes

# Metadata TLV Chain
10 00 08              # Type 0x10 (Timestamp), Length 8
00 00 01 8C 9A 3B 2F 28   # Value: 1732373147000

11 00 0C              # Type 0x11 (Source), Length 12
61 75 74 68 2D 73 65 72 76 69 63 65   # "auth-service"

12 00 09              # Type 0x12 (TraceID), Length 9
61 62 63 2D 31 32 33 2D 78   # "abc-123-x"

# Record Payload
<binary LNMP record follows>
```

**Appendix B: Complete Text Example**

```
#ENVELOPE timestamp=1732373147000 source=auth-service trace_id="abc-123-xyz" sequence=42
F12=14532
F7=1
F20="Alice"
F23=["admin","developer"]
```

---

## Change Log

- **2024-11-23**: Initial draft v1.0
