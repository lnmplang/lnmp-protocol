//! SFE Engine - Stage 2 of LNMP Pipeline

use crate::components::FieldImportance;
use lnmp::prelude::*;
use lnmp::sfe::ContextProfile;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ScoredEvent {
    pub record: LnmpRecord,
    pub score: f32,
    pub context: ContextProfile,
}

pub struct SFEEngine {
    pub stats: SFEStats,
}

#[derive(Debug, Clone, Default)]
pub struct SFEStats {
    pub input_count: usize,
    pub output_count: usize,
    pub average_score: f32,
    pub critical_events: usize,
}

impl SFEEngine {
    pub fn new() -> Self {
        Self {
            stats: SFEStats::default(),
        }
    }

    pub fn score(&mut self, messages: Vec<NetMessage>) -> Vec<ScoredEvent> {
        let mut scored = Vec::with_capacity(messages.len());
        self.stats.input_count = messages.len();
        let mut total_score = 0.0;

        for msg in messages {
            let record = msg.envelope.record.clone();
            let score = self.compute_composite_score(&record);

            // Create minimal context profile
            let context = ContextProfile {
                freshness_score: self.get_freshness(&record),
                importance: self.get_importance_u8(&record),
                risk_level: lnmp::sfe::RiskLevel::Medium,
                confidence: self.get_confidence(&record) as f64,
                llm_hints: HashMap::new(),
            };

            scored.push(ScoredEvent {
                record,
                score,
                context,
            });

            total_score += score;
            if score >= 0.9 {
                self.stats.critical_events += 1;
            }
        }

        self.stats.output_count = scored.len();
        if !scored.is_empty() {
            self.stats.average_score = total_score / scored.len() as f32;
        }

        scored
    }

    fn compute_composite_score(&self, record: &LnmpRecord) -> f32 {
        let importance = self.get_importance(record);
        let risk_level = self.get_risk_level(record);
        let confidence = self.get_confidence(record);
        let freshness = self.get_freshness(record);

        let score =
            importance * 0.40 + risk_level * 0.30 + confidence * 0.20 + freshness as f32 * 0.10;

        score.clamp(0.0, 1.0)
    }

    fn get_importance(&self, record: &LnmpRecord) -> f32 {
        let mut max_importance = 0u8;
        for field in record.fields() {
            let importance = FieldImportance::get(field.fid as u32);
            max_importance = max_importance.max(importance);
        }
        max_importance as f32 / 255.0
    }

    fn get_importance_u8(&self, record: &LnmpRecord) -> u8 {
        let mut max_importance = 0u8;
        for field in record.fields() {
            let importance = FieldImportance::get(field.fid as u32);
            max_importance = max_importance.max(importance);
        }
        max_importance
    }

    fn get_risk_level(&self, record: &LnmpRecord) -> f32 {
        let has_violence = record.get_field(50).is_some();
        let has_weapon = record.get_field(51).is_some();

        if has_violence && has_weapon {
            1.0
        } else if has_weapon {
            0.9
        } else if has_violence {
            0.85
        } else if record.get_field(70).is_some() {
            0.95
        } else {
            0.30
        }
    }

    fn get_confidence(&self, record: &LnmpRecord) -> f32 {
        if let Some(field) = record.get_field(120) {
            if let LnmpValue::Float(conf) = field.value {
                return (conf as f32).clamp(0.0, 1.0);
            }
        }
        0.85
    }

    fn get_freshness(&self, record: &LnmpRecord) -> f64 {
        if let Some(field) = record.get_field(3) {
            if let LnmpValue::Int(event_time) = field.value {
                let current_time = current_timestamp() as i64;
                let age = (current_time - event_time).max(0) as f64;

                if age < 60.0 {
                    return 1.0;
                } else if age < 300.0 {
                    return 1.0 - (age - 60.0) / 480.0;
                } else {
                    return 0.5;
                }
            }
        }
        0.9
    }

    pub fn top_k(&self, mut events: Vec<ScoredEvent>, k: usize) -> Vec<ScoredEvent> {
        events.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        events.into_iter().take(k).collect()
    }
}

impl Default for SFEEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
