const { parse_lnmp, encode_lnmp, compute_checksum } = require('./dist/index.js');

console.log('Testing WASM bindings...');

try {
    // Test Parsing
    const text = 'F12=14532;F7=1';
    console.log(`Parsing: ${text}`);
    const record = parse_lnmp(text);
    console.log('Parsed Record:', JSON.stringify(record, null, 2));

    // Test Encoding
    console.log('Encoding record...');
    const encoded = encode_lnmp(record, true, false);
    console.log(`Encoded: ${encoded}`);

    if (encoded.includes('F7=1') && encoded.includes('F12=14532')) {
        console.log('✅ Encoding success');
    } else {
        console.error('❌ Encoding failed');
        process.exit(1);
    }

    // Test Checksum
    console.log('Computing checksum...');
    const checksum = compute_checksum(12, 'i', { Int: 14532 });
    console.log(`Checksum: ${checksum}`);

    if (checksum > 0) {
        console.log('✅ Checksum success');
    } else {
        console.error('❌ Checksum failed');
        process.exit(1);
    }

} catch (error) {
    console.error('❌ Error:', error);
    process.exit(1);
}
