use crate::perf::export::{BenchmarkResult, ExportFormat};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

const HISTORY_DIR: &str = ".lnmp_perf_history";

pub struct HistoryManager {
    base_dir: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(HISTORY_DIR),
        }
    }

    pub fn init(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir(&self.base_dir)?;
        }
        Ok(())
    }

    pub fn save_result(&self, result: &BenchmarkResult) -> Result<()> {
        self.init()?;
        let filename = format!(
            "{}_{}.json",
            result.benchmark_type,
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        let path = self.base_dir.join(filename);
        result.save(&path, ExportFormat::Json)?;
        Ok(())
    }

    pub fn list_results(&self) -> Result<Vec<BenchmarkResult>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(result) = serde_json::from_str::<BenchmarkResult>(&content) {
                    results.push(result);
                }
            }
        }

        // Sort by timestamp descending
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(results)
    }

    pub fn show_trends(&self, benchmark_type: Option<String>) -> Result<()> {
        let results = self.list_results()?;
        let filtered: Vec<&BenchmarkResult> = if let Some(t) = &benchmark_type {
            results.iter().filter(|r| r.benchmark_type == *t).collect()
        } else {
            results.iter().collect()
        };

        if filtered.is_empty() {
            println!("No history found.");
            return Ok(());
        }

        println!("📈 Performance Trends");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Group by type
        let mut by_type: std::collections::HashMap<String, Vec<&BenchmarkResult>> =
            std::collections::HashMap::new();
        for r in filtered {
            by_type.entry(r.benchmark_type.clone()).or_default().push(r);
        }

        for (b_type, mut history) in by_type {
            // Sort ascending for trend line
            history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            println!("\nType: {}", b_type);
            println!(
                "{:<20} | {:<15} | {:<15}",
                "Timestamp", "Ops/Sec", "Latency"
            );
            println!("{:-<20}-+-{:-<15}-+-{:-<15}", "", "", "");

            for r in &history {
                let timestamp = &r.timestamp[0..19].replace("T", " ");
                println!(
                    "{:<20} | {:<15.2} | {:<15}",
                    timestamp,
                    r.metrics.ops_per_sec,
                    r.metrics.format_latency()
                );
            }

            // Calculate trend
            if history.len() >= 2 {
                let first = history.first().unwrap();
                let last = history.last().unwrap();
                let change = (last.metrics.ops_per_sec - first.metrics.ops_per_sec)
                    / first.metrics.ops_per_sec
                    * 100.0;

                let icon = if change > 0.0 {
                    "↗️"
                } else if change < 0.0 {
                    "↘️"
                } else {
                    "➡️"
                };
                println!("\nTrend: {} {:.2}% change since first run", icon, change);
            }
        }

        Ok(())
    }
}
