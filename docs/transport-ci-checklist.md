# LNMP Transport CI Checklist

This document describes the minimum verification matrix we expect every PR or nightly build touching `crates/lnmp-transport` to satisfy.

## Feature Test Matrix

| Feature set | Command | Notes |
| --- | --- | --- |
| none | `cargo test -p lnmp-transport --no-default-features` | Ensures examples/tests gate themselves cleanly when optional modules are missing. |
| http (default) | `cargo test -p lnmp-transport --features http` | Covers the default header bindings. |
| kafka only | `cargo test -p lnmp-transport --no-default-features --features kafka` | Validates Kafka helpers in isolation. |
| http + kafka | `cargo test -p lnmp-transport --features "http kafka"` | Catches interactions between the two most common transports. |
| full stack | `cargo test -p lnmp-transport --features "http kafka grpc nats"` | Exercises gRPC and NATS paths along with HTTP/Kafka. |

> Recommendation: add a CI workflow job that iterates across this matrix in parallel to keep runtimes manageable.

## Benchmarks & Examples

Run at least once per release (and preferably nightly):

```
cargo bench -p lnmp-transport --features "http kafka"
cargo bench -p lnmp-transport --no-default-features
cargo run  -p lnmp-transport --example transport_basic_usage --features "http kafka grpc nats"
cargo run  -p lnmp-transport --example http_full --features http
cargo run  -p lnmp-transport --example otel_integration --features http
```

This ensures:
- Feature-gated benches emit results only when transports exist.
- Examples stay compilable/runnable in automation, preventing regressions before release.

## Naming Collision Guard

Because Cargo reuses the example filename as the binary, verify that no two packages share a name by running:

```
cargo metadata --format-version 1 --no-deps \
  | jq -r '.packages[] as $pkg | $pkg.targets[]? as $t
           | select($t.kind[]? == "example")
           | "\($pkg.name): \($t.name)"' \
  | sort \
  | uniq -d
```

Fail the pipeline if this command prints anything (duplicate names).
