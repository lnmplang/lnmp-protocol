use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::codec::container::{
    parse_delta_metadata, parse_stream_metadata, ContainerBuilder, ContainerFrame, DeltaMetadata,
    StreamMetadata,
};
use lnmp::codec::Parser;
use lnmp::core::{
    LnmpFileMode, LNMP_FLAG_CHECKSUM_REQUIRED, LNMP_FLAG_COMPRESSED, LNMP_FLAG_ENCRYPTED,
    LNMP_FLAG_QKEX, LNMP_FLAG_QSIG,
};
use std::path::PathBuf;

use crate::utils::{hex_dump, hex_preview, read_file, read_text, write_file};

#[derive(Args)]
pub struct ContainerCmd {
    #[command(subcommand)]
    pub command: ContainerSubcommand,
}

#[derive(Subcommand)]
pub enum ContainerSubcommand {
    /// Inspect container header and metadata
    Inspect(InspectArgs),

    /// Decode container to text
    Decode(DecodeArgs),

    /// Encode to container
    Encode {
        #[command(subcommand)]
        mode: EncodeMode,
    },

    /// Extract metadata from container
    Metadata(MetadataArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    /// Input .lnmp container file
    pub input: PathBuf,

    /// Write metadata to file
    #[arg(long)]
    pub metadata_out: Option<PathBuf>,

    /// Show full metadata hex dump
    #[arg(long)]
    pub metadata_hex: bool,
}

#[derive(Args)]
pub struct DecodeArgs {
    /// Input .lnmp container file
    pub input: PathBuf,

    /// Write metadata to file
    #[arg(long)]
    pub metadata_out: Option<PathBuf>,

    /// Show full metadata hex dump
    #[arg(long)]
    pub metadata_hex: bool,

    /// Suppress diagnostic output (only output decoded data)
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum EncodeMode {
    /// Encode as text mode
    Text(EncodeArgs),

    /// Encode as binary mode
    Binary(EncodeArgs),

    /// Encode as stream mode
    Stream(EncodeArgs),

    /// Encode as delta mode
    Delta(EncodeArgs),
}

#[derive(Args)]
pub struct EncodeArgs {
    /// Input LNMP text file
    pub input: PathBuf,

    /// Output .lnmp container file
    pub output: PathBuf,

    /// Metadata binary file to attach
    #[arg(long)]
    pub metadata: Option<PathBuf>,

    /// Flags (comma-separated: checksum,compressed,encrypted,qsig,qkex)
    #[arg(long)]
    pub flags: Option<String>,
}

#[derive(Args)]
pub struct MetadataArgs {
    /// Input .lnmp container file
    pub input: PathBuf,

    /// Write metadata to file
    #[arg(long)]
    pub dump: Option<PathBuf>,

    /// Show full hex dump instead of preview
    #[arg(long)]
    pub raw: bool,
}

impl ContainerCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            ContainerSubcommand::Inspect(args) => inspect(args),
            ContainerSubcommand::Decode(args) => decode(args),
            ContainerSubcommand::Encode { mode } => encode(mode),
            ContainerSubcommand::Metadata(args) => metadata(args),
        }
    }
}

fn inspect(args: &InspectArgs) -> Result<()> {
    let data = read_file(&args.input)?;
    let frame = ContainerFrame::parse(&data)?;
    let header = frame.header();

    if let Some(out_path) = &args.metadata_out {
        write_file(out_path, frame.metadata())?;
        eprintln!("[lnmp-cli] Wrote metadata to {}", out_path.display());
    }

    println!("File: {}", args.input.display());
    println!("  Mode: {}", format_mode(header.mode));
    println!("  Version: {}", header.version);
    println!("  Flags: {}", format_flags(header.flags));
    println!("  Metadata length: {} bytes", header.metadata_len);

    if !frame.metadata().is_empty() {
        println!("  Metadata preview: {}", hex_preview(frame.metadata(), 32));
        if args.metadata_hex {
            println!("  Metadata hex dump: {}", hex_dump(frame.metadata()));
        }
        if let Some(details) = describe_metadata(header.mode, frame.metadata()) {
            println!("  Metadata details: {}", details);
        }
    } else if args.metadata_hex {
        println!("  Metadata hex dump: <empty>");
    }

    Ok(())
}

fn decode(args: &DecodeArgs) -> Result<()> {
    let data = read_file(&args.input)?;
    let frame = ContainerFrame::parse(&data)?;

    if !args.quiet {
        log_decode_context(&frame);
    }

    if let Some(out_path) = &args.metadata_out {
        write_file(out_path, frame.metadata())?;
        eprintln!("[lnmp-cli] Wrote metadata to {}", out_path.display());
    }

    if args.metadata_hex && !frame.metadata().is_empty() {
        eprintln!(
            "[lnmp-cli] metadata hex dump: {}",
            hex_dump(frame.metadata())
        );
        if let Some(details) = describe_metadata(frame.header().mode, frame.metadata()) {
            eprintln!("[lnmp-cli] {}", details);
        }
    } else if args.metadata_hex {
        eprintln!("[lnmp-cli] metadata hex dump: <empty>");
    }

    let text = frame.decode_to_text()?;
    println!("{}", text);

    Ok(())
}

fn encode(mode: &EncodeMode) -> Result<()> {
    let (args, file_mode) = match mode {
        EncodeMode::Text(args) => (args, LnmpFileMode::Text),
        EncodeMode::Binary(args) => (args, LnmpFileMode::Binary),
        EncodeMode::Stream(args) => (args, LnmpFileMode::Stream),
        EncodeMode::Delta(args) => (args, LnmpFileMode::Delta),
    };

    let contents = read_text(&args.input)?;
    let mut parser = Parser::new(&contents)?;
    let record = parser.parse_record()?;

    // Warn if record has no fields (common mistake with # comments)
    if record.fields().is_empty() {
        eprintln!("[lnmp-cli] warning: record has 0 fields - check LNMP syntax");
        eprintln!("[lnmp-cli] note: use 'F1:value' format, not '#field:value' (# is for comments)");
    }

    let has_checksum_hint = contents.contains('#');
    let flags = if let Some(flag_str) = &args.flags {
        parse_flags(flag_str)?
    } else {
        0
    };

    let mut builder = ContainerBuilder::new(file_mode)
        .with_flags(flags)
        .with_checksum_confirmation(has_checksum_hint);

    if let Some(metadata_path) = &args.metadata {
        let metadata = read_file(metadata_path)?;
        builder = builder.with_metadata(metadata)?;
    } else if file_mode == LnmpFileMode::Stream {
        builder = builder.with_stream_metadata(default_stream_metadata())?;
    } else if file_mode == LnmpFileMode::Delta {
        builder = builder.with_delta_metadata(default_delta_metadata())?;
    }

    let bytes = builder.encode_record(&record)?;
    write_file(&args.output, &bytes)?;

    println!("Wrote {} ({} bytes)", args.output.display(), bytes.len());
    Ok(())
}

fn metadata(args: &MetadataArgs) -> Result<()> {
    let data = read_file(&args.input)?;
    let frame = ContainerFrame::parse(&data)?;
    let metadata = frame.metadata();

    if metadata.is_empty() {
        println!("metadata: <empty>");
        return Ok(());
    }

    if let Some(out_path) = &args.dump {
        write_file(out_path, metadata)?;
        println!("metadata dumped to {}", out_path.display());
    }

    if args.raw {
        println!("{}", hex_dump(metadata));
    } else {
        println!("{}", hex_preview(metadata, 32));
    }

    if let Some(details) = describe_metadata(frame.header().mode, metadata) {
        println!("{}", details);
    }

    Ok(())
}

// Helper functions

fn format_mode(mode: LnmpFileMode) -> &'static str {
    match mode {
        LnmpFileMode::Text => "LNMP/Text",
        LnmpFileMode::Binary => "LNMP/Binary",
        LnmpFileMode::Stream => "LNMP/Stream",
        LnmpFileMode::Delta => "LNMP/Delta",
        LnmpFileMode::QuantumSafe => "LNMP/Quantum-Safe",
        LnmpFileMode::Embedding => "LNMP/Embedding",
        LnmpFileMode::Spatial => "LNMP/Spatial",
    }
}

fn format_flags(flags: u16) -> String {
    let mut labels = Vec::new();
    if flags & LNMP_FLAG_CHECKSUM_REQUIRED != 0 {
        labels.push("checksum");
    }
    if flags & LNMP_FLAG_COMPRESSED != 0 {
        labels.push("compressed");
    }
    if flags & LNMP_FLAG_ENCRYPTED != 0 {
        labels.push("encrypted");
    }
    if flags & LNMP_FLAG_QSIG != 0 {
        labels.push("qsig");
    }
    if flags & LNMP_FLAG_QKEX != 0 {
        labels.push("qkex");
    }

    if labels.is_empty() {
        format!("none (0x{:04X})", flags)
    } else {
        format!("{} (0x{:04X})", labels.join(", "), flags)
    }
}

fn parse_flags(list: &str) -> Result<u16> {
    let mut bits: u16 = 0;
    if list.trim().is_empty() {
        return Ok(bits);
    }

    for raw in list.split(',') {
        let flag = raw.trim().to_ascii_lowercase();
        if flag.is_empty() {
            continue;
        }
        let bit = match flag.as_str() {
            "checksum" => LNMP_FLAG_CHECKSUM_REQUIRED,
            "compressed" => LNMP_FLAG_COMPRESSED,
            "encrypted" => LNMP_FLAG_ENCRYPTED,
            "qsig" => LNMP_FLAG_QSIG,
            "qkex" => LNMP_FLAG_QKEX,
            other => anyhow::bail!("Unknown flag: {}", other),
        };
        bits |= bit;
    }

    Ok(bits)
}

fn log_decode_context(frame: &ContainerFrame) {
    let header = frame.header();
    eprintln!(
        "[lnmp-cli] mode={} flags={} metadata={} bytes",
        format_mode(header.mode),
        format_flags(header.flags),
        header.metadata_len
    );

    if header.flags & LNMP_FLAG_CHECKSUM_REQUIRED != 0 {
        eprintln!("[lnmp-cli] note: checksum flag is set; CLI does not validate checksums yet.");
    }

    let unsupported = header.flags & (LNMP_FLAG_COMPRESSED | LNMP_FLAG_ENCRYPTED);
    if unsupported != 0 {
        eprintln!(
            "[lnmp-cli] warning: flags {} require compression/encryption which is not implemented yet.",
            format_flags(unsupported)
        );
    }

    let quantum_bits = header.flags & (LNMP_FLAG_QSIG | LNMP_FLAG_QKEX);
    if quantum_bits != 0 && frame.metadata().is_empty() {
        eprintln!(
            "[lnmp-cli] warning: quantum-safe flags are set but no metadata bytes are present."
        );
    }

    if let Some(details) = describe_metadata(header.mode, frame.metadata()) {
        eprintln!("[lnmp-cli] {}", details);
    }
}

fn describe_metadata(mode: LnmpFileMode, metadata: &[u8]) -> Option<String> {
    if metadata.is_empty() {
        return None;
    }
    match mode {
        LnmpFileMode::Stream => Some(match parse_stream_metadata(metadata) {
            Ok(meta) => format!(
                "stream metadata: chunk_size={} bytes, checksum={}, flags={:#04X}",
                meta.chunk_size,
                stream_checksum_name(meta.checksum_type),
                meta.flags
            ),
            Err(err) => format!("stream metadata parse error: {}", err),
        }),
        LnmpFileMode::Delta => Some(match parse_delta_metadata(metadata) {
            Ok(meta) => format!(
                "delta metadata: base_snapshot={}, algorithm={}, compression={}",
                meta.base_snapshot,
                delta_algorithm_name(meta.algorithm),
                delta_compression_name(meta.compression)
            ),
            Err(err) => format!("delta metadata parse error: {}", err),
        }),
        _ => None,
    }
}

fn stream_checksum_name(code: u8) -> &'static str {
    match code {
        0x00 => "none",
        0x01 => "xor32",
        0x02 => "sc32",
        _ => "unknown",
    }
}

fn delta_algorithm_name(code: u8) -> &'static str {
    match code {
        0x00 => "op-list",
        0x01 => "merge",
        _ => "unknown",
    }
}

fn delta_compression_name(code: u8) -> &'static str {
    match code {
        0x00 => "none",
        0x01 => "varint",
        _ => "unknown",
    }
}

fn default_stream_metadata() -> StreamMetadata {
    StreamMetadata {
        chunk_size: 4096,
        checksum_type: 0x02, // SC32
        flags: 0x00,
    }
}

fn default_delta_metadata() -> DeltaMetadata {
    DeltaMetadata {
        base_snapshot: 0,
        algorithm: 0x00,
        compression: 0x00,
    }
}
