//! Lenient sanitization layer for LNMP text inputs.
//!
//! The sanitizer performs lightweight whitespace normalization, quote/escape repair,
//! and optional boolean/number canonicalization before handing text to the strict
//! LNMP parser.

mod mode;
mod sanitize;
#[cfg(test)]
mod tests;

pub use crate::mode::SanitizationLevel;
pub use crate::sanitize::{sanitize_lnmp_text, SanitizationConfig};
