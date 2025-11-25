// Stability testing module for LLM parsing reliability

/// Stability test result for a single scenario
#[derive(Debug, Clone)]
pub struct StabilityTest {
    pub scenario: String,
    pub lnmp_success: usize,
    pub lnmp_total: usize,
    pub json_success: usize,
    pub json_total: usize,
}

impl StabilityTest {
    pub fn new(scenario: String) -> Self {
        Self {
            scenario,
            lnmp_success: 0,
            lnmp_total: 0,
            json_success: 0,
            json_total: 0,
        }
    }

    pub fn lnmp_rate(&self) -> f64 {
        if self.lnmp_total == 0 {
            0.0
        } else {
            (self.lnmp_success as f64 / self.lnmp_total as f64) * 100.0
        }
    }

    pub fn json_rate(&self) -> f64 {
        if self.json_total == 0 {
            0.0
        } else {
            (self.json_success as f64 / self.json_total as f64) * 100.0
        }
    }

    pub fn difference(&self) -> f64 {
        self.lnmp_rate() - self.json_rate()
    }
}

/// Test scenario generator
pub struct ScenarioGenerator;

impl ScenarioGenerator {
    /// Generate clean, well-formed data
    pub fn clean(iteration: usize) -> (String, String) {
        let lnmp = format!(
            "F1=\"value_{}\"\nF2={}\nF3=\"test_data\"\nF4=3.14",
            iteration,
            iteration * 10
        );

        let json = format!(
            r#"{{"field1":"value_{}","field2":{},"field3":"test_data","field4":3.14}}"#,
            iteration,
            iteration * 10
        );

        (lnmp, json)
    }

    /// Generate data with missing quotes (common LLM error)
    pub fn missing_quotes(iteration: usize) -> (String, String) {
        // LNMP tolerates unquoted strings
        let lnmp = format!(
            "F1=value_{}\nF2={}\nF3=test_data",
            iteration,
            iteration * 10
        );

        // JSON requires quotes
        let json = format!(
            r#"{{"field1":value_{},"field2":{},"field3":test_data}}"#,
            iteration,
            iteration * 10
        );

        (lnmp, json)
    }

    /// Generate data with extra commas (LLM hallucination)
    pub fn extra_commas(iteration: usize) -> (String, String) {
        // LNMP doesn't use commas between fields
        let lnmp = format!(
            "F1=\"value_{}\",\nF2={},\nF3=\"test\",",
            iteration,
            iteration * 10
        );

        // JSON breaks with double commas
        let json = format!(
            r#"{{"field1":"value_{}",,,"field2":{},"field3":"test",}}"#,
            iteration,
            iteration * 10
        );

        (lnmp, json)
    }

    /// Generate truncated output (LLM cuts off mid-stream)
    pub fn truncated(iteration: usize) -> (String, String) {
        // LNMP can partial parse
        let lnmp = format!(
            "F1=\"value_{}\"\nF2={}\nF3=\"incomplete",
            iteration,
            iteration * 10
        );

        // JSON all-or-nothing
        let json = format!(
            r#"{{"field1":"value_{}","field2":{},"field3":"incomplete"#,
            iteration,
            iteration * 10
        );

        (lnmp, json)
    }

    /// Generate mixed encoding (UTF-8 challenges)
    pub fn mixed_encoding(iteration: usize) -> (String, String) {
        let lnmp = format!(
            "F1=\"Hello 世界\"\nF2=\"Emoji 🚀\"\nF3=\"Über {}\"",
            iteration
        );

        let json = format!(
            r#"{{"field1":"Hello 世界","field2":"Emoji 🚀","field3":"Über {}"}}"#,
            iteration
        );

        (lnmp, json)
    }
}

/// Run all stability tests
pub fn run_stability_tests(iterations: usize) -> anyhow::Result<Vec<StabilityTest>> {
    use lnmp::codec::Parser;

    let scenarios = vec![
        (
            "Clean responses",
            ScenarioGenerator::clean as fn(usize) -> (String, String),
        ),
        ("Missing quotes", ScenarioGenerator::missing_quotes),
        ("Extra commas", ScenarioGenerator::extra_commas),
        ("Truncated output", ScenarioGenerator::truncated),
        ("Mixed encoding", ScenarioGenerator::mixed_encoding),
    ];

    let mut results = Vec::new();

    for (name, generator) in scenarios {
        let mut test = StabilityTest::new(name.to_string());

        for i in 0..iterations {
            let (lnmp_data, json_data) = generator(i);

            // Test LNMP parsing
            test.lnmp_total += 1;
            let mut parser = Parser::new(&lnmp_data)?;
            if parser.parse_record().is_ok() {
                test.lnmp_success += 1;
            }

            // Test JSON parsing
            test.json_total += 1;
            if serde_json::from_str::<serde_json::Value>(&json_data).is_ok() {
                test.json_success += 1;
            }
        }

        results.push(test);
    }

    Ok(results)
}

/// Calculate overall success rate
pub fn calculate_overall_rate(tests: &[StabilityTest]) -> (f64, f64) {
    let lnmp_total: usize = tests.iter().map(|t| t.lnmp_total).sum();
    let lnmp_success: usize = tests.iter().map(|t| t.lnmp_success).sum();
    let json_total: usize = tests.iter().map(|t| t.json_total).sum();
    let json_success: usize = tests.iter().map(|t| t.json_success).sum();

    let lnmp_rate = if lnmp_total == 0 {
        0.0
    } else {
        (lnmp_success as f64 / lnmp_total as f64) * 100.0
    };

    let json_rate = if json_total == 0 {
        0.0
    } else {
        (json_success as f64 / json_total as f64) * 100.0
    };

    (lnmp_rate, json_rate)
}

/// Run stability tests and print report
#[allow(dead_code)]
pub fn run_stability(iterations: usize) -> anyhow::Result<()> {
    println!(
        "Running stability tests with {} iterations per scenario...",
        iterations
    );
    let results = run_stability_tests(iterations)?;

    println!("\nStability Test Results:");
    println!(
        "{:<20} | {:<10} | {:<10} | {:<10}",
        "Scenario", "LNMP %", "JSON %", "Diff"
    );
    println!("{:-<20}-+-{:-<10}-+-{:-<10}-+-{:-<10}", "", "", "", "");

    for test in &results {
        println!(
            "{:<20} | {:>9.1}% | {:>9.1}% | {:>+9.1}%",
            test.scenario,
            test.lnmp_rate(),
            test.json_rate(),
            test.difference()
        );
    }

    let (lnmp_rate, json_rate) = calculate_overall_rate(&results);
    println!("{:-<56}", "");
    println!(
        "{:<20} | {:>9.1}% | {:>9.1}% | {:>+9.1}%",
        "OVERALL",
        lnmp_rate,
        json_rate,
        lnmp_rate - json_rate
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_metrics() {
        let mut test = StabilityTest::new("test".to_string());
        test.lnmp_success = 997;
        test.lnmp_total = 1000;
        test.json_success = 503;
        test.json_total = 1000;

        assert!((test.lnmp_rate() - 99.7).abs() < 0.1);
        assert!((test.json_rate() - 50.3).abs() < 0.1);
        assert!(test.difference() > 49.0);
    }

    #[test]
    fn test_scenario_generation() {
        let (lnmp, json) = ScenarioGenerator::clean(0);
        assert!(lnmp.contains("F1="));
        assert!(json.contains("field1"));

        let (lnmp, json) = ScenarioGenerator::missing_quotes(0);
        assert!(lnmp.contains("F1=value"));
        assert!(json.contains("value_"));
    }
}
