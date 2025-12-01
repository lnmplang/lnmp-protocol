const {
    Parser, Encoder, Envelope, Network, Embedding, Spatial, Sanitizer,
    ExplainEncoder, ContextScorer,
    compute_checksum, QuantScheme
} = require('./dist/index.js');

console.log('Testing LNMP SDK (Full Feature Set)...');

try {
    // 1. Core: Parsing & Encoding
    console.log('\n--- Core ---');
    const text = 'F12=14532;F7=1';
    const parser = new Parser(text);
    const record = parser.parseRecord();
    console.log('Parsed Record:', JSON.stringify(record));

    // 2. Envelope
    console.log('\n--- Envelope ---');
    const metadata = {
        timestamp: Date.now(),
        source: 'integration-test',
        trace_id: 'abc-123',
        labels: { env: 'test' }
    };
    const envelope = Envelope.wrap(record, metadata);
    console.log('Envelope Metadata:', envelope.metadata);

    const headers = Envelope.toHeaders(envelope);
    console.log('HTTP Headers:', headers);

    const unwrapped = Envelope.unwrap(envelope);
    if (!unwrapped.fields || unwrapped.fields.length !== 2) throw new Error('Unwrap failed');

    // 3. Net
    console.log('\n--- Net ---');
    const msg = {
        envelope: envelope,
        kind: 'Event',
        priority: 200,
        ttl_ms: 5000
    };

    const importance = Network.importance(msg);
    console.log(`Importance Score: ${importance} `);

    const decision = Network.decide(msg);
    console.log(`Routing Decision: ${decision} `);

    // 4. Embedding
    console.log('\n--- Embedding ---');
    const vec1 = [1.0, 2.0, 3.0];
    const vec2 = [1.1, 2.1, 3.1];

    const delta = Embedding.computeDelta(vec1, vec2);
    console.log('Vector Delta:', JSON.stringify(delta));

    const reconstructed = Embedding.applyDelta(vec1, delta);
    console.log('Reconstructed:', reconstructed);

    // Check accuracy
    if (Math.abs(reconstructed[0] - vec2[0]) > 0.0001) throw new Error('Embedding reconstruction failed');

    // 4b. Quantization
    console.log('\n--- Quantization ---');
    const qVec = Embedding.quantize(vec1, QuantScheme.QInt8);
    console.log(`Quantized Size: ${qVec.data.length} bytes(Scheme: ${qVec.scheme})`);

    const restoredVec = Embedding.dequantize(qVec);
    console.log('Restored Vector:', restoredVec);

    // 4c. Sanitization
    console.log('\n--- Sanitization ---');
    const dirtyText = '  "Hello"   ';
    const cleanText = Sanitizer.sanitize(dirtyText);
    console.log(`Sanitized: '${cleanText}'`);
    if (cleanText !== '"Hello"') throw new Error('Sanitization failed');

    // 5. Spatial
    console.log('\n--- Spatial ---');
    const frame = {
        header: {
            mode: 'Absolute',
            sequence_id: 1,
            timestamp: BigInt(Date.now()) * 1000000n, // ns
            checksum: 0
        },
        payload: {
            S10: { // SpatialState variant
                position: { x: 10, y: 20, z: 30 },
                velocity: { vx: 1, vy: 0, vz: 0 }
            }
        }
    };

    const bytes = Spatial.encodeFrame(frame);
    console.log(`Encoded Spatial Frame: ${bytes.length} bytes`);

    const decoded = Spatial.decodeFrame(bytes);
    console.log(`Decoded Frame Mode: ${decoded.header.mode}`);

    // 6. LLB (Explain Mode)
    console.log('\n--- LLB (Explain Mode) ---');
    const dict = { 12: "user_id", 7: "is_active" };
    const explanation = ExplainEncoder.encode(record, dict);
    console.log(explanation);
    if (!explanation.includes("# user_id")) throw new Error("Explain mode failed");

    // 7. SFE (Context Scoring)
    console.log('\n--- SFE (Context Scoring) ---');
    const profile = ContextScorer.score(envelope);
    console.log('Context Profile:', profile);
    if (profile.freshness_score < 0.9) throw new Error("Freshness score too low");

    console.log('\n✅ All integration tests passed!');
} catch (e) {
    console.error('❌ Test failed:', e);
    process.exit(1);
}
