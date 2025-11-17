//! Configuration types for LNMP parsing and encoding.

use crate::equivalence::EquivalenceMapper;
use crate::normalizer::NormalizationConfig;

/// Parsing mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParsingMode {
    /// Strict mode: only accepts canonical LNMP format
    Strict,
    /// Loose mode: tolerates formatting variations (default)
    #[default]
    Loose,
}

// Default implementation derived via #[derive(Default)] on the enum

/// Parser configuration
#[derive(Debug, Clone, Copy)]
pub struct ParserConfig {
    /// Parsing mode (strict or loose)
    pub mode: ParsingMode,
    /// Whether to validate checksums when present (v0.3 feature)
    pub validate_checksums: bool,
    /// Whether to normalize text values into numeric/boolean types when possible
    pub normalize_values: bool,
    /// Whether to require checksums on all fields (v0.3 feature)
    pub require_checksums: bool,
    /// Optional maximum nesting depth; if None, no limit is enforced
    pub max_nesting_depth: Option<usize>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            mode: ParsingMode::Loose,
            validate_checksums: false,
            normalize_values: true,
            require_checksums: false,
            max_nesting_depth: None,
        }
    }
}

/// Prompt optimization configuration for LLM-optimized encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PromptOptimizationConfig {
    /// Whether to minimize symbols for better tokenization
    pub minimize_symbols: bool,
    /// Whether to align with common tokenizer boundaries
    pub align_token_boundaries: bool,
    /// Whether to optimize array encoding
    pub optimize_arrays: bool,
}

// Default implementation derived via #[derive(Default)] on the struct

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Whether to include type hints in output
    pub include_type_hints: bool,
    /// Whether to use canonical format (always true for v0.2)
    pub canonical: bool,
    /// Whether to append semantic checksums (v0.3 feature)
    pub enable_checksums: bool,
    /// Whether to enable explain mode with inline comments (v0.3 feature)
    pub enable_explain_mode: bool,
    /// Prompt optimization configuration (v0.3 feature)
    pub prompt_optimization: PromptOptimizationConfig,
    /// Value normalization configuration (v0.3 feature)
    pub normalization_config: NormalizationConfig,
    /// Semantic equivalence mapper (v0.3 feature)
    pub equivalence_mapper: Option<EquivalenceMapper>,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            include_type_hints: false,
            canonical: true,
            enable_checksums: false,
            enable_explain_mode: false,
            prompt_optimization: PromptOptimizationConfig::default(),
            normalization_config: NormalizationConfig::default(),
            equivalence_mapper: None,
        }
    }
}

impl EncoderConfig {
    /// Creates a new encoder configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables semantic checksums
    pub fn with_checksums(mut self, enable: bool) -> Self {
        self.enable_checksums = enable;
        self
    }

    /// Enables explain mode with inline comments
    pub fn with_explain_mode(mut self, enable: bool) -> Self {
        self.enable_explain_mode = enable;
        self
    }

    /// Sets prompt optimization configuration
    pub fn with_prompt_optimization(mut self, config: PromptOptimizationConfig) -> Self {
        self.prompt_optimization = config;
        self
    }

    /// Sets value normalization configuration
    pub fn with_normalization(mut self, config: NormalizationConfig) -> Self {
        self.normalization_config = config;
        self
    }

    /// Sets semantic equivalence mapper
    pub fn with_equivalence_mapper(mut self, mapper: EquivalenceMapper) -> Self {
        self.equivalence_mapper = Some(mapper);
        self
    }

    /// Enables type hints in output
    pub fn with_type_hints(mut self, enable: bool) -> Self {
        self.include_type_hints = enable;
        self
    }

    /// Sets canonical format mode
    pub fn with_canonical(mut self, enable: bool) -> Self {
        self.canonical = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_mode_default() {
        assert_eq!(ParsingMode::default(), ParsingMode::Loose);
    }

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert!(!config.include_type_hints);
        assert!(config.canonical);
        assert!(!config.enable_checksums);
        assert!(!config.enable_explain_mode);
        assert!(!config.prompt_optimization.minimize_symbols);
        assert!(!config.prompt_optimization.align_token_boundaries);
        assert!(!config.prompt_optimization.optimize_arrays);
        assert!(config.equivalence_mapper.is_none());
    }

    #[test]
    fn test_parsing_mode_equality() {
        assert_eq!(ParsingMode::Strict, ParsingMode::Strict);
        assert_eq!(ParsingMode::Loose, ParsingMode::Loose);
        assert_ne!(ParsingMode::Strict, ParsingMode::Loose);
    }

    #[test]
    fn test_encoder_config_with_checksums() {
        let config = EncoderConfig::new().with_checksums(true);
        assert!(config.enable_checksums);
    }

    #[test]
    fn test_encoder_config_with_explain_mode() {
        let config = EncoderConfig::new().with_explain_mode(true);
        assert!(config.enable_explain_mode);
    }

    #[test]
    fn test_encoder_config_with_prompt_optimization() {
        let prompt_opt = PromptOptimizationConfig {
            minimize_symbols: true,
            align_token_boundaries: true,
            optimize_arrays: true,
        };
        let config = EncoderConfig::new().with_prompt_optimization(prompt_opt);
        assert!(config.prompt_optimization.minimize_symbols);
        assert!(config.prompt_optimization.align_token_boundaries);
        assert!(config.prompt_optimization.optimize_arrays);
    }

    #[test]
    fn test_encoder_config_with_normalization() {
        use crate::normalizer::StringCaseRule;
        
        let norm_config = NormalizationConfig {
            string_case: StringCaseRule::Lower,
            float_precision: Some(2),
            remove_trailing_zeros: true,
        };
        let config = EncoderConfig::new().with_normalization(norm_config.clone());
        assert_eq!(config.normalization_config.string_case, StringCaseRule::Lower);
        assert_eq!(config.normalization_config.float_precision, Some(2));
        assert!(config.normalization_config.remove_trailing_zeros);
    }

    #[test]
    fn test_encoder_config_with_equivalence_mapper() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(7, "yes".to_string(), "1".to_string());
        
        let config = EncoderConfig::new().with_equivalence_mapper(mapper);
        assert!(config.equivalence_mapper.is_some());
        
        let mapper_ref = config.equivalence_mapper.as_ref().unwrap();
        assert_eq!(mapper_ref.map(7, "yes"), Some("1".to_string()));
    }

    #[test]
    fn test_encoder_config_with_type_hints() {
        let config = EncoderConfig::new().with_type_hints(true);
        assert!(config.include_type_hints);
    }

    #[test]
    fn test_encoder_config_with_canonical() {
        let config = EncoderConfig::new().with_canonical(false);
        assert!(!config.canonical);
    }

    #[test]
    fn test_encoder_config_builder_chain() {
        let mut mapper = EquivalenceMapper::new();
        mapper.add_mapping(7, "yes".to_string(), "1".to_string());
        
        let config = EncoderConfig::new()
            .with_checksums(true)
            .with_explain_mode(true)
            .with_type_hints(true)
            .with_equivalence_mapper(mapper);
        
        assert!(config.enable_checksums);
        assert!(config.enable_explain_mode);
        assert!(config.include_type_hints);
        assert!(config.equivalence_mapper.is_some());
    }

    #[test]
    fn test_prompt_optimization_config_default() {
        let config = PromptOptimizationConfig::default();
        assert!(!config.minimize_symbols);
        assert!(!config.align_token_boundaries);
        assert!(!config.optimize_arrays);
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert_eq!(config.mode, ParsingMode::Loose);
        assert!(!config.validate_checksums);
        assert!(!config.require_checksums);
        assert!(config.max_nesting_depth.is_none());
    }

    #[test]
    fn test_parser_config_with_checksum_validation() {
        let config = ParserConfig {
            mode: ParsingMode::Strict,
            validate_checksums: true,
            normalize_values: false,
            require_checksums: false,
            max_nesting_depth: None,
        };
        assert_eq!(config.mode, ParsingMode::Strict);
        assert!(config.validate_checksums);
        assert!(!config.require_checksums);
    }

    #[test]
    fn test_parser_config_with_required_checksums() {
        let config = ParserConfig {
            mode: ParsingMode::Strict,
            validate_checksums: true,
            normalize_values: false,
            require_checksums: true,
            max_nesting_depth: None,
        };
        assert!(config.validate_checksums);
        assert!(config.require_checksums);
    }
}
