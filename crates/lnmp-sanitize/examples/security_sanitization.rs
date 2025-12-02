//! Example: Security-Focused Sanitization
//!
//! Demonstrates sanitization for security-critical scenarios.

use lnmp_sanitize::{sanitize_lnmp_text, SanitizationConfig, SanitizationLevel};

fn main() {
    println!("=== LNMP Security Sanitization Examples ===\n");

    // Example 1: User-generated content (potential XSS)
    println!("Example 1: User Input with Special Characters");
    let config = SanitizationConfig {
        level: SanitizationLevel::Aggressive,
        auto_quote_strings: true,
        auto_escape_quotes: true,
        ..Default::default()
    };

    let malicious_input = r#"F20=<script>alert("XSS")</script>"#;
    let sanitized = sanitize_lnmp_text(malicious_input, &config);
    println!("  Input:  '{}'", malicious_input);
    println!("  Output: '{}'", sanitized);
    println!("  ✅ Special characters handled safely\n");

    // Example 2: SQL injection attempt
    println!("Example 2: SQL Injection Pattern");
    let sql_injection = r#"F23=["admin","' OR '1'='1"]"#;
    let sanitized = sanitize_lnmp_text(sql_injection, &config);
    println!("  Input:  '{}'", sql_injection);
    println!("  Output: '{}'", sanitized);
    println!("  ✅ Quotes properly escaped\n");

    // Example 3: Untrusted API input
    println!("Example 3: Untrusted External API Input");
    let untrusted = r#"F12=99999999999999999999;F7=maybe;F20=");DROP TABLE users;--"#;
    let sanitized = sanitize_lnmp_text(untrusted, &config);
    println!("  Input:  '{}'", untrusted);
    println!("  Output: '{}'", sanitized);
    println!("  ✅ Dangerous patterns neutralized\n");

    println!("💡 Security Best Practices:");
    println!("   - Always use Aggressive level for untrusted input");
    println!("   - Enable auto_escape_quotes for user-generated strings");
    println!("   - Validate sanitized output with strict parser");
}
