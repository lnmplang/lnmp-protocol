//! Context profiling for LLM decision support
//!
//! This module provides automatic scoring of LNMP records to help LLMs
//! prioritize which records to use in RAG systems and other applications.
//!
//! ## Scoring Metrics
//!
//! - **Freshness**: How recent the data is (exponential decay)
//! - **Importance**: Field-level priority (0-255)
//! - **Risk**: Source-based risk assessment
//! - **Confidence**: Data reliability score
//!
//! ## Example
//!
//! ```
//! use lnmp_sfe::context::{ContextScorer, ContextScorerConfig};
//! use lnmp_envelope::EnvelopeBuilder;
//! use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};
//!
//! let mut record = LnmpRecord::new();
//! record.add_field(LnmpField { fid: 12, value: LnmpValue::Int(14532) });
//!
//! let envelope = EnvelopeBuilder::new(record)
//!     .timestamp(1732373147000)
//!     .source("auth-service")
//!     .build();
//!
//! let scorer = ContextScorer::new();
//! let now = 1732373147000 + 3600000; // 1 hour later
//! let profile = scorer.score_envelope(&envelope, now);
//!
//! println!("Freshness: {:.2}", profile.freshness_score);
//! println!("Importance: {}", profile.importance);
//! println!("Risk: {:?}", profile.risk_level);
//! ```

use lnmp_core::LnmpRecord;
use lnmp_envelope::LnmpEnvelope;
use std::collections::HashMap;
use std::sync::Arc;

use crate::dictionary::SemanticDictionary;

/// Context profile for an LNMP record with LLM decision metrics
///
/// Provides scoring across multiple dimensions to help LLMs prioritize
/// which records to use when constructing prompts or processing RAG queries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContextProfile {
    /// Freshness score (0.0 = stale, 1.0 = fresh)
    ///
    /// Calculated using exponential decay based on timestamp age.
    /// Default decay constant: 24 hours (configurable).
    pub freshness_score: f64,

    /// Importance level (0 = lowest, 255 = critical)
    ///
    /// Can be derived from:
    /// - Semantic dictionary field importance
    /// - Record content analysis
    /// - Default configuration
    pub importance: u8,

    /// Risk assessment level
    ///
    /// Evaluated based on:
    /// - Source trustworthiness
    /// - Field content patterns
    /// - Configured suspicious sources
    pub risk_level: RiskLevel,

    /// Confidence in the data (0.0 = no confidence, 1.0 = full confidence)
    ///
    /// Influenced by:
    /// - Source trust level
    /// - Data completeness
    /// - Metadata presence
    pub confidence: f64,

    /// LLM hints and metadata
    ///
    /// Extensible key-value pairs for custom scoring signals.
    /// Example hints: "domain", "category", "verified", etc.
    pub llm_hints: HashMap<String, String>,
}

impl ContextProfile {
    /// Create a new context profile with default values
    pub fn new() -> Self {
        Self {
            freshness_score: 0.5,
            importance: 128,
            risk_level: RiskLevel::Low,
            confidence: 0.5,
            llm_hints: HashMap::new(),
        }
    }

    /// Add an LLM hint
    pub fn add_hint(&mut self, key: String, value: String) {
        self.llm_hints.insert(key, value);
    }

    /// Get a composite score for ranking (0.0-1.0)
    ///
    /// Uses default weights:
    /// - Freshness: 30%
    /// - Importance: 40% (normalized to 0-1)
    /// - Confidence: 30%
    /// - Risk: penalty factor
    pub fn composite_score(&self) -> f64 {
        self.composite_score_weighted(&ScoringWeights::default())
    }

    /// Get composite score with custom weights
    pub fn composite_score_weighted(&self, weights: &ScoringWeights) -> f64 {
        let importance_normalized = self.importance as f64 / 255.0;
        let risk_penalty = match self.risk_level {
            RiskLevel::Low => 1.0,
            RiskLevel::Medium => 0.8,
            RiskLevel::High => 0.5,
            RiskLevel::Critical => 0.1,
        };

        let score = weights.freshness_weight * self.freshness_score
            + weights.importance_weight * importance_normalized
            + weights.confidence_weight * self.confidence;

        (score * risk_penalty).clamp(0.0, 1.0)
    }
}

impl Default for ContextProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    /// Low risk - trusted source, normal content
    Low,
    /// Medium risk - unknown source or minor concerns
    Medium,
    /// High risk - suspicious source or concerning patterns
    High,
    /// Critical risk - known malicious source or dangerous content
    Critical,
}

impl RiskLevel {
    /// Get numeric risk value (0-3)
    pub fn as_u8(&self) -> u8 {
        match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        }
    }
}

/// Configuration for context scoring
#[derive(Debug, Clone)]
pub struct ContextScorerConfig {
    /// Freshness decay rate in hours (default: 24.0)
    ///
    /// Controls how quickly freshness score decays.
    /// Formula: e^(-age_hours / decay_hours)
    /// - 24.0: Half-life ~17 hours
    /// - 12.0: Half-life ~8 hours (faster decay)
    /// - 48.0: Half-life ~33 hours (slower decay)
    pub freshness_decay_hours: f64,

    /// Default importance if not specified (default: 128)
    pub default_importance: u8,

    /// Default risk level if not specified (default: Low)
    pub default_risk: RiskLevel,

    /// Default confidence if not specified (default: 0.5)
    pub default_confidence: f64,

    /// Enable dictionary-based importance lookup (default: true)
    pub use_dictionary_importance: bool,

    /// Trusted sources for confidence boosting
    ///
    /// Sources in this list get +0.2 confidence boost (capped at 1.0)
    pub trusted_sources: Vec<String>,

    /// Suspicious sources for risk elevation
    ///
    /// Sources in this list automatically get High or Critical risk level
    pub suspicious_sources: Vec<String>,
}

impl ContextScorerConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            freshness_decay_hours: 24.0,
            default_importance: 128,
            default_risk: RiskLevel::Low,
            default_confidence: 0.5,
            use_dictionary_importance: true,
            trusted_sources: Vec::new(),
            suspicious_sources: Vec::new(),
        }
    }

    /// Set freshness decay rate
    pub fn with_freshness_decay(mut self, hours: f64) -> Self {
        self.freshness_decay_hours = hours;
        self
    }

    /// Set default importance
    pub fn with_default_importance(mut self, importance: u8) -> Self {
        self.default_importance = importance;
        self
    }

    /// Set default risk level
    pub fn with_default_risk(mut self, risk: RiskLevel) -> Self {
        self.default_risk = risk;
        self
    }

    /// Add trusted source
    pub fn add_trusted_source(mut self, source: String) -> Self {
        self.trusted_sources.push(source);
        self
    }

    /// Add suspicious source
    pub fn add_suspicious_source(mut self, source: String) -> Self {
        self.suspicious_sources.push(source);
        self
    }
}

impl Default for ContextScorerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Weights for composite scoring
#[derive(Debug, Clone, Copy)]
pub struct ScoringWeights {
    /// Weight for freshness component (default: 0.3)
    pub freshness_weight: f64,
    /// Weight for importance component (default: 0.4)
    pub importance_weight: f64,
    /// Weight for confidence component (default: 0.3)
    pub confidence_weight: f64,
}

impl ScoringWeights {
    /// Create new weights (values should sum to 1.0)
    pub fn new(freshness: f64, importance: f64, confidence: f64) -> Self {
        Self {
            freshness_weight: freshness,
            importance_weight: importance,
            confidence_weight: confidence,
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(mut self) -> Self {
        let sum = self.freshness_weight + self.importance_weight + self.confidence_weight;
        if sum > 0.0 {
            self.freshness_weight /= sum;
            self.importance_weight /= sum;
            self.confidence_weight /= sum;
        }
        self
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            freshness_weight: 0.3,
            importance_weight: 0.4,
            confidence_weight: 0.3,
        }
    }
}

/// Main context scorer
///
/// Evaluates LNMP envelopes and records to produce context profiles
/// for LLM decision support.
pub struct ContextScorer {
    config: ContextScorerConfig,
    dictionary: Option<Arc<SemanticDictionary>>,
}

impl ContextScorer {
    /// Create new scorer with default configuration
    pub fn new() -> Self {
        Self {
            config: ContextScorerConfig::default(),
            dictionary: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ContextScorerConfig) -> Self {
        Self {
            config,
            dictionary: None,
        }
    }

    /// Attach semantic dictionary for importance lookup
    pub fn with_dictionary(mut self, dict: Arc<SemanticDictionary>) -> Self {
        self.dictionary = Some(dict);
        self
    }

    /// Score an envelope (metadata-based scoring)
    ///
    /// Evaluates:
    /// - Freshness from timestamp
    /// - Confidence from source trustworthiness
    /// - Risk from source patterns
    ///
    /// # Arguments
    ///
    /// * `envelope` - The envelope to score
    /// * `now` - Current timestamp in milliseconds since Unix epoch
    pub fn score_envelope(&self, envelope: &LnmpEnvelope, now: u64) -> ContextProfile {
        let mut profile = ContextProfile::new();

        // Score freshness
        profile.freshness_score = self.compute_freshness(envelope, now);

        // Score confidence and risk based on source
        if let Some(ref source) = envelope.metadata.source {
            profile.confidence = self.compute_confidence(source);
            profile.risk_level = self.compute_risk(source);
        } else {
            profile.confidence = self.config.default_confidence;
            profile.risk_level = self.config.default_risk;
        }

        // Default importance
        profile.importance = self.config.default_importance;

        profile
    }

    /// Score a record (content-based scoring)
    ///
    /// Evaluates:
    /// - Importance from field IDs (using dictionary if available)
    ///
    /// # Arguments
    ///
    /// * `record` - The record to score
    pub fn score_record(&self, record: &LnmpRecord) -> ContextProfile {
        let mut profile = ContextProfile::new();

        // Compute importance from fields
        profile.importance = self.compute_importance(record);

        // Add field count hint
        profile.add_hint("field_count".to_string(), record.fields().len().to_string());

        profile
    }

    /// Score envelope + record combined
    ///
    /// Combines metadata-based and content-based scoring for complete evaluation.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The envelope containing the record
    /// * `record` - The record to score (usually from envelope.record)
    /// * `now` - Current timestamp in milliseconds since Unix epoch
    pub fn score_combined(
        &self,
        envelope: &LnmpEnvelope,
        record: &LnmpRecord,
        now: u64,
    ) -> ContextProfile {
        let mut profile = self.score_envelope(envelope, now);
        let record_profile = self.score_record(record);

        // Merge importance from record
        profile.importance = record_profile.importance;

        // Merge hints
        for (k, v) in record_profile.llm_hints {
            profile.add_hint(k, v);
        }

        profile
    }

    /// Compute freshness score using exponential decay
    fn compute_freshness(&self, envelope: &LnmpEnvelope, now: u64) -> f64 {
        if let Some(ts) = envelope.metadata.timestamp {
            let age_ms = now.saturating_sub(ts);
            let age_hours = age_ms as f64 / 3_600_000.0;

            // Exponential decay: e^(-t/τ) where τ = decay constant
            (-age_hours / self.config.freshness_decay_hours).exp()
        } else {
            // No timestamp = medium priority
            0.5
        }
    }

    /// Compute confidence score from source
    fn compute_confidence(&self, source: &str) -> f64 {
        let mut confidence = self.config.default_confidence;

        // Boost for trusted sources
        if self
            .config
            .trusted_sources
            .iter()
            .any(|s| source.contains(s))
        {
            confidence = (confidence + 0.2).min(1.0);
        }

        confidence
    }

    /// Compute risk level from source
    fn compute_risk(&self, source: &str) -> RiskLevel {
        // Check suspicious sources
        if self
            .config
            .suspicious_sources
            .iter()
            .any(|s| source.contains(s))
        {
            return RiskLevel::High;
        }

        self.config.default_risk
    }

    /// Compute importance from record fields
    fn compute_importance(&self, record: &LnmpRecord) -> u8 {
        if let Some(ref dict) = self.dictionary {
            if self.config.use_dictionary_importance {
                // Use maximum importance from all fields
                let max_importance = record
                    .fields()
                    .iter()
                    .filter_map(|field| dict.get_importance(field.fid))
                    .max()
                    .unwrap_or(self.config.default_importance);

                return max_importance;
            }
        }

        self.config.default_importance
    }
}

impl Default for ContextScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for LLM context prioritization
///
/// Provides filtering, ranking, and selection operations to help
/// choose the best subset of contexts for LLM consumption.
pub struct ContextPrioritizer;

impl ContextPrioritizer {
    /// Filter contexts by minimum composite score
    ///
    /// # Arguments
    ///
    /// * `contexts` - List of (envelope, profile) pairs
    /// * `threshold` - Minimum composite score (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Filtered list containing only contexts with score >= threshold
    pub fn filter_by_threshold(
        contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        threshold: f64,
    ) -> Vec<(LnmpEnvelope, ContextProfile)> {
        contexts
            .into_iter()
            .filter(|(_, profile)| profile.composite_score() >= threshold)
            .collect()
    }

    /// Filter by minimum freshness score
    pub fn filter_by_freshness(
        contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        min_freshness: f64,
    ) -> Vec<(LnmpEnvelope, ContextProfile)> {
        contexts
            .into_iter()
            .filter(|(_, profile)| profile.freshness_score >= min_freshness)
            .collect()
    }

    /// Filter by minimum importance
    pub fn filter_by_importance(
        contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        min_importance: u8,
    ) -> Vec<(LnmpEnvelope, ContextProfile)> {
        contexts
            .into_iter()
            .filter(|(_, profile)| profile.importance >= min_importance)
            .collect()
    }

    /// Filter by maximum risk level
    pub fn filter_by_risk(
        contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        max_risk: RiskLevel,
    ) -> Vec<(LnmpEnvelope, ContextProfile)> {
        let max_risk_value = max_risk.as_u8();
        contexts
            .into_iter()
            .filter(|(_, profile)| profile.risk_level.as_u8() <= max_risk_value)
            .collect()
    }

    /// Rank contexts for LLM consumption (composite score)
    ///
    /// Returns contexts sorted by composite score (highest first) with scores.
    ///
    /// # Arguments
    ///
    /// * `contexts` - List of (envelope, profile) pairs
    /// * `weights` - Scoring weights for composite calculation
    ///
    /// # Returns
    ///
    /// List of (envelope, profile, score) sorted descending by score
    pub fn rank_for_llm(
        mut contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        weights: ScoringWeights,
    ) -> Vec<(LnmpEnvelope, ContextProfile, f64)> {
        let mut scored: Vec<_> = contexts
            .drain(..)
            .map(|(env, profile)| {
                let score = profile.composite_score_weighted(&weights);
                (env, profile, score)
            })
            .collect();

        // Sort descending by score
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        scored
    }

    /// Select top K contexts
    ///
    /// Returns the K highest-scoring contexts for LLM prompt construction.
    ///
    /// # Arguments
    ///
    /// * `contexts` - List of (envelope, profile) pairs
    /// * `k` - Number of contexts to select
    /// * `weights` - Scoring weights for composite calculation
    ///
    /// # Returns
    ///
    /// Up to K contexts with highest scores
    pub fn select_top_k(
        contexts: Vec<(LnmpEnvelope, ContextProfile)>,
        k: usize,
        weights: ScoringWeights,
    ) -> Vec<(LnmpEnvelope, ContextProfile)> {
        let ranked = Self::rank_for_llm(contexts, weights);
        ranked
            .into_iter()
            .take(k)
            .map(|(env, profile, _score)| (env, profile))
            .collect()
    }

    /// Compute statistics for a set of contexts
    pub fn compute_stats(contexts: &[(LnmpEnvelope, ContextProfile)]) -> ContextStats {
        if contexts.is_empty() {
            return ContextStats::default();
        }

        let mut total_freshness = 0.0;
        let mut total_importance = 0u64;
        let mut total_confidence = 0.0;
        let mut risk_counts = [0u32; 4]; // Low, Medium, High, Critical

        for (_, profile) in contexts {
            total_freshness += profile.freshness_score;
            total_importance += profile.importance as u64;
            total_confidence += profile.confidence;
            risk_counts[profile.risk_level.as_u8() as usize] += 1;
        }

        let count = contexts.len();

        ContextStats {
            count,
            avg_freshness: total_freshness / count as f64,
            avg_importance: (total_importance / count as u64) as u8,
            avg_confidence: total_confidence / count as f64,
            risk_low: risk_counts[0],
            risk_medium: risk_counts[1],
            risk_high: risk_counts[2],
            risk_critical: risk_counts[3],
        }
    }
}

/// Statistics for a collection of contexts
#[derive(Debug, Clone, PartialEq)]
pub struct ContextStats {
    /// Total number of contexts
    pub count: usize,
    /// Average freshness score
    pub avg_freshness: f64,
    /// Average importance
    pub avg_importance: u8,
    /// Average confidence
    pub avg_confidence: f64,
    /// Count of low-risk contexts
    pub risk_low: u32,
    /// Count of medium-risk contexts
    pub risk_medium: u32,
    /// Count of high-risk contexts
    pub risk_high: u32,
    /// Count of critical-risk contexts
    pub risk_critical: u32,
}

impl Default for ContextStats {
    fn default() -> Self {
        Self {
            count: 0,
            avg_freshness: 0.0,
            avg_importance: 0,
            avg_confidence: 0.0,
            risk_low: 0,
            risk_medium: 0,
            risk_high: 0,
            risk_critical: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::{LnmpField, LnmpValue};
    use lnmp_envelope::EnvelopeBuilder;

    #[test]
    fn test_context_profile_default() {
        let profile = ContextProfile::default();
        assert_eq!(profile.freshness_score, 0.5);
        assert_eq!(profile.importance, 128);
        assert_eq!(profile.risk_level, RiskLevel::Low);
        assert_eq!(profile.confidence, 0.5);
        assert!(profile.llm_hints.is_empty());
    }

    #[test]
    fn test_composite_score_default_weights() {
        let mut profile = ContextProfile::new();
        profile.freshness_score = 0.8;
        profile.importance = 200; // ~0.78 normalized
        profile.confidence = 0.9;
        profile.risk_level = RiskLevel::Low;

        let score = profile.composite_score();
        // 0.3*0.8 + 0.4*0.78 + 0.3*0.9 = 0.24 + 0.312 + 0.27 = 0.822
        assert!((score - 0.822).abs() < 0.01);
    }

    #[test]
    fn test_composite_score_with_risk_penalty() {
        let mut profile = ContextProfile::new();
        profile.freshness_score = 1.0;
        profile.importance = 255;
        profile.confidence = 1.0;
        profile.risk_level = RiskLevel::High; // 0.5 penalty

        let score = profile.composite_score();
        // (0.3*1.0 + 0.4*1.0 + 0.3*1.0) * 0.5 = 1.0 * 0.5 = 0.5
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_scorer_freshness_decay() {
        let scorer = ContextScorer::new();

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let now = 1732373147000u64;
        let envelope = EnvelopeBuilder::new(record)
            .timestamp(now)
            .source("test-service")
            .build();

        // Fresh record (0 hours old)
        let profile = scorer.score_envelope(&envelope, now);
        assert!((profile.freshness_score - 1.0).abs() < 0.01);

        // 1 hour old
        let profile = scorer.score_envelope(&envelope, now + 3_600_000);
        assert!(profile.freshness_score > 0.95);

        // 24 hours old (decay constant)
        let profile = scorer.score_envelope(&envelope, now + 86_400_000);
        assert!((profile.freshness_score - (-1.0f64).exp()).abs() < 0.01);

        // 1 week old
        let profile = scorer.score_envelope(&envelope, now + 604_800_000);
        assert!(profile.freshness_score < 0.02);
    }

    #[test]
    fn test_scorer_trusted_source() {
        let config = ContextScorerConfig::new().add_trusted_source("auth-service".to_string());

        let scorer = ContextScorer::with_config(config);

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let now = 1732373147000u64;

        // Trusted source
        let envelope = EnvelopeBuilder::new(record.clone())
            .timestamp(now)
            .source("auth-service")
            .build();

        let profile = scorer.score_envelope(&envelope, now);
        assert!(profile.confidence >= 0.7); // boosted from 0.5

        // Untrusted source
        let envelope = EnvelopeBuilder::new(record)
            .timestamp(now)
            .source("unknown-service")
            .build();

        let profile = scorer.score_envelope(&envelope, now);
        assert_eq!(profile.confidence, 0.5); // default
    }

    #[test]
    fn test_scorer_suspicious_source() {
        let config = ContextScorerConfig::new().add_suspicious_source("malicious".to_string());

        let scorer = ContextScorer::with_config(config);

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::Int(42),
        });

        let envelope = EnvelopeBuilder::new(record)
            .timestamp(1732373147000)
            .source("malicious-bot")
            .build();

        let profile = scorer.score_envelope(&envelope, 1732373147000);
        assert_eq!(profile.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_scoring_weights_normalize() {
        let weights = ScoringWeights::new(1.0, 2.0, 3.0).normalize();

        assert!((weights.freshness_weight - 1.0 / 6.0).abs() < 0.001);
        assert!((weights.importance_weight - 2.0 / 6.0).abs() < 0.001);
        assert!((weights.confidence_weight - 3.0 / 6.0).abs() < 0.001);

        let sum = weights.freshness_weight + weights.importance_weight + weights.confidence_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_scorer_with_dictionary_importance() {
        use crate::dictionary::SemanticDictionary;
        use std::sync::Arc;

        let mut dict = SemanticDictionary::new();
        dict.add_field_name(12, "user_id".to_string());
        dict.add_importance(12, 200); // High importance

        dict.add_field_name(7, "is_active".to_string());
        dict.add_importance(7, 100); // Medium importance

        dict.add_field_name(99, "comment".to_string());
        dict.add_importance(99, 50); // Low importance

        let scorer = ContextScorer::new().with_dictionary(Arc::new(dict));

        // Record with high importance field
        let mut record1 = LnmpRecord::new();
        record1.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(14532),
        });

        let profile1 = scorer.score_record(&record1);
        assert_eq!(profile1.importance, 200);

        // Record with mixed importance fields (should use max)
        let mut record2 = LnmpRecord::new();
        record2.add_field(LnmpField {
            fid: 99,
            value: LnmpValue::String("test".to_string()),
        });
        record2.add_field(LnmpField {
            fid: 7,
            value: LnmpValue::Bool(true),
        });

        let profile2 = scorer.score_record(&record2);
        assert_eq!(profile2.importance, 100); // max of 50 and 100

        // Record with no importance defined (should use default)
        let mut record3 = LnmpRecord::new();
        record3.add_field(LnmpField {
            fid: 999,
            value: LnmpValue::Int(1),
        });

        let profile3 = scorer.score_record(&record3);
        assert_eq!(profile3.importance, 128); // default
    }

    #[test]
    fn test_prioritizer_filter_by_threshold() {
        let mut contexts = Vec::new();

        for i in 0..5 {
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 1,
                value: LnmpValue::Int(i),
            });

            let envelope = EnvelopeBuilder::new(record)
                .timestamp(1732373147000 - (i as u64 * 3_600_000))
                .source(&format!("source-{}", i))
                .build();

            let scorer = ContextScorer::new();
            let profile = scorer.score_envelope(&envelope, 1732373147000);
            contexts.push((envelope, profile));
        }

        // Filter with threshold 0.8
        let filtered = ContextPrioritizer::filter_by_threshold(contexts.clone(), 0.8);
        assert!(filtered.len() < contexts.len());
        for (_, profile) in &filtered {
            assert!(profile.composite_score() >= 0.8);
        }
    }

    #[test]
    fn test_prioritizer_select_top_k() {
        let scorer = ContextScorer::new();
        let now = 1732373147000u64;
        let mut contexts = Vec::new();

        // Create contexts with different freshness
        for i in 0..10 {
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 1,
                value: LnmpValue::Int(i as i64),
            });

            let envelope = EnvelopeBuilder::new(record)
                .timestamp(now - (i * 3_600_000))
                .source("test")
                .build();

            let profile = scorer.score_envelope(&envelope, now);
            contexts.push((envelope, profile));
        }

        // Select top 3
        let top_3 = ContextPrioritizer::select_top_k(contexts, 3, ScoringWeights::default());

        assert_eq!(top_3.len(), 3);

        // Verify they're in descending order of score
        for i in 1..top_3.len() {
            let score_prev = top_3[i - 1].1.composite_score();
            let score_curr = top_3[i].1.composite_score();
            assert!(score_prev >= score_curr);
        }
    }

    #[test]
    fn test_prioritizer_compute_stats() {
        let scorer = ContextScorer::new();
        let now = 1732373147000u64;
        let mut contexts = Vec::new();

        for i in 0..5 {
            let mut record = LnmpRecord::new();
            record.add_field(LnmpField {
                fid: 1,
                value: LnmpValue::Int(i as i64),
            });

            let envelope = EnvelopeBuilder::new(record)
                .timestamp(now - (i * 86_400_000))
                .source("test")
                .build();

            let profile = scorer.score_envelope(&envelope, now);
            contexts.push((envelope, profile));
        }

        let stats = ContextPrioritizer::compute_stats(&contexts);

        assert_eq!(stats.count, 5);
        assert!(stats.avg_freshness > 0.0 && stats.avg_freshness < 1.0);
        assert_eq!(stats.avg_importance, 128);
        assert_eq!(stats.avg_confidence, 0.5);
        assert_eq!(stats.risk_low, 5);
        assert_eq!(stats.risk_medium, 0);
    }
}
