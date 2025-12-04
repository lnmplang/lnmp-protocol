//! LLM Bridge - Stage 3 of LNMP Pipeline

use crate::pipeline::sfe_engine::ScoredEvent;
use lnmp::prelude::*;

pub struct LLBConverter {
    // Simplified for now
}

#[derive(Debug, Clone)]
pub struct LLMPrompt {
    pub system_context: String,
    pub user_message: String,
    pub structured_data: Vec<u8>,
    pub event_count: usize,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub analysis: String,
    pub actions: Vec<LnmpRecord>,
    pub confidence: f32,
}

impl LLBConverter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn to_natural_language(&self, events: &[ScoredEvent]) -> LLMPrompt {
        let mut event_descriptions = Vec::new();
        let top_events = events.iter().take(20).collect::<Vec<_>>();

        for (idx, scored_event) in top_events.iter().enumerate() {
            let description = self.describe_event(scored_event, idx + 1);
            event_descriptions.push(description);
        }

        let system_context = format!(
            "You are the AI coordinator for Tokyo Smart City OS.\n\
            Total events processed: {}\n\
            Showing top 20 highest priority events.",
            events.len()
        );

        let user_message = format!(
            "Tokyo City Emergency Report - {} Critical Events\n\nTop Priority Events:\n\n{}",
            events.len(),
            event_descriptions.join("\n\n")
        );

        LLMPrompt {
            system_context,
            user_message,
            structured_data: Vec::new(),
            event_count: events.len(),
        }
    }

    fn describe_event(&self, scored_event: &ScoredEvent, index: usize) -> String {
        let record = &scored_event.record;
        let score = scored_event.score;

        let event_type = self
            .extract_string_field(record, 2)
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let location = self.extract_location(record);

        let mut description = format!("{}. {} (Score: {:.2})", index, event_type, score);

        if let Some((lat, lon, area)) = location {
            description.push_str(&format!("\n   Location: {} ({:.4}, {:.4})", area, lat, lon));
        }

        description
    }

    fn extract_location(&self, record: &LnmpRecord) -> Option<(f64, f64, String)> {
        let lat = record.get_field(10).and_then(|f| {
            if let LnmpValue::Float(v) = f.value {
                Some(v)
            } else {
                None
            }
        })?;
        let lon = record.get_field(11).and_then(|f| {
            if let LnmpValue::Float(v) = f.value {
                Some(v)
            } else {
                None
            }
        })?;
        Some((lat, lon, "Tokyo".to_string()))
    }

    fn extract_string_field(&self, record: &LnmpRecord, fid: u16) -> Option<String> {
        record.get_field(fid).and_then(|f| {
            if let LnmpValue::String(s) = &f.value {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    pub fn from_natural_language(&self, llm_response: &str) -> LLMResponse {
        let actions = self.extract_actions(llm_response);
        LLMResponse {
            analysis: llm_response.to_string(),
            actions,
            confidence: 0.8,
        }
    }

    fn extract_actions(&self, response: &str) -> Vec<LnmpRecord> {
        let mut actions = Vec::new();

        for line in response.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("dispatch") || line_lower.contains("send") {
                if let Some(action) = self.parse_dispatch_command(line) {
                    actions.push(action);
                }
            }
        }

        actions
    }

    fn parse_dispatch_command(&self, command: &str) -> Option<LnmpRecord> {
        let mut record = LnmpRecord::new();

        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String("AI_DISPATCH".to_string()),
        });

        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
        });

        let cmd_lower = command.to_lowercase();
        if cmd_lower.contains("police") {
            record.add_field(LnmpField {
                fid: 210,
                value: LnmpValue::String("POLICE".to_string()),
            });
        } else if cmd_lower.contains("ambulance") || cmd_lower.contains("medical") {
            record.add_field(LnmpField {
                fid: 210,
                value: LnmpValue::String("AMBULANCE".to_string()),
            });
        }

        Some(record)
    }
}

impl Default for LLBConverter {
    fn default() -> Self {
        Self::new()
    }
}
