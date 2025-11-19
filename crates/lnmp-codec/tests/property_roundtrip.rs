#![allow(clippy::approx_constant)]

use lnmp_codec::{Encoder, Parser, ParsingMode};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_sanitize::{sanitize_lnmp_text, SanitizationConfig, SanitizationLevel};
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::BTreeMap;

fn string_strategy() -> impl Strategy<Value = String> {
    // Keep strings small to reduce shrink noise and avoid pathological cases.
    "[a-zA-Z]{1,8}".prop_map(|s| s.trim().to_string())
}

fn value_strategy() -> impl Strategy<Value = LnmpValue> {
    prop_oneof![
        (-1_000_000i64..=1_000_000i64).prop_map(LnmpValue::Int),
        // Keep floats reasonable and finite to avoid parse edge cases.
        any::<f64>()
            .prop_filter("finite, bounded float", |f| f.is_finite() && f.abs() <= 1e6)
            .prop_map(|f| {
                let normalized = if f == -0.0 { 0.0 } else { f };
                LnmpValue::Float(normalized)
            }),
        any::<bool>().prop_map(LnmpValue::Bool),
        string_strategy().prop_map(LnmpValue::String),
        vec(string_strategy(), 0..3).prop_map(LnmpValue::StringArray),
    ]
}

fn record_strategy() -> impl Strategy<Value = LnmpRecord> {
    vec(
        (any::<u16>(), value_strategy()),
        0..16, // keep small to ensure fast property runs
    )
    .prop_map(|fields| {
        let mut dedup = BTreeMap::new();
        for (fid, value) in fields {
            dedup.entry(fid).or_insert(value);
        }
        let mut record = LnmpRecord::new();
        for (fid, value) in dedup {
            record.add_field(LnmpField { fid, value });
        }
        record
    })
}

proptest! {
    /// Canonical encoding must be idempotent.
    #[test]
    fn canonical_encode_is_stable(record in record_strategy()) {
        let encoder = Encoder::new();
        let canonical = encoder.encode(&record);

        let mut parser = Parser::with_mode(&canonical, ParsingMode::Strict).expect("parser init");
        let parsed = parser.parse_record().expect("parse canonical");
        let canonical_again = encoder.encode(&parsed);

        prop_assert_eq!(canonical, canonical_again);
    }
}

proptest! {
    /// Sanitizer must not alter already-canonical text and it must remain parseable.
    #[test]
    fn sanitize_preserves_canonical(record in record_strategy()) {
        let encoder = Encoder::new();
        let canonical = encoder.encode(&record);
        let sanitized = sanitize_lnmp_text(
            &canonical,
            &SanitizationConfig {
                level: SanitizationLevel::Minimal,
                auto_quote_strings: false,
                auto_escape_quotes: false,
                normalize_booleans: false,
                normalize_numbers: false,
            },
        );

        let mut parser = Parser::with_mode(sanitized.as_ref(), ParsingMode::Strict).expect("parser init");
        let reparsed = parser.parse_record().expect("parse sanitized canonical");
        let canonical_again = encoder.encode(&reparsed);

        prop_assert_eq!(canonical, canonical_again);
    }
}
