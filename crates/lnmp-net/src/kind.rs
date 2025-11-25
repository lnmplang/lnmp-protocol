//! Message kind classification for LNMP-Net

use std::fmt;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Semantic message classification for network routing and LLM integration
///
/// Each kind has different routing, priority, and LLM processing characteristics:
///
/// - **Event**: Asynchronous notifications (sensor data, telemetry, user actions)
/// - **State**: System/component state snapshots (health status, metrics)
/// - **Command**: Imperative actions ("start motor", "deploy model")
/// - **Query**: Information requests ("get current temperature")
/// - **Alert**: Critical warnings (health/safety/security)
///
/// # Examples
///
/// ```
/// use lnmp_net::MessageKind;
///
/// let kind = MessageKind::Event;
/// assert_eq!(kind.to_string(), "Event");
/// assert_eq!(kind.default_priority(), 100);
/// assert_eq!(kind.default_ttl_ms(), 5000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageKind {
    /// Sensor data, telemetry, system events
    #[default]
    Event,
    /// Component/system state snapshots
    State,
    /// Imperative actions ("do this")
    Command,
    /// Information requests ("give me data")
    Query,
    /// Critical health/safety/security warnings
    Alert,
}

impl MessageKind {
    /// Returns true if this is an Alert message
    pub fn is_alert(&self) -> bool {
        matches!(self, MessageKind::Alert)
    }

    /// Returns true if this is a Command message
    pub fn is_command(&self) -> bool {
        matches!(self, MessageKind::Command)
    }

    /// Returns true if this is a Query message
    pub fn is_query(&self) -> bool {
        matches!(self, MessageKind::Query)
    }

    /// Returns true if this is an Event message
    pub fn is_event(&self) -> bool {
        matches!(self, MessageKind::Event)
    }

    /// Returns true if this is a State message
    pub fn is_state(&self) -> bool {
        matches!(self, MessageKind::State)
    }

    /// Returns the default priority for this message kind (0-255)
    ///
    /// Priority ranges:
    /// - Alert: 255 (critical)
    /// - Command: 150 (high)
    /// - Query: 120 (medium-high)
    /// - State: 100 (medium)
    /// - Event: 100 (medium)
    pub fn default_priority(&self) -> u8 {
        match self {
            MessageKind::Alert => 255,
            MessageKind::Command => 150,
            MessageKind::Query => 120,
            MessageKind::State => 100,
            MessageKind::Event => 100,
        }
    }

    /// Returns the default TTL (time-to-live) in milliseconds
    ///
    /// TTL ranges:
    /// - Alert: 1000ms (1s - urgent)
    /// - Command: 2000ms (2s - timely execution)
    /// - Query: 5000ms (5s - quick response)
    /// - State: 10000ms (10s - snapshot validity)
    /// - Event: 5000ms (5s - real-time relevance)
    pub fn default_ttl_ms(&self) -> u32 {
        match self {
            MessageKind::Alert => 1000,
            MessageKind::Command => 2000,
            MessageKind::Query => 5000,
            MessageKind::State => 10000,
            MessageKind::Event => 5000,
        }
    }

    /// Returns all message kinds as an array
    pub fn all() -> [MessageKind; 5] {
        [
            MessageKind::Event,
            MessageKind::State,
            MessageKind::Command,
            MessageKind::Query,
            MessageKind::Alert,
        ]
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Event => write!(f, "Event"),
            MessageKind::State => write!(f, "State"),
            MessageKind::Command => write!(f, "Command"),
            MessageKind::Query => write!(f, "Query"),
            MessageKind::Alert => write!(f, "Alert"),
        }
    }
}

impl FromStr for MessageKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "event" => Ok(MessageKind::Event),
            "state" => Ok(MessageKind::State),
            "command" => Ok(MessageKind::Command),
            "query" => Ok(MessageKind::Query),
            "alert" => Ok(MessageKind::Alert),
            _ => Err(format!("Invalid MessageKind: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_kind_display() {
        assert_eq!(MessageKind::Event.to_string(), "Event");
        assert_eq!(MessageKind::State.to_string(), "State");
        assert_eq!(MessageKind::Command.to_string(), "Command");
        assert_eq!(MessageKind::Query.to_string(), "Query");
        assert_eq!(MessageKind::Alert.to_string(), "Alert");
    }

    #[test]
    fn test_message_kind_from_str() {
        assert_eq!("event".parse::<MessageKind>().unwrap(), MessageKind::Event);
        assert_eq!("Event".parse::<MessageKind>().unwrap(), MessageKind::Event);
        assert_eq!("EVENT".parse::<MessageKind>().unwrap(), MessageKind::Event);
        assert_eq!("alert".parse::<MessageKind>().unwrap(), MessageKind::Alert);
        assert!("invalid".parse::<MessageKind>().is_err());
    }

    #[test]
    fn test_is_methods() {
        assert!(MessageKind::Alert.is_alert());
        assert!(!MessageKind::Event.is_alert());

        assert!(MessageKind::Command.is_command());
        assert!(!MessageKind::Query.is_command());

        assert!(MessageKind::Event.is_event());
        assert!(MessageKind::State.is_state());
        assert!(MessageKind::Query.is_query());
    }

    #[test]
    fn test_default_priority_ranges() {
        assert_eq!(MessageKind::Alert.default_priority(), 255);
        assert_eq!(MessageKind::Command.default_priority(), 150);
        assert_eq!(MessageKind::Query.default_priority(), 120);
        assert_eq!(MessageKind::State.default_priority(), 100);
        assert_eq!(MessageKind::Event.default_priority(), 100);
    }

    #[test]
    fn test_default_ttl_values() {
        assert_eq!(MessageKind::Alert.default_ttl_ms(), 1000);
        assert_eq!(MessageKind::Command.default_ttl_ms(), 2000);
        assert_eq!(MessageKind::Query.default_ttl_ms(), 5000);
        assert_eq!(MessageKind::State.default_ttl_ms(), 10000);
        assert_eq!(MessageKind::Event.default_ttl_ms(), 5000);
    }

    #[test]
    fn test_all_kinds() {
        let all = MessageKind::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&MessageKind::Event));
        assert!(all.contains(&MessageKind::Alert));
    }

    #[test]
    fn test_default() {
        assert_eq!(MessageKind::default(), MessageKind::Event);
    }
}
