//! Event Types and FID Mapping for Tokyo Smart City OS
//!
//! This module defines all event categories, types, and Field ID (FID) mappings
//! according to the Tokyo Smart City specification.

use std::collections::HashMap;

/// Event categories in the Tokyo Smart City OS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// Traffic and mobility events (F20-F29)
    Traffic,
    /// Security and public safety events (F50-F59)
    Security,
    /// Disaster and natural hazard monitoring (F60-F79)
    Disaster,
    /// Public order and civic behavior (F80-F89)
    PublicOrder,
    /// Municipal operations and infrastructure (F90-F99)
    Municipal,
    /// Health and environment (F100-F109)
    Health,
}

/// Comprehensive event types for Tokyo Smart City OS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    // ========== Traffic & Mobility (F20-F29) ==========
    /// Vehicle exceeding speed limit
    Speeding,
    /// Red light violation detected
    RedLightViolation,
    /// High risk of accident detected
    AccidentRisk,
    /// Emergency vehicle requiring priority
    EmergencyVehicle,
    /// Traffic congestion level
    TrafficCongestion,
    /// Aggressive driving behavior detected by AI
    AggressiveDriving,
    /// Actual traffic accident
    TrafficAccident,

    // ========== Security & Public Safety (F50-F59) ==========
    /// Violence or assault detected
    Violence,
    /// Weapon detected by AI
    WeaponDetected,
    /// Theft or mugging incident
    Theft,
    /// Suspicious package detected
    SuspiciousPackage,
    /// Unauthorized entry or trespassing
    UnauthorizedEntry,
    /// Suspicious behavior pattern
    SuspiciousBehavior,

    // ========== Disaster & Natural Hazards (F60-F79) ==========
    /// Flood water level monitoring
    FloodLevel,
    /// Fire detected
    FireDetected,
    /// Tsunami early warning
    TsunamiAlert,
    /// Earthquake precursor signal
    EarthquakePrecursor,
    /// Storm tracking
    StormWarning,
    /// Infrastructure collapse risk
    InfrastructureCollapse,
    /// Smoke detection
    SmokeDetected,

    // ========== Public Order (F80-F89) ==========
    /// Littering detected
    Littering,
    /// Vandalism or property damage
    Vandalism,
    /// Public disturbance or noise
    PublicDisturbance,
    /// Unauthorized occupation
    UnauthorizedOccupation,

    // ========== Municipal Operations (F90-F99) ==========
    /// Traffic light malfunction
    TrafficLightFailure,
    /// Water or electricity outage
    UtilityOutage,
    /// Road damage or pothole
    RoadDamage,
    /// Garbage container full
    GarbageContainerFull,
    /// Sidewalk damage
    SidewalkDamage,
    /// Maintenance crew status update
    MaintenanceCrewUpdate,

    // ========== Health & Environment (F100-F109) ==========
    /// Medical emergency call
    MedicalEmergency,
    /// Person collapsed in public
    PersonCollapsed,
    /// Toxic air quality level
    ToxicAirQuality,
    /// Radiation level monitoring
    RadiationLevel,
    /// High PM2.5 level
    HighPM25,
}

impl EventType {
    /// Get the primary FID for this event type
    pub fn primary_fid(&self) -> u32 {
        match self {
            // Traffic
            Self::Speeding => 20,
            Self::RedLightViolation => 21,
            Self::AccidentRisk => 22,
            Self::EmergencyVehicle => 23,
            Self::TrafficCongestion => 24,
            Self::AggressiveDriving => 25,
            Self::TrafficAccident => 26,

            // Security
            Self::Violence => 50,
            Self::WeaponDetected => 51,
            Self::Theft => 52,
            Self::SuspiciousPackage => 53,
            Self::UnauthorizedEntry => 54,
            Self::SuspiciousBehavior => 55,

            // Disaster
            Self::FloodLevel => 60,
            Self::FireDetected => 61,
            Self::TsunamiAlert => 62,
            Self::StormWarning => 63,
            Self::EarthquakePrecursor => 70,
            Self::InfrastructureCollapse => 71,
            Self::SmokeDetected => 72,

            // Public Order
            Self::Littering => 80,
            Self::Vandalism => 90,
            Self::PublicDisturbance => 81,
            Self::UnauthorizedOccupation => 82,

            // Municipal
            Self::TrafficLightFailure => 91,
            Self::UtilityOutage => 92,
            Self::RoadDamage => 93,
            Self::GarbageContainerFull => 94,
            Self::SidewalkDamage => 95,
            Self::MaintenanceCrewUpdate => 96,

            // Health
            Self::MedicalEmergency => 100,
            Self::PersonCollapsed => 101,
            Self::ToxicAirQuality => 65,
            Self::RadiationLevel => 102,
            Self::HighPM25 => 103,
        }
    }

    /// Get the event category
    pub fn category(&self) -> EventCategory {
        match self {
            Self::Speeding
            | Self::RedLightViolation
            | Self::AccidentRisk
            | Self::EmergencyVehicle
            | Self::TrafficCongestion
            | Self::AggressiveDriving
            | Self::TrafficAccident => EventCategory::Traffic,

            Self::Violence
            | Self::WeaponDetected
            | Self::Theft
            | Self::SuspiciousPackage
            | Self::UnauthorizedEntry
            | Self::SuspiciousBehavior => EventCategory::Security,

            Self::FloodLevel
            | Self::FireDetected
            | Self::TsunamiAlert
            | Self::StormWarning
            | Self::EarthquakePrecursor
            | Self::InfrastructureCollapse
            | Self::SmokeDetected => EventCategory::Disaster,

            Self::Littering | Self::PublicDisturbance | Self::UnauthorizedOccupation => {
                EventCategory::PublicOrder
            }

            Self::Vandalism
            | Self::TrafficLightFailure
            | Self::UtilityOutage
            | Self::RoadDamage
            | Self::GarbageContainerFull
            | Self::SidewalkDamage
            | Self::MaintenanceCrewUpdate => EventCategory::Municipal,

            Self::MedicalEmergency
            | Self::PersonCollapsed
            | Self::ToxicAirQuality
            | Self::RadiationLevel
            | Self::HighPM25 => EventCategory::Health,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Speeding => "Vehicle exceeding speed limit",
            Self::RedLightViolation => "Red light violation detected",
            Self::AccidentRisk => "High accident risk detected",
            Self::EmergencyVehicle => "Emergency vehicle requiring priority",
            Self::TrafficCongestion => "Traffic congestion detected",
            Self::AggressiveDriving => "Aggressive driving behavior",
            Self::TrafficAccident => "Traffic accident occurred",

            Self::Violence => "Violence or assault detected",
            Self::WeaponDetected => "Weapon detected by AI",
            Self::Theft => "Theft or mugging incident",
            Self::SuspiciousPackage => "Suspicious package detected",
            Self::UnauthorizedEntry => "Unauthorized entry detected",
            Self::SuspiciousBehavior => "Suspicious behavior pattern",

            Self::FloodLevel => "Flood water level alert",
            Self::FireDetected => "Fire detected",
            Self::TsunamiAlert => "Tsunami early warning",
            Self::StormWarning => "Storm warning issued",
            Self::EarthquakePrecursor => "Earthquake precursor signal",
            Self::InfrastructureCollapse => "Infrastructure collapse risk",
            Self::SmokeDetected => "Smoke detected",

            Self::Littering => "Littering detected",
            Self::Vandalism => "Vandalism or property damage",
            Self::PublicDisturbance => "Public disturbance",
            Self::UnauthorizedOccupation => "Unauthorized occupation",

            Self::TrafficLightFailure => "Traffic light malfunction",
            Self::UtilityOutage => "Water or electricity outage",
            Self::RoadDamage => "Road damage detected",
            Self::GarbageContainerFull => "Garbage container full",
            Self::SidewalkDamage => "Sidewalk damage",
            Self::MaintenanceCrewUpdate => "Maintenance crew status",

            Self::MedicalEmergency => "Medical emergency",
            Self::PersonCollapsed => "Person collapsed in public",
            Self::ToxicAirQuality => "Toxic air quality level",
            Self::RadiationLevel => "Radiation level alert",
            Self::HighPM25 => "High PM2.5 pollution",
        }
    }
}

/// Field ID (FID) to importance score mapping
///
/// Importance scores range from 0-255:
/// - 0-100: Low priority (routine events)
/// - 101-200: Medium priority (notable events)
/// - 201-254: High priority (urgent events)
/// - 255: Critical priority (life-threatening)
pub struct FieldImportance;

impl FieldImportance {
    /// Get importance score for a given FID
    pub fn get(fid: u32) -> u8 {
        match fid {
            // Common metadata fields
            1 => 100,   // source_id
            2 => 100,   // event_type
            3 => 100,   // timestamp
            10 => 180,  // geo_lat
            11 => 180,  // geo_lon
            120 => 100, // camera_confidence

            // Traffic (F20-F29)
            20 => 160, // vehicle_speed (speeding)
            21 => 210, // red_light_violation
            22 => 255, // accident_risk (critical)
            23 => 230, // emergency_vehicle
            24 => 140, // traffic_congestion
            25 => 180, // aggressive_driving
            26 => 250, // traffic_accident

            // Security (F50-F59)
            50 => 255, // violence (critical)
            51 => 255, // weapon_detected (critical)
            52 => 240, // theft
            53 => 230, // suspicious_package
            54 => 200, // unauthorized_entry
            55 => 190, // suspicious_behavior

            // Disaster (F60-F79)
            60 => 230, // flood_level
            61 => 250, // fire_detected
            62 => 255, // tsunami_alert (critical)
            63 => 210, // storm_warning
            65 => 250, // air_quality_toxic
            70 => 255, // earthquake_precursor (critical)
            71 => 240, // infrastructure_collapse
            72 => 230, // smoke_detected

            // Public Order (F80-F89)
            80 => 80,  // littering
            81 => 140, // public_disturbance
            82 => 160, // unauthorized_occupation

            // Municipal (F90-F99)
            90 => 200, // vandalism
            91 => 190, // traffic_light_failure
            92 => 180, // utility_outage
            93 => 170, // road_damage
            94 => 120, // garbage_container_full
            95 => 150, // sidewalk_damage
            96 => 100, // maintenance_crew_update

            // Health (F100-F109)
            100 => 250, // medical_emergency
            101 => 245, // person_collapsed
            102 => 220, // radiation_level
            103 => 190, // high_pm25

            // Default for unknown fields
            _ => 100,
        }
    }

    /// Get all FID to importance mappings
    pub fn all_mappings() -> HashMap<u32, u8> {
        let mut map = HashMap::new();

        // Add all known FIDs
        for fid in [
            1, 2, 3, 10, 11, 120, 20, 21, 22, 23, 24, 25, 26, 50, 51, 52, 53, 54, 55, 60, 61, 62,
            63, 65, 70, 71, 72, 80, 81, 82, 90, 91, 92, 93, 94, 95, 96, 100, 101, 102, 103,
        ] {
            map.insert(fid, Self::get(fid));
        }

        map
    }

    /// Get human-readable field name for a FID
    pub fn field_name(fid: u32) -> &'static str {
        match fid {
            1 => "source_id",
            2 => "event_type",
            3 => "timestamp",
            10 => "geo_lat",
            11 => "geo_lon",
            20 => "vehicle_speed",
            21 => "red_light_violation",
            22 => "accident_risk",
            23 => "emergency_vehicle",
            24 => "traffic_congestion",
            25 => "aggressive_driving",
            26 => "traffic_accident",
            50 => "violence_detected",
            51 => "weapon_detected",
            52 => "theft",
            53 => "suspicious_package",
            54 => "unauthorized_entry",
            55 => "suspicious_behavior",
            60 => "flood_level",
            61 => "fire_detected",
            62 => "tsunami_alert",
            63 => "storm_warning",
            65 => "air_quality_toxic",
            70 => "earthquake_signal",
            71 => "infrastructure_collapse",
            72 => "smoke_detected",
            80 => "littering",
            81 => "public_disturbance",
            82 => "unauthorized_occupation",
            90 => "vandalism",
            91 => "traffic_light_failure",
            92 => "utility_outage",
            93 => "road_damage",
            94 => "garbage_container_full",
            95 => "sidewalk_damage",
            96 => "maintenance_crew",
            100 => "medical_emergency",
            101 => "person_collapsed",
            102 => "radiation_level",
            103 => "pm25_level",
            120 => "camera_confidence",
            _ => "unknown_field",
        }
    }
}

/// Priority levels for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl Priority {
    /// Convert importance score to priority level
    pub fn from_importance(importance: u8) -> Self {
        match importance {
            0..=100 => Self::Low,
            101..=200 => Self::Medium,
            201..=254 => Self::High,
            255 => Self::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_fid_mapping() {
        assert_eq!(EventType::Violence.primary_fid(), 50);
        assert_eq!(EventType::WeaponDetected.primary_fid(), 51);
        assert_eq!(EventType::RedLightViolation.primary_fid(), 21);
    }

    #[test]
    fn test_importance_scores() {
        // Critical events should have 255
        assert_eq!(FieldImportance::get(50), 255); // violence
        assert_eq!(FieldImportance::get(51), 255); // weapon
        assert_eq!(FieldImportance::get(70), 255); // earthquake

        // High priority
        assert_eq!(FieldImportance::get(21), 210); // red light violation

        // Low priority
        assert_eq!(FieldImportance::get(80), 80); // littering
    }

    #[test]
    fn test_event_categories() {
        assert_eq!(EventType::Violence.category(), EventCategory::Security);
        assert_eq!(EventType::FireDetected.category(), EventCategory::Disaster);
        assert_eq!(EventType::Speeding.category(), EventCategory::Traffic);
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(Priority::from_importance(50), Priority::Low);
        assert_eq!(Priority::from_importance(150), Priority::Medium);
        assert_eq!(Priority::from_importance(220), Priority::High);
        assert_eq!(Priority::from_importance(255), Priority::Critical);
    }
}
