use anyhow::Result;
use clap::Args;
use lnmp_net::{NetMessage, RoutingDecision, RoutingPolicy};
use std::path::PathBuf;

use crate::utils::read_file;

#[derive(Args)]
pub struct RouteCmd {
    /// Input network message file
    input: PathBuf,

    /// Routing threshold (0.0-1.0, default: 0.7)
    #[arg(long, default_value = "0.7")]
    threshold: f64,

    /// Current timestamp in Unix milliseconds (default: now)
    #[arg(long)]
    current_time: Option<u64>,
}

impl RouteCmd {
    pub fn execute(&self) -> Result<()> {
        // Read network message
        let data = read_file(&self.input)?;
        let net_msg: NetMessage = bincode::deserialize(&data)?;

        // Create routing policy
        let policy = RoutingPolicy::new(self.threshold)
            .with_always_route_alerts(true)
            .with_drop_expired(true);

        // Get current time
        let now_ms = self.current_time.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        });

        // Make routing decision
        let decision = policy.decide(&net_msg, now_ms)?;

        // Calculate importance score if available
        let importance = policy.base_importance(&net_msg, now_ms).ok();

        // Print results
        println!("🔀 Routing Decision:");
        println!("  Input: {}", self.input.display());
        println!("  Kind: {:?}", net_msg.kind);
        println!("  Priority: {}/255", net_msg.priority);

        if let Some(score) = importance {
            println!("  Importance: {:.2}", score);
        }

        match decision {
            RoutingDecision::SendToLLM => {
                println!("\n✓ Decision Send to LLM");
                println!("  → Route to LLM for processing");
                if net_msg.priority > 200 {
                    println!("  Reason: High priority (>200)");
                } else if let Some(score) = importance {
                    println!(
                        "  Reason: Importance {:.2} ≥ threshold {:.2}",
                        score, self.threshold
                    );
                }
            }
            RoutingDecision::ProcessLocally => {
                println!("\n✓ Decision: ProcessLocally");
                println!("  → Handle with local processing");
                if let Some(score) = importance {
                    println!(
                        "  Reason: Importance {:.2} < threshold {:.2}",
                        score, self.threshold
                    );
                }
            }
            RoutingDecision::Drop => {
                println!("\n✓ Decision: Drop");
                println!("  → Message expired or invalid");
                println!("  Reason: TTL expired");
            }
        }

        Ok(())
    }
}
