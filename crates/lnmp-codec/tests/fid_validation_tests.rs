//! FID Registry Validation Tests
//!
//! Tests for the optional FID validation feature (v0.5.14)

use lnmp_codec::{EncoderConfig, Parser, ParserConfig};
use lnmp_core::registry::{
    embedded_registry, ExpectedType, FidEntry, FidRange, FidRegistry, FidStatus, ValidationMode,
    ValidationResult,
};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use std::sync::Arc;

#[test]
fn test_parser_config_with_registry() {
    let registry = Arc::new(embedded_registry());

    let config = ParserConfig::default()
        .with_fid_registry(registry.clone())
        .with_fid_validation_mode(ValidationMode::Warn);

    assert!(config.fid_registry.is_some());
    assert_eq!(config.fid_validation_mode, ValidationMode::Warn);
}

#[test]
fn test_encoder_config_with_registry() {
    let registry = Arc::new(embedded_registry());

    let config = EncoderConfig::new()
        .with_fid_registry(registry.clone())
        .with_fid_validation_mode(ValidationMode::Error);

    assert!(config.fid_registry.is_some());
    assert_eq!(config.fid_validation_mode, ValidationMode::Error);
}

#[test]
fn test_embedded_registry_basic() {
    let registry = embedded_registry();

    // Check registry has entries
    assert!(!registry.is_empty());
    assert!(!registry.version().is_empty());

    // Check core FIDs
    let entity_id = registry.get(1).unwrap();
    assert_eq!(entity_id.name, "entity_id");
    assert_eq!(entity_id.expected_type, ExpectedType::Int);
    assert_eq!(entity_id.range, FidRange::Core);
    assert_eq!(entity_id.status, FidStatus::Active);

    let user_id = registry.get(12).unwrap();
    assert_eq!(user_id.name, "user_id");
    assert_eq!(user_id.expected_type, ExpectedType::Int);

    let is_active = registry.get(7).unwrap();
    assert_eq!(is_active.name, "is_active");
    assert_eq!(is_active.expected_type, ExpectedType::Bool);

    let name = registry.get(20).unwrap();
    assert_eq!(name.name, "name");
    assert_eq!(name.expected_type, ExpectedType::String);

    // Check standard FIDs (if available)
    if let Some(position) = registry.get(256) {
        assert_eq!(position.name, "position");
        // Range may vary depending on registry version
    }
}

#[test]
fn test_validate_valid_fields() {
    let registry = embedded_registry();

    // Valid Int field
    let field = LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);

    // Valid Bool field
    let field = LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);

    // Valid String field
    let field = LnmpField {
        fid: 20,
        value: LnmpValue::String("Alice".to_string()),
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);
}

#[test]
fn test_validate_type_mismatch() {
    let registry = embedded_registry();

    // F12 expects Int, but we give String
    let field = LnmpField {
        fid: 12,
        value: LnmpValue::String("not_an_int".to_string()),
    };

    match registry.validate_field(&field) {
        ValidationResult::TypeMismatch { fid, expected, .. } => {
            assert_eq!(fid, 12);
            assert_eq!(expected, ExpectedType::Int);
        }
        _ => panic!("Expected TypeMismatch"),
    }

    // F7 expects Bool, but we give Int
    let field = LnmpField {
        fid: 7,
        value: LnmpValue::Int(42),
    };

    match registry.validate_field(&field) {
        ValidationResult::TypeMismatch { fid, expected, .. } => {
            assert_eq!(fid, 7);
            assert_eq!(expected, ExpectedType::Bool);
        }
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_validate_private_range() {
    let registry = embedded_registry();

    // Private range FIDs (32768+) are always valid
    let field = LnmpField {
        fid: 40000,
        value: LnmpValue::String("anything".to_string()),
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);

    let field = LnmpField {
        fid: 65535,
        value: LnmpValue::Float(42.5), // Just a test value
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);
}

#[test]
fn test_validate_record() {
    let registry = embedded_registry();

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(123), // Valid
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true), // Valid
    });
    record.add_field(LnmpField {
        fid: 20,
        value: LnmpValue::String("Alice".to_string()), // Valid
    });

    let results = registry.validate_record(&record);
    assert!(results.is_empty(), "All fields should be valid");

    // Add invalid field
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::String("not_an_int".to_string()), // Invalid!
    });

    let results = registry.validate_record(&record);
    assert_eq!(results.len(), 1, "Should have one invalid field");
}

#[test]
fn test_fid_ranges() {
    assert_eq!(FidRange::from_fid(0), FidRange::Core);
    assert_eq!(FidRange::from_fid(255), FidRange::Core);
    assert_eq!(FidRange::from_fid(256), FidRange::Standard);
    assert_eq!(FidRange::from_fid(16383), FidRange::Standard);
    assert_eq!(FidRange::from_fid(16384), FidRange::Extended);
    assert_eq!(FidRange::from_fid(32767), FidRange::Extended);
    assert_eq!(FidRange::from_fid(32768), FidRange::Private);
    assert_eq!(FidRange::from_fid(65535), FidRange::Private);
}

#[test]
fn test_programmatic_registry() {
    let mut registry = FidRegistry::new();

    registry.add_entry(FidEntry {
        fid: 100,
        name: "test_field".to_string(),
        expected_type: ExpectedType::Int,
        range: FidRange::Core,
        status: FidStatus::Active,
        since: "0.1.0".to_string(),
        description: "Test field".to_string(),
    });

    let entry = registry.get(100).unwrap();
    assert_eq!(entry.name, "test_field");
    assert_eq!(entry.expected_type, ExpectedType::Int);

    // Valid field
    let field = LnmpField {
        fid: 100,
        value: LnmpValue::Int(42),
    };
    assert_eq!(registry.validate_field(&field), ValidationResult::Valid);

    // Invalid field
    let field = LnmpField {
        fid: 100,
        value: LnmpValue::String("not_int".to_string()),
    };
    match registry.validate_field(&field) {
        ValidationResult::TypeMismatch { .. } => {}
        _ => panic!("Expected TypeMismatch"),
    }
}

#[test]
fn test_parser_without_registry() {
    // Verify backward compatibility - parser works without registry
    let input = "F12=hello"; // F12 expects Int, but String is given
    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    // Parse should succeed without registry validation
    assert_eq!(record.fields().len(), 1);
}

#[test]
fn test_parser_with_registry_valid() {
    use lnmp_codec::ParserConfig;
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = ParserConfig::default()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    // Valid input: F12 expects Int, we give Int
    let input = "F12=14532";
    let mut parser = Parser::with_config(input, config).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 1);
}

#[test]
fn test_parser_with_registry_type_mismatch_error() {
    use lnmp_codec::{LnmpError, ParserConfig};
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = ParserConfig::default()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    // Invalid input: F12 expects Int, we give String
    let input = "F12=hello";
    let mut parser = Parser::with_config(input, config).unwrap();
    let result = parser.parse_record();

    assert!(result.is_err());
    match result.unwrap_err() {
        LnmpError::FidValidation { fid, reason, .. } => {
            assert_eq!(fid, 12);
            assert!(reason.contains("type mismatch"));
        }
        e => panic!("Expected FidValidation error, got: {:?}", e),
    }
}

#[test]
fn test_parser_with_registry_warn_mode() {
    use lnmp_codec::ParserConfig;
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = ParserConfig::default()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Warn);

    // Invalid input: F12 expects Int, we give String
    // But in Warn mode, parse should succeed
    let input = "F12=hello";
    let mut parser = Parser::with_config(input, config).unwrap();
    let record = parser.parse_record().unwrap();

    // Parse succeeds despite type mismatch
    assert_eq!(record.fields().len(), 1);
}

#[test]
fn test_parser_with_registry_private_range() {
    use lnmp_codec::ParserConfig;
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = ParserConfig::default()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    // Private range FIDs (32768+) should always succeed
    let input = "F40000=anything";
    let mut parser = Parser::with_config(input, config).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 1);
}

// ==================== Encoder FID Validation Tests ====================

#[test]
fn test_encoder_with_registry_valid() {
    use lnmp_codec::{Encoder, EncoderConfig};
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = EncoderConfig::new()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532), // F12 expects Int ✓
    });
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true), // F7 expects Bool ✓
    });

    let encoder = Encoder::with_config(config);
    let result = encoder.encode_validated(&record);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("F7=1"));
    assert!(output.contains("F12=14532"));
}

#[test]
fn test_encoder_with_registry_type_mismatch() {
    use lnmp_codec::{Encoder, EncoderConfig, LnmpError};
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = EncoderConfig::new()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::String("not_int".to_string()), // F12 expects Int ✗
    });

    let encoder = Encoder::with_config(config);
    let result = encoder.encode_validated(&record);

    assert!(result.is_err());
    match result.unwrap_err() {
        LnmpError::FidValidation { fid, reason, .. } => {
            assert_eq!(fid, 12);
            assert!(reason.contains("type mismatch"));
        }
        e => panic!("Expected FidValidation error, got: {:?}", e),
    }
}

#[test]
fn test_encoder_validate_record() {
    use lnmp_codec::{Encoder, EncoderConfig};
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = EncoderConfig::new()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Error);

    let mut valid_record = LnmpRecord::new();
    valid_record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(123),
    });

    let encoder = Encoder::with_config(config);
    assert!(encoder.validate_record(&valid_record).is_ok());
}

#[test]
fn test_encoder_without_registry_backward_compatible() {
    use lnmp_codec::Encoder;

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::String("wrong_type".to_string()), // F12 expects Int
    });

    // Without registry, encode should work
    let encoder = Encoder::new();
    let output = encoder.encode(&record);
    assert!(output.contains("F12="));
}

#[test]
fn test_encoder_warn_mode() {
    use lnmp_codec::{Encoder, EncoderConfig};
    use std::sync::Arc;

    let registry = Arc::new(embedded_registry());
    let config = EncoderConfig::new()
        .with_fid_registry(registry)
        .with_fid_validation_mode(ValidationMode::Warn);

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::String("wrong_type".to_string()), // F12 expects Int
    });

    let encoder = Encoder::with_config(config);
    // In Warn mode, encode_validated should succeed
    let result = encoder.encode_validated(&record);
    assert!(result.is_ok());
}
