import * as wasm from '@lnmplang/wasm-bindings';

export class Sanitizer {
    /**
     * Sanitizes LNMP text input by normalizing whitespace and fixing quotes.
     */
    static sanitize(text: string): string {
        return wasm.sanitize_text(text);
    }
}
