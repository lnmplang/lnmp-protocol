# LNMP Metadata Extension Block

**Status:** Registry frozen; activation still requires a version/flag bump (not part of v1).  
**Purpose:** Reserve a predictable channel for future metadata without changing the fixed v1 mode metadata layouts.

## Goals
- Carry future knobs (new checksum algorithms, crypto parameters, compression hints) as optional TLVs.
- Preserve the fixed v1 metadata sizes for Stream/Delta while enabling additive evolution.
- Keep forward-compat by allowing unknown TLVs to be skipped.

## Activation
- Header flag bit 15 (`ext_meta_block`) is reserved to signal the presence of an extension block that immediately follows the fixed mode metadata.
- In v1 this flag **MUST** be `0`; setting it is a conformance error. A future version/RFC will define the exact activation rules.
- Activation plan (to ratify in v1.1+): bump container version to 0x02 or formally unreserve flag bit 15; publish new fixtures where the flag is set and TLVs are present; keep v1 readers strict (reject set flag) to avoid silent downgrade.

## TLV Encoding (proposed)
- Endianness: big-endian for multi-byte fields.
- `type`: `u8` (0–255, even numbers reserved for registry, odd for vendor/experimental).
- `length`: `u16` (bytes of `value`), enabling values up to 65535 bytes.
- `value`: opaque bytes interpreted per `type`.
- Parsing rule: read TLVs until `metadata_length` is consumed. Unknown `type` values must be skipped using `length`.
- Multiple TLVs of the same `type` are allowed; last-one-wins or append semantics are left to the `type` definition.
- Ordering: SHOULD be ascending by `type` for canonicalization, but parsers must not depend on order.

### Size & DoS considerations
- Total extension chain size (sum of TLVs) SHOULD be capped to <= 64 KiB to avoid pathological inputs; implementations MAY enforce stricter limits.
- `metadata_length` still counts the fixed mode metadata plus the TLV chain; parsers must reject chains that overflow the declared length.

## Registry (frozen; inactive until flag/version bump)
- `0x01` – checksum algorithm descriptor.
- `0x02` – encryption parameters.
- `0x03` – signature parameters.
- `0x7F` – vendor/experimental (MUST NOT appear in interop fixtures).

### Proposed value layouts
These layouts are frozen with the registry; they only become active once the flag/version bump is ratified:
- `0x01 checksum descriptor`
  - `algorithm u8` (e.g., 0x01=SC32, 0x02=Blake3-256, 0x03=SHA2-256; 0xFF reserved/invalid)
  - `digest_len u8` (bytes; MUST match algo expectations)
  - `flags u8` (bit 0: truncation allowed; others reserved=0)
- `0x02 encryption parameters`
  - `suite u8` (e.g., 0x01=ChaCha20-Poly1305, 0x02=AES-256-GCM)
  - `key_hint_len u8` followed by `key_hint` bytes (e.g., key ID or KDF label)
  - Remaining bytes OPTIONAL per suite (e.g., salt); unknown extras ignored.
- `0x03 signature parameters`
  - `scheme u8` (e.g., 0x01=Ed25519, 0x02=Dilithium2)
  - `pubkey_fingerprint_len u8` followed by fingerprint bytes
  - Optional `cert_chain_len u16` + bytes (can be zero).
- `0x7F vendor`
  - Free-form; MUST NOT be in interop fixtures; length-bounded by TLV.

These codes and layouts are frozen for the initial registry; altering them requires a new RFC. Using these TLVs in production requires the activation flag and a version/flag bump.
