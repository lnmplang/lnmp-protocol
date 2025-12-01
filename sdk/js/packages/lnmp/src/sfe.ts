import * as wasm from '@lnmplang/wasm-bindings';
import { Envelope } from './envelope';
import { ContextProfile } from '@lnmplang/wasm-bindings';

export { ContextProfile };

export class ContextScorer {
    /**
     * Scores an envelope for LLM context prioritization.
     * @param envelope The envelope to score
     * @param now Current timestamp (ms)
     */
    static score(envelope: Envelope, now: number = Date.now()): ContextProfile {
        // We need to pass BigInt for u64 timestamp
        return wasm.sfe_score_context(envelope, BigInt(now)) as ContextProfile;
    }
}
