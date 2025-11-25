use crate::config::{ColorTheme, Config, OutputFormat};
use serde::Serialize;

/// Color codes for terminal output
pub struct Colors {
    pub green: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
    pub red: &'static str,
    pub cyan: &'static str,
    #[allow(dead_code)]
    pub magenta: &'static str,
    #[allow(dead_code)]
    pub white: &'static str,
    #[allow(dead_code)]
    pub black: &'static str,
}

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";

    // Dynamic colors based on theme
    pub fn new(theme: ColorTheme) -> Self {
        match theme {
            ColorTheme::Dark => Self::dark_theme(),
            ColorTheme::Light => Self::light_theme(),
            ColorTheme::Minimal => Self::minimal_theme(),
        }
    }

    fn dark_theme() -> Self {
        Self {
            // Bright colors for dark background
            green: "\x1b[92m",
            yellow: "\x1b[93m",
            blue: "\x1b[94m",
            red: "\x1b[91m",
            cyan: "\x1b[96m",
            magenta: "\x1b[95m",
            white: "\x1b[97m",
            black: "\x1b[90m",
        }
    }

    fn light_theme() -> Self {
        Self {
            // Darker colors for light background
            green: "\x1b[32m",
            yellow: "\x1b[33m",
            blue: "\x1b[34m",
            red: "\x1b[31m",
            cyan: "\x1b[36m",
            magenta: "\x1b[35m",
            white: "\x1b[37m",
            black: "\x1b[30m",
        }
    }

    fn minimal_theme() -> Self {
        Self {
            // Minimal colors (mostly monochrome)
            green: "",
            yellow: "",
            blue: "",
            red: "",
            cyan: "",
            magenta: "",
            white: "",
            black: "",
        }
    }
}

/// Output printer with config-aware formatting
pub struct Printer {
    config: Config,
    colors: Colors,
}

impl Printer {
    pub fn new(config: Config) -> Self {
        let theme = config.theme();
        Self {
            config: config.clone(),
            colors: Colors::new(theme),
        }
    }

    /// Print with optional color
    pub fn print(&self, text: &str) {
        println!("{}", text);
    }

    /// Print success message (green checkmark)
    pub fn success(&self, message: &str) {
        if self.config.use_colors() {
            println!("{}✓{} {}", self.colors.green, Colors::RESET, message);
        } else {
            println!("✓ {}", message);
        }
    }

    /// Print error message (red X)
    pub fn error(&self, message: &str) {
        if self.config.use_colors() {
            eprintln!("{}✗{} {}", self.colors.red, Colors::RESET, message);
        } else {
            eprintln!("✗ {}", message);
        }
    }

    /// Print warning message (yellow triangle)
    pub fn warning(&self, message: &str) {
        if self.config.use_colors() {
            println!("{}⚠{} {}", self.colors.yellow, Colors::RESET, message);
        } else {
            println!("⚠ {}", message);
        }
    }

    /// Print info message (blue info icon)
    pub fn info(&self, message: &str) {
        if self.config.use_colors() {
            println!("{}ℹ{} {}", self.colors.blue, Colors::RESET, message);
        } else {
            println!("ℹ {}", message);
        }
    }

    /// Print verbose message (only if verbose mode)
    pub fn verbose(&self, message: &str) {
        if self.config.is_verbose() {
            if self.config.use_colors() {
                println!("{}[VERBOSE]{} {}", Colors::DIM, Colors::RESET, message);
            } else {
                println!("[VERBOSE] {}", message);
            }
        }
    }

    /// Print debug message (cyan)
    pub fn debug(&self, message: &str) {
        if self.config.use_colors() {
            println!("{}[DEBUG]{} {}", self.colors.cyan, Colors::RESET, message);
        } else {
            println!("[DEBUG] {}", message);
        }
    }

    /// Print with custom color
    pub fn colored(&self, text: &str, color: &str) {
        if self.config.use_colors() {
            println!("{}{}{}", color, text, Colors::RESET);
        } else {
            println!("{}", text);
        }
    }

    /// Print key-value pair
    pub fn key_value(&self, key: &str, value: &str) {
        if self.config.use_colors() {
            println!("  {}{}{}: {}", Colors::BOLD, key, Colors::RESET, value);
        } else {
            println!("  {}: {}", key, value);
        }
    }

    /// Print section header
    pub fn header(&self, title: &str) {
        if self.config.use_colors() {
            println!("\n{}{}{}", Colors::BOLD, title, Colors::RESET);
        } else {
            println!("\n{}", title);
        }
    }
}

/// Format data according to output format
#[allow(dead_code)]
pub fn format_output<T: Serialize>(
    data: &T,
    format: OutputFormat,
) -> Result<String, serde_json::Error> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(data),
        OutputFormat::Compact => serde_json::to_string(data),
        OutputFormat::Text | OutputFormat::Binary => {
            // For text/binary, return JSON representation
            serde_json::to_string_pretty(data)
        }
    }
}

/// Pretty print JSON value
#[allow(dead_code)]
pub fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

/// Format bytes as hex preview
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format hex dump with line numbers and ASCII
#[allow(dead_code)]
pub fn hex_dump_detailed(data: &[u8]) -> String {
    let mut output = String::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        // Line number
        output.push_str(&format!("{:08x}: ", i * 16));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02x} ", byte));
            if j == 7 {
                output.push(' '); // Extra space in middle
            }
        }

        // Padding if last line
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                output.push_str("   ");
                if j == 7 {
                    output.push(' ');
                }
            }
        }

        // ASCII representation
        output.push_str(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        output.push('|');
        output.push('\n');
    }

    output
}

/// Format bytes as human-readable size
#[allow(dead_code)]
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
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

/// Create a table row
#[allow(dead_code)]
pub fn table_row(columns: &[&str], widths: &[usize]) -> String {
    let mut row = String::new();
    for (i, col) in columns.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(20);
        row.push_str(&format!("{:width$}", col, width = width));
        if i < columns.len() - 1 {
            row.push_str(" | ");
        }
    }
    row
}

/// Create a table separator
#[allow(dead_code)]
pub fn table_separator(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|w| "-".repeat(*w))
        .collect::<Vec<_>>()
        .join("-+-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ColorMode;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(100), "100 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_hex_preview() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        assert_eq!(hex_preview(&data, 3), "01 02 03 ...");
        assert_eq!(hex_preview(&data, 10), "01 02 03 04 05");
    }

    #[test]
    fn test_table_row() {
        let row = table_row(&["Name", "Age", "City"], &[10, 5, 10]);
        assert!(row.contains("Name"));
        assert!(row.contains("|"));
    }

    #[test]
    fn test_printer_no_colors() {
        let config = Config {
            color: ColorMode::Never,
            ..Default::default()
        };
        let printer = Printer::new(config);

        // Should not panic
        printer.success("test");
        printer.error("test");
        printer.warning("test");
    }
}
