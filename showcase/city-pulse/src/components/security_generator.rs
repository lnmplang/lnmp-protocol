//! Security and Public Safety Event Generator
//!
//! Generates AI-powered CCTV security events

use super::event_types::EventType;
use super::helpers::*;
use lnmp::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Active security incident
#[derive(Debug, Clone)]
pub struct SecurityIncident {
    pub incident_type: EventType,
    pub location: String,
    pub lat: f64,
    pub lon: f64,
    pub severity: f32,
    pub people_count: u32,
    pub weapon_present: bool,
    pub timestamp: u64,
}

/// Security event generator
pub struct SecurityGenerator {
    camera_count: usize,
    active_incidents: Vec<SecurityIncident>,
    event_counter: u64,
    monitored_areas: Vec<(&'static str, (f64, f64))>,
}

impl SecurityGenerator {
    /// Create new security generator with specified camera count
    pub fn new(camera_count: usize) -> Self {
        let monitored_areas = vec![
            ("Shibuya_Crossing", (35.6595, 139.7004)),
            ("Shinjuku_Station", (35.6896, 139.7006)),
            ("Roppongi_District", (35.6627, 139.7296)),
            ("Ginza_Shopping", (35.6717, 139.7648)),
        ];

        Self {
            camera_count,
            active_incidents: Vec::new(),
            event_counter: 0,
            monitored_areas,
        }
    }

    /// Generate security events
    pub fn generate_events(&mut self, count: usize) -> Vec<LnmpRecord> {
        let mut events = Vec::with_capacity(count);
        let mut rng_state = Self::simple_random((self.event_counter as u32).wrapping_add(54321));

        for _ in 0..count {
            let event_type_rand = Self::next_random(&mut rng_state) % 1000;

            let event = match event_type_rand {
                0..=599 => self.generate_suspicious_behavior(&mut rng_state),
                600..=799 => self.generate_theft_event(&mut rng_state),
                950..=979 => self.generate_violence_event(&mut rng_state),
                _ => self.generate_weapon_detection(&mut rng_state),
            };

            if let Some(record) = event {
                events.push(record);
            }

            self.event_counter += 1;
        }

        events
    }

    fn generate_violence_event(&mut self, rng: &mut u32) -> Option<LnmpRecord> {
        let area_idx = (Self::next_random(rng) % self.monitored_areas.len() as u32) as usize;
        let (location, coords) = self.monitored_areas[area_idx];

        let people_count = Self::next_random(rng) % 8 + 2;
        let confidence = 0.80 + (Self::next_random(rng) % 20) as f32 / 100.0;

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(
            &mut record,
            1,
            format!(
                "CCTV_{:04}",
                Self::next_random(rng) % self.camera_count as u32
            ),
        );
        add_string_field(&mut record, 2, "VIOLENCE_DETECTED");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, coords.0 as f32);
        add_float_field(&mut record, 11, coords.1 as f32);
        add_bool_field(&mut record, 50, true); // F50: violence_detected
        add_int_field(&mut record, 201, people_count as i64);
        add_float_field(&mut record, 120, confidence);
        add_string_field(&mut record, 202, location);

        Some(record)
    }

    fn generate_weapon_detection(&mut self, rng: &mut u32) -> Option<LnmpRecord> {
        let area_idx = (Self::next_random(rng) % self.monitored_areas.len() as u32) as usize;
        let (location, coords) = self.monitored_areas[area_idx];

        let confidence = 0.85 + (Self::next_random(rng) % 15) as f32 / 100.0;

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(
            &mut record,
            1,
            format!(
                "CCTV_{:04}",
                Self::next_random(rng) % self.camera_count as u32
            ),
        );
        add_string_field(&mut record, 2, "WEAPON_DETECTED");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, coords.0 as f32);
        add_float_field(&mut record, 11, coords.1 as f32);
        add_bool_field(&mut record, 51, true); // F51: weapon_detected
        add_float_field(&mut record, 120, confidence);
        add_string_field(&mut record, 202, location);

        Some(record)
    }

    fn generate_theft_event(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let area_idx = (Self::next_random(rng) % self.monitored_areas.len() as u32) as usize;
        let (location, coords) = self.monitored_areas[area_idx];

        let confidence = 0.70 + (Self::next_random(rng) % 25) as f32 / 100.0;

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(
            &mut record,
            1,
            format!(
                "CCTV_{:04}",
                Self::next_random(rng) % self.camera_count as u32
            ),
        );
        add_string_field(&mut record, 2, "THEFT");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, coords.0 as f32);
        add_float_field(&mut record, 11, coords.1 as f32);
        add_bool_field(&mut record, 52, true); // F52: theft
        add_float_field(&mut record, 120, confidence);
        add_string_field(&mut record, 202, location);

        Some(record)
    }

    fn generate_suspicious_behavior(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let area_idx = (Self::next_random(rng) % self.monitored_areas.len() as u32) as usize;
        let (location, coords) = self.monitored_areas[area_idx];

        let confidence = 0.60 + (Self::next_random(rng) % 30) as f32 / 100.0;

        if confidence < 0.70 {
            return None;
        }

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(
            &mut record,
            1,
            format!(
                "CCTV_{:04}",
                Self::next_random(rng) % self.camera_count as u32
            ),
        );
        add_string_field(&mut record, 2, "SUSPICIOUS_BEHAVIOR");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, coords.0 as f32);
        add_float_field(&mut record, 11, coords.1 as f32);
        add_bool_field(&mut record, 55, true); // F55: suspicious_behavior
        add_float_field(&mut record, 120, confidence);
        add_string_field(&mut record, 202, location);

        Some(record)
    }

    /// Get currently active incidents
    pub fn get_active_incidents(&self) -> &[SecurityIncident] {
        &self.active_incidents
    }

    fn simple_random(seed: u32) -> u32 {
        seed.wrapping_mul(1664525).wrapping_add(1013904223)
    }

    fn next_random(state: &mut u32) -> u32 {
        *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        *state
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_generator_creation() {
        let gen = SecurityGenerator::new(100);
        assert_eq!(gen.camera_count, 100);
    }

    #[test]
    fn test_generate_events() {
        let mut gen = SecurityGenerator::new(50);
        let events = gen.generate_events(100);

        assert!(!events.is_empty());
    }
}
