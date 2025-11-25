use crate::perf::metrics::BenchmarkMetrics;
use anyhow::Result;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    pub timestamp: String,
    pub benchmark_type: String,
    pub iterations: usize,
    pub metrics: BenchmarkMetrics,
    pub metadata: std::collections::HashMap<String, String>,
}

impl BenchmarkResult {
    pub fn new(
        benchmark_type: &str,
        iterations: usize,
        metrics: BenchmarkMetrics,
        metadata: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            timestamp: chrono::Local::now().to_rfc3339(),
            benchmark_type: benchmark_type.to_string(),
            iterations,
            metrics,
            metadata,
        }
    }

    pub fn save(&self, path: &Path, format: ExportFormat) -> Result<()> {
        let mut file = File::create(path)?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(self)?;
                file.write_all(json.as_bytes())?;
            }
            ExportFormat::Csv => {
                // Simple CSV format: timestamp,type,ops_per_sec,latency_ns,throughput
                writeln!(
                    file,
                    "timestamp,type,ops_per_sec,latency_ns,throughput_bytes_sec"
                )?;
                writeln!(
                    file,
                    "{},{},{},{},{}",
                    self.timestamp,
                    self.benchmark_type,
                    self.metrics.ops_per_sec,
                    self.metrics.latency_per_op.as_nanos(),
                    self.metrics.throughput_bytes_sec
                )?;
            }
        }

        println!("📝 Results exported to {}", path.display());
        Ok(())
    }
}
