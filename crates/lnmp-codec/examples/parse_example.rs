use lnmp_codec::Parser;

fn main() {
    // Example LNMP data
    let lnmp_data = r#"
# User record example
F12=14532
F7=1
F20="Halil Ibrahim"
F23=["admin","dev"]
"#;

    println!("Parsing LNMP data:");
    println!("{}", lnmp_data);

    // Parse the LNMP data
    match Parser::new(lnmp_data) {
        Ok(mut parser) => match parser.parse_record() {
            Ok(record) => {
                println!("\nParsed {} fields:", record.fields().len());

                for field in record.fields() {
                    println!("  Field {}: {:?}", field.fid, field.value);
                }

                // Access specific fields
                if let Some(field) = record.get_field(12) {
                    println!("\nUser ID (F12): {:?}", field.value);
                }
                if let Some(field) = record.get_field(20) {
                    println!("Name (F20): {:?}", field.value);
                }
                if let Some(field) = record.get_field(23) {
                    println!("Roles (F23): {:?}", field.value);
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        },
        Err(e) => eprintln!("Parser initialization error: {}", e),
    }
}
