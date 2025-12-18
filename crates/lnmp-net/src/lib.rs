#![warn(missing_docs)]
#![warn(clippy::all)]

//! # lnmp-net
//!
//! Network behavior standardization for LNMP agent networks.
//!
//! LNMP-Net provides semantic message classification, QoS primitives, and intelligent
//! routing decisions for LLM/agent networks. It builds on top of the LNMP ecosystem
//! (lnmp-core, lnmp-envelope, lnmp-transport) without replacing them.
//!
//! ## Core Concepts
//!
//! - **MessageKind**: Semantic classification (Event/State/Command/Query/Alert)
//! - **NetMessage**: Wraps LNMP envelope with network metadata (priority, TTL, class)
//! - **RoutingPolicy**: Decides whether messages go to LLM, local processing, or are dropped
//!
//! ## Quick Start
//!
//! ```
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//! use lnmp_envelope::EnvelopeBuilder;
//! use lnmp_net::{MessageKind, NetMessage, RoutingPolicy, RoutingDecision};
//!
//! // Create a record
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(42) });
//!
//! // Wrap with envelope
//! let envelope = EnvelopeBuilder::new(record)
//!     .timestamp(1700000000000)
//!     .source("sensor-01")
//!     .build();
//!
//! // Create network message
//! let msg = NetMessage::new(envelope, MessageKind::Event);
//!
//! // Make routing decision
//! let policy = RoutingPolicy::default();
//! let decision = policy.decide(&msg, 1700000001000).unwrap();
//!
//! match decision {
//!     RoutingDecision::SendToLLM => println!("Sending to LLM"),
//!     RoutingDecision::ProcessLocally => println!("Processing locally"),
//!     RoutingDecision::Drop => println!("Dropping message"),
//! }
//! ```
//!
//! ## Message Kinds
//!
//! - **Event**: Sensor data, telemetry, user actions
//! - **State**: System state snapshots, health status
//! - **Command**: Imperative actions ("start motor")
//! - **Query**: Information requests ("get temperature")
//! - **Alert**: Critical warnings (health/safety/security)
//!
//! Each kind has default priority and TTL values tuned for typical use cases.
//!
//! ## Routing Logic (ECO Profile)
//!
//! The `RoutingPolicy` implements Energy/Token Optimization:
//!
//! 1. **Expired messages** → Drop (wasteful to process)
//! 2. **Alerts** with high priority → Always send to LLM
//! 3. **Events/State**: Compute importance score (priority + SFE) → threshold check
//! 4. **Commands/Queries** → Process locally (unless complex)
//!
//! This reduces LLM API calls by 90%+ while maintaining decision quality.
//!
//! ## Features
//!
//! - `serde`: Enable serde serialization support (optional)

pub mod content_routing;
pub mod error;
pub mod kind;
pub mod message;
pub mod routing;

#[cfg(feature = "transport")]
pub mod transport;

pub use content_routing::{ContentAwarePolicy, ContentRule, FieldCondition};
pub use error::{NetError, Result};
pub use kind::MessageKind;
pub use message::{NetMessage, NetMessageBuilder};
pub use routing::{RoutingDecision, RoutingPolicy};

// Re-export commonly used types for convenience
pub use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
pub use lnmp_envelope::{EnvelopeBuilder, LnmpEnvelope};
