//! Earthquake Response Scenario
//!
//! Simulates a major seismic event in Tokyo requiring
//! coordinated response from Fire, Medical, and Police units.

use super::Scenario;
use crate::agents::AgentSystem;
use crate::components::DisasterGenerator;
use crate::LNMPPipeline;
use lnmp::prelude::*;

pub struct EarthquakeScenario {
    step_count: usize,
    disaster_gen: DisasterGenerator,
    magnitude: f32,
    resolved_fires: usize,
    evacuation_status: bool,
}

impl Default for EarthquakeScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl EarthquakeScenario {
    pub fn new() -> Self {
        Self {
            step_count: 0,
            disaster_gen: DisasterGenerator::new(0.5),
            magnitude: 0.0,
            resolved_fires: 0,
            evacuation_status: false,
        }
    }
}

impl Scenario for EarthquakeScenario {
    fn name(&self) -> &str {
        "Major Earthquake (Magnitude 7.0)"
    }

    fn init(&mut self) {
        println!("⚠️  SCENARIO STARTED: {}", self.name());
        println!("   Context: Seismic sensors detecting P-wave precursors.");
    }

    fn step(&mut self, pipeline: &mut LNMPPipeline, agents: &mut AgentSystem) -> bool {
        self.step_count += 1;

        // 1. Generate background events
        let mut events = self.disaster_gen.generate_events(30);

        // 2. Inject scenario events
        if self.step_count == 3 {
            println!("\n📍 Step 3: P-Wave Detected (Early Warning)");
            // Precursor event
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("EARTHQUAKE_PRECURSOR".to_string()),
            });
            record.add_field(LnmpField {
                fid: 70,
                value: LnmpValue::Float(4.5),
            }); // Magnitude estimate
            record.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6895),
            });
            record.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.6917),
            });
            events.push(record);
        } else if self.step_count == 6 {
            println!("\n📍 Step 6: MAIN SHOCK - MAGNITUDE 7.2");
            self.magnitude = 7.2;
            // Main shock event
            events.push(self.disaster_gen.generate_major_earthquake(7.2));

            // Secondary disasters (Fires)
            for i in 0..3 {
                let mut fire = LnmpRecord::new();
                fire.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String(format!("FIRE_OUTBREAK_{}", i)),
                });
                fire.add_field(LnmpField {
                    fid: 61,
                    value: LnmpValue::Bool(true),
                });
                fire.add_field(LnmpField {
                    fid: 10,
                    value: LnmpValue::Float(35.6895 + (i as f64 * 0.01)),
                });
                fire.add_field(LnmpField {
                    fid: 11,
                    value: LnmpValue::Float(139.6917 + (i as f64 * 0.01)),
                });
                events.push(fire);
            }
        }

        // 3. Process Pipeline
        let output = pipeline.process(events);

        // 4. LLM Decision
        if output.stats.critical_events > 0 {
            let llm_response = if self.magnitude > 7.0 {
                "MAJOR EARTHQUAKE CONFIRMED.\nINITIATE CITY-WIDE EVACUATION.\nDISPATCH ALL FIRE UNITS TO OUTBREAKS.\nDISPATCH MEDICAL TO TRIAGE CENTERS."
            } else if self.magnitude > 4.0 {
                "SEISMIC ACTIVITY DETECTED.\nISSUE EARLY WARNING.\nPUT EMERGENCY SERVICES ON STANDBY."
            } else {
                "MONITORING SEISMIC ACTIVITY."
            };

            let parsed_response = pipeline.process_llm_response(llm_response);

            // Custom parsing for this scenario (since LLB is generic)
            // We'll manually inject commands for the demo if LLB doesn't catch them all
            // But let's rely on the generic "DISPATCH" keyword logic we added to LLB

            // For the demo, we'll also manually trigger specific agent types if the generic parser misses
            // or if we want specific behavior.
            let mut commands = parsed_response.actions;

            if self.magnitude > 7.0 && !self.evacuation_status {
                self.evacuation_status = true;
                // Broadcast evacuation
                println!("   📢 SYSTEM BROADCAST: EVACUATION ORDER ISSUED");

                // Dispatch Fire Units
                let mut fire_cmd = LnmpRecord::new();
                fire_cmd.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
                });
                fire_cmd.add_field(LnmpField {
                    fid: 210,
                    value: LnmpValue::String("FIRE".to_string()),
                });
                commands.push(fire_cmd);

                // Dispatch Medical
                let mut med_cmd = LnmpRecord::new();
                med_cmd.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
                });
                med_cmd.add_field(LnmpField {
                    fid: 210,
                    value: LnmpValue::String("AMBULANCE".to_string()),
                });
                commands.push(med_cmd);
            }

            let agent_responses = agents.process_commands(commands);
            for resp in agent_responses {
                if let Some(field) = resp.get_field(2) {
                    if let LnmpValue::String(s) = &field.value {
                        println!("   🚒 AGENT ACTION: {}", s);
                    }
                }
            }
        }

        // 5. Update Agents
        let updates = agents.update_all();
        for update in updates {
            if let Some(field) = update.get_field(2) {
                if let LnmpValue::String(s) = &field.value {
                    if s == "INCIDENT_RESOLVED" {
                        self.resolved_fires += 1;
                        println!("   ✅ FIRE EXTINGUISHED / RESCUE COMPLETE");
                    }
                }
            }
        }

        // Completion condition
        if self.resolved_fires >= 3 {
            println!("\n✅ SCENARIO RESOLVED: All fires extinguished, evacuation complete.");
            return false;
        }

        if self.step_count >= 40 {
            println!("\n❌ SCENARIO TIMEOUT: Disaster response took too long.");
            return false;
        }

        true
    }

    fn status(&self) -> String {
        format!(
            "Mag: {:.1}, Fires Resolved: {}",
            self.magnitude, self.resolved_fires
        )
    }
}
