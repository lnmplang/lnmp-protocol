use anyhow::Result;
use clap::Args;
use lnmp_envelope::LnmpEnvelope;
use lnmp_net::MessageKind;
use std::path::PathBuf;

use crate::utils::read_file;

#[derive(Args)]
pub struct ClassifyCmd {
    /// Input LNMP envelope file  
    input: PathBuf,

    /// Hint keyword for classification
    #[arg(long)]
    hint: Option<String>,
}

impl ClassifyCmd {
    pub fn execute(&self) -> Result<()> {
        // Read envelope
        let data = read_file(&self.input)?;
        let envelope: LnmpEnvelope = bincode::deserialize(&data)?;

        // Simple heuristic classification
        let (kind, confidence, reason) = self.classify_envelope(&envelope);

        println!("🔍 Message Classification:");
        println!();
        println!("Suggested Kind: {:?}", kind);
        println!("Confidence: {}%", confidence);
        println!("Reason: {}", reason);
        println!();
        println!("Default QoS:");
        println!("  Priority: {}", kind.default_priority());
        println!("  TTL: {}ms", kind.default_ttl_ms());

        Ok(())
    }

    fn classify_envelope(&self, envelope: &LnmpEnvelope) -> (MessageKind, u8, &'static str) {
        // Check metadata hints
        if let Some(ref source) = envelope.metadata.source {
            if source.contains("alert") || source.contains("alarm") {
                return (MessageKind::Alert, 90, "Source indicates alert system");
            }
            if source.contains("sensor") || source.contains("telemetry") {
                return (MessageKind::Event, 85, "Source indicates sensor/telemetry");
            }
            if source.contains("command") || source.contains("control") {
                return (MessageKind::Command, 85, "Source indicates control system");
            }
        }

        // Check hint keyword if provided
        if let Some(ref hint) = self.hint {
            let hint_lower = hint.to_lowercase();
            if hint_lower.contains("alert") || hint_lower.contains("critical") {
                return (MessageKind::Alert, 95, "Hint keyword indicates alert");
            }
            if hint_lower.contains("event") || hint_lower.contains("sensor") {
                return (MessageKind::Event, 90, "Hint keyword indicates event");
            }
            if hint_lower.contains("command") || hint_lower.contains("action") {
                return (MessageKind::Command, 90, "Hint keyword indicates command");
            }
            if hint_lower.contains("query") || hint_lower.contains("request") {
                return (MessageKind::Query, 90, "Hint keyword indicates query");
            }
            if hint_lower.contains("state") || hint_lower.contains("status") {
                return (MessageKind::State, 90, "Hint keyword indicates state");
            }
        }

        // Check field count heuristics
        let field_count = envelope.record.fields().len();

        if field_count == 1 {
            return (MessageKind::Query, 70, "Single field suggests simple query");
        }

        if field_count > 10 {
            return (MessageKind::State, 75, "Many fields suggest state snapshot");
        }

        // Default to Event
        (
            MessageKind::Event,
            60,
            "Default classification for telemetry data",
        )
    }
}
