//! Example demonstrating LNMP-ShortForm encoding in LNMP v0.3
//!
//! LNMP-ShortForm is an unsafe minimal variant optimized for extreme token
//! reduction in LLM contexts. It omits the 'F' prefix and optionally type hints
//! to achieve 7-12× token reduction compared to JSON.
//!
//! WARNING: ShortForm is NOT canonical LNMP and should only be used for
//! LLM input optimization, never for storage or APIs.

use lnmp_codec::Encoder;
use lnmp_codec::config::EncoderConfig;
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.3 ShortForm Example ===\n");
    println!("WARNING: ShortForm is an UNSAFE variant for LLM optimization only!\n");

    // Example 1: Standard vs ShortForm comparison
    println!("1. Standard LNMP vs ShortForm");
    println!("------------------------------");
    standard_vs_shortform();
    println!();

    // Example 2: Token reduction metrics
    println!("2. Token Reduction Metrics");
    println!("--------------------------");
    token_reduction_metrics();
    println!();

    // Example 3: ShortForm with nested structures
    println!("3. ShortForm with Nested Structures");
    println!("------------------------------------");
    shortform_nested();
    println!();

    // Example 4: When to use ShortForm
    println!("4. When to Use ShortForm");
    println!("------------------------");
    when_to_use_shortform();
}

fn standard_vs_shortform() {
    // Create a sample record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 23,
        value: LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()]),
    });

    // Standard LNMP with type hints
    let config_standard = EncoderConfig::new()
        .with_type_hints(true)
        .with_canonical(true)
        .with_checksums(false);
    let encoder_standard = Encoder::with_config(config_standard);
    let standard_output = encoder_standard.encode(&record);

    println!("Standard LNMP:");
    println!("{}", standard_output);
    println!("Length: {} chars\n", standard_output.len());

    // ShortForm (conceptual - simulated by removing F prefix and type hints)
    let shortform_output = simulate_shortform(&record);
    println!("ShortForm:");
    println!("{}", shortform_output);
    println!("Length: {} chars\n", shortform_output.len());

    let reduction = ((standard_output.len() - shortform_output.len()) as f64 
        / standard_output.len() as f64) * 100.0;
    println!("Reduction: {:.1}%", reduction);
}

fn token_reduction_metrics() {
    // Create a more complex record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("alice".to_string()),
    });
    record.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(25),
    });
    record.add_field(LnmpField {
        fid: 3,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 4,
        value: LnmpValue::Float(98.5),
    });
    record.add_field(LnmpField {
        fid: 5,
        value: LnmpValue::StringArray(vec![
            "admin".to_string(),
            "developer".to_string(),
            "user".to_string(),
        ]),
    });

    // JSON equivalent
    let json_output = r#"{"name":"alice","age":25,"active":true,"score":98.5,"roles":["admin","developer","user"]}"#;
    println!("JSON:");
    println!("{}", json_output);
    println!("Length: {} chars\n", json_output.len());

    // Standard LNMP
    let encoder = Encoder::new();
    let lnmp_output = encoder.encode(&record);
    println!("Standard LNMP:");
    println!("{}", lnmp_output);
    println!("Length: {} chars\n", lnmp_output.len());

    // ShortForm
    let shortform_output = simulate_shortform(&record);
    println!("ShortForm:");
    println!("{}", shortform_output);
    println!("Length: {} chars\n", shortform_output.len());

    let json_reduction = ((json_output.len() - shortform_output.len()) as f64 
        / json_output.len() as f64) * 100.0;
    let lnmp_reduction = ((lnmp_output.len() - shortform_output.len()) as f64 
        / lnmp_output.len() as f64) * 100.0;

    println!("Token reduction:");
    println!("  vs JSON: {:.1}%", json_reduction);
    println!("  vs Standard LNMP: {:.1}%", lnmp_reduction);
}

fn shortform_nested() {
    // Create nested structure
    let mut inner = LnmpRecord::new();
    inner.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("nested".to_string()),
    });
    inner.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::Int(42),
    });

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::NestedRecord(Box::new(inner)),
    });
    record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("outer".to_string()),
    });

    // Standard LNMP
    let encoder = Encoder::new();
    let standard_output = encoder.encode(&record);
    println!("Standard LNMP:");
    println!("{}", standard_output);
    println!();

    // ShortForm (conceptual)
    let shortform_output = simulate_shortform_nested(&record);
    println!("ShortForm:");
    println!("{}", shortform_output);
    println!();
    println!("Note: ShortForm maintains structure but removes prefixes");
}

fn when_to_use_shortform() {
    println!("✓ Use ShortForm for:");
    println!("  - LLM prompt input where token count is critical");
    println!("  - Temporary data in LLM context windows");
    println!("  - One-way data flow (LLM input only)");
    println!();
    println!("✗ DO NOT use ShortForm for:");
    println!("  - Data storage or persistence");
    println!("  - API communication");
    println!("  - Data that needs checksums");
    println!("  - Multi-language interoperability");
    println!("  - Canonical format requirements");
    println!();
    println!("ShortForm trades safety for token efficiency!");
}

// Helper function to simulate ShortForm encoding
// (In a real implementation, this would be in the ShortFormEncoder)
fn simulate_shortform(record: &LnmpRecord) -> String {
    let fields: Vec<String> = record
        .sorted_fields()
        .iter()
        .map(|field| {
            let value_str = match &field.value {
                LnmpValue::Int(i) => i.to_string(),
                LnmpValue::Float(f) => f.to_string(),
                LnmpValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
                LnmpValue::String(s) => s.clone(),
                LnmpValue::StringArray(arr) => format!("[{}]", arr.join(",")),
                _ => String::new(),
            };
            format!("{}={}", field.fid, value_str)
        })
        .collect();
    
    fields.join(" ")
}

fn simulate_shortform_nested(record: &LnmpRecord) -> String {
    let fields: Vec<String> = record
        .sorted_fields()
        .iter()
        .map(|field| {
            let value_str = match &field.value {
                LnmpValue::Int(i) => i.to_string(),
                LnmpValue::Float(f) => f.to_string(),
                LnmpValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
                LnmpValue::String(s) => s.clone(),
                LnmpValue::StringArray(arr) => format!("[{}]", arr.join(",")),
                LnmpValue::NestedRecord(nested) => {
                    let inner_fields: Vec<String> = nested
                        .sorted_fields()
                        .iter()
                        .map(|f| {
                            let v = match &f.value {
                                LnmpValue::Int(i) => i.to_string(),
                                LnmpValue::String(s) => s.clone(),
                                _ => String::new(),
                            };
                            format!("{}={}", f.fid, v)
                        })
                        .collect();
                    format!("{{{}}}", inner_fields.join(";"))
                }
                _ => String::new(),
            };
            format!("{}={}", field.fid, value_str)
        })
        .collect();
    
    fields.join(" ")
}
