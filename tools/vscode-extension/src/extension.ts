import * as vscode from 'vscode';
import { promises as fs } from 'fs';

export function activate(context: vscode.ExtensionContext) {
    const decodedProvider = new LnmpDecodedContentProvider();
    context.subscriptions.push(
        vscode.workspace.registerTextDocumentContentProvider('lnmp-decoded', decodedProvider)
    );

    const inspectCmd = vscode.commands.registerCommand('lnmp.inspectContainer', () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }
        const filePath = editor.document.uri.fsPath;
        inspectNative(filePath).then(
            (output) => vscode.window.showInformationMessage(output),
            (error) => vscode.window.showErrorMessage(error)
        );
    });

    const openTextCmd = vscode.commands.registerCommand('lnmp.openAsText', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }
        const doc = await decodedProvider.openDecodedDocument(editor.document.uri);
        await vscode.window.showTextDocument(doc, { preview: false });
    });

    const intercepted = new Set<string>();
    const autoOpen = vscode.workspace.onDidOpenTextDocument(async (doc) => {
        if (doc.uri.scheme !== 'file' || !doc.fileName.endsWith('.lnmp')) {
            return;
        }
        const key = doc.uri.toString();
        if (intercepted.has(key)) {
            return;
        }
        intercepted.add(key);
        await vscode.commands.executeCommand('vscode.openWith', doc.uri, 'lnmp.decodeEditor');
    });

    context.subscriptions.push(
        inspectCmd,
        openTextCmd,
        vscode.window.registerCustomEditorProvider(
            'lnmp.decodeEditor',
            new LnmpCustomEditorProvider(decodedProvider),
            {
                webviewOptions: { retainContextWhenHidden: true },
                supportsMultipleEditorsPerDocument: false
            }
        ),
        autoOpen
    );
}

export function deactivate() {}

class LnmpDecodedContentProvider implements vscode.TextDocumentContentProvider {
    private readonly emitter = new vscode.EventEmitter<vscode.Uri>();
    onDidChange?: vscode.Event<vscode.Uri> | undefined = this.emitter.event;

    async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
        try {
            const decoded = await decodeLnmpFile(uri.fsPath);
            return decoded.content;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            vscode.window.showErrorMessage(message);
            return message;
        }
    }

    toDecodedUri(uri: vscode.Uri): vscode.Uri {
        return vscode.Uri.from({
            scheme: 'lnmp-decoded',
            path: uri.fsPath
        });
    }

    async openDecodedDocument(originalUri: vscode.Uri): Promise<vscode.TextDocument> {
        const decodedUri = this.toDecodedUri(originalUri);
        return vscode.workspace.openTextDocument(decodedUri);
    }
}

class LnmpCustomEditorProvider implements vscode.CustomReadonlyEditorProvider<vscode.CustomDocument> {
    constructor(private readonly decoded: LnmpDecodedContentProvider) {}

    async openCustomDocument(
        uri: vscode.Uri,
        _openContext: vscode.CustomDocumentOpenContext,
        _token: vscode.CancellationToken
    ): Promise<vscode.CustomDocument> {
        return { uri, dispose: () => { /* no-op */ } };
    }

    async resolveCustomEditor(
        document: vscode.CustomDocument,
        webviewPanel: vscode.WebviewPanel,
        _token: vscode.CancellationToken
    ): Promise<void> {
        webviewPanel.webview.options = { enableScripts: false };
        try {
            const decoded = await decodeLnmpFile(document.uri.fsPath);
            const escaped = escapeHtml(decoded.content);
            webviewPanel.webview.html = `
                <html>
                <head>
                    <style>
                        body { font-family: var(--vscode-editor-font-family); margin: 0; }
                        pre { white-space: pre-wrap; word-break: break-word; padding: 1rem; }
                    </style>
                </head>
                <body>
                    <pre>${escaped}</pre>
                </body>
                </html>
            `;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            webviewPanel.webview.html = `<pre>${escapeHtml(message)}</pre>`;
        }
    }
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

async function inspectNative(filePath: string): Promise<string> {
    const raw = await fs.readFile(filePath);
    const header = parseHeader(raw);
    const details = [`Mode: ${header.mode}`, `Version: ${header.version}`, `Flags: 0x${header.flags.toString(16).padStart(4, '0').toUpperCase()}`, `Metadata length: ${header.metadataLength} bytes`];
    return details.join(' • ');
}

async function decodeLnmpFile(filePath: string): Promise<{ content: string }> {
    const raw = await fs.readFile(filePath);
    const header = parseHeader(raw);
    const metadataEnd = 12 + header.metadataLength;
    const payload = raw.slice(metadataEnd);

    if (header.mode === 'LNMP/Text') {
        return { content: payload.toString('utf8') };
    }

    // For binary/stream/delta, show a friendly message + hex dump preview
    const preview = payload.slice(0, 256).toString('hex').match(/.{1,2}/g)?.join(' ') ?? '';
    const msg = [
        `${header.mode} payload (binary)`,
        `Flags: 0x${header.flags.toString(16).padStart(4, '0').toUpperCase()}`,
        `Metadata length: ${header.metadataLength} bytes`,
        `Payload bytes: ${payload.length}`,
        preview ? `Preview (first 256 bytes): ${preview}` : 'No payload bytes'
    ].join('\n');
    return { content: msg };
}

function parseHeader(raw: Buffer): { mode: string; version: number; flags: number; metadataLength: number } {
    if (raw.length < 12) {
        throw new Error('LNMP file too short (missing header)');
    }
    const magic = raw.slice(0, 4).toString('ascii');
    if (magic !== 'LNMP') {
        throw new Error('Not an LNMP container (magic mismatch)');
    }
    const version = raw[4];
    const modeByte = raw[5];
    const flags = raw.readUInt16BE(6);
    const metadataLength = raw.readUInt32BE(8);
    const mode = modeName(modeByte);
    return { mode, version, flags, metadataLength };
}

function modeName(mode: number): string {
    switch (mode) {
        case 0x01:
            return 'LNMP/Text';
        case 0x02:
            return 'LNMP/Binary';
        case 0x03:
            return 'LNMP/Stream';
        case 0x04:
            return 'LNMP/Delta';
        case 0x05:
            return 'LNMP/Quantum-Safe';
        default:
            return `Unknown (0x${mode.toString(16)})`;
    }
}
