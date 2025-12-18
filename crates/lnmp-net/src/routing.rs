//! Routing logic for LNMP-Net messages

use lnmp_sfe::{ContextScorer, ContextScorerConfig};

use crate::error::Result;
use crate::message::NetMessage;

/// Routing decision for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    /// Send message to LLM for processing
    SendToLLM,
    /// Process message locally at edge/service level
    ProcessLocally,
    /// Drop message (expired, low priority, etc.)
    Drop,
}

/// Policy for routing messages to LLM vs local processing
///
/// Implements the ECO (Energy/Token Optimization) profile logic:
/// - Alerts with high priority always routed to LLM
/// - Expired messages dropped
/// - Event/State messages scored using SFE and routed based on threshold
/// - Commands/Queries typically processed locally
///
/// # Examples
///
/// ```
/// use lnmp_core::LnmpRecord;
/// use lnmp_envelope::EnvelopeBuilder;
/// use lnmp_net::{MessageKind, NetMessage, RoutingPolicy, RoutingDecision};
///
/// let policy = RoutingPolicy::default();
///
/// // Alert message -> always to LLM
/// let envelope = EnvelopeBuilder::new(LnmpRecord::new())
///     .timestamp(1000)
///     .build();
/// let alert = NetMessage::new(envelope, MessageKind::Alert);
/// assert_eq!(policy.decide(&alert, 2000).unwrap(), RoutingDecision::SendToLLM);
/// ```
#[derive(Debug, Clone)]
pub struct RoutingPolicy {
    /// Minimum importance score (0.0-1.0) to route to LLM
    pub llm_threshold: f64,

    /// Always route Alert messages to LLM regardless of score
    pub always_route_alerts: bool,

    /// Automatically drop expired messages
    pub drop_expired: bool,

    /// SFE scorer for computing importance/freshness
    scorer_config: ContextScorerConfig,
}

impl RoutingPolicy {
    /// Creates a new routing policy with custom threshold
    pub fn new(llm_threshold: f64) -> Self {
        Self {
            llm_threshold,
            always_route_alerts: true,
            drop_expired: true,
            scorer_config: ContextScorerConfig::default(),
        }
    }

    /// Sets whether to always route alerts to LLM
    pub fn with_always_route_alerts(mut self, enabled: bool) -> Self {
        self.always_route_alerts = enabled;
        self
    }

    /// Sets whether to drop expired messages
    pub fn with_drop_expired(mut self, enabled: bool) -> Self {
        self.drop_expired = enabled;
        self
    }

    /// Sets custom SFE scorer configuration
    pub fn with_scorer_config(mut self, config: ContextScorerConfig) -> Self {
        self.scorer_config = config;
        self
    }

    /// Decides how to route a message
    ///
    /// Decision flow:
    /// 1. Check expiry (if enabled) -> Drop
    /// 2. Check if Alert + high priority -> SendToLLM
    /// 3. For Event/State: compute importance score -> threshold check
    /// 4. Commands/Queries -> ProcessLocally
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to route
    /// * `now_ms` - Current time in epoch milliseconds
    pub fn decide(&self, msg: &NetMessage, now_ms: u64) -> Result<RoutingDecision> {
        // 1. Check expiry
        if self.drop_expired && msg.is_expired(now_ms)? {
            return Ok(RoutingDecision::Drop);
        }

        // 2. Always route high-priority alerts
        if self.always_route_alerts && msg.kind.is_alert() && msg.priority > 200 {
            return Ok(RoutingDecision::SendToLLM);
        }

        // 3. For Event/State: compute importance and check threshold
        if msg.kind.is_event() || msg.kind.is_state() {
            let importance = self.base_importance(msg, now_ms)?;
            return if importance >= self.llm_threshold {
                Ok(RoutingDecision::SendToLLM)
            } else {
                Ok(RoutingDecision::ProcessLocally)
            };
        }

        // 4. Commands and Queries: local processing by default
        // (Future: complex queries could be routed to LLM)
        Ok(RoutingDecision::ProcessLocally)
    }

    /// Decides how to route a message (Zero-Copy View)
    ///
    /// # Arguments
    ///
    /// * `kind` - Message kind
    /// * `priority` - Message priority
    /// * `metadata` - Envelope metadata
    /// * `expires_at` - Expiration timestamp (if any)
    /// * `record_view` - The record view (currently unused by default policy but available for extensions)
    /// * `now_ms` - Current time in epoch milliseconds
    pub fn decide_view(
        &self,
        kind: crate::kind::MessageKind,
        priority: u8,
        metadata: &lnmp_envelope::EnvelopeMetadata,
        expires_at: Option<u64>,
        _record_view: &lnmp_core::LnmpRecordView,
        now_ms: u64,
    ) -> Result<RoutingDecision> {
        // 1. Check expiry
        if self.drop_expired {
            if let Some(exp) = expires_at {
                if exp <= now_ms {
                    return Ok(RoutingDecision::Drop);
                }
            }
        }

        // 2. Always route high-priority alerts
        if self.always_route_alerts && kind.is_alert() && priority > 200 {
            return Ok(RoutingDecision::SendToLLM);
        }

        // 3. For Event/State: compute importance and check threshold
        if kind.is_event() || kind.is_state() {
            let importance = self.base_importance_view(priority, metadata, now_ms)?;
            return if importance >= self.llm_threshold {
                Ok(RoutingDecision::SendToLLM)
            } else {
                Ok(RoutingDecision::ProcessLocally)
            };
        }

        // 4. Commands and Queries: local processing by default
        Ok(RoutingDecision::ProcessLocally)
    }

    /// Computes base importance score for a message (0.0-1.0)
    ///
    /// Combines:
    /// - Priority (0-255 normalized to 0.0-1.0): 50% weight
    /// - SFE score (freshness + importance from lnmp-sfe): 50% weight
    ///
    /// # Formula
    ///
    /// ```text
    /// importance = (priority / 255.0) * 0.5 + sfe_score * 0.5
    /// ```
    pub fn base_importance(&self, msg: &NetMessage, now_ms: u64) -> Result<f64> {
        // Normalize priority to 0.0-1.0
        let priority_score = msg.priority as f64 / 255.0;

        // Compute SFE score using ContextScorer
        let sfe_score = if msg.timestamp().is_some() {
            let scorer = ContextScorer::with_config(self.scorer_config.clone());
            let profile = scorer.score_envelope(&msg.envelope, now_ms);

            // Use composite_score which combines freshness + importance + confidence
            profile.composite_score()
        } else {
            // No timestamp, use half score
            0.5
        };

        // Combine priority and SFE
        Ok(priority_score * 0.5 + sfe_score * 0.5)
    }

    /// Computes base importance score from view data
    pub fn base_importance_view(
        &self,
        priority: u8,
        metadata: &lnmp_envelope::EnvelopeMetadata,
        now_ms: u64,
    ) -> Result<f64> {
        // Normalize priority to 0.0-1.0
        let priority_score = priority as f64 / 255.0;

        // Compute SFE score using ContextScorer
        let sfe_score = if metadata.timestamp.is_some() {
            let scorer = ContextScorer::with_config(self.scorer_config.clone());
            let profile = scorer.score_metadata(metadata, now_ms);
            profile.composite_score()
        } else {
            0.5
        };

        Ok(priority_score * 0.5 + sfe_score * 0.5)
    }
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self::new(0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind::MessageKind;
    use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
    use lnmp_envelope::EnvelopeBuilder;

    fn sample_record() -> LnmpRecord {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 42,
            value: LnmpValue::Int(100),
        });
        record
    }

    #[test]
    fn test_alert_always_routed_to_llm() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();
        let msg = NetMessage::new(envelope, MessageKind::Alert);

        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::SendToLLM
        );
    }

    #[test]
    fn test_expired_message_dropped() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // Event with TTL=5000ms, age=6000ms -> expired
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 100, 5000);

        assert_eq!(policy.decide(&msg, 7000).unwrap(), RoutingDecision::Drop);
    }

    #[test]
    fn test_low_priority_event_processed_locally() {
        let policy = RoutingPolicy::default(); // threshold = 0.7

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // Low priority event (30/255 = 0.12) -> below threshold
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 30, 10000);

        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::ProcessLocally
        );
    }

    #[test]
    fn test_high_priority_state_sent_to_llm() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // High priority state (220/255 = 0.86) -> above threshold
        let msg = NetMessage::with_qos(envelope, MessageKind::State, 220, 10000);

        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::SendToLLM
        );
    }

    #[test]
    fn test_command_processed_locally() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessage::new(envelope, MessageKind::Command);

        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::ProcessLocally
        );
    }

    #[test]
    fn test_query_processed_locally() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessage::new(envelope, MessageKind::Query);

        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::ProcessLocally
        );
    }

    #[test]
    fn test_base_importance_high_priority() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 255, 10000);

        let importance = policy.base_importance(&msg, 2000).unwrap();

        // High priority (1.0) with fresh timestamp should yield high score
        assert!(
            importance > 0.5,
            "Expected high importance, got {}",
            importance
        );
    }

    #[test]
    fn test_base_importance_low_priority() {
        let policy = RoutingPolicy::default();

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 10, 10000);

        let importance = policy.base_importance(&msg, 2000).unwrap();

        // Low priority should yield lower score
        assert!(
            importance < 0.7,
            "Expected low importance, got {}",
            importance
        );
    }

    #[test]
    fn test_custom_threshold() {
        let policy = RoutingPolicy::new(0.9); // Very high threshold

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // Medium priority (150/255 = 0.59)
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 150, 10000);

        // Should be processed locally due to high threshold
        assert_eq!(
            policy.decide(&msg, 2000).unwrap(),
            RoutingDecision::ProcessLocally
        );
    }

    #[test]
    fn test_disable_alert_routing() {
        let policy = RoutingPolicy::default().with_always_route_alerts(false);

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // Low priority alert (if alerts didn't auto-route, this would be local)
        let msg = NetMessage::with_qos(envelope, MessageKind::Alert, 50, 1000);

        // With alerts disabled from auto-route, should check importance
        let decision = policy.decide(&msg, 2000).unwrap();

        // Low priority alert should not auto-route
        assert_ne!(decision, RoutingDecision::SendToLLM);
    }

    #[test]
    fn test_disable_drop_expired() {
        let policy = RoutingPolicy::default().with_drop_expired(false);

        let envelope = EnvelopeBuilder::new(sample_record())
            .timestamp(1000)
            .build();

        // Expired event
        let msg = NetMessage::with_qos(envelope, MessageKind::Event, 30, 2000);

        // Should not drop even though expired
        assert_ne!(policy.decide(&msg, 10000).unwrap(), RoutingDecision::Drop);
    }

    #[test]
    fn test_decide_view_routing() {
        use lnmp_core::LnmpRecordView;
        use lnmp_envelope::EnvelopeMetadata;

        let policy = RoutingPolicy::default();

        // Create metadata representing a high priority alert
        // We can't easily construct EnvelopeMetadata if fields are private or no builder?
        // LnmpEnvelope::metadata is pub.
        // We can create an envelope and take its metadata.
        // Note: EnvelopeMetadata usually implements Clone or we can construct it if fields are pub.
        // Let's rely on standard construction via EnvelopeBuilder then extracting.

        let record = sample_record();
        let envelope = EnvelopeBuilder::new(record)
            .timestamp(1000)
            .source("auth-service") // Trusted source logic might apply if configured
            .build();

        let metadata = envelope.metadata;

        let empty_view = LnmpRecordView::from_fields(vec![]);

        // 1. Alert routing (High Priority)
        let decision = policy
            .decide_view(
                MessageKind::Alert,
                250, // High priority
                &metadata,
                None, // No explicit expiry passed
                &empty_view,
                2000,
            )
            .unwrap();
        assert_eq!(decision, RoutingDecision::SendToLLM);

        // 2. Expired view
        let decision_expired = policy
            .decide_view(
                MessageKind::Event,
                100,
                &metadata,
                Some(1500), // Expires at 1500
                &empty_view,
                2000, // Now is 2000 -> Expired
            )
            .unwrap();
        assert_eq!(decision_expired, RoutingDecision::Drop);
    }
}
