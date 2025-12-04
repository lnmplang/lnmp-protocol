//! Test Agent System

use city_pulse::agents::{AgentSystem, AmbulanceAgent, PoliceAgent};
use lnmp::prelude::*;

fn main() {
    println!("🤖 Testing Multi-Agent System...");

    let mut system = AgentSystem::new();

    // Register agents
    system.register_agent(Box::new(PoliceAgent::new(
        "POLICE_01",
        35.6800,
        139.7600,
        "PATROL",
    )));
    system.register_agent(Box::new(AmbulanceAgent::new(
        "AMBULANCE_01",
        35.6800,
        139.7600,
    )));

    println!("   Registered {} agents", system.active_agent_count());

    // Create a dispatch command
    let mut cmd = LnmpRecord::new();
    cmd.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::String("HQ".to_string()),
    });
    cmd.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("DISPATCH_COMMAND".to_string()),
    });
    cmd.add_field(LnmpField {
        fid: 210,
        value: LnmpValue::String("POLICE".to_string()),
    });

    println!("   Sending dispatch command to POLICE...");
    let responses = system.process_commands(vec![cmd]);

    println!("   Received {} responses", responses.len());
    for resp in &responses {
        if let Some(field) = resp.get_field(2) {
            println!("   Response: {:?}", field.value);
        }
    }

    // Simulate updates
    println!("   Simulating updates...");
    for _ in 0..5 {
        let updates = system.update_all();
        if !updates.is_empty() {
            println!("   Updates received: {}", updates.len());
        }
    }

    println!("✅ Agent System Test Complete");
}
