import { encode_lnmp } from '@lnmplang/wasm-bindings';
import type { LnmpRecord } from './types';

export interface EncoderConfig {
    canonical?: boolean;
    includeTypeHints?: boolean;
}

export class Encoder {
    private config: Required<EncoderConfig>;

    constructor(config: EncoderConfig = {}) {
        this.config = {
            canonical: config.canonical ?? true,
            includeTypeHints: config.includeTypeHints ?? false,
        };
    }

    /**
     * Encode record to LNMP text format
     */
    encode(record: LnmpRecord): string {
        return encode_lnmp(record, this.config.canonical, this.config.includeTypeHints);
    }
}
