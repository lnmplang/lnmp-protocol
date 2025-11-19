use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = manifest_path
        .to_str()
        .expect("manifest path contains invalid UTF-8");

    let examples = [
        "type_hints",
        "strict_vs_loose",
        "deterministic_serialization",
        "semantic_checksums",
        "nested_structures",
        "structural_canonicalization",
        "explain_mode",
        "shortform",
        "binary_encoding",
        "binary_roundtrip",
        "llb2_demo",
        "v05_nested_binary",
        "v05_streaming",
        "v05_schema_negotiation",
        "v05_delta_encoding",
        "v05_llb2_binary",
    ];

    for example in examples {
        println!("\n=== Running example: {} ===", example);
        let status = Command::new("cargo")
            .args([
                "run",
                "--offline",
                "--manifest-path",
                manifest,
                "--example",
                example,
            ])
            .status()
            .expect("Failed to run example");

        if !status.success() {
            panic!("Example {} failed with status {:?}", example, status);
        }
    }

    println!("\nAll examples completed successfully");
}
