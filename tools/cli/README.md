# LNMP CLI

Utilities and CLI tools for working with LNMP. The CLI currently ships several
commands:

- `inspect` – reads a `.lnmp` container header, prints mode/flags/metadata info, and can dump metadata bytes (`--metadata-out`, `--metadata-hex`).
- `decode` – decodes LNMP/Text or LNMP/Binary payloads and emits canonical LNMP text (with the same metadata dump options as `inspect`).
- `encode-text` – parses LNMP text and wraps it into a `.lnmp` container using text mode.
- `encode-binary` – parses LNMP text, converts it to binary encoding, and wraps it into a `.lnmp` container.
- Both encode commands accept options to attach metadata and set header flags.
- `metadata` – lightweight helper to quickly dump the metadata section and optionally hex-print it: `lnmp-cli metadata <file> --dump metadata.bin [--raw]`.

Quickstart (Rust CLI):

```bash
cargo build -p lnmp-cli
cargo run -p lnmp-cli -- --help
```

### Inspecting a `.lnmp` file

```bash
cargo run -p lnmp-cli -- inspect path/to/file.lnmp [--metadata-out metadata.bin] [--metadata-hex]
```

Sample output:

```
File: ./examples/sample.lnmp
  Mode: LNMP/Binary
  Version: 1
  Flags: checksum (0x0001)
  Metadata length: 8 bytes
  Metadata preview: AA BB CC DD EE FF 00 11
```

Pass `--metadata-out metadata.bin` to write the metadata region to disk, or `--metadata-hex` to print the entire hex dump inline.

### Decoding a `.lnmp` file

```bash
cargo run -p lnmp-cli -- decode path/to/file.lnmp [--metadata-out metadata.bin] [--metadata-hex]
```

Canonical LNMP text is printed to `stdout`, while header/flag metadata, hex dumps, and warnings are logged to `stderr`.

### Dumping metadata directly

```bash
cargo run -p lnmp-cli -- metadata path/to/file.lnmp --dump metadata.bin [--raw]
```

This prints the metadata preview (or full hex with `--raw`) and writes the bytes to disk for advanced inspection.

### Encoding `.lnmp` files

Convert canonical/loose LNMP text into a `.lnmp` container (text mode):

```bash
cargo run -p lnmp-cli -- encode-text input.lnmp.txt output.lnmp --metadata metadata.bin --flags checksum,qsig
```

Convert the same record into binary mode:

```bash
cargo run -p lnmp-cli -- encode-binary input.lnmp.txt output.lnmp --flags checksum
```

Use `--metadata` to attach an arbitrary binary blob immediately after the header, and `--flags` to set header bits (`checksum`, `compressed`, `encrypted`, `qsig`, `qkex`). `checksum` requires the input text to contain checksum hints (e.g., `F12=14532#ABCD1234`), `qsig`/`qkex` require metadata bytes, and compression/encryption flags currently error out. Leave `--flags` empty to emit a container with zeroed flags.
