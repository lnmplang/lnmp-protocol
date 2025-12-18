//! Content-aware routing policies for LNMP-Net.
//!
//! This module extends the base routing policy with content-based decision making
//! using zero-copy record views.

use crate::{NetError, Result, RoutingDecision};
use lnmp_core::{FieldId, LnmpRecordView, LnmpValueView};

/// A content-based routing rule.
///
/// Inspects specific fields in the record to make routing decisions.
#[derive(Debug, Clone)]
pub struct ContentRule {
    /// Field ID to inspect
    pub field_id: FieldId,
    /// Condition to check
    pub condition: FieldCondition,
    /// Routing decision if condition matches
    pub on_match: RoutingDecision,
    /// Description of this rule (for debugging)
    pub description: String,
}

/// Field condition for content-based routing.
#[derive(Debug, Clone)]
pub enum FieldCondition {
    /// String field equals exact value
    StringEquals(String),
    /// String field contains substring
    StringContains(String),
    /// String field matches any of the values
    StringIn(Vec<String>),
    /// Integer field in range [min, max] (inclusive)
    IntInRange(i64, i64),
    /// Integer field greater than threshold
    IntGreaterThan(i64),
    /// Integer field less than threshold
    IntLessThan(i64),
    /// Field exists (any value)
    Exists,
    /// Field does not exist
    NotExists,
}

impl ContentRule {
    /// Creates a new content rule.
    pub fn new(field_id: FieldId, condition: FieldCondition, on_match: RoutingDecision) -> Self {
        let description = format!("F{}: {:?} → {:?}", field_id, condition, on_match);
        Self {
            field_id,
            condition,
            on_match,
            description,
        }
    }

    /// Creates a rule that routes critical status to LLM.
    pub fn critical_status_to_llm(field_id: FieldId) -> Self {
        Self::new(
            field_id,
            FieldCondition::StringEquals("critical".to_string()),
            RoutingDecision::SendToLLM,
        )
    }

    /// Creates a rule that drops spam messages.
    pub fn drop_spam(field_id: FieldId) -> Self {
        Self::new(
            field_id,
            FieldCondition::StringContains("spam".to_string()),
            RoutingDecision::Drop,
        )
    }

    /// Checks if this rule matches the given record view.
    ///
    /// Returns `Some(decision)` if the condition matches, `None` otherwise.
    pub fn check(&self, record_view: &LnmpRecordView) -> Result<Option<RoutingDecision>> {
        match record_view.get_field(self.field_id) {
            Some(field) => {
                if self.check_condition(&field.value)? {
                    Ok(Some(self.on_match))
                } else {
                    Ok(None)
                }
            }
            None => {
                // Field doesn't exist
                if matches!(self.condition, FieldCondition::NotExists) {
                    Ok(Some(self.on_match))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn check_condition(&self, value: &LnmpValueView) -> Result<bool> {
        match (&self.condition, value) {
            (FieldCondition::StringEquals(target), LnmpValueView::String(s)) => {
                Ok(*s == target.as_str())
            }
            (FieldCondition::StringContains(substr), LnmpValueView::String(s)) => {
                Ok(s.contains(substr.as_str()))
            }
            (FieldCondition::StringIn(values), LnmpValueView::String(s)) => {
                Ok(values.iter().any(|v| v.as_str() == *s))
            }
            (FieldCondition::IntInRange(min, max), LnmpValueView::Int(i)) => {
                Ok(*i >= *min && *i <= *max)
            }
            (FieldCondition::IntGreaterThan(threshold), LnmpValueView::Int(i)) => {
                Ok(*i > *threshold)
            }
            (FieldCondition::IntLessThan(threshold), LnmpValueView::Int(i)) => Ok(*i < *threshold),
            (FieldCondition::Exists, _) => Ok(true),
            _ => Ok(false), // Type mismatch or condition doesn't apply
        }
    }
}

/// Content-aware routing policy builder.
///
/// Extends the base routing policy with content-based rules.
#[derive(Debug, Clone)]
pub struct ContentAwarePolicy {
    /// Base importance threshold for LLM routing
    pub llm_threshold: f64,
    /// Content-based rules (evaluated in order)
    pub content_rules: Vec<ContentRule>,
    /// Whether to drop expired messages
    pub drop_expired: bool,
    /// Whether to always route high-priority alerts
    pub always_route_alerts: bool,
}

impl ContentAwarePolicy {
    /// Creates a new content-aware policy with default threshold (0.7).
    pub fn new() -> Self {
        Self {
            llm_threshold: 0.7,
            content_rules: Vec::new(),
            drop_expired: true,
            always_route_alerts: true,
        }
    }

    /// Sets the LLM routing threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.llm_threshold = threshold;
        self
    }

    /// Adds a content-based rule.
    pub fn with_rule(mut self, rule: ContentRule) -> Self {
        self.content_rules.push(rule);
        self
    }

    /// Adds multiple content rules.
    pub fn with_rules(mut self, rules: Vec<ContentRule>) -> Self {
        self.content_rules.extend(rules);
        self
    }

    /// Decides routing based on content inspection (zero-copy).
    ///
    /// Decision flow:
    /// 1. Check expiry → Drop
    /// 2. Evaluate content rules (in order) → First match wins
    /// 3. Fallback to priority-based routing
    pub fn decide_content_aware(
        &self,
        kind: crate::kind::MessageKind,
        priority: u8,
        expires_at: Option<u64>,
        record_view: &LnmpRecordView,
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

        // 2. Evaluate content rules (zero-copy field access!)
        for rule in &self.content_rules {
            if let Some(decision) = rule.check(record_view)? {
                return Ok(decision);
            }
        }

        // 3. Fallback to priority-based routing
        if self.always_route_alerts && kind.is_alert() && priority > 200 {
            return Ok(RoutingDecision::SendToLLM);
        }

        // Simple priority threshold for fallback
        let priority_score = priority as f64 / 255.0;
        if priority_score >= self.llm_threshold {
            Ok(RoutingDecision::SendToLLM)
        } else {
            Ok(RoutingDecision::ProcessLocally)
        }
    }
}

impl Default for ContentAwarePolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

    // Helper: Create a view from a record using binary encode/decode
    fn encode_decode_view<'a>(
        record: &LnmpRecord,
        buffer: &'a mut Vec<u8>,
    ) -> lnmp_core::LnmpRecordView<'a> {
        use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};

        let encoder = BinaryEncoder::new();
        *buffer = encoder.encode(record).unwrap();

        let decoder = BinaryDecoder::new();
        decoder.decode_view(buffer).unwrap()
    }

    fn sample_record_with_status(status: &str) -> LnmpRecord {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::String(status.to_string()),
        });
        record
    }

    #[test]
    fn test_content_rule_string_equals() {
        let rule = ContentRule::critical_status_to_llm(50);
        let record = sample_record_with_status("critical");

        let mut buffer = Vec::new();
        let view = encode_decode_view(&record, &mut buffer);
        let decision = rule.check(&view).unwrap();
        assert_eq!(decision, Some(RoutingDecision::SendToLLM));
    }

    #[test]
    fn test_content_rule_string_contains() {
        let rule = ContentRule::drop_spam(50);
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::String("this is spam message".to_string()),
        });

        let mut buffer = Vec::new();
        let view = encode_decode_view(&record, &mut buffer);
        let decision = rule.check(&view).unwrap();
        assert_eq!(decision, Some(RoutingDecision::Drop));
    }

    #[test]
    fn test_content_rule_int_range() {
        let rule = ContentRule::new(
            32,
            FieldCondition::IntInRange(200, 255),
            RoutingDecision::SendToLLM,
        );

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 32,
            value: LnmpValue::Int(220),
        });

        let mut buffer = Vec::new();
        let view = encode_decode_view(&record, &mut buffer);
        let decision = rule.check(&view).unwrap();
        assert_eq!(decision, Some(RoutingDecision::SendToLLM));
    }

    #[test]
    fn test_content_aware_policy() {
        let policy = ContentAwarePolicy::new()
            .with_rule(ContentRule::critical_status_to_llm(50))
            .with_rule(ContentRule::drop_spam(24));

        let critical_record = sample_record_with_status("critical");
        let mut buffer = Vec::new();
        let view = encode_decode_view(&critical_record, &mut buffer);

        let decision = policy
            .decide_content_aware(crate::kind::MessageKind::Event, 100, None, &view, 1000)
            .unwrap();

        assert_eq!(decision, RoutingDecision::SendToLLM);
    }

    #[test]
    fn test_fallback_to_priority() {
        let policy = ContentAwarePolicy::new(); // No content rules

        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Int(42),
        });

        let mut buffer = Vec::new();
        let view = encode_decode_view(&record, &mut buffer);

        // High priority → LLM
        let decision = policy
            .decide_content_aware(crate::kind::MessageKind::Event, 250, None, &view, 1000)
            .unwrap();
        assert_eq!(decision, RoutingDecision::SendToLLM);

        // Low priority → Local (need fresh decode since view borrows buffer)
        let view2 = encode_decode_view(&record, &mut buffer);
        let decision = policy
            .decide_content_aware(crate::kind::MessageKind::Event, 50, None, &view2, 1000)
            .unwrap();
        assert_eq!(decision, RoutingDecision::ProcessLocally);
    }
}
