//! # lnmp-core
//!
//! Core type definitions for LNMP (LLM Native Minimal Protocol).
//!
//! This crate provides the fundamental data structures for representing LNMP data:
//! - [`FieldId`]: Type alias for field identifiers (u16, range 0-65535)
//! - [`LnmpValue`]: Enum representing all supported value types
//! - [`LnmpField`]: A field ID and value pair
//! - [`LnmpRecord`]: A collection of fields representing a complete record
//! - [`TypeHint`]: Type annotations for values (v0.2+)
//!
//! ## LNMP v0.2 Features
//!
//! Version 0.2 adds:
//! - **Type hints**: Optional type annotations (`:i`, `:f`, `:b`, `:s`, `:sa`)
//! - **Sorted fields**: `sorted_fields()` method for deterministic ordering
//!
//! ## Example
//!
//! ```
//! use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
//!
//! // Create a new record
//! let mut record = LnmpRecord::new();
//!
//! // Add fields
//! record.add_field(LnmpField {
//!     fid: 12,
//!     value: LnmpValue::Int(14532),
//! });
//!
//! record.add_field(LnmpField {
//!     fid: 7,
//!     value: LnmpValue::Bool(true),
//! });
//!
//! // Access fields
//! if let Some(field) = record.get_field(12) {
//!     println!("Field 12: {:?}", field.value);
//! }
//!
//! // Get sorted fields (v0.2)
//! let sorted = record.sorted_fields();
//! for field in sorted {
//!     println!("F{} = {:?}", field.fid, field.value);
//! }
//! ```
//!
//! ## Type Hints (v0.2)
//!
//! ```
//! use lnmp_core::{TypeHint, LnmpValue};
//!
//! // Parse type hint from string
//! let hint = TypeHint::from_str("i").unwrap();
//! assert_eq!(hint.as_str(), "i");
//!
//! // Validate value matches type hint
//! let value = LnmpValue::Int(42);
//! assert!(hint.validates(&value));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod checksum;
pub mod record;
pub mod types;

pub use record::{LnmpField, LnmpRecord};
pub use types::{FieldId, LnmpValue, TypeHint};
