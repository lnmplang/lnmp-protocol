use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::quant::adaptive::{quantize_adaptive, AccuracyTarget};
use lnmp::quant::{dequantize_embedding, quantize_embedding, QuantScheme};
use std::path::PathBuf;

use crate::utils::{format_bytes, read_file, write_file};

#[derive(Args)]
pub struct QuantCmd {
    #[command(subcommand)]
    pub command: QuantSubcommand,
}

#[derive(Subcommand)]
pub enum QuantSubcommand {
    /// Quantize embedding vector
    Quantize {
        /// Input vector file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Quantization scheme (fp16/qint8/qint4/binary)
        #[arg(long)]
        scheme: String,
    },

    /// Dequantize to full precision
    Dequantize {
        /// Input quantized file
        input: PathBuf,

        /// Output vector file
        output: PathBuf,
    },

    /// Auto-select scheme for accuracy target
    Adaptive {
        /// Input vector file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Accuracy target (maximum/high/balanced/compact)
        #[arg(long)]
        target: String,
    },

    /// Process multiple vectors in batch
    Batch {
        /// Input directory or file list
        input: PathBuf,

        /// Output directory
        output: PathBuf,

        /// Quantization scheme
        #[arg(long)]
        scheme: String,
    },

    /// Benchmark quantization schemes  
    Benchmark {
        /// Input vector file
        input: PathBuf,
    },
}

impl QuantCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            QuantSubcommand::Quantize {
                input,
                output,
                scheme,
            } => quantize(input, output, scheme),
            QuantSubcommand::Dequantize { input, output } => dequantize(input, output),
            QuantSubcommand::Adaptive {
                input,
                output,
                target,
            } => adaptive(input, output, target),
            QuantSubcommand::Batch {
                input,
                output,
                scheme,
            } => batch(input, output, scheme),
            QuantSubcommand::Benchmark { input } => benchmark(input),
        }
    }
}

fn quantize(input: &PathBuf, output: &PathBuf, scheme_str: &str) -> Result<()> {
    use lnmp::embedding::Decoder;

    let data = read_file(input)?;
    let vector = Decoder::decode(&data)?;

    let scheme = parse_scheme(scheme_str)?;
    let quantized = quantize_embedding(&vector, scheme)?;

    let encoded = serde_json::to_vec(&quantized)?;
    write_file(output, &encoded)?;

    let orig_size = vector.dim as usize * 4;
    let quant_size = quantized.data_size();
    println!("Quantized with scheme: {:?}", scheme);
    println!("  Original size: {}", format_bytes(orig_size));
    println!("  Quantized size: {}", format_bytes(quant_size));
    println!("  Compression ratio: {:.1}x", quantized.compression_ratio());

    Ok(())
}

fn dequantize(input: &PathBuf, output: &PathBuf) -> Result<()> {
    use lnmp::embedding::Encoder;

    let data = read_file(input)?;
    let quantized = serde_json::from_slice(&data)?;

    let restored = dequantize_embedding(&quantized)?;
    let encoded = Encoder::encode(&restored)?;

    write_file(output, &encoded)?;
    println!("Dequantized to {} dimensions", restored.dim);

    Ok(())
}

fn adaptive(input: &PathBuf, output: &PathBuf, target_str: &str) -> Result<()> {
    use lnmp::embedding::Decoder;

    let data = read_file(input)?;
    let vector = Decoder::decode(&data)?;

    let target = match target_str {
        "maximum" => AccuracyTarget::Maximum,
        "high" => AccuracyTarget::High,
        "balanced" => AccuracyTarget::Balanced,
        "compact" => AccuracyTarget::Compact,
        _ => anyhow::bail!(
            "Invalid target: {} (use maximum/high/balanced/compact)",
            target_str
        ),
    };

    let quantized = quantize_adaptive(&vector, target)?;
    let encoded = serde_json::to_vec(&quantized)?;
    write_file(output, &encoded)?;

    println!("Auto-selected scheme for {:?} accuracy", target);
    println!("  Compression ratio: {:.1}x", quantized.compression_ratio());

    Ok(())
}

fn batch(_input: &PathBuf, _output: &PathBuf, scheme_str: &str) -> Result<()> {
    let scheme = parse_scheme(scheme_str)?;
    println!("Batch processing with scheme: {:?}", scheme);
    println!("(Batch processing not yet implemented)");
    Ok(())
}

fn benchmark(input: &PathBuf) -> Result<()> {
    use lnmp::embedding::Decoder;

    let data = read_file(input)?;
    let vector = Decoder::decode(&data)?;

    println!(
        "Benchmarking quantization schemes on {} dimensions\n",
        vector.dim
    );

    for scheme in &[
        QuantScheme::FP16Passthrough,
        QuantScheme::QInt8,
        QuantScheme::QInt4,
        QuantScheme::Binary,
    ] {
        let quantized = quantize_embedding(&vector, *scheme)?;
        println!("{:?}:", scheme);
        println!("  Compression: {:.1}x", quantized.compression_ratio());
        println!("  Size: {}", format_bytes(quantized.data_size()));
    }

    Ok(())
}

fn parse_scheme(s: &str) -> Result<QuantScheme> {
    match s.to_lowercase().as_str() {
        "fp16" => Ok(QuantScheme::FP16Passthrough),
        "qint8" => Ok(QuantScheme::QInt8),
        "qint4" => Ok(QuantScheme::QInt4),
        "binary" => Ok(QuantScheme::Binary),
        _ => anyhow::bail!("Invalid scheme: {} (use fp16/qint8/qint4/binary)", s),
    }
}
