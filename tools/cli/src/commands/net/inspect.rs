use anyhow::Result;
use clap::Args;
use lnmp_net::NetMessage;
use std::path::PathBuf;

use crate::utils::read_file;

#[derive(Args)]
pub struct InspectCmd {
    /// Input network message file
    input: PathBuf,
}

impl InspectCmd {
    pub fn execute(&self) -> Result<()> {
        // Read network message
        let data = read_file(&self.input)?;
        let net_msg: NetMessage = bincode::deserialize(&data)?;

        // Get envelope metadata
        let envelope = &net_msg.envelope;

        // Calculate age if timestamp available
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        println!("📨 Network Message Inspection:");
        println!();
        println!("Network Metadata:");
        println!("  Kind: {:?}", net_msg.kind);
        println!(
            "  Priority: {}/255 ({})",
            net_msg.priority,
            priority_label(net_msg.priority)
        );
        println!("  TTL: {}ms", net_msg.ttl_ms);

        if let Some(ref class) = net_msg.class {
            println!("  Class: {}", class);
        }

        println!();
        println!("Envelope Metadata:");

        if let Some(ts) = envelope.metadata.timestamp {
            let age_ms = now_ms.saturating_sub(ts);
            let freshness = if age_ms < net_msg.ttl_ms as u64 {
                "Fresh"
            } else {
                "Expired"
            };
            println!(
                "  Timestamp: {} ({}s ago, {})",
                ts,
                age_ms / 1000,
                freshness
            );
        }

        if let Some(ref source) = envelope.metadata.source {
            println!("  Source: {}", source);
        }

        if let Some(ref trace_id) = envelope.metadata.trace_id {
            println!("  TraceID: {}", trace_id);
        }

        println!();
        println!("Payload:");
        println!("  Fields: {}", envelope.record.fields().len());

        // Show first few fields
        for (i, field) in envelope.record.fields().iter().take(3).enumerate() {
            println!("  F{}: {:?}", field.fid, field.value);
            if i == 2 && envelope.record.fields().len() > 3 {
                println!("  ... ({} more fields)", envelope.record.fields().len() - 3);
            }
        }

        Ok(())
    }
}

fn priority_label(priority: u8) -> &'static str {
    match priority {
        0..=50 => "Very Low",
        51..=100 => "Low",
        101..=150 => "Normal",
        151..=200 => "High",
        201..=254 => "Very High",
        255 => "Critical",
    }
}
