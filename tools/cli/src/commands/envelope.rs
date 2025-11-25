use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::{read_file, write_file};

#[derive(Args)]
pub struct EnvelopeCmd {
    #[command(subcommand)]
    pub command: EnvelopeSubcommand,
}

#[derive(Subcommand)]
pub enum EnvelopeSubcommand {
    /// Create envelope with metadata
    Create {
        /// Source identifier
        #[arg(long)]
        source: String,

        /// Trace ID (optional, auto-generated if not provided)
        #[arg(long)]
        trace_id: Option<String>,

        /// Sequence number
        #[arg(long, default_value = "0")]
        sequence: u64,

        /// Output file
        output: PathBuf,
    },

    /// Wrap record with envelope
    Wrap {
        /// Input LNMP record file
        input: PathBuf,

        /// Output wrapped file
        output: PathBuf,

        /// Source identifier
        #[arg(long)]
        source: String,

        /// Trace ID
        #[arg(long)]
        trace_id: Option<String>,
    },

    /// Unwrap envelope and extract record
    Unwrap {
        /// Input wrapped file
        input: PathBuf,

        /// Output record file
        output: PathBuf,

        /// Also output envelope metadata to JSON
        #[arg(long)]
        envelope_out: Option<PathBuf>,
    },

    /// Extract envelope metadata only
    Extract {
        /// Input wrapped file
        input: PathBuf,

        /// Output envelope JSON file
        output: PathBuf,
    },
}

impl EnvelopeCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            EnvelopeSubcommand::Create {
                source,
                trace_id,
                sequence,
                output,
            } => create(source, trace_id.as_deref(), *sequence, output),
            EnvelopeSubcommand::Wrap {
                input,
                output,
                source,
                trace_id,
            } => wrap(input, output, source, trace_id.as_deref()),
            EnvelopeSubcommand::Unwrap {
                input,
                output,
                envelope_out,
            } => unwrap(input, output, envelope_out.as_ref()),
            EnvelopeSubcommand::Extract { input, output } => extract(input, output),
        }
    }
}

fn create(source: &str, trace_id: Option<&str>, sequence: u64, output: &PathBuf) -> Result<()> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

    let trace = trace_id
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let envelope = serde_json::json!({
        "timestamp": timestamp,
        "source": source,
        "trace_id": trace,
        "sequence": sequence,
    });

    write_file(output, serde_json::to_string_pretty(&envelope)?.as_bytes())?;

    println!("Created envelope:");
    println!("  Timestamp: {}", timestamp);
    println!("  Source: {}", source);
    println!("  Trace ID: {}", trace);
    println!("  Sequence: {}", sequence);

    Ok(())
}

fn wrap(input: &PathBuf, output: &PathBuf, source: &str, trace_id: Option<&str>) -> Result<()> {
    let record_data = read_file(input)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    let trace = trace_id
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Simple wrapper format: envelope JSON + separator + record data
    let envelope = serde_json::json!({
        "timestamp": timestamp,
        "source": source,
        "trace_id": trace,
        "sequence": 0,
    });

    let envelope_json = serde_json::to_string(&envelope)?;
    let separator = b"\n---LNMP-ENVELOPE-SEPARATOR---\n";

    let mut wrapped = Vec::new();
    wrapped.extend_from_slice(envelope_json.as_bytes());
    wrapped.extend_from_slice(separator);
    wrapped.extend_from_slice(&record_data);

    write_file(output, &wrapped)?;

    println!("Wrapped record with envelope:");
    println!("  Source: {}", source);
    println!("  Trace ID: {}", trace);

    Ok(())
}

fn unwrap(input: &PathBuf, output: &PathBuf, envelope_out: Option<&PathBuf>) -> Result<()> {
    let data = read_file(input)?;
    let separator = b"\n---LNMP-ENVELOPE-SEPARATOR---\n";

    // Find separator
    let pos = data
        .windows(separator.len())
        .position(|window| window == separator)
        .ok_or_else(|| anyhow::anyhow!("Invalid wrapped format: separator not found"))?;

    let envelope_data = &data[..pos];
    let record_data = &data[pos + separator.len()..];

    if let Some(env_path) = envelope_out {
        write_file(env_path, envelope_data)?;
        println!("Envelope metadata written to {}", env_path.display());
    }

    write_file(output, record_data)?;
    println!("Record extracted to {}", output.display());

    Ok(())
}

fn extract(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let data = read_file(input)?;
    let separator = b"\n---LNMP-ENVELOPE-SEPARATOR---\n";

    let pos = data
        .windows(separator.len())
        .position(|window| window == separator)
        .ok_or_else(|| anyhow::anyhow!("Invalid wrapped format: separator not found"))?;

    let envelope_data = &data[..pos];
    write_file(output, envelope_data)?;

    let envelope: serde_json::Value = serde_json::from_slice(envelope_data)?;
    println!("Extracted envelope:");
    println!("{}", serde_json::to_string_pretty(&envelope)?);

    Ok(())
}
