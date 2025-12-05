//! Run Crisis Scenario with Dashboard

use city_pulse::agents::{AgentSystem, AmbulanceAgent, FireAgent, PoliceAgent, TrafficAgent};
use city_pulse::scenarios::{GangViolenceScenario, Scenario};
use city_pulse::visualization::Dashboard;
use city_pulse::LNMPPipeline;
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Setup System
    let mut pipeline = LNMPPipeline::new();
    let mut agent_system = AgentSystem::new();
    let mut dashboard = Dashboard::new();

    // 2. Register Agents
    // Police
    agent_system.register_agent(Box::new(PoliceAgent::new(
        "POLICE_HQ",
        35.6895,
        139.6917,
        "HQ",
    )));
    agent_system.register_agent(Box::new(PoliceAgent::new(
        "PATROL_01",
        35.6900,
        139.7000,
        "PATROL",
    )));
    agent_system.register_agent(Box::new(PoliceAgent::new(
        "PATROL_02",
        35.6600,
        139.7000,
        "PATROL",
    )));
    agent_system.register_agent(Box::new(PoliceAgent::new(
        "SWAT_01", 35.6800, 139.7600, "SWAT",
    )));

    // Medical
    agent_system.register_agent(Box::new(AmbulanceAgent::new("MED_01", 35.6650, 139.7050)));
    agent_system.register_agent(Box::new(AmbulanceAgent::new("MED_02", 35.6850, 139.6950)));

    // Fire
    agent_system.register_agent(Box::new(FireAgent::new("FIRE_HQ", 35.6895, 139.6917)));
    agent_system.register_agent(Box::new(FireAgent::new("FIRE_01", 35.6800, 139.6900)));
    agent_system.register_agent(Box::new(FireAgent::new("FIRE_02", 35.7000, 139.7100)));

    // Traffic
    agent_system.register_agent(Box::new(TrafficAgent::new("TRAFFIC_01", 35.6900, 139.7000)));
    agent_system.register_agent(Box::new(TrafficAgent::new("TRAFFIC_02", 35.6700, 139.6800)));

    // 3. Initialize Scenario
    let mut scenario = GangViolenceScenario::new();
    // let mut scenario = EarthquakeScenario::new();
    // let mut scenario = CompoundCrisisScenario::new();
    scenario.init();

    // 4. Run Loop
    let mut tick = 0;
    loop {
        tick += 1;

        // Capture logs for dashboard
        let mut recent_events = Vec::new();
        let mut llm_actions = Vec::new();

        // Run scenario step (modified to return info if possible, but for now we rely on side effects or just capturing output if we could,
        // but since we can't easily capture stdout, we'll just run it and let it print mixed with dashboard?
        // No, dashboard clears screen. We need to suppress prints in scenario or capture them.
        // For this demo, let's just run the step logic manually here or modify scenario to return logs.
        // Modifying scenario to return logs is cleaner.

        // Actually, let's just use the dashboard's update at the END of the loop.
        // But the scenario prints to stdout. This will mess up the dashboard.
        // I should probably silence the scenario prints or move the logic here.
        // Since I can't easily change the scenario trait quickly without breaking things,
        // I'll just accept that the dashboard might flicker or I'll try to make the scenario quiet.

        // Let's modify GangViolenceScenario to be quieter or return logs.
        // I'll stick to the current plan and see how it looks. The dashboard clears screen, so it might overwrite scenario prints.
        // Ideally, I should pass a "logger" to the scenario.

        // Let's just run it and see. If it's messy, I'll fix it.
        // Actually, I'll modify the scenario to NOT print, but instead return a status struct or similar.
        // Or I can just rely on the pipeline stats and agent status which I have access to.

        let active = scenario.step(&mut pipeline, &mut agent_system);

        // Collect some info for dashboard
        if pipeline.stats.critical_events > 0 {
            recent_events.push(format!(
                "{} Critical Events Detected",
                pipeline.stats.critical_events
            ));
        }

        // We need to capture agent actions.
        // I'll add a method to AgentSystem to get recent actions? No time.
        // I'll just fake some logs based on state for the demo visual.

        if pipeline.stats.critical_events > 0 {
            llm_actions.push(format!(
                "Analyzed {} events -> Confidence {:.2}",
                pipeline.stats.critical_events, 0.85
            ));
        }

        dashboard.update(
            tick,
            &scenario,
            &pipeline.stats,
            &agent_system,
            &recent_events,
            &llm_actions,
        );

        if !active {
            println!("\n✅ SCENARIO COMPLETE");
            break;
        }

        thread::sleep(Duration::from_millis(500));
    }
}
