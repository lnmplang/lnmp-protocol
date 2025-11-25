use anyhow::Result;
use clap::Args;
use lnmp_net::{NetMessage, RoutingDecision, RoutingPolicy};
use std::path::PathBuf;

use crate::utils::read_file;

#[derive(Args)]
pub struct BatchRouteCmd {
    /// Directory containing network message files
    dir: PathBuf,

    /// Routing threshold (0.0-1.0, default: 0.7)
    #[arg(long, default_value = "0.7")]
    threshold: f64,

    /// Show statistics
    #[arg(long)]
    stats: bool,
}

impl BatchRouteCmd {
    pub fn execute(&self) -> Result<()> {
        // Verify directory exists
        if !self.dir.is_dir() {
            anyhow::bail!("Path is not a directory: {}", self.dir.display());
        }

        // Create routing policy
        let policy = RoutingPolicy::new(self.threshold)
            .with_always_route_alerts(true)
            .with_drop_expired(true);

        // Get current time
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Counters
        let mut total = 0;
        let mut send_to_llm = 0;
        let mut process_locally = 0;
        let mut drop = 0;
        let mut errors = 0;

        // Process all files in directory
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Try to read and route
            match self.route_message(&path, &policy, now_ms) {
                Ok(decision) => {
                    total += 1;
                    match decision {
                        RoutingDecision::SendToLLM => send_to_llm += 1,
                        RoutingDecision::ProcessLocally => process_locally += 1,
                        RoutingDecision::Drop => drop += 1,
                    }

                    if !self.stats {
                        println!(
                            "{}: {:?}",
                            path.file_name().unwrap().to_string_lossy(),
                            decision
                        );
                    }
                }
                Err(e) => {
                    errors += 1;
                    if !self.stats {
                        eprintln!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Print statistics
        if self.stats {
            println!("📊 Batch Routing Statistics:");
            println!();
            println!("Total Messages: {}", total);
            println!(
                "SendToLLM: {} ({:.1}%)",
                send_to_llm,
                percentage(send_to_llm, total)
            );
            println!(
                "ProcessLocally: {} ({:.1}%)",
                process_locally,
                percentage(process_locally, total)
            );
            println!("Drop: {} ({:.1}%)", drop, percentage(drop, total));

            if errors > 0 {
                println!("Errors: {}", errors);
            }

            println!();
            let token_savings = percentage(process_locally + drop, total);
            println!("💰 Token Savings: {:.1}%", token_savings);

            if total > 0 {
                println!("   (Avoided {} LLM API calls)", process_locally + drop);
            }
        }

        Ok(())
    }

    fn route_message(
        &self,
        path: &PathBuf,
        policy: &RoutingPolicy,
        now_ms: u64,
    ) -> Result<RoutingDecision> {
        let data = read_file(path)?;
        let net_msg: NetMessage = bincode::deserialize(&data)?;
        let decision = policy.decide(&net_msg, now_ms)?;
        Ok(decision)
    }
}

fn percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}
