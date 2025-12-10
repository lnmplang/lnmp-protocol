# LNMP Codec CI Checklist

This checklist covers `lnmp-codec` (text/binary encoder, container/streaming/delta layers).

## Mandatory Commands

| Feature set / focus | Command | Notes |
| --- | --- | --- |
| Unit + doc tests | `cargo test -p lnmp-codec` | Runs all codec unit/doc tests. |
| Container fixtures | `cargo test -p lnmp-codec --test container_conformance` | Validates `.lnmp` container header rules. |
| Streaming fixtures | `cargo test -p lnmp-codec --test streaming_layer_tests` | Covers chunk/checksum logic. |
| Delta fixtures | `cargo test -p lnmp-codec --test delta_encoding_tests` | Applies delta fixture corpus. |

## Benchmarks / Performance (run at least once per release)

```
cargo bench -p lnmp-codec
```

Record the latest benchmark summary in `crates/lnmp-codec/OPTIMIZATIONS.md`.

## Spec Linkage

- Container rules: `spec/lnmp-container-format.md`
- Streaming: `spec/lnmp-container-overview.md`
- Delta: `spec/lnmp-container-format.md` delta metadata section

When a spec section changes, update the relevant fixture/test listed above in the same PR.
