//! Gang Violence Escalation Scenario
//!
//! Simulates a rising conflict in Shibuya district that requires
//! coordinated police and medical response.

use super::Scenario;
use crate::agents::AgentSystem;
use crate::{LNMPPipeline, SecurityGenerator, TrafficGenerator};
use lnmp::prelude::*;

pub struct GangViolenceScenario {
    step_count: usize,
    traffic_gen: TrafficGenerator,
    security_gen: SecurityGenerator,
    escalation_level: f32, // 0.0 to 1.0
    resolved: bool,
}

impl Default for GangViolenceScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl GangViolenceScenario {
    pub fn new() -> Self {
        Self {
            step_count: 0,
            traffic_gen: TrafficGenerator::new(100),
            security_gen: SecurityGenerator::new(50),
            escalation_level: 0.0,
            resolved: false,
        }
    }
}

impl Scenario for GangViolenceScenario {
    fn name(&self) -> &str {
        "Gang Violence Escalation (Shibuya)"
    }

    fn init(&mut self) {
        println!("⚠️  SCENARIO STARTED: {}", self.name());
        println!("   Context: Intelligence reports rising tensions in Shibuya district.");
    }

    fn step(&mut self, pipeline: &mut LNMPPipeline, agents: &mut AgentSystem) -> bool {
        self.step_count += 1;

        // 1. Generate background noise events
        let mut events = self.traffic_gen.generate_events(50);
        events.extend(self.security_gen.generate_events(20));

        // 2. Inject scenario-specific events based on step
        if self.step_count == 2 {
            println!("\n📍 Step 2: Initial Disturbance");
            // Add suspicious behavior events
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("SUSPICIOUS_GATHERING".to_string()),
            });
            record.add_field(LnmpField {
                fid: 55,
                value: LnmpValue::Bool(true),
            }); // suspicious
            record.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6595),
            }); // Shibuya
            record.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.7004),
            });
            events.push(record);
        } else if self.step_count == 5 {
            println!("\n📍 Step 5: Escalation to Violence");
            self.escalation_level = 0.5;
            // Add violence event
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("VIOLENCE_DETECTED".to_string()),
            });
            record.add_field(LnmpField {
                fid: 50,
                value: LnmpValue::Bool(true),
            }); // violence
            record.add_field(LnmpField {
                fid: 201,
                value: LnmpValue::Int(5),
            }); // 5 people
            record.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6595),
            });
            record.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.7004),
            });
            events.push(record);
        } else if self.step_count == 8 {
            println!("\n📍 Step 8: Weapon Sighting - CRITICAL");
            self.escalation_level = 1.0;
            // Add weapon event
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 2,
                value: LnmpValue::String("WEAPON_DETECTED".to_string()),
            });
            record.add_field(LnmpField {
                fid: 51,
                value: LnmpValue::Bool(true),
            }); // weapon
            record.add_field(LnmpField {
                fid: 120,
                value: LnmpValue::Float(0.95),
            }); // high confidence
            record.add_field(LnmpField {
                fid: 10,
                value: LnmpValue::Float(35.6595),
            });
            record.add_field(LnmpField {
                fid: 11,
                value: LnmpValue::Float(139.7004),
            });
            events.push(record);
        }

        // 3. Process through Pipeline
        let output = pipeline.process(events);

        // 4. AI Decision Making (Real OpenAI API if key available)
        if output.stats.critical_events > 0 {
            // Generate context-aware prompt based on situation
            let prompt = format!(
                "TOKYO EMERGENCY DISPATCH CENTER\n\
                Location: Shibuya District (35.6595, 139.7004)\n\
                Critical Events Detected: {}\n\
                Escalation Level: {:.1}/1.0\n\
                Current Situation: {}\n\n\
                Based on this intelligence, what immediate response actions should be taken?",
                output.stats.critical_events,
                self.escalation_level,
                if self.escalation_level >= 1.0 {
                    "WEAPON CONFIRMED - Armed conflict in progress, civilians at risk"
                } else if self.escalation_level >= 0.5 {
                    "VIOLENCE DETECTED - Physical altercation between multiple individuals"
                } else {
                    "Suspicious gathering reported, monitoring for escalation"
                }
            );

            // 5. Execute Actions via Agents (with real AI or simulated)
            println!("\n   🤖 Calling AI for decision...");
            let parsed_response = pipeline.process_llm_response(&prompt);
            println!("   💬 AI Analysis: {} chars", parsed_response.analysis.len());
            println!("   📋 AI Actions: {} commands", parsed_response.actions.len());
            
            let agent_responses = agents.process_commands(parsed_response.actions);

            for resp in agent_responses {
                if let Some(field) = resp.get_field(2) {
                    if let LnmpValue::String(s) = &field.value {
                        println!("   👮 AGENT ACTION: {}", s);
                    }
                }
            }
        }

        // 6. Update Agents
        let updates = agents.update_all();
        for update in updates {
            if let Some(field) = update.get_field(2) {
                if let LnmpValue::String(s) = &field.value {
                    if s == "INCIDENT_RESOLVED" {
                        println!("\n✅ SCENARIO RESOLVED: Police neutralized the threat.");
                        self.resolved = true;
                        return false; // End scenario
                    }
                }
            }
        }

        // Check timeout
        if self.step_count >= 30 {
            println!("\n❌ SCENARIO TIMEOUT: Incident not resolved in time.");
            return false;
        }

        true // Continue
    }

    fn status(&self) -> String {
        format!(
            "Step: {}, Escalation: {:.1}",
            self.step_count, self.escalation_level
        )
    }
}
