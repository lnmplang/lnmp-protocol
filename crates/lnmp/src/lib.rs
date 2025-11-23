//! # LNMP - LLM Native Minimal Protocol
//!
//! This is a meta crate that re-exports all LNMP modules, providing a unified entry point
//! for users who want to work with the complete LNMP ecosystem without managing individual
//! module dependencies.
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lnmp = "0.5.4"
//! ```
//!
//! ## Available Modules
//!
//! - **`core`**: Core type definitions and protocol structures
//! - **`codec`**: Encoding and decoding functionality
//! - **`embedding`**: Embedding vector operations and delta compression
//! - **`llb`**: Large Language Block operations
//! - **`quant`**: Quantization utilities for efficient data representation
//! - **`sanitize`**: Data sanitization and validation
//! - **`sfe`**: Secure Function Evaluation primitives
//! - **`spatial`**: Spatial data streaming and hybrid protocols
//! - **`transport`**: Transport protocol bindings (HTTP, Kafka, gRPC, NATS) with W3C Trace Context
//!
//! ## Usage Examples
//!
//! ```rust
//! // Access core types
//! use lnmp::core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! // Use codec functionality
//! use lnmp::codec::{Encoder, Parser};
//!
//! // Work with embeddings
//! use lnmp::embedding::{VectorDelta, Vector};
//!
//! // Use spatial streaming
//! use lnmp::spatial::protocol::SpatialStreamer;
//! ```
//!
//! ## Individual Module Usage
//!
//! If you prefer to use only specific modules, you can depend on them individually:
//!
//! - `lnmp-core` - Core protocol definitions
//! - `lnmp-codec` - Encoding/decoding
//! - `lnmp-embedding` - Embedding operations
//! - `lnmp-llb` - Large Language Blocks
//! - `lnmp-quant` - Quantization
//! - `lnmp-sanitize` - Sanitization
//! - `lnmp-sfe` - Secure Function Evaluation
//! - `lnmp-spatial` - Spatial streaming
//! - `lnmp-transport` - Transport bindings
//!
//! ## Documentation
//!
//! For detailed documentation on each module, visit:
//! - [LNMP Protocol Documentation](https://lnmp.io)
//! - [GitHub Repository](https://github.com/lnmplang/lnmp-protocol)

// Re-export all LNMP modules
pub use lnmp_codec as codec;
pub use lnmp_core as core;
pub use lnmp_embedding as embedding;
pub use lnmp_envelope as envelope;
pub use lnmp_llb as llb;
pub use lnmp_quant as quant;
pub use lnmp_sanitize as sanitize;
pub use lnmp_sfe as sfe;
pub use lnmp_spatial as spatial;
pub use lnmp_transport as transport;

// Re-export commonly used types for convenience
pub mod prelude {
    //! Prelude module with commonly used types and traits

    // Core types
    pub use lnmp_core::{FieldId, LnmpField, LnmpRecord, LnmpValue, TypeHint};

    // Codec
    pub use lnmp_codec::{Encoder, Parser};

    // Embedding types
    pub use lnmp_embedding::{DeltaChange, UpdateStrategy, Vector, VectorDelta};

    // Spatial types
    pub use lnmp_spatial::protocol::{SpatialFrame, SpatialStreamer};
}
