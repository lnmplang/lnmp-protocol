use lnmp_core::{LnmpField, LnmpRecord, LnmpValue, TypeHint};
use lnmp_embedding::Vector;
use lnmp_quant::{quantize_embedding, QuantScheme};

fn main() {
    println!("=== LNMP-QUANT Integration Example ===\n");

    // 1. Create an embedding vector
    let embedding = Vector::from_f32(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    println!("Original embedding: {:?}", embedding.as_f32().unwrap());
    println!("Original size: {} bytes\n", embedding.dim * 4);

    // 2. Quantize it
    let quantized = quantize_embedding(&embedding, QuantScheme::QInt8).unwrap();
    println!("Quantized scheme: {:?}", quantized.scheme);
    println!("Quantized size: {} bytes", quantized.data_size());
    println!("Compression ratio: {:.1}x\n", quantized.compression_ratio());

    // 3. Create an LNMP record with quantized embedding
    let mut record = LnmpRecord::new();

    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("user_query".to_string()),
    });

    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::QuantizedEmbedding(quantized.clone()),
    });

    println!("✓ Created LNMP record with quantized embedding");
    println!("  - Field 1: String (user_query)");
    println!("  - Field 2: QuantizedEmbedding");
    println!("\nRecord has {} fields\n", record.fields().len());

    // 4. Demonstrate type hint
    let hint = TypeHint::QuantizedEmbedding;
    println!("Type hint for quantized embedding: :{}", hint.as_str());

    let value = LnmpValue::QuantizedEmbedding(quantized);
    println!("Type hint validates: {}", hint.validates(&value));

    println!("\n=== Integration Complete ===");
}
