# LNMP VSCode Extension (Draft)

The VSCode extension will give `.lnmp` containers a first-class experience inside the editor. The initial milestone sits between Phase 1 and Phase 2 of the roadmap and focuses on recognition + metadata inspection.

## Goals

1. **File association** – register `.lnmp` as `lnmp-language`, provide icon + status bar badge and ensure VSCode treats the files as text (UTF-8) by default.
2. **Syntax highlighting** – supply a TextMate grammar for LNMP/Text (field IDs, values, metadata markers) so container payloads remain readable.
3. **Metadata inspector** – add a command palette entry (`LNMP: Inspect Container`) that parses containers natively (no CLI dependency) and displays mode/flags/metadata.
4. **Status bar diagnostics** – when a `.lnmp` file is active, show the detected mode and flag summary (e.g., `LNMP/Stream • checksum+compressed`).

## Proposed Structure

```
tools/
  vscode-extension/
    package.json         # VSCode manifest
    syntaxes/lnmp.tmLanguage.json
    language-configuration.json
    src/extension.ts     # Command + metadata integration
    media/icon.png       # File icon
    README.md            # Marketplace description
```

## Next Steps

- Run `npm install` inside `tools/vscode-extension/` (or `pnpm install` if you prefer) to pull down TypeScript + VSCode type definitions.
- Use `npm run compile` (invokes `tsc -p ./`) to produce `dist/extension.js`. Launch the extension in VSCode via `F5` (Extension Development Host), then execute `LNMP: Inspect Container` from the command palette on any `.lnmp` file.
- The inspect command shells out to `lnmp-cli metadata --raw`, so make sure `lnmp-cli` is discoverable in your `$PATH` (or adjust the `spawn` invocation inside `src/extension.ts`).
- To preview the custom icon, install the generated VSIX: `npm run compile && vsce package`, then `code --install-extension lnmp-vscode-extension-0.0.1.vsix`. Select the "LNMP Icons" file-icon theme inside VSCode (Preferences → File Icon Theme).
- Future work: embed richer metadata panels (chunk breakdown, delta op tree) leveraging the CLI’s JSON output once available, and publish the extension via VSCE when Phase 2 features stabilize.
