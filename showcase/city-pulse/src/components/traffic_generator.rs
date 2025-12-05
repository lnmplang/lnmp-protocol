//! Traffic and Mobility Event Generator
//!
//! Generates realistic traffic events including:
//! - Red light violations
//! - Speeding
//! - Accident risks
//! - Emergency vehicle routing  
//! - Traffic congestion
//! - Aggressive driving behavior

use super::helpers::*;
use lnmp::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Vehicle in the traffic system
#[derive(Debug, Clone)]
struct Vehicle {
    id: String,
    lat: f64,
    lon: f64,
    speed: f32,
    is_emergency: bool,
    aggressive_score: f32, // 0.0-1.0
}

/// Traffic light state
#[derive(Debug, Clone)]
struct TrafficLight {
    id: String,
    lat: f64,
    lon: f64,
}

/// Traffic generator for Tokyo
pub struct TrafficGenerator {
    vehicles: Vec<Vehicle>,
    traffic_lights: Vec<TrafficLight>,
    event_counter: u64,

    // Tokyo coordinates (Shibuya, Harajuku, Shinjuku, etc.)
    districts: Vec<(&'static str, (f64, f64))>,
}

impl TrafficGenerator {
    /// Create a new traffic generator with specified number of vehicles
    pub fn new(vehicle_count: usize) -> Self {
        let districts = vec![
            ("Shibuya", (35.6620, 139.7030)),
            ("Harajuku", (35.6702, 139.7026)),
            ("Shinjuku", (35.6896, 139.6917)),
            ("Roppongi", (35.6627, 139.7296)),
            (" Akihabara", (35.7022, 139.7744)),
            ("Ginza", (35.6717, 139.7648)),
            ("Ikebukuro", (35.7295, 139.7109)),
            ("Ueno", (35.7148, 139.7772)),
        ];

        let mut vehicles = Vec::with_capacity(vehicle_count);
        let mut rng_state = Self::simple_random(42);

        for i in 0..vehicle_count {
            let district_idx =
                (Self::next_random(&mut rng_state) % districts.len() as u32) as usize;
            let (_, base_coords) = districts[district_idx];

            // Add some randomness around the base coordinates
            let lat_offset = (Self::next_random(&mut rng_state) % 200) as f64 / 10000.0 - 0.01;
            let lon_offset = (Self::next_random(&mut rng_state) % 200) as f64 / 10000.0 - 0.01;

            vehicles.push(Vehicle {
                id: format!("VEH_{:06}", i),
                lat: base_coords.0 + lat_offset,
                lon: base_coords.1 + lon_offset,
                speed: (Self::next_random(&mut rng_state) % 80) as f32 + 20.0, // 20-100 km/h
                is_emergency: (Self::next_random(&mut rng_state) % 100) < 2, // 2% emergency vehicles
                aggressive_score: (Self::next_random(&mut rng_state) % 100) as f32 / 100.0,
            });
        }

        // Create traffic lights
        let mut traffic_lights = Vec::new();
        for (i, (_, coords)) in districts.iter().enumerate() {
            let lat = coords.0;
            let lon = coords.1;
            traffic_lights.push(TrafficLight {
                id: format!("TL{:04}", i),
                lat,
                lon,
            });
        }

        Self {
            vehicles,
            traffic_lights,
            event_counter: 0,
            districts,
        }
    }

    /// Generate traffic events
    pub fn generate_events(&mut self, count: usize) -> Vec<LnmpRecord> {
        let mut events = Vec::with_capacity(count);
        let mut rng_state = Self::simple_random((self.event_counter as u32).wrapping_add(12345));

        for _ in 0..count {
            let event_type_rand = Self::next_random(&mut rng_state) % 100;

            let event = match event_type_rand {
                // 40% traffic congestion (most common)
                0..=39 => self.generate_congestion_event(&mut rng_state),
                // 25% speeding
                40..=64 => self.generate_speeding_event(&mut rng_state),
                // 15% red light violations
                65..=79 => self.generate_red_light_violation(&mut rng_state),
                // 10% aggressive driving
                80..=89 => self.generate_aggressive_driving(&mut rng_state),
                // 5% accident risk
                90..=94 => self.generate_accident_risk(&mut rng_state),
                // 3% emergency vehicle
                95..=97 => self.generate_emergency_vehicle(&mut rng_state),
                // 2% traffic accidents
                _ => self.generate_traffic_accident(&mut rng_state),
            };

            if let Some(record) = event {
                events.push(record);
            }

            self.event_counter += 1;
        }

        // Update vehicle positions
        self.update_vehicles();

        events
    }

    fn generate_speeding_event(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let vehicle_idx = (Self::next_random(rng) % self.vehicles.len() as u32) as usize;
        let vehicle = &self.vehicles[vehicle_idx];

        // Only generate event if actually speeding (>80 km/h in city)
        if vehicle.speed <= 80.0 {
            return None;
        }

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(
            &mut record,
            1,
            format!("TRAFFIC_SENSOR_{}", vehicle_idx % 100),
        );
        add_string_field(&mut record, 2, "SPEEDING");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, vehicle.lat as f32);
        add_float_field(&mut record, 11, vehicle.lon as f32);
        add_float_field(&mut record, 20, vehicle.speed); // F20: vehicle_speed
        add_string_field(&mut record, 200, vehicle.id.clone());

        Some(record)
    }

    fn generate_red_light_violation(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let vehicle_idx = (Self::next_random(rng) % self.vehicles.len() as u32) as usize;
        let light_idx = (Self::next_random(rng) % self.traffic_lights.len() as u32) as usize;

        let vehicle = &self.vehicles[vehicle_idx];
        let light = &self.traffic_lights[light_idx];

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, format!("CCTV_{}", light.id));
        add_string_field(&mut record, 2, "RED_LIGHT_VIOLATION");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, light.lat as f32);
        add_float_field(&mut record, 11, light.lon as f32);
        add_bool_field(&mut record, 21, true); // F21: red_light_violation
        add_float_field(&mut record, 20, vehicle.speed);
        add_string_field(&mut record, 200, vehicle.id.clone());
        add_float_field(
            &mut record,
            120,
            0.85 + (Self::next_random(rng) % 15) as f32 / 100.0,
        ); // confidence

        Some(record)
    }

    fn generate_accident_risk(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let vehicle_idx = (Self::next_random(rng) % self.vehicles.len() as u32) as usize;
        let vehicle = &self.vehicles[vehicle_idx];

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, format!("AI_PREDICTOR_{}", vehicle_idx % 50));
        add_string_field(&mut record, 2, "ACCIDENT_RISK");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, vehicle.lat as f32);
        add_float_field(&mut record, 11, vehicle.lon as f32);
        add_bool_field(&mut record, 22, true); // F22: accident_risk
        add_float_field(&mut record, 20, vehicle.speed);
        add_float_field(
            &mut record,
            120,
            0.75 + (Self::next_random(rng) % 20) as f32 / 100.0,
        );

        Some(record)
    }

    fn generate_emergency_vehicle(&self, rng: &mut u32) -> Option<LnmpRecord> {
        // Find an emergency vehicle
        let emergency_vehicles: Vec<_> = self.vehicles.iter().filter(|v| v.is_emergency).collect();

        if emergency_vehicles.is_empty() {
            return None;
        }

        let vehicle_idx = (Self::next_random(rng) % emergency_vehicles.len() as u32) as usize;
        let vehicle = emergency_vehicles[vehicle_idx];

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, "EMERGENCY_DISPATCH");
        add_string_field(&mut record, 2, "EMERGENCY_VEHICLE");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, vehicle.lat as f32);
        add_float_field(&mut record, 11, vehicle.lon as f32);
        add_bool_field(&mut record, 23, true); // F23: emergency_vehicle
        add_float_field(&mut record, 20, vehicle.speed);
        add_string_field(&mut record, 200, vehicle.id.clone());

        Some(record)
    }

    fn generate_congestion_event(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let district_idx = (Self::next_random(rng) % self.districts.len() as u32) as usize;
        let (district_name, coords) = self.districts[district_idx];

        let congestion_level = (Self::next_random(rng) % 100) as f32 / 100.0; // 0.0-1.0

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, format!("TRAFFIC_CENTER_{}", district_name));
        add_string_field(&mut record, 2, "TRAFFIC_CONGESTION");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, coords.0 as f32);
        add_float_field(&mut record, 11, coords.1 as f32);
        add_float_field(&mut record, 24, congestion_level); // F24: traffic_congestion

        Some(record)
    }

    fn generate_aggressive_driving(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let vehicle_idx = (Self::next_random(rng) % self.vehicles.len() as u32) as usize;
        let vehicle = &self.vehicles[vehicle_idx];

        // Only generate if aggressive score is high
        if vehicle.aggressive_score < 0.7 {
            return None;
        }

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, format!("CCTV_AI_{}", vehicle_idx % 100));
        add_string_field(&mut record, 2, "AGGRESSIVE_DRIVING");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, vehicle.lat as f32);
        add_float_field(&mut record, 11, vehicle.lon as f32);
        add_float_field(&mut record, 25, vehicle.aggressive_score); // F25: aggressive_driving
        add_float_field(&mut record, 20, vehicle.speed);
        add_string_field(&mut record, 200, vehicle.id.clone());
        add_float_field(
            &mut record,
            120,
            0.80 + (Self::next_random(rng) % 15) as f32 / 100.0,
        );

        Some(record)
    }

    fn generate_traffic_accident(&self, rng: &mut u32) -> Option<LnmpRecord> {
        let vehicle_idx = (Self::next_random(rng) % self.vehicles.len() as u32) as usize;
        let vehicle = &self.vehicles[vehicle_idx];

        let mut record = LnmpRecord::new();
        let timestamp = current_timestamp();

        add_string_field(&mut record, 1, format!("EMERGENCY_911_{}", vehicle_idx));
        add_string_field(&mut record, 2, "TRAFFIC_ACCIDENT");
        add_int_field(&mut record, 3, timestamp as i64);
        add_float_field(&mut record, 10, vehicle.lat as f32);
        add_float_field(&mut record, 11, vehicle.lon as f32);
        add_bool_field(&mut record, 26, true); // F26: traffic_accident
        add_int_field(&mut record, 201, (Self::next_random(rng) % 5 + 1) as i64); // vehicles involved

        Some(record)
    }

    fn update_vehicles(&mut self) {
        // Simple movement simulation
        let mut rng = Self::simple_random((self.event_counter as u32).wrapping_add(9999));

        for vehicle in &mut self.vehicles {
            // Small random movement
            let lat_delta = (Self::next_random(&mut rng) % 40) as f64 / 100000.0 - 0.0002;
            let lon_delta = (Self::next_random(&mut rng) % 40) as f64 / 100000.0 - 0.0002;

            vehicle.lat += lat_delta;
            vehicle.lon += lon_delta;

            // Slightly vary speed
            let speed_delta = (Self::next_random(&mut rng) % 20) as f32 - 10.0;
            vehicle.speed = (vehicle.speed + speed_delta).clamp(10.0, 120.0);
        }
    }

    // Simple LCG random number generator for reproducibility
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
    fn test_traffic_generator_creation() {
        let gen = TrafficGenerator::new(100);
        assert_eq!(gen.vehicles.len(), 100);
        assert_eq!(gen.traffic_lights.len(), 8); // 8 districts
    }

    #[test]
    fn test_generate_events() {
        let mut gen = TrafficGenerator::new(50);
        let events = gen.generate_events(100);

        assert!(events.len() <= 100); // Some events may be filtered
        assert!(!events.is_empty());
    }

    #[test]
    fn test_event_has_required_fields() {
        let mut gen = TrafficGenerator::new(10);
        let events = gen.generate_events(10);

        for event in &events {
            // All events should have timestamp and location
            assert!(event.get_field(3).is_some()); // timestamp
            assert!(event.get_field(10).is_some()); // lat
            assert!(event.get_field(11).is_some()); // lon
        }
    }
}
