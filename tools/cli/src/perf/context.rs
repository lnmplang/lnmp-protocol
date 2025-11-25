use anyhow::Result;
use serde::Serialize;
use std::time::SystemTime;

/// Context quality metrics
#[derive(Debug, Clone, Serialize)]
pub struct ContextMetrics {
    /// Time since creation (freshness)
    pub age_seconds: f64,

    /// Importance score (0.0 - 1.0)
    pub importance: f64,

    /// Risk score (0.0 - 1.0)
    pub risk: f64,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Number of tokens in context
    pub token_count: usize,
}

impl ContextMetrics {
    pub fn new(age: f64, importance: f64, risk: f64, confidence: f64, tokens: usize) -> Self {
        Self {
            age_seconds: age,
            importance,
            risk,
            confidence,
            token_count: tokens,
        }
    }

    /// Calculate overall quality score (weighted average)
    pub fn quality_score(&self) -> f64 {
        let freshness_score = (1.0 - (self.age_seconds / 3600.0)).max(0.0); // Decay over 1 hour

        (self.importance * 0.4)
            + (self.confidence * 0.3)
            + (freshness_score * 0.2)
            + ((1.0 - self.risk) * 0.1)
    }
}

/// Run context analysis simulation
pub fn analyze_context(samples: usize) -> Result<Vec<ContextMetrics>> {
    let mut metrics = Vec::new();
    let _now = SystemTime::now();

    // Simulate context objects
    for i in 0..samples {
        // Simulate different scenarios
        let (age, importance, risk, confidence) = match i % 4 {
            0 => (5.0, 0.9, 0.1, 0.95),    // High quality, fresh
            1 => (300.0, 0.7, 0.2, 0.85),  // Good, slightly old
            2 => (1800.0, 0.4, 0.4, 0.60), // Medium, old
            _ => (10.0, 0.2, 0.8, 0.40),   // Low quality, risky
        };

        metrics.push(ContextMetrics::new(
            age,
            importance,
            risk,
            confidence,
            128 + (i * 10) % 512, // Random token count
        ));
    }

    Ok(metrics)
}

/// Print context analysis report
pub fn print_context_report(metrics: &[ContextMetrics]) {
    println!("\n🧠 Context Quality Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Analyzed {} context objects\n", metrics.len());

    let avg_quality: f64 =
        metrics.iter().map(|m| m.quality_score()).sum::<f64>() / metrics.len() as f64;
    let avg_importance: f64 =
        metrics.iter().map(|m| m.importance).sum::<f64>() / metrics.len() as f64;
    let avg_risk: f64 = metrics.iter().map(|m| m.risk).sum::<f64>() / metrics.len() as f64;
    let avg_confidence: f64 =
        metrics.iter().map(|m| m.confidence).sum::<f64>() / metrics.len() as f64;

    println!("Average Metrics:");
    println!("  Quality Score:  {:.2}/1.0", avg_quality);
    println!("  Importance:     {:.2}", avg_importance);
    println!("  Risk:           {:.2}", avg_risk);
    println!("  Confidence:     {:.2}", avg_confidence);

    println!("\nDistribution:");
    let high_quality = metrics.iter().filter(|m| m.quality_score() > 0.8).count();
    let medium_quality = metrics
        .iter()
        .filter(|m| m.quality_score() > 0.5 && m.quality_score() <= 0.8)
        .count();
    let low_quality = metrics.iter().filter(|m| m.quality_score() <= 0.5).count();

    println!(
        "  High Quality:   {} ({:.1}%)",
        high_quality,
        (high_quality as f64 / metrics.len() as f64) * 100.0
    );
    println!(
        "  Medium Quality: {} ({:.1}%)",
        medium_quality,
        (medium_quality as f64 / metrics.len() as f64) * 100.0
    );
    println!(
        "  Low Quality:    {} ({:.1}%)",
        low_quality,
        (low_quality as f64 / metrics.len() as f64) * 100.0
    );
}
