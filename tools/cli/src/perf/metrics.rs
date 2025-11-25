use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance metrics for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Number of operations performed
    pub operations: u64,

    /// Total duration
    pub duration: Duration,

    /// Operations per second
    pub ops_per_sec: f64,

    /// Average latency per operation (microseconds)
    pub avg_latency_us: f64,

    /// Memory used (bytes)
    pub memory_bytes: usize,

    /// Peak memory (bytes)
    pub peak_memory_bytes: usize,

    /// Latency per operation
    pub latency_per_op: Duration,

    /// Throughput in bytes per second
    pub throughput_bytes_sec: f64,
}

impl BenchmarkMetrics {
    pub fn new(operations: u64, duration: Duration, memory_bytes: usize) -> Self {
        let duration_secs = duration.as_secs_f64();
        let ops_per_sec = if duration_secs > 0.0 {
            operations as f64 / duration_secs
        } else {
            0.0
        };

        let avg_latency_us = if operations > 0 {
            (duration.as_micros() as f64) / (operations as f64)
        } else {
            0.0
        };

        Self {
            operations,
            duration,
            ops_per_sec,
            avg_latency_us,
            memory_bytes,
            peak_memory_bytes: memory_bytes,
            latency_per_op: if operations > 0 {
                duration / operations as u32
            } else {
                Duration::default()
            },
            throughput_bytes_sec: if duration_secs > 0.0 {
                (operations as f64 * 100.0) / duration_secs
            } else {
                0.0
            }, // Estimate
        }
    }

    /// Format ops/sec in human-readable form
    pub fn format_ops_per_sec(&self) -> String {
        if self.ops_per_sec >= 1_000_000.0 {
            format!("{:.2}M ops/sec", self.ops_per_sec / 1_000_000.0)
        } else if self.ops_per_sec >= 1_000.0 {
            format!("{:.2}K ops/sec", self.ops_per_sec / 1_000.0)
        } else {
            format!("{:.2} ops/sec", self.ops_per_sec)
        }
    }

    /// Format latency in appropriate unit
    pub fn format_latency(&self) -> String {
        if self.avg_latency_us < 1.0 {
            format!("{:.2} ns", self.avg_latency_us * 1000.0)
        } else if self.avg_latency_us < 1000.0 {
            format!("{:.2} μs", self.avg_latency_us)
        } else {
            format!("{:.2} ms", self.avg_latency_us / 1000.0)
        }
    }
}

/// Comparison result between two metrics
#[derive(Debug)]
#[allow(dead_code)]
pub struct ComparisonResult {
    pub lnmp: BenchmarkMetrics,
    pub baseline: BenchmarkMetrics,
    pub baseline_name: String,
    pub speedup: f64,
    pub memory_ratio: f64,
}

impl ComparisonResult {
    #[allow(dead_code)]
    pub fn new(lnmp: BenchmarkMetrics, baseline: BenchmarkMetrics, baseline_name: String) -> Self {
        let speedup = lnmp.ops_per_sec / baseline.ops_per_sec.max(1.0);
        let memory_ratio = baseline.memory_bytes as f64 / lnmp.memory_bytes.max(1) as f64;

        Self {
            lnmp,
            baseline,
            baseline_name,
            speedup,
            memory_ratio,
        }
    }
}

/// Benchmark timer helper
pub struct BenchmarkTimer {
    start: Instant,
    operations: u64,
}

impl BenchmarkTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            operations: 0,
        }
    }

    pub fn record_op(&mut self) {
        self.operations += 1;
    }

    pub fn finish(self, memory_bytes: usize) -> BenchmarkMetrics {
        let duration = self.start.elapsed();
        BenchmarkMetrics::new(self.operations, duration, memory_bytes)
    }
}

/// Timer for detailed breakdown of operations
pub struct DetailedTimer {
    stages: Vec<(String, std::time::Duration)>,
    current_start: Option<std::time::Instant>,
}

impl DetailedTimer {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            current_start: None,
        }
    }

    pub fn start_stage(&mut self, _name: &str) {
        self.current_start = Some(std::time::Instant::now());
        // If stage name already exists, we might want to aggregate,
        // but for simple sequential benchmarks, appending is fine or we handle it in print.
    }

    pub fn end_stage(&mut self, name: &str) {
        if let Some(start) = self.current_start {
            let duration = start.elapsed();
            self.stages.push((name.to_string(), duration));
        }
    }

    pub fn print_breakdown(&self, iterations: usize) {
        println!("\n📊 Timing Breakdown (Average per iteration):");

        // Aggregate by stage name
        use std::collections::HashMap;
        let mut totals: HashMap<String, std::time::Duration> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();

        for (name, duration) in &self.stages {
            *totals.entry(name.clone()).or_default() += *duration;
            *counts.entry(name.clone()).or_default() += 1;
        }

        // Sort by total duration descending
        let mut sorted_stages: Vec<_> = totals.iter().collect();
        sorted_stages.sort_by(|a, b| b.1.cmp(a.1));

        let total_duration: std::time::Duration = self.stages.iter().map(|(_, d)| *d).sum();

        for (name, total) in sorted_stages {
            let avg = *total / (counts[name] as u32);
            let percent = (total.as_secs_f64() / total_duration.as_secs_f64()) * 100.0;
            println!("  {:<15}: {:>8.2?} ({:>5.1}%)", name, avg, percent);
        }

        println!(
            "  {:<15}: {:>8.2?}",
            "Total",
            total_duration / (iterations as u32)
        );
    }
}
impl Default for BenchmarkTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_metrics() {
        let metrics = BenchmarkMetrics::new(1_000_000, Duration::from_secs(1), 1024);

        assert_eq!(metrics.operations, 1_000_000);
        assert_eq!(metrics.ops_per_sec, 1_000_000.0);
        assert_eq!(metrics.format_ops_per_sec(), "1.00M ops/sec");
    }

    #[test]
    fn test_comparison_result() {
        let lnmp = BenchmarkMetrics::new(2_000_000, Duration::from_secs(1), 512);
        let json = BenchmarkMetrics::new(650_000, Duration::from_secs(1), 1024);

        let comparison = ComparisonResult::new(lnmp, json, "JSON".to_string());

        assert!(comparison.speedup > 3.0);
        assert!(comparison.memory_ratio > 1.9);
    }
    #[test]
    fn test_detailed_timer() {
        let mut timer = DetailedTimer::new();

        timer.start_stage("test");
        std::thread::sleep(Duration::from_millis(10));
        timer.end_stage("test");

        assert_eq!(timer.stages.len(), 1);
        assert_eq!(timer.stages[0].0, "test");
        assert!(timer.stages[0].1 >= Duration::from_millis(10));
    }

    #[test]
    fn test_format_ops_per_sec() {
        let metrics = BenchmarkMetrics::new(1000, Duration::from_secs(1), 0);
        assert_eq!(metrics.format_ops_per_sec(), "1.00K ops/sec");

        let metrics = BenchmarkMetrics::new(1_500_000, Duration::from_secs(1), 0);
        assert_eq!(metrics.format_ops_per_sec(), "1.50M ops/sec");
    }

    #[test]
    fn test_format_latency() {
        let metrics = BenchmarkMetrics::new(1, Duration::from_micros(500), 0);
        assert_eq!(metrics.format_latency(), "500.00 μs");

        let metrics = BenchmarkMetrics::new(1, Duration::from_millis(2), 0);
        assert_eq!(metrics.format_latency(), "2.00 ms");
    }
}
