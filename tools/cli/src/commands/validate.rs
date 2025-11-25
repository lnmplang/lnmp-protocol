use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::codec::container::ContainerFrame;
use lnmp::codec::Parser;
use lnmp::core::LnmpProfile;
use std::path::PathBuf;

use crate::utils::{read_file, read_text, write_text};

#[derive(Args)]
pub struct ValidateCmd {
    #[command(subcommand)]
    pub command: ValidateSubcommand,
}

#[derive(Subcommand)]
pub enum ValidateSubcommand {
    /// Sanitize input for security
    Sanitize {
        /// Input LNMP file
        input: PathBuf,

        /// Output sanitized file
        output: PathBuf,
    },

    /// Validate record structure
    Check {
        /// Input LNMP file
        input: PathBuf,

        /// Validation profile (loose/standard/strict)
        #[arg(long, default_value = "standard")]
        profile: String,
    },

    /// Strict validation with canonical ordering
    Strict {
        /// Input LNMP file
        input: PathBuf,
    },

    /// Compliance check against spec
    Compliance {
        /// Input LNMP file
        input: PathBuf,
    },
}

impl ValidateCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            ValidateSubcommand::Sanitize { input, output } => sanitize(input, output),
            ValidateSubcommand::Check { input, profile } => check(input, profile),
            ValidateSubcommand::Strict { input } => strict(input),
            ValidateSubcommand::Compliance { input } => compliance(input),
        }
    }
}

fn sanitize(input: &PathBuf, output: &PathBuf) -> Result<()> {
    // Auto-detect and decode container files
    let text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)?;
        frame.decode_to_text()?
    } else {
        read_text(input)?
    };

    // Pre-filter lines (lenient mode) - remove invalid field IDs
    let sanitized_lines: Vec<String> = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            
            // Keep empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return Some(line.to_string());
            }
            
            // Check field lines
            if trimmed.starts_with('F') {
                // Extract field ID
                if let Some(eq_pos) = trimmed.find('=') {
                    let fid_str = &trimmed[1..eq_pos];
                    if let Ok(fid) = fid_str.parse::<u32>() {
                        if fid >= 1 && fid <= 65535 {
                            // Valid field ID
                            return Some(line.to_string());
                        } else {
                            eprintln!("[sanitize] Skipped invalid field ID: F{}", fid);
                            return None;
                        }
                    }
                }
                eprintln!("[sanitize] Skipped malformed line: {}", trimmed);
                return None;
            }
            
            // Keep other lines
            Some(line.to_string())
        })
        .collect();
    
    let sanitized = sanitized_lines.join("\n");

    // Validate after sanitization
    let mut parser = Parser::new(&sanitized)?;
    let record = parser.parse_record()?;

    write_text(output, &sanitized)?;
    println!("✓ Sanitized and validated");
    println!("  Input fields: {}", text.lines().filter(|l| l.trim().starts_with('F')).count());
    println!("  Output fields: {}", record.fields().len());

    Ok(())
}

fn check(input: &PathBuf, profile: &str) -> Result<()> {
    // Auto-detect and decode container files
    let text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)?;
        frame.decode_to_text()?
    } else {
        read_text(input)?
    };

    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    let profile_mode = match profile {
        "loose" => LnmpProfile::Loose,
        "standard" => LnmpProfile::Standard,
        "strict" => LnmpProfile::Strict,
        _ => anyhow::bail!("Invalid profile: {} (use loose/standard/strict)", profile),
    };

    // Check field ordering for strict mode
    if profile_mode == LnmpProfile::Strict {
        let mut prev_fid = 0;
        for field in record.fields() {
            if field.fid <= prev_fid {
                anyhow::bail!(
                    "Validation failed: Field F{} appears after F{} (not in canonical order)",
                    field.fid,
                    prev_fid
                );
            }
            prev_fid = field.fid;
        }
    }

    // Check for duplicate FIDs
    let mut seen_fids = std::collections::HashSet::new();
    for field in record.fields() {
        if !seen_fids.insert(field.fid) {
            anyhow::bail!("Validation failed: Duplicate field ID F{}", field.fid);
        }
    }

    println!("✓ Validation passed ({} profile)", profile);
    println!("  Fields: {}", record.fields().len());
    println!("  No duplicates");
    if profile_mode == LnmpProfile::Strict {
        println!("  Canonical ordering: OK");
    }

    Ok(())
}

fn strict(input: &PathBuf) -> Result<()> {
    check(input, "strict")
}

fn compliance(input: &PathBuf) -> Result<()> {
    // Auto-detect and decode container files
    let text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)?;
        frame.decode_to_text()?
    } else {
        read_text(input)?
    };

    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    println!("LNMP Compliance Check:");
    println!();

    // Check 1: Valid syntax
    println!("✓ Syntax: Valid LNMP format");

    // Check 2: Field IDs in valid range (FID 0 is reserved)
    let invalid_fids: Vec<_> = record
        .fields()
        .iter()
        .filter(|f| f.fid == 0)
        .map(|f| f.fid)
        .collect();

    if invalid_fids.is_empty() {
        println!("✓ Field IDs: All in valid range (1-65535)");
    } else {
        println!("✗ Field IDs: Invalid FIDs found: {:?}", invalid_fids);
    }

    // Check 3: Canonical ordering
    let mut prev_fid = 0;
    let mut ordered = true;
    for field in record.fields() {
        if field.fid <= prev_fid {
            ordered = false;
            break;
        }
        prev_fid = field.fid;
    }

    if ordered {
        println!("✓ Ordering: Canonical (sorted by FID)");
    } else {
        println!("⚠ Ordering: Not canonical (fields not sorted)");
    }

    // Check 4: No duplicates
    let mut seen = std::collections::HashSet::new();
    let mut has_duplicates = false;
    for field in record.fields() {
        if !seen.insert(field.fid) {
            has_duplicates = true;
            break;
        }
    }

    if !has_duplicates {
        println!("✓ Uniqueness: No duplicate FIDs");
    } else {
        println!("✗ Uniqueness: Duplicate FIDs found");
    }

    println!();
    if invalid_fids.is_empty() && ordered && !has_duplicates {
        println!("Result: ✓ FULLY COMPLIANT");
    } else {
        println!("Result: ⚠ PARTIALLY COMPLIANT");
    }

    Ok(())
}
