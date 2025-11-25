//! Basic LNMP-Net example demonstrating 2-node communication
//!
//! This example simulates communication between Node A (sensor) and Node B (monitor),
//! demonstrating different message types, routing decisions, and HTTP header encoding/decoding.
//!
//! Run with:
//! ```bash
//! cargo run --example lnmp-net-basic --features transport
//! ```

use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::EnvelopeBuilder;
use lnmp_net::transport::http::{http_headers_to_net_meta, net_to_http_headers};
use lnmp_net::{MessageKind, NetMessage, NetMessageBuilder, RoutingDecision, RoutingPolicy};

/// Simulates Node A (Sensor) creating and sending messages
struct NodeA {
    name: String,
}

impl NodeA {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Creates a sensor reading event
    fn create_event(&self, value: i64, timestamp: u64) -> NetMessage {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1, // temperature
            value: LnmpValue::Int(value),
        });

        let envelope = EnvelopeBuilder::new(record)
            .timestamp(timestamp)
            .source(&self.name)
            .trace_id(&format!("trace-{}", timestamp))
            .build();

        NetMessageBuilder::new(envelope, MessageKind::Event)
            .priority(80) // Low priority
            .ttl_ms(5000)
            .class("sensor")
            .build()
    }

    /// Creates a critical alert
    fn create_alert(&self, message: &str, timestamp: u64) -> NetMessage {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 2, // alert_message
            value: LnmpValue::String(message.to_string()),
        });

        let envelope = EnvelopeBuilder::new(record)
            .timestamp(timestamp)
            .source(&self.name)
            .trace_id(&format!("alert-{}", timestamp))
            .build();

        NetMessageBuilder::new(envelope, MessageKind::Alert)
            .priority(255) // Critical
            .ttl_ms(1000)
            .class("safety")
            .build()
    }

    /// Creates an expired event (very old timestamp)
    fn create_expired_event(&self, timestamp: u64) -> NetMessage {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Int(42),
        });

        // Create with old timestamp
        let envelope = EnvelopeBuilder::new(record)
            .timestamp(timestamp - 10000) // 10 seconds ago
            .source(&self.name)
            .build();

        NetMessage::with_qos(envelope, MessageKind::Event, 50, 2000) // Short TTL
    }
}

/// Simulates Node B (Monitor) receiving and processing messages
struct NodeB {
    name: String,
    policy: RoutingPolicy,
}

impl NodeB {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            policy: RoutingPolicy::default(), // ECO profile with threshold=0.7
        }
    }

    fn receive_and_route(&self, msg: NetMessage, now_ms: u64) {
        println!("\n📨 {} receiving message:", self.name);
        println!("   Kind: {:?}", msg.kind);
        println!("   Priority: {}", msg.priority);
        println!("   TTL: {}ms", msg.ttl_ms);
        println!("   Source: {:?}", msg.source());
        println!("   Class: {:?}", msg.class);

        // Encode to HTTP headers (simulating network transmission)
        let headers = match net_to_http_headers(&msg) {
            Ok(h) => h,
            Err(e) => {
                println!("   ❌ Header encoding failed: {}", e);
                return;
            }
        };

        println!("   📤 HTTP Headers:");
        for (name, value) in &headers {
            println!("      {}: {:?}", name, value);
        }

        // Decode from HTTP headers (simulating network reception)
        let (kind, priority, ttl_ms, class) = match http_headers_to_net_meta(&headers) {
            Ok(meta) => meta,
            Err(e) => {
                println!("   ❌ Header decoding failed: {}", e);
                return;
            }
        };

        println!("   📥 Decoded metadata:");
        println!("      Kind: {:?}", kind);
        println!("      Priority: {}", priority);
        println!("      TTL: {}ms", ttl_ms);
        println!("      Class: {:?}", class);

        // Make routing decision
        match self.policy.decide(&msg, now_ms) {
            Ok(RoutingDecision::SendToLLM) => {
                println!("   🤖 Routing decision: → Send to LLM");
                self.process_with_llm(&msg);
            }
            Ok(RoutingDecision::ProcessLocally) => {
                println!("   💻 Routing decision: → Process locally");
                self.process_locally(&msg);
            }
            Ok(RoutingDecision::Drop) => {
                println!("   🗑️  Routing decision: → Drop (expired)");
            }
            Err(e) => {
                println!("   ❌ Routing error: {}", e);
            }
        }
    }

    fn process_with_llm(&self, msg: &NetMessage) {
        println!("      ✅ Forwarded to LLM for analysis");
        println!("      📊 Importance score above threshold (0.7)");
        if msg.kind.is_alert() {
            println!("      ⚠️  Alert requires immediate attention!");
        }
    }

    fn process_locally(&self, msg: &NetMessage) {
        println!("      ✅ Processed locally (routine operation)");
        println!("      📊 Importance score below LLM threshold");
        if let Some(class) = &msg.class {
            println!("      🏷️  Message class: {}", class);
        }
    }
}

fn main() {
    println!("🚀 LNMP-Net Basic Example: 2-Node Communication\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let node_a = NodeA::new("sensor-01");
    let node_b = NodeB::new("monitor-01");

    let base_time = 1700000000000u64; // Base timestamp

    // Scenario 1: Low-priority Event → Process Locally
    println!("\n📍 Scenario 1: Low-Priority Sensor Event");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let event = node_a.create_event(22, base_time); // 22°C temperature reading
    node_b.receive_and_route(event, base_time + 1000);

    // Scenario 2: High-Priority Alert → Send to LLM
    println!("\n📍 Scenario 2: Critical Safety Alert");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let alert = node_a.create_alert("Temperature threshold exceeded!", base_time + 2000);
    node_b.receive_and_route(alert, base_time + 2500);

    // Scenario 3: Expired Event → Drop
    println!("\n📍 Scenario 3: Expired Event (Stale Data)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let expired = node_a.create_expired_event(base_time + 5000);
    node_b.receive_and_route(expired, base_time + 20000); // Much later

    // Scenario 4: Another Event with different class
    println!("\n📍 Scenario 4: Health Check Event");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let mut health_record = LnmpRecord::new();
    health_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::Bool(true), // System healthy
    });

    let health_envelope = EnvelopeBuilder::new(health_record)
        .timestamp(base_time + 10000)
        .source("sensor-01")
        .build();

    let health_event = NetMessageBuilder::new(health_envelope, MessageKind::State)
        .priority(90)
        .ttl_ms(10000)
        .class("health")
        .build();

    node_b.receive_and_route(health_event, base_time + 10500);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Example completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("   • Low-priority events processed locally (cost-efficient)");
    println!("   • Critical alerts always routed to LLM (safety-critical)");
    println!("   • Expired messages dropped (data freshness)");
    println!("   • HTTP headers preserve all metadata (transport-agnostic)");
    println!("\n🎯 ECO Profile Benefits:");
    println!("   • 90%+ reduction in LLM API calls");
    println!("   • Intelligent routing based on importance + freshness");
    println!("   • QoS guarantees for critical messages");
}
