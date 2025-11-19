use lnmp_codec::{Encoder, EncoderConfig, Parser};

fn main() {
    // Original LNMP data
    let original = r#"F12=14532;F7=1;F23=["admin","dev"]"#;

    println!("Original LNMP:");
    println!("{}\n", original);

    // Parse
    println!("Parsing...");
    let mut parser = Parser::new(original).unwrap();
    let record = parser.parse_record().unwrap();
    println!("Parsed {} fields\n", record.fields().len());

    // Display parsed data
    println!("Parsed fields:");
    for field in record.fields() {
        println!("  F{} = {:?}", field.fid, field.value);
    }
    println!();

    // Encode back (multiline format)
    println!("Encoding to multiline format:");
    let encoder = Encoder::new();
    let multiline = encoder.encode(&record);
    println!("{}\n", multiline);

    // Encode back (inline format)
    println!("Encoding to inline format:");
    let encoder_inline = Encoder::with_config(EncoderConfig {
        canonical: false,
        ..EncoderConfig::default()
    });
    let inline = encoder_inline.encode(&record);
    println!("{}\n", inline);

    // Verify round-trip
    let mut parser2 = Parser::new(&inline).unwrap();
    let record2 = parser2.parse_record().unwrap();

    println!("Round-trip verification:");
    println!("  Original fields: {}", record.fields().len());
    println!("  Round-trip fields: {}", record2.fields().len());
    println!(
        "  Match: {}",
        record.fields().len() == record2.fields().len()
    );
}
