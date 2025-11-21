use std::{env, error::Error, fs};

use lnmp_codec::{
    container::{
        parse_delta_metadata, parse_stream_metadata, ContainerBuilder, ContainerFrame,
        DeltaMetadata, StreamMetadata,
    },
    Parser,
};
use lnmp_core::{
    LnmpFileMode, LNMP_FLAG_CHECKSUM_REQUIRED, LNMP_FLAG_COMPRESSED, LNMP_FLAG_ENCRYPTED,
    LNMP_FLAG_QKEX, LNMP_FLAG_QSIG,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    match args[0].as_str() {
        "inspect" => inspect_file(parse_inspect_args(&args[1..])?),
        "decode" => decode_file(parse_decode_args(&args[1..])?),
        "encode-text" => encode_container(parse_encode_args(&args[1..])?, LnmpFileMode::Text),
        "encode-binary" => encode_container(parse_encode_args(&args[1..])?, LnmpFileMode::Binary),
        "metadata" => metadata_command(&args[1..]),
        "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        unknown => Err(format!("Unknown command `{unknown}`").into()),
    }
}

fn print_usage() {
    eprintln!(
        "lnmp-cli - LNMP container utilities\n\n\
         Usage:\n  lnmp-cli inspect <file.lnmp> [--metadata-out file] [--metadata-hex]\n  lnmp-cli decode <file.lnmp> [--metadata-out file] [--metadata-hex]\n  lnmp-cli encode-text <input.lnmp> <output.lnmp> [--metadata file] [--flags list]\n  lnmp-cli encode-binary <input.lnmp> <output.lnmp> [--metadata file] [--flags list]\n  lnmp-cli metadata <file.lnmp> --dump out.bin [--raw]\n  lnmp-cli --help"
    );
}

fn inspect_file(opts: InspectArgs) -> Result<(), Box<dyn Error>> {
    let data = fs::read(&opts.path)?;
    let frame = ContainerFrame::parse(&data)?;
    let header = frame.header();

    if let Some(out_path) = opts.metadata_out.as_deref() {
        fs::write(out_path, frame.metadata())?;
        eprintln!("[lnmp-cli] wrote metadata to {out_path}");
    }

    println!("File: {}", opts.path);
    println!("  Mode: {}", format_mode(header.mode));
    println!("  Version: {}", header.version);
    println!("  Flags: {}", format_flags(header.flags));
    println!("  Metadata length: {} bytes", header.metadata_len);
    if !frame.metadata().is_empty() {
        println!(
            "  Metadata preview: {}",
            format_metadata_preview(frame.metadata())
        );
        if opts.metadata_hex {
            println!("  Metadata hex dump: {}", to_hex(frame.metadata()));
        }
        if let Some(details) = describe_metadata(header.mode, frame.metadata()) {
            println!("  Metadata details: {details}");
        }
    } else if opts.metadata_hex {
        println!("  Metadata hex dump: <empty>");
    }
    Ok(())
}

fn decode_file(opts: DecodeArgs) -> Result<(), Box<dyn Error>> {
    let data = fs::read(&opts.path)?;
    let frame = ContainerFrame::parse(&data)?;
    log_decode_context(&frame);
    if let Some(out_path) = opts.metadata_out.as_deref() {
        fs::write(out_path, frame.metadata())?;
        eprintln!("[lnmp-cli] wrote metadata to {out_path}");
    }
    if opts.metadata_hex && !frame.metadata().is_empty() {
        eprintln!("[lnmp-cli] metadata hex dump: {}", to_hex(frame.metadata()));
        if let Some(details) = describe_metadata(frame.header().mode, frame.metadata()) {
            eprintln!("[lnmp-cli] {details}");
        }
    } else if opts.metadata_hex {
        eprintln!("[lnmp-cli] metadata hex dump: <empty>");
    }
    let text = frame.decode_to_text()?;
    println!("{text}");
    Ok(())
}

fn encode_container(opts: EncodeArgs, mode: LnmpFileMode) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&opts.input)?;
    let mut parser = Parser::new(&contents)?;
    let record = parser.parse_record()?;
    let has_checksum_hint = contents.contains('#');
    let mut builder = ContainerBuilder::new(mode)
        .with_flags(opts.flags)
        .with_checksum_confirmation(has_checksum_hint);
    if let Some(path) = opts.metadata.as_deref() {
        let metadata = fs::read(path)?;
        builder = builder.with_metadata(metadata)?;
    } else if mode == LnmpFileMode::Stream {
        builder = builder.with_stream_metadata(default_stream_metadata())?;
    } else if mode == LnmpFileMode::Delta {
        builder = builder.with_delta_metadata(default_delta_metadata())?;
    }
    let bytes = builder.encode_record(&record)?;
    fs::write(&opts.output, bytes)?;
    println!("Wrote {}", opts.output);
    Ok(())
}

fn format_metadata_preview(metadata: &[u8]) -> String {
    const MAX_PREVIEW: usize = 32;
    let mut parts = Vec::new();
    for byte in metadata.iter().take(MAX_PREVIEW) {
        parts.push(format!("{:02X}", byte));
    }
    if metadata.len() > MAX_PREVIEW {
        parts.push("...".to_string());
    }
    parts.join(" ")
}

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
        format!("none (0x{flags:04X})")
    } else {
        format!("{} (0x{flags:04X})", labels.join(", "))
    }
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
        eprintln!("[lnmp-cli] {details}");
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
struct EncodeArgs {
    input: String,
    output: String,
    metadata: Option<String>,
    flags: u16,
}

fn parse_encode_args(args: &[String]) -> Result<EncodeArgs, Box<dyn Error>> {
    if args.len() < 2 {
        return Err("encode command expects at least <input> <output>".into());
    }

    let input = args[0].clone();
    let output = args[1].clone();
    let mut metadata: Option<String> = None;
    let mut flags: u16 = 0;
    let mut idx = 2;

    while idx < args.len() {
        match args[idx].as_str() {
            "--metadata" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("`--metadata` expects a file path".into());
                }
                metadata = Some(args[idx].clone());
            }
            "--flags" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("`--flags` expects comma-separated flag names".into());
                }
                flags = parse_flag_list(&args[idx])?;
            }
            other => return Err(format!("Unknown option `{other}`").into()),
        }
        idx += 1;
    }

    Ok(EncodeArgs {
        input,
        output,
        metadata,
        flags,
    })
}

fn parse_flag_list(list: &str) -> Result<u16, Box<dyn Error>> {
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
            other => return Err(format!("Unknown flag `{other}`").into()),
        };
        bits |= bit;
    }

    Ok(bits)
}

#[derive(Debug)]
struct InspectArgs {
    path: String,
    metadata_out: Option<String>,
    metadata_hex: bool,
}

fn parse_inspect_args(args: &[String]) -> Result<InspectArgs, Box<dyn Error>> {
    if args.is_empty() {
        return Err("inspect expects <file.lnmp>".into());
    }
    let mut opts = InspectArgs {
        path: args[0].clone(),
        metadata_out: None,
        metadata_hex: false,
    };
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--metadata-out" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("`--metadata-out` expects a file path".into());
                }
                opts.metadata_out = Some(args[idx].clone());
            }
            "--metadata-hex" => {
                opts.metadata_hex = true;
            }
            other => return Err(format!("Unknown option `{other}`").into()),
        }
        idx += 1;
    }
    Ok(opts)
}

#[derive(Debug)]
struct DecodeArgs {
    path: String,
    metadata_out: Option<String>,
    metadata_hex: bool,
}

fn parse_decode_args(args: &[String]) -> Result<DecodeArgs, Box<dyn Error>> {
    if args.is_empty() {
        return Err("decode expects <file.lnmp>".into());
    }
    let mut opts = DecodeArgs {
        path: args[0].clone(),
        metadata_out: None,
        metadata_hex: false,
    };
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--metadata-out" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("`--metadata-out` expects a file path".into());
                }
                opts.metadata_out = Some(args[idx].clone());
            }
            "--metadata-hex" => opts.metadata_hex = true,
            other => return Err(format!("Unknown option `{other}`").into()),
        }
        idx += 1;
    }
    Ok(opts)
}
fn metadata_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        return Err("metadata expects <file.lnmp>".into());
    }
    let path = args[0].clone();
    let mut dump: Option<String> = None;
    let mut raw = false;
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--dump" => {
                idx += 1;
                if idx >= args.len() {
                    return Err("`--dump` expects a file path".into());
                }
                dump = Some(args[idx].clone());
            }
            "--raw" => raw = true,
            other => return Err(format!("Unknown option `{other}`").into()),
        }
        idx += 1;
    }

    let data = fs::read(&path)?;
    let frame = ContainerFrame::parse(&data)?;
    let metadata = frame.metadata();

    if metadata.is_empty() {
        println!("metadata: <empty>");
        return Ok(());
    }

    if let Some(out) = dump.as_deref() {
        fs::write(out, metadata)?;
        println!("metadata dumped to {out}");
    }

    if raw {
        println!("{}", to_hex(metadata));
    } else {
        println!("{}", format_metadata_preview(metadata));
    }
    if let Some(details) = describe_metadata(frame.header().mode, metadata) {
        println!("{details}");
    }

    Ok(())
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
            Err(err) => format!("stream metadata parse error: {err}"),
        }),
        LnmpFileMode::Delta => Some(match parse_delta_metadata(metadata) {
            Ok(meta) => format!(
                "delta metadata: base_snapshot={}, algorithm={}, compression={}",
                meta.base_snapshot,
                delta_algorithm_name(meta.algorithm),
                delta_compression_name(meta.compression)
            ),
            Err(err) => format!("delta metadata parse error: {err}"),
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
