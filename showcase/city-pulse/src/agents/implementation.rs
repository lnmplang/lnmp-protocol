//! Specific Agent Implementations

use super::system::{Agent, AgentStatus, AgentType};
use lnmp::prelude::*;

// ============================================================================
// Police Agent
// ============================================================================

pub struct PoliceAgent {
    id: String,
    status: AgentStatus,
    lat: f64,
    lon: f64,
    target_lat: Option<f64>,
    target_lon: Option<f64>,
}

impl PoliceAgent {
    pub fn new(id: &str, lat: f64, lon: f64, _unit_type: &str) -> Self {
        Self {
            id: id.to_string(),
            status: AgentStatus::Idle,
            lat,
            lon,
            target_lat: None,
            target_lon: None,
        }
    }
}

impl Agent for PoliceAgent {
    fn id(&self) -> &str {
        &self.id
    }
    fn agent_type(&self) -> AgentType {
        AgentType::Police
    }
    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
    fn location(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    fn handle_command(&mut self, command: &LnmpRecord) -> Vec<LnmpRecord> {
        let mut responses = Vec::new();

        // Check if it's a dispatch command
        if let Some(field) = command.get_field(2) {
            // Event Type
            if let LnmpValue::String(s) = &field.value {
                if s == "DISPATCH_COMMAND" {
                    // Extract target location
                    // In a real system, we'd parse coordinates from fields
                    // For demo, we'll simulate receiving coordinates
                    self.status = AgentStatus::Dispatched;
                    self.target_lat = Some(35.6595); // Shibuya
                    self.target_lon = Some(139.7004);

                    // Create acknowledgment event
                    let mut ack = LnmpRecord::new();
                    ack.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    ack.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("UNIT_DISPATCHED".to_string()),
                    });
                    ack.add_field(LnmpField {
                        fid: 210,
                        value: LnmpValue::String("POLICE".to_string()),
                    });
                    ack.add_field(LnmpField {
                        fid: 211,
                        value: LnmpValue::String(self.status_string()),
                    });
                    responses.push(ack);
                }
            }
        }

        responses
    }

    fn update(&mut self) -> Vec<LnmpRecord> {
        let mut updates = Vec::new();

        match self.status {
            AgentStatus::Dispatched => {
                // Simulate movement
                if let (Some(target_lat), Some(target_lon)) = (self.target_lat, self.target_lon) {
                    let lat_diff = target_lat - self.lat;
                    let lon_diff = target_lon - self.lon;

                    // Move 20% of the way (faster)
                    self.lat += lat_diff * 0.2;
                    self.lon += lon_diff * 0.2;

                    // Check arrival
                    if lat_diff.abs() < 0.0001 && lon_diff.abs() < 0.0001 {
                        self.status = AgentStatus::OnScene;

                        // Emit arrival event
                        let mut arrival = LnmpRecord::new();
                        arrival.add_field(LnmpField {
                            fid: 1,
                            value: LnmpValue::String(self.id.clone()),
                        });
                        arrival.add_field(LnmpField {
                            fid: 2,
                            value: LnmpValue::String("UNIT_ARRIVED".to_string()),
                        });
                        arrival.add_field(LnmpField {
                            fid: 10,
                            value: LnmpValue::Float(self.lat),
                        });
                        arrival.add_field(LnmpField {
                            fid: 11,
                            value: LnmpValue::Float(self.lon),
                        });
                        updates.push(arrival);
                    }
                }
            }
            AgentStatus::OnScene => {
                // Simulate resolving the issue (guaranteed for demo)
                if true {
                    self.status = AgentStatus::Returning;

                    let mut resolved = LnmpRecord::new();
                    resolved.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    resolved.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("INCIDENT_RESOLVED".to_string()),
                    });
                    updates.push(resolved);
                }
            }
            AgentStatus::Returning => {
                self.status = AgentStatus::Idle;
                self.target_lat = None;
                self.target_lon = None;
            }
            _ => {}
        }

        updates
    }
}

impl PoliceAgent {
    fn status_string(&self) -> String {
        match self.status {
            AgentStatus::Idle => "IDLE".to_string(),
            AgentStatus::Dispatched => "DISPATCHED".to_string(),
            AgentStatus::OnScene => "ON_SCENE".to_string(),
            AgentStatus::Returning => "RETURNING".to_string(),
            AgentStatus::Busy => "BUSY".to_string(),
        }
    }
}

// ============================================================================
// Ambulance Agent
// ============================================================================

pub struct AmbulanceAgent {
    id: String,
    status: AgentStatus,
    lat: f64,
    lon: f64,
    target_lat: Option<f64>,
    target_lon: Option<f64>,
    has_patient: bool,
}

impl AmbulanceAgent {
    pub fn new(id: &str, lat: f64, lon: f64) -> Self {
        Self {
            id: id.to_string(),
            status: AgentStatus::Idle,
            lat,
            lon,
            target_lat: None,
            target_lon: None,
            has_patient: false,
        }
    }
}

impl Agent for AmbulanceAgent {
    fn id(&self) -> &str {
        &self.id
    }
    fn agent_type(&self) -> AgentType {
        AgentType::Ambulance
    }
    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
    fn location(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    fn handle_command(&mut self, command: &LnmpRecord) -> Vec<LnmpRecord> {
        let mut responses = Vec::new();

        if let Some(field) = command.get_field(2) {
            if let LnmpValue::String(s) = &field.value {
                if s == "DISPATCH_COMMAND" {
                    self.status = AgentStatus::Dispatched;
                    // Simulate target
                    self.target_lat = Some(35.6895);
                    self.target_lon = Some(139.6917);

                    let mut ack = LnmpRecord::new();
                    ack.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    ack.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("MEDICAL_DISPATCHED".to_string()),
                    });
                    responses.push(ack);
                }
            }
        }
        responses
    }

    fn update(&mut self) -> Vec<LnmpRecord> {
        let mut updates = Vec::new();
        // Simplified movement logic similar to Police
        if self.status == AgentStatus::Dispatched {
            if let (Some(target_lat), Some(target_lon)) = (self.target_lat, self.target_lon) {
                self.lat += (target_lat - self.lat) * 0.1;
                self.lon += (target_lon - self.lon) * 0.1;

                if (target_lat - self.lat).abs() < 0.0001 {
                    self.status = AgentStatus::OnScene;
                    let mut arrival = LnmpRecord::new();
                    arrival.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    arrival.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("MEDICAL_ARRIVED".to_string()),
                    });
                    updates.push(arrival);
                }
            }
        } else if self.status == AgentStatus::OnScene {
            if simple_random(self.lat as u32) % 100 < 10 {
                self.status = AgentStatus::Returning; // Transporting to hospital
                self.has_patient = true;
                let mut transport = LnmpRecord::new();
                transport.add_field(LnmpField {
                    fid: 1,
                    value: LnmpValue::String(self.id.clone()),
                });
                transport.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String("PATIENT_TRANSPORT".to_string()),
                });
                updates.push(transport);
            }
        } else if self.status == AgentStatus::Returning {
            // Simulate returning to base/hospital
            if simple_random(self.lat as u32) % 100 < 10 {
                self.status = AgentStatus::Idle;
                self.has_patient = false;
            }
        }
        updates
    }
}

// Simple LCG random number generator
fn simple_random(seed: u32) -> u32 {
    seed.wrapping_mul(1664525).wrapping_add(1013904223)
}

// ============================================================================
// Fire Agent
// ============================================================================

pub struct FireAgent {
    id: String,
    status: AgentStatus,
    lat: f64,
    lon: f64,
    target_lat: Option<f64>,
    target_lon: Option<f64>,
    water_level: f32,
}

impl FireAgent {
    pub fn new(id: &str, lat: f64, lon: f64) -> Self {
        Self {
            id: id.to_string(),
            status: AgentStatus::Idle,
            lat,
            lon,
            target_lat: None,
            target_lon: None,
            water_level: 100.0,
        }
    }
}

impl Agent for FireAgent {
    fn id(&self) -> &str {
        &self.id
    }
    fn agent_type(&self) -> AgentType {
        AgentType::Fire
    }
    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
    fn location(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    fn handle_command(&mut self, command: &LnmpRecord) -> Vec<LnmpRecord> {
        let mut responses = Vec::new();

        if let Some(field) = command.get_field(2) {
            if let LnmpValue::String(s) = &field.value {
                if s == "DISPATCH_COMMAND" {
                    self.status = AgentStatus::Dispatched;
                    // Simulate target (e.g., near Shinjuku)
                    self.target_lat = Some(35.6895);
                    self.target_lon = Some(139.6917);

                    let mut ack = LnmpRecord::new();
                    ack.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    ack.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("FIRE_UNIT_DISPATCHED".to_string()),
                    });
                    responses.push(ack);
                }
            }
        }
        responses
    }

    fn update(&mut self) -> Vec<LnmpRecord> {
        let mut updates = Vec::new();

        match self.status {
            AgentStatus::Dispatched => {
                if let (Some(target_lat), Some(target_lon)) = (self.target_lat, self.target_lon) {
                    let lat_diff = target_lat - self.lat;
                    let lon_diff = target_lon - self.lon;

                    // Heavy vehicle, moves slower (15%)
                    self.lat += lat_diff * 0.15;
                    self.lon += lon_diff * 0.15;

                    if lat_diff.abs() < 0.0001 && lon_diff.abs() < 0.0001 {
                        self.status = AgentStatus::OnScene;
                        let mut arrival = LnmpRecord::new();
                        arrival.add_field(LnmpField {
                            fid: 1,
                            value: LnmpValue::String(self.id.clone()),
                        });
                        arrival.add_field(LnmpField {
                            fid: 2,
                            value: LnmpValue::String("FIRE_UNIT_ARRIVED".to_string()),
                        });
                        updates.push(arrival);
                    }
                }
            }
            AgentStatus::OnScene => {
                // Simulate extinguishing fire
                if self.water_level > 0.0 {
                    self.water_level -= 10.0;
                    if self.water_level <= 50.0 {
                        // 50% chance to resolve per tick once water is used
                        if simple_random(self.lat as u32) % 100 < 50 {
                            self.status = AgentStatus::Returning;
                            let mut resolved = LnmpRecord::new();
                            resolved.add_field(LnmpField {
                                fid: 1,
                                value: LnmpValue::String(self.id.clone()),
                            });
                            resolved.add_field(LnmpField {
                                fid: 2,
                                value: LnmpValue::String("INCIDENT_RESOLVED".to_string()),
                            });
                            updates.push(resolved);
                        }
                    }
                } else {
                    // Out of water, return to refill
                    self.status = AgentStatus::Returning;
                }
            }
            AgentStatus::Returning => {
                // Return to base
                self.status = AgentStatus::Idle;
                self.water_level = 100.0;
            }
            _ => {}
        }

        updates
    }
}

// ============================================================================
// Traffic Control Agent
// ============================================================================

pub struct TrafficAgent {
    id: String,
    status: AgentStatus,
    lat: f64,
    lon: f64,
    target_lat: Option<f64>,
    target_lon: Option<f64>,
}

impl TrafficAgent {
    pub fn new(id: &str, lat: f64, lon: f64) -> Self {
        Self {
            id: id.to_string(),
            status: AgentStatus::Idle,
            lat,
            lon,
            target_lat: None,
            target_lon: None,
        }
    }
}

impl Agent for TrafficAgent {
    fn id(&self) -> &str {
        &self.id
    }
    fn agent_type(&self) -> AgentType {
        AgentType::TrafficControl
    }
    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
    fn location(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    fn handle_command(&mut self, command: &LnmpRecord) -> Vec<LnmpRecord> {
        let mut responses = Vec::new();

        if let Some(field) = command.get_field(2) {
            if let LnmpValue::String(s) = &field.value {
                if s == "DISPATCH_COMMAND" {
                    self.status = AgentStatus::Dispatched;
                    // Simulate target
                    self.target_lat = Some(35.6895);
                    self.target_lon = Some(139.6917);

                    let mut ack = LnmpRecord::new();
                    ack.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    ack.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("TRAFFIC_UNIT_DISPATCHED".to_string()),
                    });
                    responses.push(ack);
                }
            }
        }
        responses
    }

    fn update(&mut self) -> Vec<LnmpRecord> {
        let mut updates = Vec::new();

        match self.status {
            AgentStatus::Dispatched => {
                if let (Some(target_lat), Some(target_lon)) = (self.target_lat, self.target_lon) {
                    let lat_diff = target_lat - self.lat;
                    let lon_diff = target_lon - self.lon;

                    // Fast movement (motorcycles)
                    self.lat += lat_diff * 0.25;
                    self.lon += lon_diff * 0.25;

                    if lat_diff.abs() < 0.0001 && lon_diff.abs() < 0.0001 {
                        self.status = AgentStatus::OnScene;
                        let mut arrival = LnmpRecord::new();
                        arrival.add_field(LnmpField {
                            fid: 1,
                            value: LnmpValue::String(self.id.clone()),
                        });
                        arrival.add_field(LnmpField {
                            fid: 2,
                            value: LnmpValue::String("TRAFFIC_UNIT_ARRIVED".to_string()),
                        });
                        updates.push(arrival);
                    }
                }
            }
            AgentStatus::OnScene => {
                // Simulate clearing traffic/accident
                if simple_random(self.lat as u32) % 100 < 20 {
                    self.status = AgentStatus::Returning;
                    let mut resolved = LnmpRecord::new();
                    resolved.add_field(LnmpField {
                        fid: 1,
                        value: LnmpValue::String(self.id.clone()),
                    });
                    resolved.add_field(LnmpField {
                        fid: 2,
                        value: LnmpValue::String("INCIDENT_RESOLVED".to_string()),
                    });
                    updates.push(resolved);
                }
            }
            AgentStatus::Returning => {
                self.status = AgentStatus::Idle;
            }
            _ => {}
        }

        updates
    }
}
