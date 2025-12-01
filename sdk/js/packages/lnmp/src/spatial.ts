import * as wasm from '@lnmplang/wasm-bindings';
import { SpatialFrame } from '@lnmplang/wasm-bindings';

export class Spatial {
    /**
     * Encodes a spatial frame to binary format.
     */
    static encodeFrame(frame: SpatialFrame): Uint8Array {
        const bytes = wasm.spatial_encode_frame(frame);
        return new Uint8Array(bytes);
    }

    /**
     * Decodes a spatial frame from binary format.
     */
    static decodeFrame(bytes: Uint8Array): SpatialFrame {
        return wasm.spatial_decode_frame(bytes) as SpatialFrame;
    }
}
