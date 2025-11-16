#![allow(clippy::approx_constant)]

use lnmp_codec::Parser;
use lnmp_core::LnmpValue;

#[test]
fn test_spec_example_inline() {
    // Example from spec: inline format with semicolons
    let input = r#"F12=14532;F7=1;F23=["admin","dev"]"#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 3);
    assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(
        record.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

#[test]
fn test_spec_example_multiline() {
    // Example from spec: multiline format
    let input = r#"F12=14532
F7=1
F20="Halil"
F23=["admin","dev"]"#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 4);
    assert_eq!(record.get_field(12).unwrap().value, LnmpValue::Int(14532));
    assert_eq!(record.get_field(7).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(
        record.get_field(20).unwrap().value,
        LnmpValue::String("Halil".to_string())
    );
    assert_eq!(
        record.get_field(23).unwrap().value,
        LnmpValue::StringArray(vec!["admin".to_string(), "dev".to_string()])
    );
}

#[test]
fn test_spec_integer_values() {
    let input = "F1=42;F2=-123;F3=0";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Int(-123));
    assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Bool(false));
}

#[test]
fn test_spec_float_values() {
    let input = "F1=3.14;F2=-2.5;F3=0.0";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Float(3.14));
    assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Float(-2.5));
    assert_eq!(record.get_field(3).unwrap().value, LnmpValue::Float(0.0));
}

#[test]
fn test_spec_boolean_values() {
    let input = "F1=1;F2=0";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Bool(true));
    assert_eq!(record.get_field(2).unwrap().value, LnmpValue::Bool(false));
}

#[test]
fn test_spec_string_values() {
    let input = r#"F1="hello world";F2=simple_string;F3="with \"quotes\"""#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(
        record.get_field(1).unwrap().value,
        LnmpValue::String("hello world".to_string())
    );
    assert_eq!(
        record.get_field(2).unwrap().value,
        LnmpValue::String("simple_string".to_string())
    );
    assert_eq!(
        record.get_field(3).unwrap().value,
        LnmpValue::String("with \"quotes\"".to_string())
    );
}

#[test]
fn test_spec_string_array_values() {
    let input = r#"F1=["a","b","c"];F2=[simple,strings];F3=[]"#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(
        record.get_field(1).unwrap().value,
        LnmpValue::StringArray(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
    assert_eq!(
        record.get_field(2).unwrap().value,
        LnmpValue::StringArray(vec!["simple".to_string(), "strings".to_string()])
    );
    assert_eq!(
        record.get_field(3).unwrap().value,
        LnmpValue::StringArray(vec![])
    );
}

#[test]
fn test_spec_escape_sequences() {
    let input = r#"F1="line1\nline2";F2="tab\there";F3="back\\slash";F4="quote\"here""#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(
        record.get_field(1).unwrap().value,
        LnmpValue::String("line1\nline2".to_string())
    );
    assert_eq!(
        record.get_field(2).unwrap().value,
        LnmpValue::String("tab\there".to_string())
    );
    assert_eq!(
        record.get_field(3).unwrap().value,
        LnmpValue::String("back\\slash".to_string())
    );
    assert_eq!(
        record.get_field(4).unwrap().value,
        LnmpValue::String("quote\"here".to_string())
    );
}

#[test]
fn test_spec_comments() {
    let input = r#"# This is a comment
F1=42
# Another comment
F2=test"#;

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 2);
    assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        record.get_field(2).unwrap().value,
        LnmpValue::String("test".to_string())
    );
}

#[test]
fn test_spec_whitespace_handling() {
    let input = "F1  =  42  ;  F2  =  test";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(record.fields().len(), 2);
    assert_eq!(record.get_field(1).unwrap().value, LnmpValue::Int(42));
    assert_eq!(
        record.get_field(2).unwrap().value,
        LnmpValue::String("test".to_string())
    );
}

#[test]
fn test_spec_field_id_range() {
    // Test minimum and maximum field IDs
    let input = "F0=min;F65535=max";

    let mut parser = Parser::new(input).unwrap();
    let record = parser.parse_record().unwrap();

    assert_eq!(
        record.get_field(0).unwrap().value,
        LnmpValue::String("min".to_string())
    );
    assert_eq!(
        record.get_field(65535).unwrap().value,
        LnmpValue::String("max".to_string())
    );
}

#[test]
fn test_spec_error_invalid_field_id() {
    // Field ID out of range
    let input = "F99999=test";

    let result = Parser::new(input);
    assert!(result.is_ok());

    let mut parser = result.unwrap();
    let result = parser.parse_record();
    assert!(result.is_err());
}

#[test]
fn test_spec_error_unterminated_string() {
    let input = r#"F1="unterminated"#;

    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse_record();
    assert!(result.is_err());
}

#[test]
fn test_spec_error_invalid_escape() {
    let input = r#"F1="invalid\xescape""#;

    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse_record();
    assert!(result.is_err());
}
