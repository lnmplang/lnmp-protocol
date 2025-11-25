use anyhow::Result;
use clap::Args;
use lnmp_envelope::LnmpEnvelope;
use lnmp_net::{MessageKind, NetMessageBuilder};
use std::path::PathBuf;

use crate::utils::{read_file, write_file};

#[derive(Args)]
pub struct CreateCmd {
    /// Input LNMP envelope file
    input: PathBuf,

    /// Message kind
    #[arg(long, value_enum)]
    kind: MessageKindArg,

    /// Priority (0-255, default based on kind)
    #[arg(long)]
    priority: Option<u8>,

    /// TTL in milliseconds (default based on kind)
    #[arg(long)]
    ttl: Option<u64>,

    /// Domain class (optional)
    #[arg(long)]
    class: Option<String>,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Clone, clap::ValueEnum)]
enum MessageKindArg {
    Event,
    State,
    Command,
    Query,
    Alert,
}

impl From<MessageKindArg> for MessageKind {
    fn from(arg: MessageKindArg) -> Self {
        match arg {
            MessageKindArg::Event => MessageKind::Event,
            MessageKindArg::State => MessageKind::State,
            MessageKindArg::Command => MessageKind::Command,
            MessageKindArg::Query => MessageKind::Query,
            MessageKindArg::Alert => MessageKind::Alert,
        }
    }
}

impl CreateCmd {
    pub fn execute(&self) -> Result<()> {
        // Read envelope
        let data = read_file(&self.input)?;
        let envelope: LnmpEnvelope = bincode::deserialize(&data)?;

        // Convert kind
        let kind: MessageKind = self.kind.clone().into();

        // Create network message
        let mut builder = NetMessageBuilder::new(envelope, kind);

        if let Some(priority) = self.priority {
            builder = builder.priority(priority);
        }

        if let Some(ttl) = self.ttl {
            builder = builder.ttl_ms(ttl as u32);
        }

        if let Some(ref class) = self.class {
            builder = builder.class(class);
        }

        let net_msg = builder.build();

        // Serialize and save
        let serialized = bincode::serialize(&net_msg)?;
        write_file(&self.output, &serialized)?;

        println!("✓ Network message created:");
        println!("  Kind: {:?}", net_msg.kind);
        println!("  Priority: {}/255", net_msg.priority);
        println!("  TTL: {}ms", net_msg.ttl_ms);
        if let Some(ref class) = net_msg.class {
            println!("  Class: {}", class);
        }
        println!("  Output: {}", self.output.display());

        Ok(())
    }
}
