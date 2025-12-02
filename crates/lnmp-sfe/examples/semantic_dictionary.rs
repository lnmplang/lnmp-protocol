//! Example: Semantic Dictionary Usage
//!
//! Demonstrates semantic dictionary for field ID to name mapping.

use lnmp_sfe::SemanticDictionary;

fn main() {
    println!("=== LNMP Semantic Dictionary Examples ===\n");

    // Example 1: Create and use a dictionary
    println!("Example 1: Basic Dictionary");
    let mut dict = SemanticDictionary::new();

    dict.add_field_name(12, "user_id".to_string());
    dict.add_field_name(7, "is_active".to_string());
    dict.add_field_name(20, "username".to_string());
    dict.add_field_name(23, "roles".to_string());

    dict.add_importance(12, 200); // High importance
    dict.add_importance(7, 150);
    dict.add_importance(20, 180);
    dict.add_importance(23, 220); // Critical importance

    println!(
        "  Field 12: {} (importance: {})",
        dict.get_field_name(12).unwrap_or("unknown"),
        dict.get_importance(12).unwrap_or(0)
    );
    println!(
        "  Field 7:  {} (importance: {})",
        dict.get_field_name(7).unwrap_or("unknown"),
        dict.get_importance(7).unwrap_or(0)
    );
    println!();

    // Example 2: Equivalence mappings
    println!("Example 2: Value Equivalences");
    dict.add_equivalence(7, "yes".to_string(), "1".to_string());
    dict.add_equivalence(7, "true".to_string(), "1".to_string());
    dict.add_equivalence(7, "no".to_string(), "0".to_string());
    dict.add_equivalence(7, "false".to_string(), "0".to_string());

    println!(
        "  'yes' → {}",
        dict.get_equivalence(7, "yes").unwrap_or("no mapping")
    );
    println!(
        "  'true' → {}",
        dict.get_equivalence(7, "true").unwrap_or("no mapping")
    );
    println!(
        "  'no' → {}",
        dict.get_equivalence(7, "no").unwrap_or("no mapping")
    );
    println!();

    // Example 3:Normalized lookup (case-insensitive)
    println!("Example 3: Normalized Lookups");
    println!(
        "  'YES' (normalized) → {}",
        dict.get_equivalence_normalized(7, "YES")
            .unwrap_or("no mapping")
    );
    println!(
        "  ' True ' (normalized) → {}",
        dict.get_equivalence_normalized(7, " True ")
            .unwrap_or("no mapping")
    );
    println!();

    // Example 4: List all fields
    println!("Example 4: All Registered Fields");
    for (fid, name) in dict.field_name_entries() {
        let importance = dict.get_importance(fid).unwrap_or(0);
        println!("  F{}: {} (importance: {})", fid, name, importance);
    }
    println!();

    println!("Field count: {}", dict.field_count());
    println!("Equivalence mappings: {}", dict.equivalence_count());
    println!("Importance definitions: {}", dict.importance_count());
    println!();

    println!("✅ Semantic dictionary examples completed!");
    println!("\n💡 Use Cases:");
    println!("   - Human-readable explain mode");
    println!("   - API documentation generation");
    println!("   - Field validation");
    println!("   - Schema introspection");
}
