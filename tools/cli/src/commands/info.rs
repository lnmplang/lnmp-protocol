use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::codec::Parser;
use std::path::PathBuf;

use crate::utils::{format_bytes, read_text};

#[derive(Args)]
pub struct InfoCmd {
    #[command(subcommand)]
    pub command: InfoSubcommand,
}

#[derive(Subcommand)]
pub enum InfoSubcommand {
    /// Show version and build info
    Version,

    /// List supported features
    Features,

    /// Show file/record statistics
    Stats {
        /// Input LNMP file
        input: PathBuf,

        /// Show detailed breakdown
        #[arg(long)]
        detailed: bool,
    },

    /// Context profiling scores
    Profile {
        /// Input LNMP file
        input: PathBuf,

        /// Show profiling scores
        #[arg(long)]
        scores: bool,
    },
}

impl InfoCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            InfoSubcommand::Version => version(),
            InfoSubcommand::Features => features(),
            InfoSubcommand::Stats { input, detailed } => stats(input, *detailed),
            InfoSubcommand::Profile { input, scores } => profile(input, *scores),
        }
    }
}

fn version() -> Result<()> {
    println!("lnmp-cli {}", env!("CARGO_PKG_VERSION"));
    println!("LNMP Protocol v0.5.7");
    println!();
    println!("Build info:");
    println!("  Rust: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!(
        "  Profile: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    Ok(())
}

fn features() -> Result<()> {
    println!("Supported LNMP features:");
    println!();
    println!("Core:");
    println!("  ✓ Text format (v0.1)");
    println!("  ✓ Binary format (v0.4)");
    println!("  ✓ Nested structures (v0.3)");
    println!("  ✓ Generic arrays (v0.5.5)");
    println!();
    println!("Container modes:");
    println!("  ✓ Text");
    println!("  ✓ Binary");
    println!("  ✓ Stream");
    println!("  ✓ Delta");
    println!("  ✓ Quantum-Safe");
    println!("  ✓ Embedding");
    println!("  ✓ Spatial");
    println!();
    println!("Advanced:");
    println!("  ✓ Semantic checksums (SC32)");
    println!("  ✓ Embedding vectors with delta encoding");
    println!("  ✓ Spatial data (Position, Rotation, Velocity, etc.)");
    println!("  ✓ Quantization (FP16, QInt8, QInt4, Binary)");
    println!("  ✓ Transport bindings (HTTP, Kafka, gRPC, NATS)");
    println!("  ✓ Envelope metadata");
    println!();
    println!("Validation:");
    println!("  ✓ Loose / Standard / Strict profiles");
    println!("  ✓ Input sanitization");

    Ok(())
}

fn stats(input: &PathBuf, detailed: bool) -> Result<()> {
    let text = read_text(input)?;
    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    println!("File: {}", input.display());
    println!("Statistics:");
    println!("  File size: {}", format_bytes(text.len()));
    println!("  Fields: {}", record.fields().len());

    if detailed {
        println!("\nField breakdown:");
        let mut int_count = 0;
        let mut float_count = 0;
        let mut bool_count = 0;
        let mut str_count = 0;
        let mut array_count = 0;
        let mut nested_count = 0;

        for field in record.fields() {
            use lnmp::core::LnmpValue;
            match &field.value {
                LnmpValue::Int(_) => int_count += 1,
                LnmpValue::Float(_) => float_count += 1,
                LnmpValue::Bool(_) => bool_count += 1,
                LnmpValue::String(_) => str_count += 1,
                LnmpValue::StringArray(_)
                | LnmpValue::IntArray(_)
                | LnmpValue::FloatArray(_)
                | LnmpValue::BoolArray(_) => array_count += 1,
                LnmpValue::NestedRecord(_) | LnmpValue::NestedArray(_) => nested_count += 1,
                LnmpValue::Embedding(_)
                | LnmpValue::EmbeddingDelta(_)
                | LnmpValue::QuantizedEmbedding(_) => array_count += 1,
            }
        }

        println!("  Integers: {}", int_count);
        println!("  Floats: {}", float_count);
        println!("  Booleans: {}", bool_count);
        println!("  Strings: {}", str_count);
        println!("  Arrays: {}", array_count);
        println!("  Nested: {}", nested_count);
    }

    Ok(())
}

fn profile(_input: &PathBuf, _scores: bool) -> Result<()> {
    println!("Context profiling not yet implemented");
    println!("(Would use lnmp-sfe for freshness, importance, risk, confidence scores)");
    Ok(())
}
