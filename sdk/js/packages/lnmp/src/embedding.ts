import * as wasm from '@lnmplang/wasm-bindings';
import { VectorDelta, QuantScheme, QuantizedVector } from '@lnmplang/wasm-bindings';

export { QuantScheme };

export class Embedding {
    /**
     * Computes the delta between two vectors.
     */
    static computeDelta(base: number[], updated: number[]): VectorDelta {
        // WASM expects Float32Array for Vec<f32>
        return wasm.embedding_delta(
            new Float32Array(base),
            new Float32Array(updated)
        ) as VectorDelta;
    }

    /**
     * Applies a delta to a base vector to reconstruct the updated vector.
     */
    static applyDelta(base: number[], delta: VectorDelta): number[] {
        const result = wasm.embedding_apply_delta(
            new Float32Array(base),
            delta
        );
        // Convert Float32Array back to number[]
        return Array.from(result);
    }

    /**
     * Quantizes a vector using the specified scheme.
     */
    static quantize(vector: number[], scheme: QuantScheme): QuantizedVector {
        return wasm.quantize_embedding(
            new Float32Array(vector),
            scheme
        ) as QuantizedVector;
    }

    /**
     * Dequantizes a vector back to floating point values.
     */
    static dequantize(quantized: QuantizedVector): number[] {
        const result = wasm.dequantize_embedding(quantized);
        return Array.from(result);
    }
}
