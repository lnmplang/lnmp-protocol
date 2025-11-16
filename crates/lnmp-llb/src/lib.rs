//! LNMP-LLM Bridge Layer (LLB)
//!
//! This crate provides optimization strategies for LLM (Large Language Model) contexts,
//! including prompt visibility optimization, explain mode encoding, and ShortForm encoding.
//!
//! # Modules
//!
//! - `explain`: Explain mode encoding with human-readable annotations
//! - `prompt_opt`: Prompt visibility optimization for tokenization efficiency
//! - `shortform`: ShortForm encoding for extreme token reduction (planned)
//!
//! # Examples
//!
//! ## Explain Mode
//!
//! ```
//! use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
//! use lnmp_llb::{ExplainEncoder, SemanticDictionary};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField {
//!     fid: 12,
//!     value: LnmpValue::Int(14532),
//! });
//!
//! let dict = SemanticDictionary::from_pairs(vec![(12, "user_id")]);
//! let encoder = ExplainEncoder::new(dict);
//! let output = encoder.encode_with_explanation(&record);
//! // Output: F12:i=14532         # user_id
//! ```
//!
//! ## Prompt Optimization
//!
//! ```
//! use lnmp_core::{LnmpField, LnmpValue};
//! use lnmp_llb::{PromptOptimizer, PromptOptConfig};
//!
//! let optimizer = PromptOptimizer::default();
//! let arr = vec!["admin".to_string(), "developer".to_string()];
//! let result = optimizer.optimize_array(&arr);
//! // Output: [admin,developer]
//! ```

pub mod explain;
pub mod llb2;
pub mod prompt_opt;
pub mod shortform;

// Re-export main types for convenience
pub use explain::{ExplainEncoder, SemanticDictionary};
pub use llb2::{LlbConfig, LlbConverter, LlbError};
pub use prompt_opt::{PromptOptConfig, PromptOptimizer};
