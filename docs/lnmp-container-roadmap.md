# LNMP Container Roadmap

## Goal

Standardize `.lnmp` as the single, forward-compatible container for every LNMP mode. The header is **magic + mode byte + flags + metadata length**, giving one visible file type while allowing per-mode evolution. Tooling (CLI/IDE/SDK packaging) is explicitly out of scope here; this roadmap is only about the container schema and reference behavior.

## Phase 0 – Core Architecture (complete)
- Locked header layout (magic, version, mode byte, flags, metadata length, alignment rules) and documented in `spec/`.
- Implemented header types in `lnmp-core` with encode/decode + errors.
- Provided mode detection helpers so any `.lnmp` file identifies instantly.

## Phase 1 – LNMP/Text & LNMP/Binary (complete)
- Container-first APIs (`lnmp_codec::container::{ContainerFrame, ContainerBuilder}`) and CLI paths emit/inspect `.lnmp`.
- Header flags and metadata plumbed through text/binary encode/decode paths with checksum/compression support.
- Docs and samples default to `.lnmp` artifacts; metadata inspection wired into `lnmp-cli`.

## Phase 2 – LNMP/Stream & LNMP/Delta (complete)
- Stream metadata schema published (chunk size, checksum type, flow-control flags) and serialized in container headers; stream chunk/checksum fixtures validated in tests.
- Delta metadata published (base snapshot ID, algorithm, compression hints) and serialized; invalid algorithm/compression rejected; base/algorithm honored at apply time via metadata-aware delta context; real delta payload fixtures generated and applied in tests.
- Header + payload conformance fixtures live in `fixtures/` with coverage in `crates/lnmp-codec/tests/{container_conformance.rs,streaming_layer_tests.rs,delta_encoding_tests.rs}`.
- Deliverables: updated `spec/lnmp-container-format.md`, conformance matrix (`docs/lnmp-conformance-checklist.md`), payload plan (`docs/lnmp-payload-conformance.md`), regression fixtures, and the minimum interoperable subset for implementers.
- Actionables for CI: add conformance/fixture tests to the pipeline (`container_conformance`, streaming/delta fixture tests).

## Phase 3 – Schema Finalization (in progress)
- Freeze v1 metadata envelopes after stream/delta bake-in; declare reserved bits/bytes for future PQ work.
- Lock CI gates on conformance fixtures (header + payload) and publish the negative matrix alongside the spec.
- Thread container-derived delta apply context into SDK/CLI so base/algorithm checks are automatic.
- Publish final v1 fixture set and announce “no header changes without version bump”.
- CI hook (`scripts/run-conformance.sh`) runs container_conformance + streaming_layer_tests + delta_encoding_tests + full suite.

## Phase 4 – LNMP/Quantum-Safe (forward-looking)
- Draft PQ metadata layout (key exchange + signatures) and flag matrix for quantum mode.
- Produce a minimal reference proof (key exchange + signing) using a lightweight PQ library.

## Out of Scope
- CLI/IDE packaging, marketplace publishing, and installer flows.
- Non-Rust SDK bindings and distribution pipelines (tracked separately).

## Ongoing
- Update `spec/` after each phase; keep MIME/icon definitions aligned with the header spec.
- Maintain backward/forward compatibility matrix in the changelog and publish fixture sets for third parties.
