# LNMP Codec – Architectural Findings & v0.7 Roadmap
*(Technical Assessment & Action Plan)*

Goal: deliver strict M2M correctness + LLM‐optimized lenient path + stable error semantics + zero‐copy fast path.

## 1) Findings
- **Sanitizer** (crates/lnmp-sanitize/src/sanitize.rs): single pass mixes whitespace cleanup, escape repair, and boolean/number normalization; `auto_quote_strings` unused; levels are not distinct; backslash handling can over-escape already-correct sequences. *Risk:* lenient sanitize mutates valid input.
- **Parser spans** (crates/lnmp-codec/src/parser.rs): owns sanitized text but loses original→sanitized offset mapping. *Risk:* error positions diverge from original input in lenient mode.
- **Lexer ownership** (crates/lnmp-codec/src/lexer.rs): `Cow` forces owned allocations on strict paths in many cases. *Risk:* avoidable overhead in hot paths/WASM.
- **Binary encode strictness** (crates/lnmp-codec/src/binary/encoder.rs): `encode_text` uses `text_input_mode` but leaves `parsing_mode` at default loose. *Risk:* callers expect strict+strict but get lenient parsing.
- **Defaults** (crates/lnmp-codec/src/config.rs): `TextInputMode::Strict` + `ParsingMode::Loose` may surprise LLM SDK users who expect lenient defaults.
- **Compliance runner** (tests/compliance/rust/runner.rs): never exercises lenient/sanitize path. *Risk:* regressions in lenient behavior go unnoticed.
- **Examples** (examples/examples/Cargo.toml): rely on workspace deps; external builds need git/path overrides. *Risk:* friction for out-of-tree users.

## 2) Architecture & Performance Recommendations
- **Multi-pass sanitizer (level-aware):** Pass 1 whitespace/semicolon cleanup; Pass 2 quote/escape repair (respect existing escapes, implement `auto_quote_strings`); Pass 3 semantic normalization (booleans/numbers) gated by level (Aggressive).
- **Streaming sanitizer iterator:** emit tokens/segments to avoid full-buffer copies on large LLM outputs; keep `Cow` return for small/clean inputs.
- **Dual-span error mapping:** maintain sanitized→original offset map; surface both spans in errors for lenient mode.
- **Zero-allocation strict fast path:** replace `Cow` with Borrowed/Owned enum in lexer; ASCII-first scan; avoid allocations on strict inputs.
- **API defaults:** provide explicit constructors/profiles: `Parser::for_llm` (Loose + Lenient), `Parser::for_m2m` (Strict + Strict); document defaults for SDKs.
- **Testing/fuzzing:** property tests (proptest) for sanitize→parse invariants; fuzz quote/backslash repairs; add lenient-mode compliance cases; roundtrip stress tests.
- **Benchmarking:** Criterion suites for strict/lenient parse, sanitize on large inputs, binary encode; set target overhead budget (<X% vs strict). Add WASM benches for Node/Browser.

## 3) v0.7 Actionable Roadmap
**Phase 1 — Sanitizer Overhaul**
- Refactor to 3 passes; implement `auto_quote_strings`; protect valid `\\` sequences; add property tests for quote/escape repairs.

**Phase 2 — Parser/Lexer Refactor**
- Switch lexer storage to Borrowed/Owned enum; zero-copy strict path; introduce sanitized→original span map; expose spans in error structs.

**Phase 3 — Binary Encoder**
- Add strict+strict helper/flag; align `text_input_mode` defaults with `parsing_mode` intent; surface `text_input_mode` in bindings.

**Phase 4 — SDK Defaults**
- TS/Go/Python SDKs default to lenient LLM profile; document M2M strict profile.

**Phase 5 — Compliance & Fuzz**
- Add lenient compliance suite; sanitize fuzzers; parse+encode roundtrip stress.

**Phase 6 — Benchmarks**
- Add Criterion and WASM benches; publish overhead targets.

## 4) Executive Summary
The codec is memory-safe, workspace-clean, and supports lenient mode, but to reach v0.7 “Stable Architecture” we need: multi-pass sanitizer, span-aware errors, zero-copy strict path, clear strict/lenient profiles, and coverage/benchmarks. Executing the roadmap positions LNMP as a protocol combining Protobuf-grade rigor, JSON5 practicality, and LLM-friendly resilience while retaining strict M2M guarantees.
