//! Compound Crisis Scenario (Multi-Hazard)
//!
//! Simulates a traffic accident that causes a fire, requiring
//! coordinated response from Traffic Control and Fire Department.

use super::Scenario;
use crate::agents::AgentSystem;
use crate::components::TrafficGenerator;
use crate::LNMPPipeline;
use lnmp::prelude::*;

pub struct CompoundCrisisScenario {
    step_count: usize,
    traffic_gen: TrafficGenerator,
    resolved_traffic: bool,
    resolved_fire: bool,
}

impl Default for CompoundCrisisScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl CompoundCrisisScenario {
    pub fn new() -> Self {
        Self {
            step_count: 0,
            traffic_gen: TrafficGenerator::new(100),
            resolved_traffic: false,
            resolved_fire: false,
        }
    }
}

impl Scenario for CompoundCrisisScenario {
    fn name(&self) -> &str {
        "Multi-Hazard: Accident + Fire"
    }

    fn init(&mut self) {
        println!("⚠️  SCENARIO STARTED: {}", self.name());
        println!("   Context: Major intersection accident reported.");
    }

    fn step(&mut self, pipeline: &mut LNMPPipeline, agents: &mut AgentSystem) -> bool {
        self.step_count += 1;

        // 1. Generate background events
        let mut events = self.traffic_gen.generate_events(40);

        // 2. Inject scenario events
        if self.step_count == 3 {
            println!("\n📍 Step 3: MAJOR ACCIDENT DETECTED");
            // Traffic Accident (F26)
            let mut acc = LnmpRecord::new();
            acc.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("TRAFFIC_ACCIDENT".to_string()),
            });
            acc.add_field(LnmpField {
                fid: 26,
                value: LnmpValue::Bool(true),
            }); // Accident
            acc.add_field(LnmpField {
                fid: 22,
                value: LnmpValue::Float(0.95),
            }); // High risk
            acc.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6895),
            });
            acc.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.6917),
            });
            events.push(acc);
        } else if self.step_count == 6 {
            println!("\n📍 Step 6: FIRE OUTBREAK AT ACCIDENT SITE");
            // Fire (F61)
            let mut fire = LnmpRecord::new();
            fire.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("FIRE_DETECTED".to_string()),
            });
            fire.add_field(LnmpField {
                fid: 61,
                value: LnmpValue::Bool(true),
            });
            fire.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6895),
            });
            fire.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.6917),
            });
            events.push(fire);
        }

        // 3. Process Pipeline
        let output = pipeline.process(events);

        // 4. LLM Decision (Real AI if API key is available)
        if output.stats.critical_events > 0 {
            // Generate context-aware prompt
            let prompt = if self.step_count >= 6 {
                format!(
                    "URGENT: Traffic accident has escalated to fire outbreak!\n\
                    Location: Tokyo intersection (35.6895, 139.6917)\n\
                    Critical Events: {}\n\
                    Status: {} ongoing fires, traffic {} cleared\n\n\
                    What emergency response actions are needed?",
                    output.stats.critical_events,
                    if self.resolved_fire { "Fire" } else { "Active" },
                    if self.resolved_traffic { "" } else { "NOT" }
                )
            } else if self.step_count >= 3 {
                format!(
                    "Major traffic accident detected at Tokyo intersection.\n\
                    Critical Events: {}\n\
                    Vehicles involved, potential injuries.\n\n\
                    What immediate response is required?",
                    output.stats.critical_events
                )
            } else {
                // If no critical events and step_count < 3, we don't need an LLM prompt for critical response
                // The original code would have returned "MONITORING TRAFFIC FLOW."
                // For this new prompt generation logic, we can skip LLM processing if not critical.
                return true; // Continue to next step without LLM interaction for critical events
            };

            // Call LLM (real API or simulated)
            let llm_response = pipeline.process_llm_response(&prompt);
            let mut commands = llm_response.actions;

            // Manual dispatch logic for demo
            if self.step_count == 4 && !self.resolved_traffic {
                // Dispatch Traffic
                let mut cmd = LnmpRecord::new();
                cmd.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
                });
                cmd.add_field(LnmpField {
                    fid: 210,
                    value: LnmpValue::String("TRAFFIC_CONTROL".to_string()),
                });
                commands.push(cmd);
            }

            if self.step_count == 7 && !self.resolved_fire {
                // Dispatch Fire
                let mut cmd = LnmpRecord::new();
                cmd.add_field(LnmpField {
                    fid: 2,
                    value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
                });
                cmd.add_field(LnmpField {
                    fid: 210,
                    value: LnmpValue::String("FIRE".to_string()),
                });
                commands.push(cmd);
            }

            let agent_responses = agents.process_commands(commands);
            for resp in agent_responses {
                if let Some(field) = resp.get_field(2) {
                    if let LnmpValue::String(s) = &field.value {
                        println!("   🚓 AGENT ACTION: {}", s);
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
                        // Check agent type to know what was resolved
                        // For demo simplicity, we'll assume based on timing or just count resolutions
                        // But we can check the source ID if we tracked it.
                        // Let's just say if we get a resolution, we check what's pending.

                        if !self.resolved_traffic {
                            self.resolved_traffic = true;
                            println!("   ✅ TRAFFIC CLEARED");
                        } else if !self.resolved_fire {
                            self.resolved_fire = true;
                            println!("   ✅ FIRE EXTINGUISHED");
                        }
                    }
                }
            }
        }

        // Completion condition
        if self.resolved_traffic && self.resolved_fire {
            println!("\n✅ SCENARIO RESOLVED: Accident cleared and fire extinguished.");
            return false;
        }

        if self.step_count >= 30 {
            println!("\n❌ SCENARIO TIMEOUT");
            return false;
        }

        true
    }

    fn status(&self) -> String {
        format!(
            "Traffic: {}, Fire: {}",
            if self.resolved_traffic {
                "CLEARED"
            } else {
                "BLOCKED"
            },
            if self.resolved_fire {
                "EXTINGUISHED"
            } else {
                "ACTIVE"
            }
        )
    }
}
