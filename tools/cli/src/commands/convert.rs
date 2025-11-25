use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::codec::binary::{BinaryDecoder, BinaryEncoder};
use lnmp::codec::container::ContainerFrame;
use lnmp::codec::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::{pretty_json, read_file, read_text, write_file, write_text};

#[derive(Args)]
pub struct ConvertCmd {
    #[command(subcommand)]
    pub command: ConvertSubcommand,
}

#[derive(Subcommand)]
pub enum ConvertSubcommand {
    /// Convert LNMP to JSON
    ToJson {
        /// Input LNMP file
        input: PathBuf,

        /// Output JSON file
        output: PathBuf,

        /// Pretty print
        #[arg(long)]
        pretty: bool,
    },

    /// Convert JSON to LNMP (with field mapping)
    FromJson {
        /// Input JSON file
        input: PathBuf,

        /// Output LNMP file
        output: PathBuf,

        /// Field mapping file (JSON: {"json_key": fid})
        #[arg(long)]
        mapping: Option<PathBuf>,
    },

    /// Convert text to binary
    ToBinary {
        /// Input LNMP text file
        input: PathBuf,

        /// Output binary file
        output: PathBuf,
    },

    /// Convert binary to text
    FromBinary {
        /// Input binary file
        input: PathBuf,

        /// Output text file
        output: PathBuf,
    },

    /// Convert to ShortForm (LLB)
    ToShortform {
        /// Input LNMP text file
        input: PathBuf,

        /// Output shortform file
        output: PathBuf,
    },

    /// Parse ShortForm to LNMP
    FromShortform {
        /// Input shortform file
        input: PathBuf,

        /// Output LNMP text file
        output: PathBuf,
    },
}

impl ConvertCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            ConvertSubcommand::ToJson {
                input,
                output,
                pretty,
            } => to_json(input, output, *pretty),
            ConvertSubcommand::FromJson {
                input,
                output,
                mapping,
            } => from_json(input, output, mapping.as_ref()),
            ConvertSubcommand::ToBinary { input, output } => to_binary(input, output),
            ConvertSubcommand::FromBinary { input, output } => from_binary(input, output),
            ConvertSubcommand::ToShortform { input, output } => to_shortform(input, output),
            ConvertSubcommand::FromShortform { input, output } => from_shortform(input, output),
        }
    }
}

fn to_json(input: &PathBuf, output: &PathBuf, pretty: bool) -> Result<()> {
    // Auto-detect and decode container files
    let text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        // Container file - decode first
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)
            .map_err(|e| anyhow::anyhow!("Failed to parse container: {}", e))?;
        frame
            .decode_to_text()
            .map_err(|e| anyhow::anyhow!("Failed to decode container: {}", e))?
    } else {
        // Plain LNMP text
        read_text(input)?
    };

    let mut parser = Parser::new(&text)?;
    let record = parser.parse_record()?;

    // Convert to JSON
    let mut json_obj = serde_json::Map::new();
    for field in record.fields() {
        let value = match &field.value {
            lnmp::core::LnmpValue::Int(i) => serde_json::json!(i),
            lnmp::core::LnmpValue::Float(f) => serde_json::json!(f),
            lnmp::core::LnmpValue::Bool(b) => serde_json::json!(b),
            lnmp::core::LnmpValue::String(s) => serde_json::json!(s),
            lnmp::core::LnmpValue::StringArray(arr) => serde_json::json!(arr),
            lnmp::core::LnmpValue::IntArray(arr) => serde_json::json!(arr),
            lnmp::core::LnmpValue::FloatArray(arr) => serde_json::json!(arr),
            lnmp::core::LnmpValue::BoolArray(arr) => serde_json::json!(arr),
            _ => serde_json::json!(null),
        };
        json_obj.insert(format!("F{}", field.fid), value);
    }

    let json = serde_json::Value::Object(json_obj);
    let json_str = if pretty {
        pretty_json(&json)
    } else {
        serde_json::to_string(&json)?
    };

    write_text(output, &json_str)?;
    println!("Converted to JSON: {} fields", record.fields().len());

    Ok(())
}

fn from_json(input: &PathBuf, output: &PathBuf, mapping: Option<&PathBuf>) -> Result<()> {
    // Parse JSON
    let json_text = read_text(input)?;
    let json: serde_json::Value = serde_json::from_str(&json_text)?;

    // Load or generate field mapping
    let field_map = if let Some(map_path) = mapping {
        load_json_mapping(map_path)?
    } else {
        generate_auto_mapping(&json)?
    };

    // Convert JSON → LNMP
    let lnmp_text = json_object_to_lnmp(&json, &field_map)?;

    write_text(output, &lnmp_text)?;
    println!(
        "Converted {} JSON keys → {} LNMP fields",
        count_json_keys(&json),
        field_map.len()
    );

    Ok(())
}

fn load_json_mapping(path: &PathBuf) -> Result<HashMap<String, u16>> {
    let text = read_text(path)?;
    let mapping: HashMap<String, u16> = serde_json::from_str(&text)?;

    // Validate FIDs
    for &fid in mapping.values() {
        if fid == 0 {
            anyhow::bail!("Invalid FID in mapping: {} (must be 1-65535)", fid);
        }
    }

    Ok(mapping)
}

fn generate_auto_mapping(json: &serde_json::Value) -> Result<HashMap<String, u16>> {
    let mut mapping = HashMap::new();
    let mut next_fid = 1u32; // Use u32 to allow checking for overflow

    // Only map top-level keys - keep it simple like gRPC/Protobuf
    if let serde_json::Value::Object(map) = json {
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();

        for key in keys {
            if next_fid > 65535 {
                anyhow::bail!("Too many JSON keys (max 65535 fields)");
            }

            mapping.insert(key.clone(), next_fid as u16);
            next_fid += 1;
        }
    }

    Ok(mapping)
}

fn json_object_to_lnmp(json: &serde_json::Value, mapping: &HashMap<String, u16>) -> Result<String> {
    let mut fields = Vec::new();

    if let serde_json::Value::Object(map) = json {
        for (key, value) in map {
            if let Some(&fid) = mapping.get(key) {
                let lnmp_value = json_value_to_lnmp_value(value)?;
                fields.push((fid, lnmp_value));
            }
        }
    } else {
        anyhow::bail!("JSON root must be an object");
    }

    // Sort by FID for canonical form
    fields.sort_by_key(|(fid, _)| *fid);

    let lnmp_lines: Vec<String> = fields
        .into_iter()
        .map(|(fid, value)| format!("F{}={}", fid, value))
        .collect();

    Ok(lnmp_lines.join("\n"))
}

fn json_value_to_lnmp_value(value: &serde_json::Value) -> Result<String> {
    use serde_json::Value;

    match value {
        Value::String(s) => {
            // Quote if contains special chars
            if s.contains(['=', ' ', '#', '@', ':']) || s.is_empty() {
                Ok(format!("\"{}\"", s.replace('"', "\\\"")))
            } else {
                Ok(s.clone())
            }
        }
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::Array(arr) => convert_json_array(arr),
        Value::Object(_) => {
            // For nested objects, serialize as JSON string
            // User should handle complex schemas themselves
            let json_str = serde_json::to_string(value)?;
            Ok(format!("\"{}\"", json_str.replace('"', "\\\"")))
        }
    }
}

fn convert_json_array(arr: &[serde_json::Value]) -> Result<String> {
    if arr.is_empty() {
        return Ok("[]".to_string());
    }

    // Only handle simple homogeneous arrays - keep it simple
    let all_strings = arr.iter().all(|v| v.is_string());
    let all_ints = arr.iter().all(|v| v.is_i64());
    let all_floats = arr.iter().all(|v| v.is_f64() || v.is_i64());
    let all_bools = arr.iter().all(|v| v.is_boolean());

    if all_strings {
        let strings: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        Ok(format!("[{}]", strings.join(",")))
    } else if all_ints {
        let ints: Vec<i64> = arr.iter().filter_map(|v| v.as_i64()).collect();
        Ok(format!(
            "[{}]",
            ints.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ))
    } else if all_floats {
        let floats: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
        Ok(format!(
            "[{}]",
            floats
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ))
    } else if all_bools {
        let bools: Vec<bool> = arr.iter().filter_map(|v| v.as_bool()).collect();
        Ok(format!(
            "[{}]",
            bools
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ))
    } else {
        // Complex/mixed arrays - serialize as JSON string
        let json_str = serde_json::to_string(arr)?;
        Ok(format!("\"{}\"", json_str.replace('"', "\\\"")))
    }
}

fn count_json_keys(json: &serde_json::Value) -> usize {
    if let serde_json::Value::Object(map) = json {
        map.len()
    } else {
        0
    }
}

fn to_binary(input: &PathBuf, output: &PathBuf) -> Result<()> {
    // Auto-detect and decode container files
    let text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)?;
        frame.decode_to_text()?
    } else {
        read_text(input)?
    };

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode_text(&text)?;

    write_file(output, &binary)?;
    println!(
        "Converted to binary: {} bytes (from {} bytes text)",
        binary.len(),
        text.len()
    );

    Ok(())
}

fn from_binary(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let binary = read_file(input)?;
    let decoder = BinaryDecoder::new();
    let text = decoder.decode_to_text(&binary)?;

    write_text(output, &text)?;
    println!(
        "Converted to text: {} bytes (from {} bytes binary)",
        text.len(),
        binary.len()
    );

    Ok(())
}

fn to_shortform(input: &PathBuf, _output: &PathBuf) -> Result<()> {
    // Auto-detect container files (for when shortform is implemented)
    let _text = if input.extension().and_then(|s| s.to_str()) == Some("lnmp") {
        let data = read_file(input)?;
        let frame = ContainerFrame::parse(&data)?;
        frame.decode_to_text()?
    } else {
        read_text(input)?
    };

    println!("ShortForm conversion not yet implemented");
    println!("(Would use lnmp-llb for token-optimized output)");
    Ok(())
}

fn from_shortform(_input: &PathBuf, _output: &PathBuf) -> Result<()> {
    println!("ShortForm parsing not yet implemented");
    println!("(Would use lnmp-llb to parse compressed format)");
    Ok(())
}
