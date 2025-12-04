//! Crisis Scenarios for Tokyo Smart City OS

pub mod gang_violence;

use crate::agents::AgentSystem;
use crate::LNMPPipeline;

/// Trait for crisis scenarios
pub trait Scenario {
    /// Initialize the scenario
    fn init(&mut self);

    /// Run one step of the scenario
    /// Returns true if scenario is still active
    fn step(&mut self, pipeline: &mut LNMPPipeline, agents: &mut AgentSystem) -> bool;

    /// Get scenario name
    fn name(&self) -> &str;

    /// Get current status description
    fn status(&self) -> String;
}

pub use gang_violence::GangViolenceScenario;

pub mod earthquake;
pub use earthquake::EarthquakeScenario;

pub mod compound_crisis;
pub use compound_crisis::CompoundCrisisScenario;
