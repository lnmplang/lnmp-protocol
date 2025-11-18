//! # lnmp-codec
//!
//! Parser and encoder implementations for LNMP (LLM Native Minimal Protocol) text format.
//!
//! This crate provides:
//! - [`Parser`]: Converts LNMP text format into structured [`LnmpRecord`] objects
//! - [`Encoder`]: Converts [`LnmpRecord`] objects into LNMP text format (canonical v0.2 format)
//! - [`ParsingMode`]: Strict or loose parsing modes for validation
//! - [`EncoderConfig`]: Configuration for encoder behavior (type hints, etc.)
//! - [`LnmpError`]: Error types for parsing and encoding operations
//!
//! ## LNMP v0.2 Features
//!
//! Version 0.2 introduces semantic stability with:
//! - **Deterministic serialization**: Fields are always sorted by FID
//! - **Canonical format**: Newline-separated, no extra whitespace
//! - **Type hints**: Optional type annotations (`:i`, `:f`, `:b`, `:s`, `:sa`)
//! - **Strict mode**: Validates canonical format compliance
//! - **Loose mode**: Accepts format variations (default)
//!
//! ## Parsing Examples
//!
//! ### Basic Parsing (Loose Mode)
//!
//! ```
//! use lnmp_codec::Parser;
//!
//! // Loose mode accepts various formats
//! let lnmp_text = r#"F12=14532;F7=1;F23=["admin","dev"]"#;
//!
//! let mut parser = Parser::new(lnmp_text).unwrap();
//! let record = parser.parse_record().unwrap();
//!
//! println!("Parsed {} fields", record.fields().len());
//! ```
//!
//! ### Strict Mode Parsing
//!
//! ```
//! use lnmp_codec::{Parser, ParsingMode};
//!
//! // Strict mode requires canonical format (sorted, newlines)
//! let canonical = "F7=1\nF12=14532\nF23=[admin,dev]";
//!
//! let mut parser = Parser::with_mode(canonical, ParsingMode::Strict).unwrap();
//! let record = parser.parse_record().unwrap();
//! ```
//!
//! ### Parsing with Type Hints
//!
//! ```
//! use lnmp_codec::Parser;
//!
//! let with_hints = "F12:i=14532\nF5:f=3.14\nF7:b=1";
//!
//! let mut parser = Parser::new(with_hints).unwrap();
//! let record = parser.parse_record().unwrap();
//! ```
//!
//! ## Encoding Examples
//!
//! ### Canonical Format (v0.2 Default)
//!
//! ```
//! use lnmp_codec::Encoder;
//! use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField {
//!     fid: 12,
//!     value: LnmpValue::Int(14532),
//! });
//! record.add_field(LnmpField {
//!     fid: 7,
//!     value: LnmpValue::Bool(true),
//! });
//!
//! // Canonical format: sorted by FID, newline-separated
//! let encoder = Encoder::new();
//! let output = encoder.encode(&record);
//! println!("{}", output); // F7=1\nF12=14532
//! ```
//!
//! ### Encoding with Type Hints
//!
//! ```
//! use lnmp_codec::{Encoder, EncoderConfig};
//! use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField {
//!     fid: 12,
//!     value: LnmpValue::Int(14532),
//! });
//!
//! let config = EncoderConfig::new()
//!     .with_type_hints(true)
//!     .with_canonical(true);
//! let encoder = Encoder::with_config(config);
//! let output = encoder.encode(&record);
//! println!("{}", output); // F12:i=14532
//! ```
//!
//! ## Deterministic Round-Trip
//!
//! ```
//! use lnmp_codec::{Parser, Encoder};
//!
//! // Any input becomes canonical after encode
//! let loose_input = "F23=[a,b];F7=1;F12=100"; // Unsorted, semicolons
//!
//! let mut parser = Parser::new(loose_input).unwrap();
//! let record = parser.parse_record().unwrap();
//!
//! let encoder = Encoder::new();
//! let canonical = encoder.encode(&record);
//! // Output: F7=1\nF12=100\nF23=[a,b]  (sorted, newlines)
//!
//! // Multiple encodes produce identical output
//! assert_eq!(canonical, encoder.encode(&record));
//! ```
//!
//! ## Migration from v0.1
//!
//! v0.2 is backward compatible with v0.1 for parsing, but encoding behavior has changed:
//!
//! - **v0.1**: Fields in insertion order, semicolons optional
//! - **v0.2**: Fields sorted by FID, newlines only (canonical)
//!
//! To maintain v0.1 behavior temporarily, use the deprecated `with_semicolons()` method,
//! but note that fields will still be sorted in v0.2.
//!
//! ## Error Handling
//!
//! ```
//! use lnmp_codec::{Parser, LnmpError};
//!
//! let invalid_lnmp = "F99999=test"; // Field ID out of range
//!
//! match Parser::new(invalid_lnmp) {
//!     Ok(mut parser) => match parser.parse_record() {
//!         Ok(record) => println!("Success!"),
//!         Err(e) => eprintln!("Parse error: {}", e),
//!     },
//!     Err(e) => eprintln!("Parser init error: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod binary;
pub mod config;
pub mod encoder;
pub mod equivalence;
pub mod error;
pub mod lexer;
pub mod normalizer;
pub mod parser;

pub use config::{EncoderConfig, ParsingMode, TextInputMode};
pub use encoder::{canonicalize_record, Encoder};
pub use equivalence::EquivalenceMapper;
pub use error::LnmpError;
pub use normalizer::{NormalizationConfig, StringCaseRule, ValueNormalizer};
pub use parser::Parser;
