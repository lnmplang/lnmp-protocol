//! CityPulse LLM Integration Demo
//!
//! Demonstrates how LNMP format reduces LLM API costs by using fewer tokens.
//! Compares JSON vs LNMP for the same sensor data analysis task.
//!
//! Run: `cargo run -p city-pulse --bin llm_demo`

use lnmp::prelude::*;

#[derive(Clone)]
struct TrafficSensor {
    id: String,
    speed: f64,
    vehicle_count: i64,
}

impl TrafficSensor {
    fn new(index: usize, speed: f64, count: i64) -> Self {
        Self {
            id: format!("traffic-{:03}", index),
            speed,
            vehicle_count: count,
        }
    }

    fn to_json(&self) -> String {
        serde_json::json!({
            "sensorId": self.id,
            "speed": self.speed,
            "vehicleCount": self.vehicle_count,
        })
        .to_string()
    }

    fn to_lnmp(&self) -> String {
        let mut record = LnmpRecord::new();
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.id.clone()),
        });
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Float(self.speed),
        });
        record.add_field(LnmpField {
            fid: 21,
            value: LnmpValue::Int(self.vehicle_count),
        });

        Encoder::new().encode(&record)
    }
}

/// Approximate token count (4 chars ≈ 1 token for English text)
fn estimate_tokens(text: &str) -> usize {
    // Rough approximation: 1 token ≈ 4 characters
    // This is a simplified model, real tokenizers vary
    (text.len() as f64 / 4.0).ceil() as usize
}

fn build_json_prompt(sensors: &[TrafficSensor]) -> String {
    let mut prompt = String::from("Analyze these traffic sensors and identify congestion:\n\n");

    for sensor in sensors {
        prompt.push_str(&format!("{}\n", sensor.to_json()));
    }

    prompt.push_str("\nIdentify sensors with congestion (speed < 20 km/h and high vehicle count) and suggest actions.");
    prompt
}

fn build_lnmp_prompt(sensors: &[TrafficSensor]) -> String {
    let mut prompt =
        String::from("Analyze these traffic sensors (LNMP format) and identify congestion:\n\n");
    prompt.push_str("Field mapping: F1=sensor_id, F20=speed_kmh, F21=vehicle_count\n\n");

    for sensor in sensors {
        prompt.push_str(&format!("{}\n", sensor.to_lnmp()));
    }

    prompt.push_str("\nIdentify sensors with congestion (speed < 20 km/h and high vehicle count) and suggest actions.");
    prompt
}

fn simulate_llm_response(sensors: &[TrafficSensor]) -> String {
    let mut congested: Vec<&TrafficSensor> = sensors
        .iter()
        .filter(|s| s.speed < 20.0 && s.vehicle_count > 30)
        .collect();

    congested.sort_by(|a, b| a.speed.partial_cmp(&b.speed).unwrap());

    let mut response = String::from("TRAFFIC ANALYSIS RESULTS:\n\n");

    if congested.is_empty() {
        response.push_str("✅ No significant congestion detected.\n");
    } else {
        response.push_str("🚨 CONGESTION DETECTED:\n\n");
        for sensor in &congested {
            let severity = if sensor.speed < 10.0 {
                "CRITICAL"
            } else {
                "WARNING"
            };
            response.push_str(&format!(
                "  {} - {} ({:.1} km/h, {} vehicles)\n",
                sensor.id, severity, sensor.speed, sensor.vehicle_count
            ));
        }

        response.push_str("\nRECOMMENDED ACTIONS:\n");
        response.push_str("1. Activate alternate route signage\n");
        response.push_str("2. Alert traffic management center\n");
        if congested.len() > 3 {
            response.push_str("3. Consider traffic signal timing adjustment\n");
        }
        response.push_str(&format!(
            "4. Dispatch officers to {} priority locations\n",
            congested.len().min(3)
        ));
    }

    response
}

fn main() {
    println!("🤖 CityPulse LLM Integration Demo\n");
    println!("Demonstrating token efficiency with LNMP format\n");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Generate realistic sensor data
    let sensors = vec![
        TrafficSensor::new(1, 45.0, 23),  // Normal
        TrafficSensor::new(2, 55.0, 15),  // Normal
        TrafficSensor::new(3, 8.0, 67),   // CRITICAL congestion
        TrafficSensor::new(4, 42.0, 28),  // Normal
        TrafficSensor::new(5, 15.0, 52),  // WARNING congestion
        TrafficSensor::new(6, 38.0, 31),  // Normal
        TrafficSensor::new(7, 12.0, 58),  // WARNING congestion
        TrafficSensor::new(8, 50.0, 18),  // Normal
        TrafficSensor::new(9, 6.0, 72),   // CRITICAL congestion
        TrafficSensor::new(10, 35.0, 25), // Normal
    ];

    println!(
        "Scenario: {} traffic sensors reporting data\n",
        sensors.len()
    );

    // JSON approach
    println!("📊 Approach 1: JSON Format");
    println!("─────────────────────────────────────────────────────────────\n");

    let json_prompt = build_json_prompt(&sensors);
    let json_tokens = estimate_tokens(&json_prompt);
    let json_cost = (json_tokens as f64) * 0.000003; // $3 per 1M tokens (GPT-4o pricing)

    println!("Prompt preview (first 200 chars):");
    println!("{}\n", &json_prompt.chars().take(200).collect::<String>());
    println!("Total prompt size: {} bytes", json_prompt.len());
    println!("Estimated tokens: {}", json_tokens);
    println!("Estimated cost: ${:.6}\n", json_cost);

    // LNMP approach
    println!("📊 Approach 2: LNMP Format");
    println!("─────────────────────────────────────────────────────────────\n");

    let lnmp_prompt = build_lnmp_prompt(&sensors);
    let lnmp_tokens = estimate_tokens(&lnmp_prompt);
    let lnmp_cost = (lnmp_tokens as f64) * 0.000003;

    println!("Prompt preview (first 200 chars):");
    println!("{}\n", &lnmp_prompt.chars().take(200).collect::<String>());
    println!("Total prompt size: {} bytes", lnmp_prompt.len());
    println!("Estimated tokens: {}", lnmp_tokens);
    println!("Estimated cost: ${:.6}\n", lnmp_cost);

    // Comparison
    let size_reduction =
        ((json_prompt.len() - lnmp_prompt.len()) as f64 / json_prompt.len() as f64) * 100.0;
    let token_reduction = ((json_tokens - lnmp_tokens) as f64 / json_tokens as f64) * 100.0;
    let cost_savings = json_cost - lnmp_cost;

    println!("═══════════════════════════════════════════════════════════════");
    println!("                    COMPARISON RESULTS                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("┌──────────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Metric           │ JSON        │ LNMP        │ Improvement │");
    println!("├──────────────────┼─────────────┼─────────────┼─────────────┤");
    println!(
        "│ Prompt Size      │ {:>7} B   │ {:>7} B   │ {:>9.1}% │",
        json_prompt.len(),
        lnmp_prompt.len(),
        size_reduction
    );
    println!(
        "│ Token Count      │ {:>9}   │ {:>9}   │ {:>9.1}% │",
        json_tokens, lnmp_tokens, token_reduction
    );
    println!(
        "│ Cost (1 query)   │ ${:>9.6} │ ${:>9.6} │ ${:>8.6}  │",
        json_cost, lnmp_cost, cost_savings
    );
    println!("└──────────────────┴─────────────┴─────────────┴─────────────┘\n");

    // Scale analysis
    println!("📈 Cost at Scale:\n");

    let queries_per_day = [100, 1000, 10000];
    for queries in queries_per_day {
        let json_monthly = json_cost * queries as f64 * 30.0;
        let lnmp_monthly = lnmp_cost * queries as f64 * 30.0;
        let monthly_savings = json_monthly - lnmp_monthly;

        println!("  {:>6} queries/day:", queries);
        println!("    JSON:  ${:>8.2}/month", json_monthly);
        println!("    LNMP:  ${:>8.2}/month", lnmp_monthly);
        println!(
            "    SAVED: ${:>8.2}/month (${:>9.2}/year)",
            monthly_savings,
            monthly_savings * 12.0
        );
        println!();
    }

    // Context window analysis
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  CONTEXT WINDOW EFFICIENCY                    ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let context_limit = 8000; // tokens
    let json_sensor_tokens = json_tokens / sensors.len();
    let lnmp_sensor_tokens = lnmp_tokens / sensors.len();

    let json_max_sensors = context_limit / json_sensor_tokens;
    let lnmp_max_sensors = context_limit / lnmp_sensor_tokens;
    let capacity_increase =
        ((lnmp_max_sensors - json_max_sensors) as f64 / json_max_sensors as f64) * 100.0;

    println!("With 8K token context window:");
    println!("  JSON format:  ~{} sensors max", json_max_sensors);
    println!("  LNMP format:  ~{} sensors max", lnmp_max_sensors);
    println!("  Capacity:     +{:.1}% more sensors!\n", capacity_increase);

    // Simulated LLM response
    println!("═══════════════════════════════════════════════════════════════");
    println!("                   SIMULATED LLM RESPONSE                      ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let response = simulate_llm_response(&sensors);
    println!("{}", response);

    println!("\n✅ Demo complete!");
    println!("\n💡 Key Takeaways:");
    println!("   • {:.1}% fewer tokens with LNMP", token_reduction);
    println!(
        "   • {:.1}% capacity increase in context window",
        capacity_increase
    );
    println!("   • Significant cost savings at scale");
    println!("   • LLMs understand LNMP with field mapping");
}
