//! # Zero-Copy vs Owned Decoding Example
//!
//! This example demonstrates when to use:
//! - `decode_view` (Zero-Copy): For high-performance routing, filtering, and inspection.
//! - `decode` (Owned): For business logic, mutation, and long-term storage.
//!
//! Run with: `cargo run -p lnmp-codec --example zero_copy_usage`

use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LNMP Zero-Copy Usage Example ===\n");

    // 1. Prepare Data
    let mut original_record = LnmpRecord::new();
    original_record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(12345),
    });
    original_record.add_field(LnmpField {
        fid: 10,
        value: LnmpValue::String("system_alert".to_string()),
    });
    original_record.add_field(LnmpField {
        fid: 50,
        value: LnmpValue::String("Critical battery level".to_string()),
    });

    let encoder = BinaryEncoder::new();
    let payload = encoder.encode(&original_record)?;
    let decoder = BinaryDecoder::new();

    println!("Payload size: {} bytes", payload.len());

    // ---------------------------------------------------------
    // SCENARIO 1: Routing / Inspection (Zero-Copy)
    // Goal: Check if message is an alert and route it.
    // Requirement: Extreme speed, no memory allocation for strings.
    // ---------------------------------------------------------
    println!("\n[Scenario 1] High-Speed Routing (Zero-Copy)");
    {
        // 'view' borrows from 'payload'. It cannot outlive 'payload'.
        let view = decoder.decode_view(&payload)?;

        // Access fields directly without copying strings
        let fields = view.fields();
        let mut is_alert = false;

        for field in fields {
            // Check for type="system_alert" (FID 10)
            if field.fid == 10 {
                if let lnmp_core::LnmpValueView::String(s) = field.value {
                    println!("  Found Type: {}", s);
                    if s == "system_alert" {
                        is_alert = true;
                    }
                }
            }
        }

        if is_alert {
            println!("  Decision: ROUTE to Admin Queue");
        } else {
            println!("  Decision: ROUTE to General Queue");
        }

        // 'view' is dropped here. Minimal overhead.
    }

    // ---------------------------------------------------------
    // SCENARIO 2: Processing / Storage (Owned)
    // Goal: Modify the record and save it to a database/channel.
    // Requirement: Independent ownership, mutation.
    // ---------------------------------------------------------
    println!("\n[Scenario 2] Processing & Storage (Owned)");
    {
        // Decode into a fully owned struct
        let mut owned_record = decoder.decode(&payload)?;

        // We can drop 'payload' now if we wanted, owned_record survives.
        // drop(payload);

        // Modify record
        println!("  Original: {:?}", owned_record);
        owned_record.add_field(LnmpField {
            fid: 999,
            value: LnmpValue::Bool(true),
        }); // Mark processed

        // This record can now be sent to another thread or stored
        process_record(owned_record);
    }

    Ok(())
}

fn process_record(record: LnmpRecord) {
    println!(
        "  Processed Record (Ready for DB): {} fields",
        record.fields().len()
    );
}
