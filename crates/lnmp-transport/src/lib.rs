//! # lnmp-transport
//!
//! Transport bindings for the LNMP protocol, providing standard mappings between
//! LNMP records with Envelope metadata and various transport protocols (HTTP, Kafka, gRPC).
//!
//! This crate does NOT implement HTTP/Kafka/gRPC clients or servers - it only provides
//! helpers to map LNMP data to/from transport-specific headers and bodies.

#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "kafka")]
pub mod kafka;
#[cfg(feature = "nats")]
pub mod nats;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    #[error("Invalid header value for {0}: {1}")]
    InvalidHeaderValue(String, String),
    #[error("Codec error: {0}")]
    CodecError(#[from] lnmp_codec::LnmpError),
    #[error("Binary encoding error: {0}")]
    BinaryError(#[from] lnmp_codec::binary::BinaryError),
    #[error("Envelope error: {0}")]
    EnvelopeError(String),
}

pub type Result<T> = std::result::Result<T, TransportError>;
