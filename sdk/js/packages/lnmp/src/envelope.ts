import * as wasm from '@lnmplang/wasm-bindings';
import { LnmpRecord, EnvelopeMetadata, LnmpEnvelope } from '@lnmplang/wasm-bindings';

export class Envelope {
    /**
     * Wraps an LNMP record with metadata to create an envelope.
     */
    static wrap(record: LnmpRecord, metadata: EnvelopeMetadata): LnmpEnvelope {
        return wasm.envelope_wrap(record, metadata) as LnmpEnvelope;
    }

    /**
     * Unwraps an envelope to retrieve the original record.
     */
    static unwrap(envelope: LnmpEnvelope): LnmpRecord {
        return wasm.envelope_unwrap(envelope) as LnmpRecord;
    }

    /**
     * Converts an envelope to HTTP headers (X-LNMP-...).
     */
    static toHeaders(envelope: LnmpEnvelope): Record<string, string> {
        return wasm.transport_to_http(envelope) as Record<string, string>;
    }

    /**
     * Creates an envelope metadata structure from HTTP headers.
     * Note: This returns metadata, which can be combined with a record to form an envelope.
     */
    static fromHeaders(headers: Record<string, string>): EnvelopeMetadata {
        return wasm.transport_from_http(headers) as EnvelopeMetadata;
    }
}
