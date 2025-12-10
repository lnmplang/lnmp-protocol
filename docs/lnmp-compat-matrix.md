# LNMP Compatibility Matrix (v1)

This document captures the container v1 compatibility expectations and points to reusable fixture bundles for third parties.

## Container v1 (header version 0x01)
- Modes: Text (0x01), Binary (0x02), Stream (0x03), Delta (0x04) — all supported in v1.
- Delta metadata: fixed 10 bytes; `algorithm` and `compression` must be 0x00/0x01 only.
- Stream metadata: fixed 6 bytes; `checksum_type` 0x00/0x02 validated; reserved bits zero.
- Flags: only checksum flag is meaningful; others reserved and must be zero.
- Future reservation: flag bit 15 is earmarked to signal a Metadata Extension Block (TLV chain) in a later version. It MUST remain `0` in v1; consumers may ignore absent chains.
- Extension registry: `spec/lnmp-metadata-extension-rfc.md` defines the TLV shape and frozen registry codes (checksum/encryption/signature/vendor). Still inactive until a version/flag bump (planned header v0x02 or flag 15 unreserve with fixtures).

## Fixture Sets (v1)
- Source fixtures live under `fixtures/` and are used by conformance tests.
- To publish a bundle for external use: `scripts/package-fixtures.sh` produces `artifacts/lnmp-fixtures-v1.tar.gz`.

### Automated Status Table (update via CI)

| Date (UTC) | Commit | Container | Streaming | Delta | Notes |
| --- | --- | --- | --- | --- | --- |
| _pending automation_ | `_` | `_` | `_` | `_` | Populate via CI pipeline |

> See `docs/compat-reporting-guide.md` for how to keep this table current.

## Versioning / Freeze Policy
- Header/mode bytes and stream/delta metadata layouts above are frozen for v1. Any change to magic/version/mode/flags/layouts requires a version bump + RFC.
- Conformance fixtures/tests must remain green across releases.
