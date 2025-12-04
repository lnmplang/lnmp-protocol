use super::helpers::*;
use lnmp::prelude::*;
// use rand::Rng; // Removed to avoid dependency

pub struct DisasterGenerator {
    base_seismic_activity: f32,
    seed: u32,
}

impl DisasterGenerator {
    pub fn new(base_seismic_activity: f32) -> Self {
        Self {
            base_seismic_activity,
            seed: 12345,
        }
    }

    fn next_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed % 10000) as f32 / 10000.0
    }

    fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_random()
    }

    pub fn generate_events(&mut self, count: usize) -> Vec<LnmpRecord> {
        let mut events = Vec::new();

        for _ in 0..count {
            let mut record = LnmpRecord::new();

            // Add source ID
            add_string_field(
                &mut record,
                1,
                format!(
                    "SENSOR_DISASTER_{}",
                    (self.next_random() * 9000.0) as u32 + 1000
                ),
            );

            // Add timestamp (simulated)
            add_int_field(&mut record, 3, 1678886400);

            // Determine event type based on probability
            let roll = self.next_random();

            if roll < 0.05 {
                // Earthquake Precursor (F70) - Rare but critical
                add_string_field(&mut record, 2, "EARTHQUAKE_SIGNAL".to_string());
                add_float_field(
                    &mut record,
                    70,
                    self.base_seismic_activity + self.next_range(0.0, 2.0),
                );
                add_float_field(&mut record, 10, 35.6895 + self.next_range(-0.1, 0.1)); // Tokyo Lat
                add_float_field(&mut record, 11, 139.6917 + self.next_range(-0.1, 0.1));
            // Tokyo Lon
            } else if roll < 0.15 {
                // Fire Detected (F61)
                add_string_field(&mut record, 2, "FIRE_SENSOR".to_string());
                add_bool_field(&mut record, 61, self.next_random() < 0.01); // 1% chance of actual fire in background noise
                add_float_field(&mut record, 10, 35.6895 + self.next_range(-0.2, 0.2));
                add_float_field(&mut record, 11, 139.6917 + self.next_range(-0.2, 0.2));
            } else {
                // Background Environmental Data
                add_string_field(&mut record, 2, "ENV_MONITOR".to_string());
                add_float_field(&mut record, 65, self.next_range(0.0, 50.0)); // Air quality
                add_float_field(&mut record, 10, 35.6895 + self.next_range(-0.5, 0.5));
                add_float_field(&mut record, 11, 139.6917 + self.next_range(-0.5, 0.5));
            }

            events.push(record);
        }

        events
    }

    // Helper to generate a specific major event
    pub fn generate_major_earthquake(&self, magnitude: f32) -> LnmpRecord {
        let mut record = LnmpRecord::new();
        add_string_field(&mut record, 1, "SEISMIC_NET_PRIMARY".to_string());
        add_string_field(&mut record, 2, "EARTHQUAKE_CRITICAL".to_string());
        add_float_field(&mut record, 70, magnitude); // Magnitude
        add_bool_field(&mut record, 71, true); // Infrastructure risk
        add_float_field(&mut record, 10, 35.6895);
        add_float_field(&mut record, 11, 139.6917);
        record
    }
}
