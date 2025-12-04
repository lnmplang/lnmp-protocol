//! Tokyo Smart City OS - Initial Test
//!
//! Tests the basic event generation components

use city_pulse::components::{EventType, FieldImportance, SecurityGenerator, TrafficGenerator};

fn main() {
    println!("🏙️  Tokyo Smart City OS - Component Test\n");
    println!("═══════════════════════════════════════════════════════\n");

    // Test event type system
    println!("📋 Event Type System:");
    println!(
        "   Violence FID: {} (Importance: {})",
        EventType::Violence.primary_fid(),
        FieldImportance::get(EventType::Violence.primary_fid())
    );
    println!(
        "   Weapon FID: {} (Importance: {})",
        EventType::WeaponDetected.primary_fid(),
        FieldImportance::get(EventType::WeaponDetected.primary_fid())
    );
    println!(
        "   Red Light FID: {} (Importance: {})",
        EventType::RedLightViolation.primary_fid(),
        FieldImportance::get(EventType::RedLightViolation.primary_fid())
    );
    println!(
        "   Littering FID: {} (Importance: {})\n",
        EventType::Littering.primary_fid(),
        FieldImportance::get(EventType::Littering.primary_fid())
    );

    // Test traffic generator
    println!("🚗 Traffic Generator:");
    let mut traffic_gen = TrafficGenerator::new(1000);
    let traffic_events = traffic_gen.generate_events(10000);
    println!(
        "   Generated {} traffic events from 1000 vehicles",
        traffic_events.len()
    );

    if let Some(sample) = traffic_events.first() {
        println!("   Sample event fields: {}", count_fields(sample));
    }

    // Test security generator
    println!("\n🚨 Security Generator:");
    let mut security_gen = SecurityGenerator::new(500);
    let security_events = security_gen.generate_events(5000);
    println!(
        "   Generated {} security events from 500 cameras",
        security_events.len()
    );
    println!(
        "   Active incidents: {}",
        security_gen.get_active_incidents().len()
    );

    if let Some(sample) = security_events.first() {
        println!("   Sample event fields: {}", count_fields(sample));
    }

    // Combined stats
    println!("\n📊 Combined Statistics:");
    let total_events = traffic_events.len() + security_events.len();
    println!("   Total events generated: {}", total_events);
    println!(
        "   Event rate: ~{}K events/min (simulated)",
        total_events / 1000
    );

    // Show importance distribution
    println!("\n🎯 Importance Score Distribution:");
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for event in traffic_events.iter().chain(security_events.iter()) {
        if event.get_field(2).is_some() {
            // Count by approximate importance
            let importance = estimate_importance(event);
            match importance {
                255 => critical += 1,
                201..=254 => high += 1,
                101..=200 => medium += 1,
                _ => low += 1,
            }
        }
    }

    println!(
        "   Critical (255):     {:6} events ({:5.2}%)",
        critical,
        critical as f32 / total_events as f32 * 100.0
    );
    println!(
        "   High (201-254):     {:6} events ({:5.2}%)",
        high,
        high as f32 / total_events as f32 * 100.0
    );
    println!(
        "   Medium (101-200):   {:6} events ({:5.2}%)",
        medium,
        medium as f32 / total_events as f32 * 100.0
    );
    println!(
        "   Low (0-100):        {:6} events ({:5.2}%)",
        low,
        low as f32 / total_events as f32 * 100.0
    );

    println!("\n✅ Component test complete!");
    println!("═══════════════════════════════════════════════════════\n");
}

fn count_fields(record: &lnmp::prelude::LnmpRecord) -> usize {
    // Count non-empty fields
    let mut count = 0;
    for fid in 0..250u16 {
        if record.get_field(fid).is_some() {
            count += 1;
        }
    }
    count
}

fn estimate_importance(record: &lnmp::prelude::LnmpRecord) -> u8 {
    // Check for critical fields
    if record.get_field(50).is_some() {
        return 255;
    } // violence
    if record.get_field(51).is_some() {
        return 255;
    } // weapon
    if record.get_field(70).is_some() {
        return 255;
    } // earthquake
    if record.get_field(22).is_some() {
        return 255;
    } // accident_risk
    if record.get_field(21).is_some() {
        return 210;
    } // red_light
    if record.get_field(52).is_some() {
        return 240;
    } // theft
    if record.get_field(61).is_some() {
        return 250;
    } // fire
    if record.get_field(20).is_some() {
        return 160;
    } // speeding
    if record.get_field(24).is_some() {
        return 140;
    } // congestion
    if record.get_field(55).is_some() {
        return 190;
    } // suspicious

    100 // default
}
