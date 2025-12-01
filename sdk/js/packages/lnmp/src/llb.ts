import * as wasm from '@lnmplang/wasm-bindings';
import { LnmpRecord } from '@lnmplang/wasm-bindings';

export class ExplainEncoder {
    /**
     * Encodes a record with human-readable explanations.
     * @param record The record to encode
     * @param dictionary Mapping of Field IDs to names (e.g., { 12: "user_id" })
     */
    static encode(record: LnmpRecord, dictionary: Record<number, string>): string {
        // Convert dictionary to the format expected by Rust (HashMap<u64, String>)
        // In JS, object keys are strings, so we need to ensure they are parsed correctly on Rust side.
        // The Rust side expects a map-like structure. passing a JS object works for serde_wasm_bindgen map deserialization.

        // However, serde_wasm_bindgen expects Map for HashMap by default, or Object if configured.
        // Let's try passing the object directly.

        // We need to construct the SemanticDictionary struct structure: { field_names: { ... } }
        const rustDict = {
            field_names: new Map(Object.entries(dictionary).map(([k, v]) => [Number(k), v]))
        };

        return wasm.llb_explain(record, rustDict);
    }
}
