import * as wasm from '@lnmplang/wasm-bindings';
import { NetMessage, RoutingDecision } from '@lnmplang/wasm-bindings';

export class Network {
    /**
     * Decides how to route a message based on the default policy.
     */
    static decide(message: NetMessage, nowMs: number = Date.now()): RoutingDecision {
        // WASM returns string representation of enum
        const decision = wasm.network_decide(message, nowMs) as string;
        return decision as RoutingDecision;
    }

    /**
     * Calculates the importance score of a message (0.0 - 1.0).
     */
    static importance(message: NetMessage, nowMs: number = Date.now()): number {
        return wasm.network_importance(message, nowMs);
    }
}
