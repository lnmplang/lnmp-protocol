//! Example demonstrating strict vs loose parsing modes in LNMP v0.2

use lnmp_codec::{Parser, ParsingMode};

fn main() {
    println!("=== LNMP v0.2 Strict vs Loose Parsing ===\n");

    // Example 1: Canonical format (works in both modes)
    println!("1. Canonical format (sorted, newlines):");
    let canonical = "F7=1\nF12=14532\nF23=[admin,dev]";
    println!("Input: {:?}\n", canonical);

    println!("Loose mode:");
    match Parser::new(canonical) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields", r.fields().len()),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("Strict mode:");
    match Parser::with_mode(canonical, ParsingMode::Strict) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields\n", r.fields().len()),
            Err(e) => println!("✗ Error: {}\n", e),
        },
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 2: Unsorted fields (loose accepts, strict rejects)
    println!("2. Unsorted fields:");
    let unsorted = "F23=[admin,dev]\nF7=1\nF12=14532";
    println!("Input: {:?}\n", unsorted);

    println!("Loose mode:");
    match Parser::new(unsorted) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields", r.fields().len()),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("Strict mode:");
    match Parser::with_mode(unsorted, ParsingMode::Strict) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields\n", r.fields().len()),
            Err(e) => println!("✗ Error: {}\n", e),
        },
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 3: Semicolons (loose accepts, strict rejects)
    println!("3. Semicolon separators:");
    let semicolons = "F7=1;F12=14532;F23=[admin,dev]";
    println!("Input: {:?}\n", semicolons);

    println!("Loose mode:");
    match Parser::new(semicolons) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields", r.fields().len()),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("Strict mode:");
    match Parser::with_mode(semicolons, ParsingMode::Strict) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields\n", r.fields().len()),
            Err(e) => println!("✗ Error: {}\n", e),
        },
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 4: Comments (loose accepts, strict rejects)
    println!("4. Comments:");
    let with_comments = "# This is a comment\nF1=42\nF2=100";
    println!("Input: {:?}\n", with_comments);

    println!("Loose mode:");
    match Parser::new(with_comments) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields", r.fields().len()),
            Err(e) => println!("✗ Error: {}", e),
        },
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("Strict mode:");
    match Parser::with_mode(with_comments, ParsingMode::Strict) {
        Ok(mut p) => match p.parse_record() {
            Ok(r) => println!("✓ Parsed {} fields\n", r.fields().len()),
            Err(e) => println!("✗ Error: {}\n", e),
        },
        Err(e) => println!("✗ Error: {}\n", e),
    }

    println!("=== Summary ===");
    println!("Loose mode: Accepts format variations (default)");
    println!("Strict mode: Requires canonical format (sorted, newlines, no comments)");
}
