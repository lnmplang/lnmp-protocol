use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::codec::binary::BinaryDecoder;
use lnmp::codec::{Encoder, EncoderConfig, Parser};
use lnmp::core::LnmpProfile;
use std::path::PathBuf;

use crate::utils::{read_file, read_text, write_text};

#[derive(Args)]
pub struct CodecCmd {
    #[command(subcommand)]
    pub command: CodecSubcommand,
}

#[derive(Subcommand)]
pub enum CodecSubcommand {
    /// Parse LNMP text and display AST
    Parse {
        /// Input LNMP text file
        input: PathBuf,

        /// Show detailed AST
        #[arg(long)]
        detailed: bool,
    },

    /// Format/canonicalize LNMP text
    Format {
        /// Input LNMP text file
        input: PathBuf,

        /// Output file (defaults to stdout)
        output: Option<PathBuf>,

        /// Enable checksums
        #[arg(long)]
        checksums: bool,
    },

    /// Validate LNMP syntax
    Validate {
        /// Input LNMP text file
        input: PathBuf,

        /// Validation profile (loose/standard/strict)
        #[arg(long, default_value = "standard")]
        profile: String,
    },

    /// Add checksums to fields
    Checksum {
        /// Input LNMP text file
        input: PathBuf,

        /// Output file
        output: PathBuf,
    },

    /// Normalize values
    Normalize {
        /// Input LNMP text file
        input: PathBuf,

        /// Output file
        output: PathBuf,
    },
}

impl CodecCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            CodecSubcommand::Parse { input, detailed } => parse(input, *detailed),
            CodecSubcommand::Format {
                input,
                output,
                checksums,
            } => format_text(input, output.as_ref(), *checksums),
            CodecSubcommand::Validate { input, profile } => validate(input, profile),
            CodecSubcommand::Checksum { input, output } => checksum(input, output),
            CodecSubcommand::Normalize { input, output } => normalize(input, output),
        }
    }
}

fn parse(input: &PathBuf, detailed: bool) -> Result<()> {
    // Read file as bytes first
    let data = read_file(input)?;
    
    // Auto-detect format: binary codec starts with magic bytes
    // Binary codec format: [varint_len][varint_field_count][fields...]
    // First byte is typically 0x04 or similar (small varint)
    let is_binary = data.len() >= 2 && data[0] < 0x20 && data[1] < 0x20;
    
    let record = if is_binary {
        // Binary codec - use BinaryDecoder
        let decoder = BinaryDecoder::new();
        decoder.decode(&data)
            .map_err(|e| anyhow::anyhow!("Binary decode failed: {}", e))?
    } else {
        // Text codec - use Parser
        let text = String::from_utf8(data)
            .map_err(|e| anyhow::anyhow!("Not valid UTF-8 text: {}", e))?;
        let mut parser = Parser::new(&text)?;
        parser.parse_record()?
    };

    println!("Parsed successfully!");
    println!("Fields: {}", record.fields().len());

    if detailed {
        println!("\nRecord AST:");
        for field in record.fields() {
            println!("  F{} = {:?}", field.fid, field.value);
        }
    } else {
        println!(
            "FIDs: {:?}",
            record.fields().iter().map(|f| f.fid).collect::<Vec<_>>()
        );
    }

    Ok(())
}

fn format_text(input: &PathBuf, output: Option<&PathBuf>, checksums: bool) -> Result<()> {
    let text = read_text(input)?;
    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    let config = EncoderConfig {
        enable_checksums: checksums,
        ..Default::default()
    };
    let encoder = Encoder::with_config(config);
    let formatted = encoder.encode(&record);

    if let Some(out_path) = output {
        write_text(out_path, &formatted)?;
        println!("Formatted text written to {}", out_path.display());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

fn validate(input: &PathBuf, profile: &str) -> Result<()> {
    let text = read_text(input)?;

    let profile_mode = match profile {
        "loose" => LnmpProfile::Loose,
        "standard" => LnmpProfile::Standard,
        "strict" => LnmpProfile::Strict,
        _ => anyhow::bail!("Invalid profile: {} (use loose/standard/strict)", profile),
    };

    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    // Validate field ordering for strict mode
    if profile_mode == LnmpProfile::Strict {
        let mut prev_fid = 0;
        for field in record.fields() {
            if field.fid <= prev_fid {
                anyhow::bail!(
                    "Strict validation failed: Field F{} appears after F{} (not in canonical order)",
                    field.fid,
                    prev_fid
                );
            }
            prev_fid = field.fid;
        }
    }

    println!("✓ Validation passed ({} profile)", profile);
    println!("  Fields: {}", record.fields().len());

    Ok(())
}

fn checksum(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let text = read_text(input)?;
    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    let config = EncoderConfig {
        enable_checksums: true,
        ..Default::default()
    };
    let encoder = Encoder::with_config(config);
    let checksummed = encoder.encode(&record);

    write_text(output, &checksummed)?;
    println!("Checksums added, written to {}", output.display());

    Ok(())
}

fn normalize(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let text = read_text(input)?;
    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    // Re-encode to normalize values
    let encoder = Encoder::new();
    let normalized = encoder.encode(&record);

    write_text(output, &normalized)?;
    println!("Values normalized, written to {}", output.display());

    Ok(())
}
