use lnmp_codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp_codec::{Encoder, EncoderConfig, Parser};
use lnmp_core::{LnmpRecord, LnmpValue};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let workspace_root = workspace_root();
    let text_dir = workspace_root.join("spec/examples/text");
    let binary_dir = workspace_root.join("spec/examples/binary");
    let container_dir = workspace_root.join("spec/examples/container");

    let mut failures = Vec::new();
    let mut text_count = 0usize;
    let mut binary_count = 0usize;
    let mut container_count = 0usize;

    if text_dir.exists() {
        for entry in fs::read_dir(&text_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("lnmp") {
                continue;
            }
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains(".canonical"))
                .unwrap_or(false)
            {
                continue;
            }
            text_count += 1;
            if let Err(err) = verify_text_fixture(&path) {
                failures.push(format!("{}: {}", path.display(), err));
            }
        }
    }

    if binary_dir.exists() {
        for entry in fs::read_dir(&binary_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("hex") {
                continue;
            }
            binary_count += 1;
            if let Err(err) = verify_binary_fixture(&path, &text_dir) {
                failures.push(format!("{}: {}", path.display(), err));
            }
        }
    }

    if container_dir.exists() {
        for entry in fs::read_dir(&container_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("hex") {
                continue;
            }
            container_count += 1;
            if let Err(err) = verify_container_fixture(&path, &container_dir) {
                failures.push(format!("{}: {}", path.display(), err));
            }
        }
    }

    if failures.is_empty() {
        println!(
            "Verified {} text fixture(s), {} binary fixture(s), and {} container fixture(s).",
            text_count, binary_count, container_count
        );
        Ok(())
    } else {
        eprintln!("Fixture verification failed:");
        for failure in &failures {
            eprintln!("  - {}", failure);
        }
        Err("fixture verification failed".into())
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn verify_text_fixture(path: &Path) -> Result<(), Box<dyn Error>> {
    let canonical_path = canonical_partner(path)?;
    if !canonical_path.exists() {
        return Err(format!("canonical fixture missing: {}", canonical_path.display()).into());
    }

    let input_text = load_fixture_text(path)?;
    let expected_canonical = load_fixture_text(&canonical_path)?;
    if expected_canonical.is_empty() {
        return Err("canonical text is empty".into());
    }

    let (encoder, requires_annotations) = encoder_for(&expected_canonical);
    let mut parser = Parser::new(&input_text)?;
    let record = parser.parse_record()?;
    let supports_binary = record_supports_binary(&record);
    let canonical_from_input = encoder.encode(&record);
    assert_text_eq(
        &canonical_from_input,
        &expected_canonical,
        "Canonical text mismatch",
    )?;

    // Ensure canonical text is stable when re-canonicalized
    let mut parser = Parser::new(&expected_canonical)?;
    let record = parser.parse_record()?;
    let canonical_again = encoder.encode(&record);
    assert_text_eq(
        &canonical_again,
        &expected_canonical,
        "Canonical text is not idempotent",
    )?;

    // Text -> Binary -> Text round-trip
    if supports_binary && !requires_annotations {
        let bin_encoder = BinaryEncoder::new();
        let binary = bin_encoder.encode_text(&input_text)?;
        let decoder = BinaryDecoder::new();
        let text_from_binary = decoder.decode_to_text(&binary)?;
        assert_text_eq(
            &text_from_binary,
            &expected_canonical,
            "Binary round-trip mismatch",
        )?;

        let binary_from_canonical = bin_encoder.encode_text(&expected_canonical)?;
        if binary != binary_from_canonical {
            return Err("Binary output differs between input and canonical text".into());
        }
    }

    Ok(())
}

fn verify_binary_fixture(path: &Path, text_dir: &Path) -> Result<(), Box<dyn Error>> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("binary fixture missing valid name")?;
    let canonical_text_path = text_dir.join(format!("{stem}.canonical.lnmp"));
    if !canonical_text_path.exists() {
        return Err(format!("canonical text fixture missing for binary base '{}'", stem).into());
    }

    let canonical_text = load_fixture_text(&canonical_text_path)?;
    let expected_hex = load_binary_hex(path)?;
    if expected_hex.is_empty() {
        return Err("binary fixture is empty".into());
    }

    let bin_encoder = BinaryEncoder::new();
    let actual_binary = bin_encoder.encode_text(&canonical_text)?;
    let actual_hex = hex::encode(&actual_binary);
    if actual_hex != expected_hex {
        return Err(format!(
            "binary encoding mismatch (expected {}, got {})",
            expected_hex, actual_hex
        )
        .into());
    }

    let decoder = BinaryDecoder::new();
    let decoded_text = decoder.decode_to_text(&actual_binary)?;
    assert_text_eq(
        &decoded_text,
        &canonical_text,
        "Decoding binary fixture did not yield canonical text",
    )?;

    Ok(())
}

fn load_fixture_text(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let filtered: String = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(filtered.trim().to_string())
}

fn load_binary_hex(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let filtered: String = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<String>();
    let normalized: String = filtered
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    Ok(normalized)
}

fn load_hex_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = load_binary_hex(path)?;
    hex_to_bytes(&hex)
}

fn canonical_partner(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("fixture has no valid file stem")?;
    Ok(path.with_file_name(format!("{stem}.canonical.lnmp")))
}

fn assert_text_eq(actual: &str, expected: &str, context: &str) -> Result<(), Box<dyn Error>> {
    let normalized_actual = normalize_text(actual);
    let normalized_expected = normalize_text(expected);
    if normalized_actual == normalized_expected {
        Ok(())
    } else {
        Err(format!(
            "{}: expected {:?} but got {:?}",
            context, normalized_expected, normalized_actual
        )
        .into())
    }
}

fn normalize_text(input: &str) -> String {
    input.replace("\r\n", "\n").trim().to_string()
}

fn encoder_for(expected_canonical: &str) -> (Encoder, bool) {
    let mut needs_type_hints = false;
    let mut needs_checksums = false;

    for line in expected_canonical.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            if trimmed[..eq_pos].contains(':') {
                needs_type_hints = true;
            }
        }
        if trimmed.contains('#') {
            needs_checksums = true;
        }
    }

    let mut config = EncoderConfig::new();
    if needs_type_hints {
        config = config.with_type_hints(true);
    }
    if needs_checksums {
        config = config.with_checksums(true);
    }
    let encoder = Encoder::with_config(config);
    (encoder, needs_type_hints || needs_checksums)
}

fn record_supports_binary(record: &LnmpRecord) -> bool {
    record
        .fields()
        .iter()
        .all(|field| value_supports_binary(&field.value))
}

fn value_supports_binary(value: &LnmpValue) -> bool {
    !matches!(
        value,
        LnmpValue::NestedRecord(_) | LnmpValue::NestedArray(_)
    )
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if !hex.len().is_multiple_of(2) {
        return Err("hex string has odd length".into());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn verify_container_fixture(path: &Path, manifest_dir: &Path) -> Result<(), Box<dyn Error>> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("container fixture has no valid stem")?;
    let manifest_path = manifest_dir.join(format!("{stem}.yaml"));
    if !manifest_path.exists() {
        return Err(format!("container manifest missing: {}", manifest_path.display()).into());
    }
    let manifest_str = fs::read_to_string(&manifest_path)?;
    let manifest: ContainerManifest = serde_yaml::from_str(&manifest_str)?;

    let bytes = load_hex_bytes(path)?;
    match validate_container(&bytes, &manifest) {
        Ok(_) => {
            if let Some(expected) = &manifest.expect_error {
                Err(format!(
                    "expected container validation to fail ({}) but it succeeded",
                    expected
                )
                .into())
            } else {
                Ok(())
            }
        }
        Err(err) => {
            if let Some(expected) = &manifest.expect_error {
                if err.to_lowercase().contains(&expected.to_lowercase()) {
                    Ok(())
                } else {
                    Err(format!(
                        "container error mismatch: expected '{}', got '{}'",
                        expected, err
                    )
                    .into())
                }
            } else {
                Err(err.into())
            }
        }
    }
}

fn normalize_hex(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn mode_byte_for(mode: &str) -> Option<u8> {
    match mode.to_ascii_lowercase().as_str() {
        "text" => Some(0x01),
        "binary" => Some(0x02),
        "stream" => Some(0x03),
        "delta" => Some(0x04),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct ContainerManifest {
    mode: String,
    version: Option<u8>,
    flags: u16,
    metadata_length: u32,
    metadata_hex: Option<String>,
    stream: Option<StreamManifest>,
    delta: Option<DeltaManifest>,
    expect_error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamManifest {
    chunk_size: u32,
    checksum_type: u8,
    flags: u8,
}

#[derive(Debug, Deserialize)]
struct DeltaManifest {
    base_snapshot: u64,
    algorithm: u8,
    compression: u8,
}

fn validate_container(bytes: &[u8], manifest: &ContainerManifest) -> Result<(), String> {
    if bytes.len() < 12 {
        return Err("container shorter than 12-byte header".to_string());
    }
    if &bytes[0..4] != b"LNMP" {
        return Err("invalid magic".to_string());
    }

    let version = bytes[4];
    let expected_version = manifest.version.unwrap_or(1);
    if version != expected_version {
        return Err(format!(
            "unsupported version {} (expected {})",
            version, expected_version
        ));
    }

    let mode_byte = bytes[5];
    let expected_mode_byte =
        mode_byte_for(&manifest.mode).ok_or_else(|| format!("unknown mode {}", manifest.mode))?;
    if mode_byte != expected_mode_byte {
        return Err(format!(
            "mode mismatch {:#04X} vs expected {:#04X}",
            mode_byte, expected_mode_byte
        ));
    }

    let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
    if flags != manifest.flags {
        return Err(format!(
            "flags mismatch {:#06X} vs expected {:#06X}",
            flags, manifest.flags
        ));
    }
    if flags & 0xFFFE != 0 {
        return Err("reserved flag bit set".to_string());
    }

    let metadata_len = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if metadata_len != manifest.metadata_length {
        return Err(format!(
            "metadata length mismatch {} vs expected {}",
            metadata_len, manifest.metadata_length
        ));
    }
    if bytes.len() < 12 + metadata_len as usize {
        return Err("metadata length exceeds available bytes".to_string());
    }
    let metadata = &bytes[12..12 + metadata_len as usize];

    match expected_mode_byte {
        0x01 => {
            if metadata_len != 0 {
                return Err("text mode MUST omit metadata".to_string());
            }
        }
        0x02 => {
            if let Some(expected_hex) = manifest.metadata_hex.as_deref() {
                let normalized_expected = normalize_hex(expected_hex);
                let actual_hex = hex::encode(metadata);
                if actual_hex != normalized_expected {
                    return Err(format!(
                        "binary metadata mismatch: {} vs {}",
                        actual_hex, normalized_expected
                    ));
                }
            }
        }
        0x03 => {
            let stream = manifest
                .stream
                .as_ref()
                .ok_or_else(|| "stream metadata missing in manifest".to_string())?;
            if metadata_len != 6 {
                return Err(format!(
                    "stream metadata must be 6 bytes (got {})",
                    metadata_len
                ));
            }
            let chunk_size =
                u32::from_be_bytes([metadata[0], metadata[1], metadata[2], metadata[3]]);
            let checksum_type = metadata[4];
            let stream_flags = metadata[5];
            if chunk_size == 0 {
                return Err("stream chunk_size must be > 0".to_string());
            }
            if chunk_size != stream.chunk_size
                || checksum_type != stream.checksum_type
                || stream_flags != stream.flags
            {
                return Err("stream metadata mismatch".to_string());
            }
        }
        0x04 => {
            let delta = manifest
                .delta
                .as_ref()
                .ok_or_else(|| "delta metadata missing in manifest".to_string())?;
            if metadata_len != 10 {
                return Err(format!(
                    "delta metadata must be 10 bytes (got {})",
                    metadata_len
                ));
            }
            let base_snapshot = u64::from_be_bytes([
                metadata[0],
                metadata[1],
                metadata[2],
                metadata[3],
                metadata[4],
                metadata[5],
                metadata[6],
                metadata[7],
            ]);
            let algorithm = metadata[8];
            let compression = metadata[9];
            if base_snapshot == 0 {
                return Err("delta base_snapshot must be non-zero".to_string());
            }
            if base_snapshot != delta.base_snapshot
                || algorithm != delta.algorithm
                || compression != delta.compression
            {
                return Err("delta metadata mismatch".to_string());
            }
            if algorithm > 0x01 {
                return Err(format!("unknown delta algorithm {:#04X}", algorithm));
            }
            if compression > 0x01 {
                return Err(format!("unknown delta compression {:#04X}", compression));
            }
        }
        other => {
            return Err(format!("unsupported container mode {:#04X}", other));
        }
    }

    Ok(())
}
