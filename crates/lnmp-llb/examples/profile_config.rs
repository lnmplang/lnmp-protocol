use lnmp_core::StrictDeterministicConfig;
use lnmp_llb::{LlbConfig, LlbConverter};

fn main() {
    println!("LNMP-LLB Profile Configuration Example");
    println!("======================================");

    // 1. Create a profile that enforces field ordering
    // We disable require_type_hints because ShortForm format doesn't currently support them
    let config_strict = StrictDeterministicConfig {
        reject_unsorted_fields: true,
        require_type_hints: false,
        ..StrictDeterministicConfig::standard()
    };

    println!("Configured profile: StrictDeterministic (reject_unsorted_fields=true)");

    // 2. Configure LLB with the profile
    let config = LlbConfig::new().with_strict_config(config_strict);
    let converter = LlbConverter::new(config);

    // 3. Test with sorted fields (Valid)
    // ShortForm: "1=42;2=test" -> FullText: "F1=42\nF2=test"
    let valid_shortform = "1=42;2=test";
    println!(
        "\nAttempting to parse sorted ShortForm: '{}'",
        valid_shortform
    );
    match converter.shortform_to_binary(valid_shortform) {
        Ok(_) => println!("SUCCESS: Valid ShortForm parsed successfully"),
        Err(e) => println!("FAILURE: Error parsing valid ShortForm: {}", e),
    }

    // 4. Test with unsorted fields (Invalid)
    // ShortForm: "2=test;1=42" -> FullText: "F2=test\nF1=42"
    let invalid_shortform = "2=test;1=42";
    println!(
        "\nAttempting to parse unsorted ShortForm: '{}'",
        invalid_shortform
    );
    match converter.shortform_to_binary(invalid_shortform) {
        Ok(_) => println!("SUCCESS: Unsorted ShortForm parsed (Unexpected!)"),
        Err(e) => println!("FAILURE: Expected error parsing unsorted ShortForm: {}", e),
    }
}
