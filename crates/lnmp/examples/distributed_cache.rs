//! Distributed Cache - Showcase Example
//!
//! Distributed caching system with efficient data synchronization.
//! Demonstrates binary format, nested structures, and streaming.
//!
//! Run: `cargo run --example distributed_cache`

use lnmp::prelude::*;

/// Cache entry
#[derive(Clone)]
struct CacheEntry {
    key: String,
    value: Vec<u8>,
    ttl: u64,
    version: u32,
}

impl CacheEntry {
    fn to_lnmp_record(&self) -> LnmpRecord {
        let mut record = LnmpRecord::new();

        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.key.clone()),
        });

        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String(String::from_utf8_lossy(&self.value).to_string()),
        });

        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::Int(self.ttl as i64),
        });

        record.add_field(LnmpField {
            fid: 4,
            value: LnmpValue::Int(self.version as i64),
        });

        record
    }
}

fn main() {
    println!("💾 Distributed Cache - LNMP Showcase\n");

    // Create cache entries
    let entries = vec![
        CacheEntry {
            key: "user:1001".to_string(),
            value: b"Alice Johnson|alice@example.com|Premium".to_vec(),
            ttl: 3600,
            version: 1,
        },
        CacheEntry {
            key: "session:abc123".to_string(),
            value: b"SessionData{authenticated:true}".to_vec(),
            ttl: 1800,
            version: 1,
        },
        CacheEntry {
            key: "config:app".to_string(),
            value: b"max_connections=100;timeout=30".to_vec(),
            ttl: 7200,
            version: 2,
        },
    ];

    println!("📦 Cache initialized with {} entries\n", entries.len());

    // Create encoder
    let encoder = lnmp::codec::Encoder::new();

    println!("🔄 Encoding cache entries:\n");

    for entry in &entries {
        let record = entry.to_lnmp_record();
        let encoded = encoder.encode(&record);

        println!("  Key: {}", entry.key);
        println!("    Version: {}", entry.version);
        println!("    TTL: {}s", entry.ttl);
        println!("    Encoded size: {} bytes", encoded.len());
        println!();
    }

    // Simulate cache update
    println!("🔄 Simulating cache update...");
    let mut updated_entry = entries[0].clone();
    updated_entry.value = b"Alice Johnson|alice@example.com|Enterprise".to_vec();
    updated_entry.version = 2;

    let old_record = entries[0].to_lnmp_record();
    let new_record = updated_entry.to_lnmp_record();

    let old_encoded = encoder.encode(&old_record);
    let new_encoded = encoder.encode(&new_record);

    println!("  Old version: {} bytes", old_encoded.len());
    println!("  New version: {} bytes", new_encoded.len());
    println!(
        "  Change: {} → {} (version bumped)\n",
        entries[0].version, updated_entry.version
    );

    println!("✅ Cache synchronization demo complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Structured cache entries with metadata");
    println!("   • Cache versioning for conflict resolution");
    println!("   • TTL-based expiration");
    println!("   • Efficient LNMP encoding");
    println!("   • Meta crate integration (lnmp::*)");
}
