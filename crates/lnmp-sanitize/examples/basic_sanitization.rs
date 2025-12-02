//! Example: Basic Input Sanitization
//!
//! Demonstrates how to use the sanitization layer to clean up LNMP text input.

use lnmp_sanitize::{sanitize_lnmp_text, SanitizationConfig, SanitizationLevel};

fn main() {
    println!("=== LNMP Input Sanitization Examples ===\n");

    // Example 1: Minimal sanitization (whitespace only)
    println!("Example 1: Minimal Sanitization");
    let config = SanitizationConfig {
        level: SanitizationLevel::Minimal,
        ..Default::default()
    };
    let input = "  F12=14532  ;  F7=1  ";
    let sanitized = sanitize_lnmp_text(input, &config);
    println!("  Input:  '{}'", input);
    println!("  Output: '{}'", sanitized);
    println!();

    // Example 2: Auto-quote unquoted strings
    println!("Example 2: Auto-Quote Strings");
    let config = SanitizationConfig {
        level: SanitizationLevel::Normal,
        auto_quote_strings: true,
        ..Default::default()
    };
    let input = "F20=Hello World";
    let sanitized = sanitize_lnmp_text(input, &config);
    println!("  Input:  '{}'", input);
    println!("  Output: '{}'", sanitized);
    println!();

    // Example 3: Normalize booleans
    println!("Example 3: Normalize Booleans");
    let config = SanitizationConfig {
        level: SanitizationLevel::Normal,
        normalize_booleans: true,
        ..Default::default()
    };
    let input = "F7=true;F8=false;F9=yes;F10=no";
    let sanitized = sanitize_lnmp_text(input, &config);
    println!("  Input:  '{}'", input);
    println!("  Output: '{}'", sanitized);
    println!();

    // Example 4: Aggressive sanitization
    println!("Example 4: Aggressive Sanitization (All Features)");
    let config = SanitizationConfig {
        level: SanitizationLevel::Aggressive,
        auto_quote_strings: true,
        auto_escape_quotes: true,
        normalize_booleans: true,
        normalize_numbers: true,
    };
    let input = "  F12 = +042  ; F7=TRUE ; F20=User Name  ";
    let sanitized = sanitize_lnmp_text(input, &config);
    println!("  Input:  '{}'", input);
    println!("  Output: '{}'", sanitized);
    println!();

    println!("✅ All sanitization examples completed!");
}
