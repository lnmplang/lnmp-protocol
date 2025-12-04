//! Network Filter - Stage 1 of LNMP Pipeline
//!
//! Uses lnmp-net for intelligent message routing and filtering

use lnmp::envelope::EnvelopeBuilder;
use lnmp::net::MessageKind;
use lnmp::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NetworkFilter {
    dedup_cache: HashMap<u64, u64>,
    pub stats: FilterStats,
}

#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    pub input_count: usize,
    pub output_count: usize,
    pub duplicates_removed: usize,
    pub low_priority_dropped: usize,
    pub validation_failures: usize,
}

impl NetworkFilter {
    pub fn new() -> Self {
        Self {
            dedup_cache: HashMap::new(),
            stats: FilterStats::default(),
        }
    }

    pub fn filter(&mut self, events: Vec<LnmpRecord>) -> Vec<NetMessage> {
        let mut filtered = Vec::new();
        self.stats.input_count += events.len();

        for record in events {
            // Check for duplicates
            let signature = self.compute_signature(&record);
            let current_time = current_timestamp();

            if let Some(&last_seen) = self.dedup_cache.get(&signature) {
                if current_time - last_seen < 60 {
                    self.stats.duplicates_removed += 1;
                    continue;
                }
            }
            self.dedup_cache.insert(signature, current_time);

            // Determine message kind and priority
            let (msg_kind, priority) = self.classify_event(&record);

            // Drop very low priority messages
            if priority < 50 && self.should_drop_low_priority() {
                self.stats.low_priority_dropped += 1;
                continue;
            }

            // Create NetMessage with envelope
            let envelope = EnvelopeBuilder::new(record)
                .timestamp(current_time * 1000)
                .source("city-pulse-filter")
                .build();

            let net_msg = NetMessage::with_qos(envelope, msg_kind, priority, 30000);
            filtered.push(net_msg);
            self.stats.output_count += 1;
        }

        self.dedup_cache
            .retain(|_, &mut timestamp| current_timestamp() - timestamp < 300);

        filtered
    }

    fn compute_signature(&self, record: &LnmpRecord) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        if let Some(field) = record.get_field(1) {
            format!("{:?}", field.value).hash(&mut hasher);
        }
        if let Some(field) = record.get_field(2) {
            format!("{:?}", field.value).hash(&mut hasher);
        }
        hasher.finish()
    }

    fn classify_event(&self, record: &LnmpRecord) -> (MessageKind, u8) {
        let is_critical = record.get_field(50).is_some()
            || record.get_field(51).is_some()
            || record.get_field(70).is_some();

        let is_high_priority = record.get_field(21).is_some()
            || record.get_field(22).is_some()
            || record.get_field(61).is_some();

        if is_critical {
            (MessageKind::Alert, 255)
        } else if is_high_priority {
            (MessageKind::Event, 200)
        } else {
            (MessageKind::State, 100)
        }
    }

    fn should_drop_low_priority(&self) -> bool {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(current_timestamp());
        (hasher.finish() % 100) < 30
    }

    pub fn compression_ratio(&self) -> f32 {
        if self.stats.input_count == 0 {
            return 0.0;
        }
        (self.stats.input_count - self.stats.output_count) as f32 / self.stats.input_count as f32
    }
}

impl Default for NetworkFilter {
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
