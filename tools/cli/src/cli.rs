use clap::{Parser, Subcommand};

use crate::commands::{
    codec::CodecCmd, container::ContainerCmd, convert::ConvertCmd, embedding::EmbeddingCmd,
    envelope::EnvelopeCmd, info::InfoCmd, perf::PerfCmd, quant::QuantCmd, spatial::SpatialCmd,
    transport::TransportCmd, validate::ValidateCmd,
};

/// LNMP CLI - Command-line tools for LNMP protocol
#[derive(Parser)]
#[command(name = "lnmp-cli")]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Container file operations (.lnmp files)
    Container(ContainerCmd),

    /// Text codec operations (parse, format, validate)
    Codec(CodecCmd),

    /// Vector embedding operations (encode, decode, delta)
    Embedding(EmbeddingCmd),

    /// Spatial data operations (position, rotation, streaming)
    Spatial(SpatialCmd),

    /// Quantization operations (compress embeddings)
    Quant(QuantCmd),

    /// Transport protocol operations (HTTP, Kafka, gRPC, NATS)
    Transport(TransportCmd),

    /// Envelope metadata operations (wrap, unwrap, extract)
    Envelope(EnvelopeCmd),

    /// Format conversion utilities (JSON, binary, shortform)
    Convert(ConvertCmd),

    /// Information and diagnostics (version, features, stats)
    Info(InfoCmd),

    /// Validation and security (sanitize, check, compliance)
    Validate(ValidateCmd),

    /// Performance benchmarking and comparison
    Perf(PerfCmd),

    /// Interactive Terminal UI
    Tui,
}
