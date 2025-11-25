use anyhow::Result;
use clap::{Args, Subcommand};

mod batch;
mod classify;
mod create;
mod headers;
mod inspect;
mod route;

pub use batch::BatchRouteCmd;
pub use classify::ClassifyCmd;
pub use create::CreateCmd;
pub use headers::HeadersCmd;
pub use inspect::InspectCmd;
pub use route::RouteCmd;

#[derive(Args)]
pub struct NetCmd {
    #[command(subcommand)]
    pub command: NetSubcommand,
}

#[derive(Subcommand)]
pub enum NetSubcommand {
    /// Create network message with QoS metadata
    Create(CreateCmd),

    /// Make routing decision (LLM vs Local vs Drop)
    Route(RouteCmd),

    /// Inspect network message metadata
    Inspect(InspectCmd),

    /// Generate transport headers (HTTP, Kafka, NATS, gRPC)
    Headers(HeadersCmd),

    /// Batch routing analysis with statistics
    BatchRoute(BatchRouteCmd),

    /// Auto-classify message kind
    Classify(ClassifyCmd),
}

impl NetCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            NetSubcommand::Create(cmd) => cmd.execute(),
            NetSubcommand::Route(cmd) => cmd.execute(),
            NetSubcommand::Inspect(cmd) => cmd.execute(),
            NetSubcommand::Headers(cmd) => cmd.execute(),
            NetSubcommand::BatchRoute(cmd) => cmd.execute(),
            NetSubcommand::Classify(cmd) => cmd.execute(),
        }
    }
}
