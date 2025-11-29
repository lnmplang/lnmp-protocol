//! Network messaging example
//!
//! This example demonstrates:
//! - Message kinds and priorities
//! - QoS levels
//! - Network routing concepts

use lnmp::net::MessageKind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Network Example ===\n");

    // 1. Message kinds
    println!("--- Message Kinds ---");
    println!("Available message kinds:");
    for kind in MessageKind::all() {
        println!(
            "  • {}: priority={}, TTL={}ms",
            kind,
            kind.default_priority(),
            kind.default_ttl_ms()
        );
    }

    // 2. Example message
    println!("\n--- Example Message ---");
    let event_kind = MessageKind::Event;
    println!("Message kind: {}", event_kind);
    println!("Is event: {}", event_kind.is_event());
    println!("Default priority: {}", event_kind.default_priority());
    println!("Default TTL: {}ms", event_kind.default_ttl_ms());

    // 3. Alert message
    println!("\n--- Alert Message ---");
    let alert_kind = MessageKind::Alert;
    println!("Message kind: {}", alert_kind);
    println!("Is alert: {}", alert_kind.is_alert());
    println!(
        "Default priority: {} (critical)",
        alert_kind.default_priority()
    );
    println!("Default TTL: {}ms (urgent)", alert_kind.default_ttl_ms());

    // 4. Routing info
    println!("\n--- Network Routing ---");
    println!("LNMP-Net provides:");
    println!("  • MessageKind classification (Event/State/Command/Query/Alert)");
    println!("  • Priority-based routing (0-255)");
    println!("  • TTL management for message freshness");
    println!("  • ECO (Efficient Collaborative Optimization) routing");

    println!("\n✅ Network operations completed successfully!");

    Ok(())
}
