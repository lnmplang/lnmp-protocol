//! Transport bindings for LNMP-Net.
//!
//! This module provides helpers to map LNMP-Net metadata (kind, priority, ttl, class)
//! to/from transport-specific headers for HTTP, Kafka, NATS, and gRPC.
//!
//! These bindings complement the existing lnmp-transport module, which handles
//! LNMP Envelope metadata (timestamp, source, trace_id, sequence).
//!
//! # Usage
//!
//! ```rust,ignore
//! use lnmp_net::transport::http;
//!
//! // Encode LNMP-Net metadata to HTTP headers
//! let net_headers = http::net_to_http_headers(&msg)?;
//!
//! // Decode from HTTP headers
//! let (kind, priority, ttl_ms, class) = http::http_headers_to_net_meta(&headers)?;
//! ```

pub mod grpc;
pub mod http;
pub mod kafka;
pub mod nats;

// Re-export commonly used functions
#[cfg(feature = "transport")]
pub use http::{http_headers_to_net_meta, net_to_http_headers};

pub use grpc::{grpc_metadata_to_net_meta, net_to_grpc_metadata};
pub use kafka::{kafka_headers_to_net_meta, net_to_kafka_headers};
pub use nats::{nats_headers_to_net_meta, net_to_nats_headers};
