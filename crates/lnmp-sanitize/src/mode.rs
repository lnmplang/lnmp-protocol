#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanitizationLevel {
    /// Only whitespace normalization and basic trimming
    Minimal,
    /// Quote/escape repair and whitespace normalization
    Normal,
    /// Aggressive best-effort repair for LLM outputs
    Aggressive,
}
