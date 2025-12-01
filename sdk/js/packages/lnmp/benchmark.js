const {
    Parser, Encoder, Envelope, Network, Embedding, Spatial, Sanitizer,
    ExplainEncoder, ContextScorer, QuantScheme
} = require('./dist/index.js');

const ITERATIONS = 10000;

function runBenchmark(name, fn) {
    const start = performance.now();
    for (let i = 0; i < ITERATIONS; i++) {
        fn();
    }
    const end = performance.now();
    const duration = end - start;
    const ops = (ITERATIONS / duration) * 1000;
    console.log(`${name}: ${ops.toFixed(0)} ops/sec`);
    return ops;
}

console.log(`Running Benchmarks (${ITERATIONS} iterations)...\n`);

// --- Core ---
const text = 'F12=14532;F7=1;F23=[admin,dev]';
const parser = new Parser(text);
const record = parser.parseRecord();
const encoder = new Encoder();

runBenchmark('Core: Parse', () => {
    new Parser(text).parseRecord();
});

runBenchmark('Core: Encode', () => {
    encoder.encode(record);
});

// --- Embedding ---
const vec1 = new Float32Array(1536).fill(0.1);
const vec2 = new Float32Array(1536).fill(0.2);

runBenchmark('Embedding: Delta', () => {
    Embedding.computeDelta(vec1, vec2);
});

runBenchmark('Embedding: Quantize (QInt8)', () => {
    Embedding.quantize(vec1, QuantScheme.QInt8);
});

// --- Spatial ---
const frame = {
    header: {
        mode: 'Absolute',
        sequence_id: 1,
        timestamp: BigInt(Date.now()) * 1000000n,
        checksum: 0
    },
    payload: {
        S10: {
            position: { x: 10, y: 20, z: 30 },
            velocity: { vx: 0, vy: 0, vz: 0 },
            orientation: { roll: 0, pitch: 0, yaw: 0 }
        }
    }
};
const frameBytes = Spatial.encodeFrame(frame);

runBenchmark('Spatial: Encode', () => {
    Spatial.encodeFrame(frame);
});

runBenchmark('Spatial: Decode', () => {
    Spatial.decodeFrame(frameBytes);
});

// --- SFE ---
const envelope = Envelope.wrap(record, { source: 'bench', labels: {} });

runBenchmark('SFE: Score Context', () => {
    ContextScorer.score(envelope);
});

console.log('\nDone!');
