# LNMP Core CI Checklist

This checklist covers `lnmp-core` primitives (records, fields, profiles, checksums).

## Mandatory Commands

| Scope | Command | Notes |
| --- | --- | --- |
| Unit + doc tests | `cargo test -p lnmp-core` | Exercises record builder, checksum, profile logic. |
| Examples | `cargo test -p lnmp-core --examples` | Ensures canonical ordering demos build/run. |

## Optional (run before release)

```
cargo bench -p lnmp-core
```

## Spec Linkage

- Record & canonical rules: `spec/lnmp-container-format.md` (Record layer)
- Checksum semantics: `spec/lnmp-container-format.md` + `spec/lnmp-envelope-v1.0.md`

Any change to these sections must keep the commands above green.
