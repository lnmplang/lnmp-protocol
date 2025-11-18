use crate::{sanitize_lnmp_text, SanitizationConfig, SanitizationLevel};

#[test]
fn returns_borrowed_when_clean() {
    let input = "F1=1;F2=\"ok\"";
    let config = SanitizationConfig::default();
    let sanitized = sanitize_lnmp_text(input, &config);
    assert!(matches!(sanitized, std::borrow::Cow::Borrowed(_)));
    assert_eq!(sanitized, input);
}

#[test]
fn trims_and_compacts_whitespace() {
    let input = "  F1=1 ;  F2=\"hi\"  \n";
    let config = SanitizationConfig::default();
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, "F1=1;F2=\"hi\"\n");
}

#[test]
fn repairs_unterminated_quote() {
    let input = "F1=\"hello";
    let config = SanitizationConfig::default();
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, "F1=\"hello\"");
}

#[test]
fn normalizes_booleans() {
    let input = "F1=true;F2=no";
    let config = SanitizationConfig {
        normalize_numbers: true,
        level: SanitizationLevel::Aggressive,
        ..Default::default()
    };
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, "F1=1;F2=0");
}

#[test]
fn respects_minimal_level_for_hashes() {
    let input = "#comment\nF1=1";
    let config = SanitizationConfig {
        level: SanitizationLevel::Minimal,
        ..Default::default()
    };
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, input);
}

#[test]
fn keeps_already_escaped_quotes_intact() {
    let input = r#"F1="Hello \"world\"""#;
    let config = SanitizationConfig::default();
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, input);
}

#[test]
fn auto_quotes_unquoted_value_with_quotes() {
    let input = r#"F1=Hello "world";F2=ok"#;
    let config = SanitizationConfig {
        auto_quote_strings: true,
        ..Default::default()
    };
    let sanitized = sanitize_lnmp_text(input, &config);
    assert_eq!(sanitized, r#"F1="Hello \"world\"";F2=ok"#);
}
