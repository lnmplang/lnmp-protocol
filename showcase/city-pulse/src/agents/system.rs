//! Core Agent System Definitions

use lnmp::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentType {
    Police,
    Ambulance,
    Fire,
    TrafficControl,
    Municipal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Dispatched,
    OnScene,
    Returning,
    Busy,
}

/// Trait for all Smart City Agents
pub trait Agent: Send + Sync {
    /// Get agent ID
    fn id(&self) -> &str;

    /// Get agent type
    fn agent_type(&self) -> AgentType;

    /// Get current status
    fn status(&self) -> AgentStatus;

    /// Get current location (lat, lon)
    fn location(&self) -> (f64, f64);

    /// Handle an incoming command (LNMP record)
    /// Returns a list of response events (e.g., status updates)
    fn handle_command(&mut self, command: &LnmpRecord) -> Vec<LnmpRecord>;

    /// Update agent state (simulation tick)
    fn update(&mut self) -> Vec<LnmpRecord>;
}

/// Orchestrator for all agents
pub struct AgentSystem {
    agents: HashMap<String, Box<dyn Agent>>,
}

impl Default for AgentSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentSystem {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents.insert(agent.id().to_string(), agent);
    }

    pub fn get_agent(&self, id: &str) -> Option<&dyn Agent> {
        self.agents.get(id).map(|a| a.as_ref())
    }

    // pub fn get_agent_mut(&mut self, id: &str) -> Option<&mut (dyn Agent + '_)> {
    //     self.agents.get_mut(id).map(|a| a.as_mut())
    // }

    /// Process a list of command records (e.g., from LLB)
    pub fn process_commands(&mut self, commands: Vec<LnmpRecord>) -> Vec<LnmpRecord> {
        let mut responses = Vec::new();

        for cmd in commands {
            // Extract target agent type or ID from command
            // For demo, we'll broadcast to relevant agent types
            // Real system would have specific routing

            if let Some(target_type) = self.extract_target_type(&cmd) {
                for agent in self.agents.values_mut() {
                    if agent.agent_type() == target_type && agent.status() == AgentStatus::Idle {
                        let agent_responses = agent.handle_command(&cmd);
                        responses.extend(agent_responses);
                        break; // Dispatch to first available agent
                    }
                }
            }
        }

        responses
    }

    /// Update all agents
    pub fn update_all(&mut self) -> Vec<LnmpRecord> {
        let mut updates = Vec::new();
        for agent in self.agents.values_mut() {
            updates.extend(agent.update());
        }
        updates
    }

    fn extract_target_type(&self, cmd: &LnmpRecord) -> Option<AgentType> {
        // Look for F210 (Agent Type field)
        if let Some(field) = cmd.get_field(210) {
            if let LnmpValue::String(s) = &field.value {
                return match s.as_str() {
                    "POLICE" => Some(AgentType::Police),
                    "AMBULANCE" => Some(AgentType::Ambulance),
                    "FIRE" => Some(AgentType::Fire),
                    "TRAFFIC" => Some(AgentType::TrafficControl),
                    _ => None,
                };
            }
        }
        None
    }

    pub fn active_agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn busy_agent_count(&self) -> usize {
        self.agents
            .values()
            .filter(|a| a.status() != AgentStatus::Idle)
            .count()
    }
}
