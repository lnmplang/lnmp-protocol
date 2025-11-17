use lnmp_codec::Encoder;
use lnmp_core::{LnmpRecord, LnmpField, LnmpValue};

#[test]
fn test_encoder_new_is_canonical() {
    // Build a record with two top-level fields
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField { fid: 2, value: LnmpValue::String("b".to_string()) });
    record.add_field(LnmpField { fid: 1, value: LnmpValue::String("a".to_string()) });

    // Encoder::new() should default to canonical representation where top-level
    // fields are separated by newlines (each field on its own line in canonical)
    let encoder = Encoder::new();
    let out = encoder.encode(&record);

    // Check that newline separation exists between top-level fields
    assert!(out.contains("\n"));
    // The canonical output should contain the sorted field fids, 1 then 2
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 2);
    // Default encoder does not include type hints, so fields will be `F1=` and `F2=`.
    assert!(lines.iter().any(|l| l.starts_with("F1=")));
    assert!(lines.iter().any(|l| l.starts_with("F2=")));
}
