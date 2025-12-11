# LNMP Container Format (.lnmp)

This RFC locks the `.lnmp` container v1 file layout. A single file extension fronts all LNMP transport modes while the header selects the decoding path.

## File Identity
- Extension: `.lnmp`
- Magic: ASCII `LNMP` (`0x4C 0x4E 0x4D 0x50`)
- Version: `0x01` (v1 of the container header)
- Endianness: all multi-byte integers are big-endian
- MIME (proposed): `application/x-lnmp`

## Design Rules
1) **Single artifact** – only `.lnmp` is exposed; the mode byte and metadata guide decoding.  
2) **Header first** – magic + version + mode + flags + metadata length is always 12 bytes.  
3) **Extensible metadata** – the header declares how many metadata bytes follow. Parsers must skip bytes they do not understand.  
4) **Forward-compatibility** – unknown modes/flags fail fast; unknown metadata is ignored after validation of length.

## Header Layout (12 bytes)

| Offset | Length | Field           | Description |
|--------|--------|-----------------|-------------|
| 0      | 4      | Magic           | ASCII `LNMP` |
| 4      | 1      | Version         | `0x01` (required by this spec) |
| 5      | 1      | Mode            | `0x01` Text, `0x02` Binary, `0x03` Stream, `0x04` Delta, `0x05` Quantum-Safe (reserved) |
| 6      | 2      | Flags           | Bitfield, see table below |
| 8      | 4      | Metadata length | Length in bytes of the metadata block that immediately follows the header |

`metadata_length = 0` is allowed. The payload starts immediately after the metadata segment.

## Modes
- **LNMP/Text (0x01)** – Payload is UTF-8 LNMP text. Metadata is typically empty; optional keys can express charset/newline policies.  
- **LNMP/Binary (0x02)** – Payload is `lnmp-codec` binary encoding. Metadata may advertise variant/depth limits.  
- **LNMP/Stream (0x03)** – Payload is a stream chunk sequence. Metadata is **REQUIRED** to convey chunk policy and checksum type.  
- **LNMP/Delta (0x04)** – Payload is a delta operation set. Metadata is **REQUIRED** to declare the base snapshot and algorithm.  
- **LNMP/Quantum-Safe (0x05)** – Reserved for PQ-safe payloads; semantics follow a future RFC.

## Flags (u16)

| Bit | Name                  | Meaning (v1) |
|-----|-----------------------|--------------|
| 0   | `checksum`            | Payload carries checksums (e.g., SC32). In v1 this is a hint only; set when checksums are present. |
| 1   | `compressed`          | Reserved; MUST be `0` in v1. |
| 2   | `encrypted`           | Reserved; MUST be `0` in v1. |
| 3   | `qsig`                | Reserved for PQ signatures; MUST be `0` in v1. |
| 4   | `qkex`                | Reserved for PQ key exchange; MUST be `0` in v1. |
| 5-14| reserved              | MUST be `0` in v1. |
| 15  | `ext_meta_block` (TBD)| Reserved for signaling a metadata extension TLV chain after fixed metadata. MUST be `0` in v1; future RFC will define semantics. |

Consumers MUST reject containers that set reserved bits. Producers MUST set `metadata_length > 0` whenever flags imply additional material (e.g., quantum flags in the future).

## Metadata Layouts

Mode metadata blocks begin immediately after the 12-byte header. All integers are big-endian.

### Stream Metadata (`mode = 0x03`, length = 6 bytes)

| Offset | Field           | Type | Description |
|--------|-----------------|------|-------------|
| 0      | `chunk_size`    | u32  | Preferred chunk size in bytes (SHOULD be honored). |
| 4      | `checksum_type` | u8   | `0x00` none, `0x01` XOR32, `0x02` SC32. Unknown values MUST be tolerated but may downgrade checks. |
| 5      | `flags`         | u8   | Bit 0: compression hint, Bit 1: ACK required, others reserved (`0`). |

### Delta Metadata (`mode = 0x04`, length = 10 bytes)

| Offset | Field           | Type | Description |
|--------|-----------------|------|-------------|
| 0      | `base_snapshot` | u64  | Snapshot identifier this delta applies to. |
| 8      | `algorithm`     | u8   | `0x00` op-list (default), `0x01` merge; others reserved. |
| 9      | `compression`   | u8   | `0x00` none, `0x01` varint-packed payload; others reserved. |

Producers MUST emit these metadata blocks for Stream and Delta modes. Consumers MUST validate length before parsing.

## Parsing & Validation
- Always validate magic and version before mode dispatch.  
- Reject unknown versions or modes.  
- Verify `metadata_length` does not exceed available bytes.  
- Reserved-flag violations are errors in v1.  
- After metadata is sliced, payload handling is delegated to the mode-specific codec.

## Minimum Interoperable Subset (v1)
- Stream: `metadata_length = 6`, `chunk_size > 0`, reserved bits in `flags` MUST be zero, and `checksum_type` MAY be ignored if unknown but must not break decoding.  
- Delta: `metadata_length = 10`, `base_snapshot` is required (non-zero recommended), reserved bytes MUST be zero, and `algorithm`/`compression` MUST be in the allowed set (`algorithm` = 0x00/0x01, `compression` = 0x00/0x01); other codes are errors.  
- Flags: only `checksum` is meaningful in v1; all other bits MUST be zero.  
- Payload: parsers MUST reject files whose metadata length overflows or runs past the buffer.  
- Unknown metadata bytes beyond the defined fields are tolerated only if covered by `metadata_length` and non-reserved.

## Conformance Fixture Ideas (Phase 2)
- Valid: text with checksum flag, binary with 16B metadata, stream with SC32 and 4KiB chunks, delta with base snapshot and op-list algorithm.  
- Invalid: wrong magic, unsupported version, unknown mode, reserved flag set, truncated metadata, zero-length metadata in stream/delta, out-of-range metadata length.

Reference fixtures for the valid headers above now live under `spec/examples/container/` and are automatically checked by `lnmp-verify-examples` so that documentation and parser behavior never drifts. The same directory contains `invalid_*.hex` fixtures that exercise each error in the mapping below; the verifier expects them to fail with the documented reason, keeping the spec and parser diagnostics in lockstep.

### Where to Find the Fixtures/Tests
- **Container header & metadata:** `crates/lnmp-codec/tests/container_conformance.rs`
- **Streaming layer:** `crates/lnmp-codec/tests/streaming_layer_tests.rs`
- **Delta encoding:** `crates/lnmp-codec/tests/delta_encoding_tests.rs`
- **Binary payload examples:** `crates/lnmp-codec/tests/binary_*`
- **Transport header mappings:** `crates/lnmp-transport/tests/integration_tests.rs`

> When updating this spec, keep the referenced fixtures green or update them alongside the change.

## Header Error Mapping (reference)

When using the published fixtures, parsers should surface the following errors:
- `invalid-bad-magic.lnmp` → `InvalidMagic`
- `invalid-version-ff.lnmp` → `UnsupportedVersion(0xFF)`
- `invalid-mode-ff.lnmp` → `UnknownMode(0xFF)`
- `invalid-reserved-flag.lnmp` → `ReservedFlags`
- `invalid-stream-no-meta.lnmp` → `InvalidMetadataLength` (expected 6, actual 0)
- `invalid-delta-no-meta.lnmp` → `InvalidMetadataLength` (expected 10, actual 0)
- `invalid-meta-overflow.lnmp` → `TruncatedMetadata`
- `invalid-delta-unknown-algorithm.lnmp` → `InvalidMetadataValue` (algorithm)
- `invalid-delta-unknown-compression.lnmp` → `InvalidMetadataValue` (compression)

## Examples

Text mode with checksum flag:
```
4C 4E 4D 50 01 01 00 01 00 00 00 00
```

Binary mode with 16-byte metadata:
```
4C 4E 4D 50 01 02 00 00 00 00 00 10
```

Stream mode with 6-byte metadata (chunk=4096, SC32, no flags):
```
4C 4E 4D 50 01 03 00 00 00 00 00 06
00 00 10 00 02 00
```

## Versioning & Compatibility
- Header version is bumped only when the header layout changes; mode evolution happens inside metadata.  
- New modes or metadata fields require a recorded update in `CHANGELOG` and the corresponding RFC.  
- Unknown metadata bytes are skipped using the advertised length; only known subfields influence behavior.  
- **Freeze policy (v1):** Header/mode bytes and the stream/delta metadata layouts above are frozen for v1. Any change to magic/version/mode/flags/layouts requires a version bump and a new RFC. Bug fixes must stay within these fields and container tests/fixtures must remain green.

This document is the authoritative definition of `.lnmp` container v1. Implementations must conform to stay compatible.

## Forward-Looking: Metadata Extension Block (planned)
- Not part of v1. Proposal: after the fixed mode metadata, allow an optional extension chain encoded as TLVs (`type u8`, `length u16|u32`, `value`), guarded by a reserved header flag.  
- Flag reservation: header flag bit 15 (`ext_meta_block`) is earmarked to signal presence of this chain but MUST stay `0` in v1.  
- Unknown TLVs must be skipped using length; known TLVs can describe future algorithms, checksums, or encryption parameters without altering the fixed v1 metadata.  
- Enabling this requires a version bump or an agreed flag+RFC; current v1 conformance remains strict (fixed lengths for stream/delta).  
- Registry frozen (inactive): `spec/lnmp-metadata-extension-rfc.md` defines the TLV layout and registry codes (0x01 checksum, 0x02 encryption, 0x03 signature, 0x7F vendor). Activation still requires a version/flag bump.
