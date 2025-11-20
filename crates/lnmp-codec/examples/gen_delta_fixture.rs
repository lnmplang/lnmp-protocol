use lnmp_codec::binary::{DeltaConfig, DeltaDecoder, DeltaEncoder};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut base = LnmpRecord::new();
    base.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(1),
    });

    let mut updated = base.clone();
    updated.add_field(LnmpField {
        fid: 2,
        value: LnmpValue::String("added".into()),
    });

    let cfg = DeltaConfig::new().with_enable_delta(true);
    let encoder = DeltaEncoder::with_config(cfg.clone());
    let decoder = DeltaDecoder::with_config(cfg);

    let ops = encoder.compute_delta(&base, &updated).expect("compute delta");
    let encoded = encoder.encode_delta(&ops).expect("encode delta");

    let fixtures_dir = PathBuf::from("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    // valid delta payload (metadata already in file; we overwrite the payload region).
    let valid_path = fixtures_dir.join("valid-delta-base1-oplist.lnmp");
    rewrite_payload(&valid_path, &encoded);

    // base mismatch uses same payload; metadata already declares base=2.
    let mismatch_path = fixtures_dir.join("invalid-delta-base-mismatch.lnmp");
    rewrite_payload(&mismatch_path, &encoded);

    println!("Updated delta payload fixtures.");

    // sanity check: decoded ops round-trip
    let decoded_ops = decoder.decode_delta(&encoded).expect("decode delta");
    assert_eq!(ops.len(), decoded_ops.len());
}

fn rewrite_payload(path: &PathBuf, payload: &[u8]) {
    let mut bytes = fs::read(path).expect("read fixture");
    // header is 12 bytes + 10 bytes metadata = 22 bytes
    const OFFSET: usize = 12 + 10;
    bytes.truncate(OFFSET);
    bytes.extend_from_slice(payload);
    fs::write(path, &bytes).expect("write fixture");
}
