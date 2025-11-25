use anyhow::Result;
use clap::{Args, Subcommand};
use lnmp::embedding::{Decoder, Encoder, SimilarityMetric, Vector, VectorDelta};
use std::path::PathBuf;

use crate::utils::{parse_float_list, read_file, read_text, write_file};

#[derive(Args)]
pub struct EmbeddingCmd {
    #[command(subcommand)]
    pub command: EmbeddingSubcommand,
}

#[derive(Subcommand)]
pub enum EmbeddingSubcommand {
    /// Encode vector to LNMP binary format
    Encode {
        /// Vector data (comma-separated floats or @file.txt)
        #[arg(long)]
        vector: String,

        /// Output file
        output: PathBuf,
    },

    /// Decode LNMP binary to vector
    Decode {
        /// Input binary file
        input: PathBuf,
    },

    /// Delta operations
    Delta {
        #[command(subcommand)]
        action: DeltaAction,
    },

    /// Compute similarity between vectors
    Similarity {
        /// First vector file
        vec1: PathBuf,

        /// Second vector file
        vec2: PathBuf,

        /// Similarity metric (cosine/euclidean/dot)
        #[arg(long, default_value = "cosine")]
        metric: String,
    },
}

#[derive(Subcommand)]
pub enum DeltaAction {
    /// Compute delta between two vectors
    Compute {
        /// Base vector file
        base: PathBuf,

        /// Target vector file
        target: PathBuf,

        /// Output delta file
        output: PathBuf,
    },

    /// Apply delta to vector
    Apply {
        /// Base vector file
        base: PathBuf,

        /// Delta file
        delta: PathBuf,

        /// Output result file
        output: PathBuf,
    },
}

impl EmbeddingCmd {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            EmbeddingSubcommand::Encode { vector, output } => encode(vector, output),
            EmbeddingSubcommand::Decode { input } => decode(input),
            EmbeddingSubcommand::Delta { action } => match action {
                DeltaAction::Compute {
                    base,
                    target,
                    output,
                } => delta_compute(base, target, output),
                DeltaAction::Apply {
                    base,
                    delta,
                    output,
                } => delta_apply(base, delta, output),
            },
            EmbeddingSubcommand::Similarity { vec1, vec2, metric } => {
                similarity(vec1, vec2, metric)
            }
        }
    }
}

fn encode(vector_str: &str, output: &PathBuf) -> Result<()> {
    // Parse vector data
    let values = if let Some(path) = vector_str.strip_prefix('@') {
        // Read from file
        let text = read_text(path)?;
        parse_float_list(&text)?
    } else {
        // Parse from command line
        parse_float_list(vector_str)?
    };

    let vector = Vector::from_f32(values);
    let encoded = Encoder::encode(&vector)?;

    write_file(output, &encoded)?;
    println!(
        "Encoded {} dimensions to {} ({} bytes)",
        vector.dim,
        output.display(),
        encoded.len()
    );

    Ok(())
}

fn decode(input: &PathBuf) -> Result<()> {
    let data = read_file(input)?;
    let vector = Decoder::decode(&data)?;

    println!("Dimensions: {}", vector.dim);
    println!("Type: {:?}", vector.dtype);
    println!("Values:");
    for (i, val) in vector.data.iter().enumerate() {
        print!("{:.6}", val);
        if i < vector.data.len() - 1 {
            print!(", ");
        }
        if (i + 1) % 10 == 0 {
            println!();
        }
    }
    println!();

    Ok(())
}

fn delta_compute(base: &PathBuf, target: &PathBuf, output: &PathBuf) -> Result<()> {
    let base_data = read_file(base)?;
    let target_data = read_file(target)?;

    let base_vec = Decoder::decode(&base_data)?;
    let target_vec = Decoder::decode(&target_data)?;

    let delta = VectorDelta::from_vectors(&base_vec, &target_vec, 0)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Serialize delta (simplified - would use proper serialization)
    let delta_bytes = serde_json::to_vec(&delta)?;
    write_file(output, &delta_bytes)?;

    println!(
        "Computed delta: {} changes, written to {}",
        delta.changes.len(),
        output.display()
    );

    Ok(())
}

fn delta_apply(base: &PathBuf, delta: &PathBuf, output: &PathBuf) -> Result<()> {
    let base_data = read_file(base)?;
    let delta_data = read_file(delta)?;

    let base_vec = Decoder::decode(&base_data)?;
    let delta: VectorDelta = serde_json::from_slice(&delta_data)?;

    let result = delta
        .apply(&base_vec)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let encoded = Encoder::encode(&result)?;

    write_file(output, &encoded)?;

    println!(
        "Applied delta: result has {} dimensions, written to {}",
        result.dim,
        output.display()
    );

    Ok(())
}

fn similarity(vec1: &PathBuf, vec2: &PathBuf, metric_str: &str) -> Result<()> {
    let data1 = read_file(vec1)?;
    let data2 = read_file(vec2)?;

    let v1 = Decoder::decode(&data1)?;
    let v2 = Decoder::decode(&data2)?;

    let metric = match metric_str {
        "cosine" => SimilarityMetric::Cosine,
        "euclidean" => SimilarityMetric::Euclidean,
        "dot" => SimilarityMetric::DotProduct,
        _ => anyhow::bail!("Invalid metric: {} (use cosine/euclidean/dot)", metric_str),
    };

    let score = v1
        .similarity(&v2, metric)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Similarity ({:?}): {:.6}", metric, score);

    Ok(())
}
