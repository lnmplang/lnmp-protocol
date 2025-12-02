//! Secure API Gateway - Showcase Example
//!
//! API gateway that sanitizes and routes untrusted external data.
//! Demonstrates input sanitization, envelopes, transport headers, and routing.
//!
//! Run: `cargo run --example secure_api_gateway`

use lnmp::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Simulated external API request
struct ApiRequest {
    endpoint: String,
    user_input: String,
    source_ip: String,
}

fn main() {
    println!("🔒 Secure API Gateway - LNMP Showcase\n");

    // Simulated untrusted API requests
    let requests = [
        ApiRequest {
            endpoint: "/api/users".to_string(),
            user_input: r#"F20=<script>alert("XSS")</script>;F7=true"#.to_string(),
            source_ip: "203.0.113.42".to_string(),
        },
        ApiRequest {
            endpoint: "/api/search".to_string(),
            user_input: r#"F30=' OR '1'='1;F31=DROP TABLE"#.to_string(),
            source_ip: "untrusted-bot".to_string(),
        },
        ApiRequest {
            endpoint: "/api/config".to_string(),
            user_input: "F50=critical-data;F51=admin".to_string(),
            source_ip: "trusted-service".to_string(),
        },
    ];

    // Configure aggressive sanitization for untrusted input
    let sanitize_config = lnmp::sanitize::SanitizationConfig {
        level: lnmp::sanitize::SanitizationLevel::Aggressive,
        auto_quote_strings: true,
        auto_escape_quotes: true,
        normalize_booleans: true,
        normalize_numbers: true,
    };

    println!("🛡️  Processing {} API requests:\n", requests.len());

    for (idx, request) in requests.iter().enumerate() {
        println!("Request #{} - {}", idx + 1, request.endpoint);
        println!("  Source: {}", request.source_ip);
        println!("  Raw input: {}", request.user_input);

        // Step 1: Sanitize untrusted input
        let sanitized = lnmp::sanitize::sanitize_lnmp_text(&request.user_input, &sanitize_config);
        let was_modified = sanitized != request.user_input;

        if was_modified {
            println!("  ⚠️  Input sanitized for security");
            println!("  Sanitized: {}", sanitized);
        } else {
            println!("  ✅ Input clean");
        }

        // Step 2: Parse sanitized LNMP
        let record = match Parser::new_lenient(&sanitized) {
            Ok(mut parser) => match parser.parse_record() {
                Ok(r) => r,
                Err(e) => {
                    println!("  ❌ Parse error: {}", e);
                    continue;
                }
            },
            Err(e) => {
                println!("  ❌ Parser init error: {}", e);
                continue;
            }
        };

        // Step 3: Wrap in operational envelope
        let now = current_timestamp();
        let envelope = lnmp::envelope::EnvelopeBuilder::new(record)
            .timestamp(now)
            .source("api-gateway")
            .trace_id(format!("trace-{}", idx + 1))
            .build();

        // Step 4: Determine routing based on source trust
        let is_trusted = request.source_ip.contains("trusted");
        let priority = if is_trusted { 200 } else { 100 };

        let route = if is_trusted {
            "🚀 FAST_LANE"
        } else {
            "🐌 INSPECTION_QUEUE"
        };

        println!("  Route: {} (priority: {})", route, priority);

        // Step 5: Trace context for observability
        if let Some(trace) = &envelope.metadata.trace_id {
            println!("  Trace ID: {}", trace);
        }

        println!();
    }

    println!("✅ All requests processed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Aggressive input sanitization (XSS/SQL injection protection)");
    println!("   • Lenient parsing for robustness");
    println!("   • Operational envelopes with trace context");
    println!("   • Trust-based routing decisions");
    println!("   • Meta crate integration (lnmp::*)");
}
