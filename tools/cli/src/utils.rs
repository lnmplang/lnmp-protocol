use anyhow::{Context, Result};

use std::path::Path;

// Compatibility wrappers for existing code using anyhow::Result
/// Read file to bytes (anyhow::Result wrapper)
pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    crate::io::read_bytes(path).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Read file to string (anyhow::Result wrapper)
pub fn read_text(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    crate::io::read_text(path).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Write bytes to file (anyhow::Result wrapper)
pub fn write_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    let path = path.as_ref();
    crate::io::write_bytes(path, data).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Write string to file (anyhow::Result wrapper)
pub fn write_text(path: impl AsRef<Path>, text: &str) -> Result<()> {
    write_file(path, text.as_bytes())
}

/// Format hex preview with optional max bytes
pub fn hex_preview(data: &[u8], max_bytes: usize) -> String {
    let mut parts = Vec::new();
    for byte in data.iter().take(max_bytes) {
        parts.push(format!("{:02X}", byte));
    }
    if data.len() > max_bytes {
        parts.push("...".to_string());
    }
    parts.join(" ")
}

/// Format full hex dump
pub fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pretty print JSON value
pub fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Parse comma-separated floats
pub fn parse_float_list(input: &str) -> Result<Vec<f32>> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f32>()
                .with_context(|| format!("Failed to parse float: {}", s))
        })
        .collect()
}

/// Parse comma or newline-separated string list
#[allow(dead_code)]
pub fn parse_string_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
