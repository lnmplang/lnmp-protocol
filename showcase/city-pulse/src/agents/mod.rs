//! Multi-Agent System for Tokyo Smart City OS
//!
//! Defines the Agent trait and specific implementations for:
//! - Police Dispatch
//! - Ambulance/Medical
//! - Fire & Rescue
//! - Traffic Control

pub mod implementation;
pub mod system;

pub use implementation::{AmbulanceAgent, FireAgent, PoliceAgent, TrafficAgent};
pub use system::{Agent, AgentStatus, AgentSystem, AgentType};
