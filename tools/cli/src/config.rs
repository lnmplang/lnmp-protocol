use std::env;
use std::path::PathBuf;

/// Global CLI configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Log level (error, warn, info, debug, trace)
    pub log_level: LogLevel,

    /// Default output format for conversions
    pub default_format: OutputFormat,

    /// Verbose mode (detailed output)
    pub verbose: bool,

    /// Quiet mode (minimal output)
    pub quiet: bool,

    /// Color output
    pub color: ColorMode,

    /// Default validation profile
    pub validation_profile: ValidationProfile,

    /// Config file path (optional)
    pub config_file: Option<PathBuf>,
}

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Binary,
    Compact,
}

/// Color output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Validation profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationProfile {
    Loose,
    Standard,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorTheme {
    #[default]
    Dark, // Bright colors on dark background (default)
    Light,   // Dark colors on light background
    Minimal, // Minimal colors, mostly monochrome
}

impl ColorTheme {
    pub fn from_env() -> Self {
        match std::env::var("LNMP_THEME")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "dark" => Self::Dark,
            "light" => Self::Light,
            "minimal" => Self::Minimal,
            _ => Self::Dark,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            default_format: OutputFormat::Text,
            verbose: false,
            quiet: false,
            color: ColorMode::Auto,
            validation_profile: ValidationProfile::Standard,
            config_file: None,
        }
    }
}

impl Config {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Config::default();

        // LNMP_LOG_LEVEL
        if let Ok(level) = env::var("LNMP_LOG_LEVEL") {
            config.log_level = LogLevel::from_str(&level);
        }

        // LNMP_FORMAT
        if let Ok(format) = env::var("LNMP_FORMAT") {
            config.default_format = OutputFormat::from_str(&format);
        }

        // LNMP_VERBOSE
        if env::var("LNMP_VERBOSE").is_ok() {
            config.verbose = true;
        }

        // LNMP_QUIET
        if env::var("LNMP_QUIET").is_ok() {
            config.quiet = true;
        }

        // LNMP_COLOR
        if let Ok(color) = env::var("LNMP_COLOR") {
            config.color = ColorMode::from_str(&color);
        }

        // NO_COLOR (standard env var)
        if env::var("NO_COLOR").is_ok() {
            config.color = ColorMode::Never;
        }

        // LNMP_PROFILE
        if let Ok(profile) = env::var("LNMP_PROFILE") {
            config.validation_profile = ValidationProfile::from_str(&profile);
        }

        // LNMP_CONFIG
        if let Ok(path) = env::var("LNMP_CONFIG") {
            config.config_file = Some(PathBuf::from(path));
        }

        config
    }

    /// Check if verbose output should be shown
    pub fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }

    /// Check if quiet mode is active
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Check if colors should be used
    pub fn use_colors(&self) -> bool {
        match self.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                // Auto-detect based on terminal
                atty::is(atty::Stream::Stdout)
            }
        }
    }

    pub fn theme(&self) -> ColorTheme {
        // Return a default theme or one based on config if added later
        ColorTheme::default()
    }
}

impl LogLevel {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" | "err" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }
}

impl OutputFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "text" | "txt" => OutputFormat::Text,
            "json" => OutputFormat::Json,
            "binary" | "bin" => OutputFormat::Binary,
            "compact" => OutputFormat::Compact,
            _ => OutputFormat::Text,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
            OutputFormat::Binary => "binary",
            OutputFormat::Compact => "compact",
        }
    }
}

impl ColorMode {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "always" | "yes" | "true" => ColorMode::Always,
            "never" | "no" | "false" => ColorMode::Never,
            "auto" => ColorMode::Auto,
            _ => ColorMode::Auto,
        }
    }
}

impl ValidationProfile {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "loose" => ValidationProfile::Loose,
            "standard" | "normal" => ValidationProfile::Standard,
            "strict" => ValidationProfile::Strict,
            _ => ValidationProfile::Standard,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationProfile::Loose => "loose",
            ValidationProfile::Standard => "standard",
            ValidationProfile::Strict => "strict",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.default_format, OutputFormat::Text);
        assert!(!config.verbose);
        assert!(!config.quiet);
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("invalid"), LogLevel::Info);
    }

    #[test]
    fn test_output_format_parsing() {
        assert_eq!(OutputFormat::from_str("json"), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("TEXT"), OutputFormat::Text);
    }
}
