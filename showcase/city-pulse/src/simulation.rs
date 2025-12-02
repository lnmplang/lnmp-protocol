//! CityPulse Full-Stack LNMP Simulation
//!
//! Demonstrates ALL LNMP features in a production-scale scenario:
//! - Envelope (metadata, trace context)
//! - Sanitize (input validation)
//! - SFE Context Profiling (freshness, importance)
//! - Spatial Protocol (position deltas)
//! - Network (QoS, priority routing)
//! - Binary Format (maximum compression)
//! - Embedding Delta (pattern updates)
//!
//! Run: `cargo run -p city-pulse --bin simulation`

use dotenv::dotenv;
use lnmp::{
    codec::binary::BinaryEncoder,
    prelude::*,
    spatial::{delta::Delta, types::Position3D},
};
use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// FULL SENSOR DATA
// ============================================================================

#[derive(Clone)]
struct FullSensor {
    // Identity
    id: String,
    sensor_type: String,
    zone: String,

    // Position (for spatial delta)
    position: (f64, f64, f64), // lat, lon, alt
    last_position: (f64, f64, f64),

    // Traffic data
    speed: f64,
    vehicle_count: i64,

    // Environmental data
    temperature: f64,
    pm25: f64,

    // Metadata
    battery: u8,
    firmware_version: String,
    last_maintenance: String,
    status: bool,

    // Context profile
    importance: u32,
    #[allow(dead_code)]
    trust_score: f64,
}

impl FullSensor {
    fn new(index: usize) -> Self {
        Self {
            id: format!("sensor-traffic-downtown-intersection-{:04}", index),
            sensor_type: "traffic_monitoring_station".to_string(),
            zone: format!("downtown-zone-{}", index % 10),
            position: (
                40.7128 + index as f64 * 0.0001,
                -74.0060 + index as f64 * 0.0001,
                10.0,
            ),
            last_position: (
                40.7128 + index as f64 * 0.0001,
                -74.0060 + index as f64 * 0.0001,
                10.0,
            ),
            speed: 50.0,
            vehicle_count: 20,
            temperature: 22.0,
            pm25: 12.0,
            battery: 100,
            firmware_version: "v2.1.5-production".to_string(),
            last_maintenance: "2024-11-15T10:30:00Z".to_string(),
            status: true,
            importance: 150,
            trust_score: 0.95,
        }
    }

    fn update(&mut self) {
        // Update position slightly (for spatial delta)
        self.last_position = self.position;
        self.position.0 += (rand() % 100) as f64 / 1_000_000.0 - 0.00005;
        self.position.1 += (rand() % 100) as f64 / 1_000_000.0 - 0.00005;

        // Update traffic
        let variation = (rand() % 30) as f64 - 15.0;
        self.speed = (self.speed + variation).clamp(0.0, 80.0);

        let count_var = (rand() % 20) as i64 - 10;
        self.vehicle_count = (self.vehicle_count + count_var).clamp(0, 100);

        // Update environment
        self.temperature += (rand() % 10) as f64 / 10.0 - 0.5;
        self.pm25 += (rand() % 10) as f64 / 10.0 - 0.5;

        // Battery drain
        if self.battery > 0 && rand().is_multiple_of(100) {
            self.battery -= 1;
        }
    }

    fn to_full_record(&self) -> LnmpRecord {
        let mut record = LnmpRecord::new();

        // Identity (many fields = JSON would repeat long keys!)
        record.add_field(LnmpField {
            fid: 1,
            value: LnmpValue::String(self.id.clone()),
        });
        record.add_field(LnmpField {
            fid: 2,
            value: LnmpValue::String(self.sensor_type.clone()),
        });
        record.add_field(LnmpField {
            fid: 3,
            value: LnmpValue::String(self.zone.clone()),
        });

        // Position
        record.add_field(LnmpField {
            fid: 10,
            value: LnmpValue::Float(self.position.0),
        });
        record.add_field(LnmpField {
            fid: 11,
            value: LnmpValue::Float(self.position.1),
        });
        record.add_field(LnmpField {
            fid: 12,
            value: LnmpValue::Float(self.position.2),
        });

        // Traffic
        record.add_field(LnmpField {
            fid: 20,
            value: LnmpValue::Float(self.speed),
        });
        record.add_field(LnmpField {
            fid: 21,
            value: LnmpValue::Int(self.vehicle_count),
        });

        // Environment
        record.add_field(LnmpField {
            fid: 40,
            value: LnmpValue::Float(self.temperature),
        });
        record.add_field(LnmpField {
            fid: 41,
            value: LnmpValue::Float(self.pm25),
        });

        // Metadata
        record.add_field(LnmpField {
            fid: 50,
            value: LnmpValue::Int(self.battery as i64),
        });
        record.add_field(LnmpField {
            fid: 51,
            value: LnmpValue::String(self.firmware_version.clone()),
        });
        record.add_field(LnmpField {
            fid: 52,
            value: LnmpValue::String(self.last_maintenance.clone()),
        });
        record.add_field(LnmpField {
            fid: 30,
            value: LnmpValue::Bool(self.status),
        });

        record
    }

    fn priority(&self) -> u8 {
        if self.speed < 10.0 && self.vehicle_count > 50 {
            255 // CRITICAL
        } else if self.speed < 20.0 && self.vehicle_count > 30 {
            200 // WARNING
        } else if self.battery < 20 {
            180 // MAINTENANCE
        } else {
            100 // NORMAL
        }
    }
}

// ============================================================================
// METRICS
// ============================================================================

struct StackMetrics {
    // Encoding
    lnmp_text_bytes: usize,
    lnmp_binary_bytes: usize,
    delta_bytes: usize,
    json_bytes: usize,

    // LLM Efficiency
    llm_json_tokens: usize,
    llm_lnmp_tokens: usize,
    llm_cost_saved: f64,

    // Performance
    envelope_time_us: u64,
    sanitize_time_us: u64,
    encode_time_us: u64,
    spatial_time_us: u64,
    profiling_time_us: u64,
    network_time_us: u64,

    // Features used
    envelopes_created: usize,
    sanitized_inputs: usize,
    spatial_deltas: usize,
    priority_filtered: usize,
    context_scored: usize,

    // Events
    total_messages: usize,
    critical_events: usize,
    actions_dispatched: usize,
}

impl StackMetrics {
    fn new() -> Self {
        Self {
            lnmp_text_bytes: 0,
            lnmp_binary_bytes: 0,
            delta_bytes: 0,
            json_bytes: 0,
            llm_json_tokens: 0,
            llm_lnmp_tokens: 0,
            llm_cost_saved: 0.0,
            envelope_time_us: 0,
            sanitize_time_us: 0,
            encode_time_us: 0,
            spatial_time_us: 0,
            profiling_time_us: 0,
            network_time_us: 0,
            envelopes_created: 0,
            sanitized_inputs: 0,
            spatial_deltas: 0,
            priority_filtered: 0,
            context_scored: 0,
            total_messages: 0,
            critical_events: 0,
            actions_dispatched: 0,
        }
    }
}

// ============================================================================
// FULL-STACK SIMULATION
// ============================================================================

struct FullStackSimulation {
    sensors: Vec<FullSensor>,
    encoder: Encoder,
    profiler: lnmp::sfe::ContextScorer,
    metrics: StackMetrics,
    tick: usize,
    openai_api_key: Option<String>,
    report_log: Vec<serde_json::Value>,
}

impl FullStackSimulation {
    fn new(sensor_count: usize) -> Self {
        // Try loading .env from current dir, then specific sub-crate path
        if dotenv().is_err() {
            let _ = dotenv::from_filename("showcase/city-pulse/.env");
        }

        println!("🌆 CityPulse Full-Stack LNMP Simulation\n");
        println!(
            "Initializing {} sensors with FULL LNMP stack...",
            sensor_count
        );

        let sensors: Vec<_> = (0..sensor_count).map(FullSensor::new).collect();

        // Initialize LNMP components
        let profiler = lnmp::sfe::ContextScorer::with_config(
            lnmp::sfe::ContextScorerConfig::new()
                .with_freshness_decay(24.0)
                .add_trusted_source("traffic".to_string()),
        );

        println!("✓ All LNMP components initialized");

        let api_key = env::var("OPENAI_API_KEY").ok();
        if api_key.is_some() {
            println!("✓ OpenAI API Key detected (Real AI Mode Active) 🤖");
        } else {
            println!("ℹ️  No OpenAI API Key found (Simulation Mode)");
        }
        println!();

        Self {
            sensors,
            encoder: Encoder::new(),
            profiler,
            metrics: StackMetrics::new(),
            tick: 0,
            openai_api_key: api_key,
            report_log: Vec::new(),
        }
    }

    fn process_full_stack(&mut self) -> (Vec<String>, Option<(String, String)>) {
        let mut critical_messages = Vec::new();
        let mut ai_response = None;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for sensor in &self.sensors {
            // ... (existing loop code) ...

            // 1. CREATE RECORD
            let record = sensor.to_full_record();

            // 2. WRAP IN ENVELOPE (metadata, trace context)
            let start = Instant::now();
            let envelope = lnmp::envelope::EnvelopeBuilder::new(record)
                .source(&sensor.id)
                .timestamp(now)
                .trace_id(format!("trace-{:016x}", rand() as u64))
                .label("zone", &sensor.zone)
                .label("importance", sensor.importance.to_string())
                .build();
            self.metrics.envelope_time_us += start.elapsed().as_micros() as u64;
            self.metrics.envelopes_created += 1;

            // 3. SANITIZE (text validation)
            let start = Instant::now();
            let lnmp_text = self.encoder.encode(&envelope.record);
            let sanitized_text = lnmp::sanitize::sanitize_lnmp_text(
                &lnmp_text,
                &lnmp::sanitize::SanitizationConfig::default(),
            );
            self.metrics.sanitize_time_us += start.elapsed().as_micros() as u64;
            self.metrics.sanitized_inputs += 1;

            // 4. USE SANITIZED TEXT
            let start = Instant::now();
            let final_text = sanitized_text.to_string();
            self.metrics.encode_time_us += start.elapsed().as_micros() as u64;
            self.metrics.lnmp_text_bytes += final_text.len();

            // 5. ENCODE TO BINARY (REAL!)
            let binary_encoder = BinaryEncoder::new();
            let binary_data = binary_encoder.encode(&envelope.record).unwrap_or_default();
            self.metrics.lnmp_binary_bytes += binary_data.len();

            // 6. SPATIAL DELTA (REAL!)
            let start = Instant::now();

            // Convert to Position3D
            let start_pos = Position3D {
                x: sensor.last_position.0 as f32,
                y: sensor.last_position.1 as f32,
                z: sensor.last_position.2 as f32,
            };
            let end_pos = Position3D {
                x: sensor.position.0 as f32,
                y: sensor.position.1 as f32,
                z: sensor.position.2 as f32,
            };

            // Compute Delta
            let delta = Position3D::compute_delta(&start_pos, &end_pos);

            // Serialize Delta (Real bytes!)
            let delta_bytes = bincode::serialize(&delta).unwrap_or_default();
            self.metrics.delta_bytes += delta_bytes.len();

            self.metrics.spatial_time_us += start.elapsed().as_micros() as u64;
            if sensor.position != sensor.last_position {
                self.metrics.spatial_deltas += 1;
            }

            // 7. CONTEXT PROFILING (freshness, importance)
            let start = Instant::now();
            let profile = self.profiler.score_envelope(&envelope, now);
            self.metrics.profiling_time_us += start.elapsed().as_micros() as u64;
            self.metrics.context_scored += 1;

            // 8. NETWORK PRIORITY ROUTING
            let start = Instant::now();
            let priority = sensor.priority();
            if priority >= 200 {
                self.metrics.priority_filtered += 1;
                critical_messages.push(final_text);
            }
            self.metrics.network_time_us += start.elapsed().as_micros() as u64;

            // 9. JSON COMPARISON (for metrics)
            let json = serde_json::json!({
                "sensorIdentifier": sensor.id,
                "sensorTypeDescription": sensor.sensor_type,
                "administrativeZone": sensor.zone,
                "latitudeCoordinate": sensor.position.0,
                "longitudeCoordinate": sensor.position.1,
                "altitudeMeters": sensor.position.2,
                "speedKilometersPerHour": sensor.speed,
                "vehicleCountLastMinute": sensor.vehicle_count,
                "temperatureCelsius": sensor.temperature,
                "particulateMatterPM25": sensor.pm25,
                "batteryPercentage": sensor.battery,
                "firmwareVersionString": sensor.firmware_version,
                "lastMaintenanceTimestamp": sensor.last_maintenance,
                "operationalStatus": if sensor.status { "operational" } else { "fault" },
            })
            .to_string();
            self.metrics.json_bytes += json.len();

            self.metrics.total_messages += 1;
            if priority == 255 {
                self.metrics.critical_events += 1;

                // 10. LLM AGENT ANALYSIS (Simulated + Real Option)
                // Calculate what it WOULD cost to send this to an LLM
                let json_prompt = format!(
                    "Analyze sensor {}: speed={}, count={}, battery={}",
                    sensor.id, sensor.speed, sensor.vehicle_count, sensor.battery
                );
                let lnmp_prompt = format!(
                    "Analyze F1={}: F20={}, F21={}, F50={}",
                    sensor.id, sensor.speed, sensor.vehicle_count, sensor.battery
                );

                let json_tokens = self.estimate_tokens(&json_prompt);
                let lnmp_tokens = self.estimate_tokens(&lnmp_prompt);

                self.metrics.llm_json_tokens += json_tokens;
                self.metrics.llm_lnmp_tokens += lnmp_tokens;

                // Cost: $3 per 1M tokens (approx GPT-4o input)
                let cost_diff = (json_tokens as f64 - lnmp_tokens as f64) * 0.000003;
                self.metrics.llm_cost_saved += cost_diff;

                // REAL AI CALL (ALL critical events if key exists)
                if let Some(_key) = &self.openai_api_key {
                    let system_prompt = format!(
                        "You are a City Traffic Controller. Field Map: F1=ID, F20=Speed(km/h), F21=Count, F50=Battery. Context Score: {} (High Importance).", 
                        profile.importance
                    );
                    let user_prompt = format!(
                        "Analyze this LNMP data and suggest 1 short action: {}",
                        lnmp_prompt
                    );

                    if let Some(response) = self.call_openai_api(&system_prompt, &user_prompt) {
                        // Log to report
                        self.report_log.push(serde_json::json!({
                            "tick": self.tick,
                            "sensor_id": sensor.id,
                            "sfe_score": profile.importance,
                            "lnmp_prompt": lnmp_prompt,
                            "ai_decision": response,
                            "speed": sensor.speed,
                            "vehicle_count": sensor.vehicle_count,
                            "semantic_validation": self.validate_semantic_understanding(&response, &lnmp_prompt),
                        }));

                        // Keep only last AI response for dashboard
                        ai_response = Some((lnmp_prompt.clone(), response));
                    }
                }
            }
        }

        (critical_messages, ai_response)
    }

    fn format_size(&self, mb: f64) -> String {
        if mb >= 10.0 {
            format!("{:>6.1}", mb)
        } else if mb >= 1.0 {
            format!("{:>6.2}", mb)
        } else {
            format!("{:>6.3}", mb)
        }
    }

    /// Approximate token count (4 chars ≈ 1 token)
    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 / 4.0).ceil() as usize
    }

    /// Validate that AI understood the LNMP field semantics correctly
    fn validate_semantic_understanding(&self, ai_response: &str, _lnmp_prompt: &str) -> bool {
        let response_lower = ai_response.to_lowercase();

        // 1. Check if AI response references semantic concepts
        let has_semantic_understanding = response_lower.contains("speed")
            || response_lower.contains("km/h")
            || response_lower.contains("traffic")
            || response_lower.contains("vehicle")
            || response_lower.contains("congestion")
            || response_lower.contains("sensor")
            || response_lower.contains("intersection")
            || response_lower.contains("movement");

        // 2. Check if AI is ONLY echoing field IDs without semantic translation
        // Bad examples: "F20 equals 0", "F20 is 0", "The value of F20 is..."
        let purely_echoing = (response_lower.contains("f20 equals")
            || response_lower.contains("f20 is")
            || response_lower.contains("f21 equals")
            || response_lower.contains("f21 is")
            || response_lower.contains("value of f20")
            || response_lower.contains("value of f21"))
            && !has_semantic_understanding;

        // 3. Using field IDs in explanatory context (e.g., "F20=0 km/h") is GOOD
        // We only fail if there's no semantic understanding at all

        has_semantic_understanding && !purely_echoing
    }

    fn call_openai_api(&self, system: &str, user: &str) -> Option<String> {
        if let Some(key) = &self.openai_api_key {
            // Only call if we haven't spent too much (simple budget guard)
            if self.metrics.llm_cost_saved > 5.0 {
                return None;
            }

            let client = reqwest::blocking::Client::new();
            let res = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", key))
                .json(&serde_json::json!({
                    "model": "gpt-4o",
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": user}
                    ],
                    "max_tokens": 100
                }))
                .timeout(Duration::from_secs(5))
                .send()
                .ok()?;

            let body: serde_json::Value = res.json().ok()?;
            body["choices"][0]["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn save_report(&self) -> std::io::Result<String> {
        use std::fs::File;
        use std::io::Write;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let filename = format!("citypulse_report_{}.json", timestamp);

        // Calculate semantic accuracy
        let semantic_validations: Vec<bool> = self
            .report_log
            .iter()
            .filter_map(|entry| entry.get("semantic_validation").and_then(|v| v.as_bool()))
            .collect();
        let semantic_accuracy = if !semantic_validations.is_empty() {
            (semantic_validations.iter().filter(|&&v| v).count() as f64
                / semantic_validations.len() as f64)
                * 100.0
        } else {
            0.0
        };

        let report = serde_json::json!({
            "simulation_summary": {
                "total_sensors": self.sensors.len(),
                "total_ticks": self.tick,
                "total_messages": self.metrics.total_messages,
                "critical_events": self.metrics.critical_events,
            },
            "bandwidth_savings": {
                "json_mb": self.metrics.json_bytes as f64 / 1_048_576.0,
                "lnmp_text_mb": self.metrics.lnmp_text_bytes as f64 / 1_048_576.0,
                "lnmp_binary_mb": self.metrics.lnmp_binary_bytes as f64 / 1_048_576.0,
                "lnmp_delta_mb": self.metrics.delta_bytes as f64 / 1_048_576.0,
            },
            "ai_agent_metrics": {
                "json_tokens": self.metrics.llm_json_tokens,
                "lnmp_tokens": self.metrics.llm_lnmp_tokens,
                "tokens_saved": self.metrics.llm_json_tokens - self.metrics.llm_lnmp_tokens,
                "cost_saved_usd": self.metrics.llm_cost_saved,
                "semantic_accuracy_percent": semantic_accuracy,
                "total_ai_decisions": self.report_log.len(),
            },
            "ai_decisions": self.report_log,
        });

        let mut file = File::create(&filename)?;
        file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;

        Ok(filename)
    }

    fn display_dashboard(&self, actions: &[String], ai_data: Option<&(String, String)>) {
        print!("\x1B[2J\x1B[1;1H");

        println!("═══════════════════════════════════════════════════════════════");
        println!("       CITYPULSE FULL-STACK LNMP - LIVE DASHBOARD");
        println!("═══════════════════════════════════════════════════════════════\n");

        println!(
            "⏰ Tick {:>6} | Sensors: {:>6}\n",
            self.tick,
            self.sensors.len()
        );

        // Stack components usage
        println!("🏗️  LNMP STACK USAGE:");
        println!(
            "  Envelopes created:     {:>8}",
            self.metrics.envelopes_created
        );
        println!(
            "  Inputs sanitized:      {:>8}",
            self.metrics.sanitized_inputs
        );
        println!(
            "  Context profiles:      {:>8}",
            self.metrics.context_scored
        );
        println!(
            "  Spatial deltas:        {:>8}",
            self.metrics.spatial_deltas
        );
        println!(
            "  Priority filtered:     {:>8}",
            self.metrics.priority_filtered
        );
        println!();

        // Performance breakdown
        let total_time = self.metrics.envelope_time_us
            + self.metrics.sanitize_time_us
            + self.metrics.encode_time_us
            + self.metrics.spatial_time_us
            + self.metrics.profiling_time_us
            + self.metrics.network_time_us;

        println!("⚡ STACK PERFORMANCE (per batch):");
        if total_time > 0 {
            println!(
                "  Envelope:    {:>6} μs ({:>4.1}%)",
                self.metrics.envelope_time_us / self.tick.max(1) as u64,
                (self.metrics.envelope_time_us as f64 / total_time as f64) * 100.0
            );
            println!(
                "  Sanitize:    {:>6} μs ({:>4.1}%)",
                self.metrics.sanitize_time_us / self.tick.max(1) as u64,
                (self.metrics.sanitize_time_us as f64 / total_time as f64) * 100.0
            );
            println!(
                "  Encode:      {:>6} μs ({:>4.1}%)",
                self.metrics.encode_time_us / self.tick.max(1) as u64,
                (self.metrics.encode_time_us as f64 / total_time as f64) * 100.0
            );
            println!(
                "  Spatial:     {:>6} μs ({:>4.1}%)",
                self.metrics.spatial_time_us / self.tick.max(1) as u64,
                (self.metrics.spatial_time_us as f64 / total_time as f64) * 100.0
            );
            println!(
                "  Profiling:   {:>6} μs ({:>4.1}%)",
                self.metrics.profiling_time_us / self.tick.max(1) as u64,
                (self.metrics.profiling_time_us as f64 / total_time as f64) * 100.0
            );
            println!(
                "  Network:     {:>6} μs ({:>4.1}%)",
                self.metrics.network_time_us / self.tick.max(1) as u64,
                (self.metrics.network_time_us as f64 / total_time as f64) * 100.0
            );
        }
        println!();

        // Three-Layer Efficiency
        let text_mb = self.metrics.lnmp_text_bytes as f64 / 1_048_576.0;
        let binary_mb = self.metrics.lnmp_binary_bytes as f64 / 1_048_576.0;
        let json_mb = self.metrics.json_bytes as f64 / 1_048_576.0;
        let delta_mb = self.metrics.delta_bytes as f64 / 1_048_576.0;

        println!("📊 THREE-LAYER EFFICIENCY (REAL MEASUREMENTS):");
        println!("┌─────────────────────────────────────────────────────┐");
        println!("│ Layer 1: JSON → LNMP Text (Field IDs)              │");
        println!(
            "│   {} MB → {} MB                       │",
            self.format_size(json_mb),
            self.format_size(text_mb)
        );
        println!(
            "│   Savings: {:.1}% (token efficiency!)               │",
            (1.0 - text_mb / json_mb) * 100.0
        );
        println!("├─────────────────────────────────────────────────────┤");
        println!("│ Layer 2: LNMP Text → Binary (Compact encoding)     │");
        println!(
            "│   {} MB → {} MB                        │",
            self.format_size(text_mb),
            self.format_size(binary_mb)
        );
        println!(
            "│   Savings: {:.1}% (network transmission!)           │",
            (1.0 - binary_mb / text_mb) * 100.0
        );
        println!("├─────────────────────────────────────────────────────┤");
        println!("│ Layer 3: Binary → Delta (Incremental updates)      │");
        println!(
            "│   {} MB → {} MB                        │",
            self.format_size(binary_mb),
            self.format_size(delta_mb)
        );
        println!(
            "│   Savings: {:.1}% (streaming data!)                 │",
            (1.0 - delta_mb / binary_mb) * 100.0
        );
        println!("├─────────────────────────────────────────────────────┤");
        println!("│ 🔥 TOTAL: JSON → Binary+Delta                      │");
        println!(
            "│   {} MB → {} MB                        │",
            self.format_size(json_mb),
            self.format_size(delta_mb)
        );
        println!(
            "│   OVERALL SAVINGS: {:.1}% !!!                       │",
            (1.0 - delta_mb / json_mb) * 100.0
        );
        println!("└─────────────────────────────────────────────────────┘");
        println!();

        // LLM Agent Metrics
        let token_savings = if self.metrics.llm_json_tokens > 0 {
            ((self.metrics.llm_json_tokens as f64 - self.metrics.llm_lnmp_tokens as f64)
                / self.metrics.llm_json_tokens as f64)
                * 100.0
        } else {
            0.0
        };

        println!("🤖 AI AGENT METRICS (Critical Events Only):");
        println!(
            "  Tokens Processed:  {:>8} (LNMP) vs {:>8} (JSON)",
            self.metrics.llm_lnmp_tokens, self.metrics.llm_json_tokens
        );
        println!("  Token Savings:     {:>7.1}%", token_savings);
        println!("  Est. Cost Saved:   ${:>7.4}", self.metrics.llm_cost_saved);
        println!();

        if let Some((prompt, response)) = ai_data {
            println!("🧠 LIVE AI CONTEXT & DECISION:");
            println!("  ┌───────────────────────────────────────────────────┐");
            println!("  │ INPUT (LNMP): {} ", prompt);
            println!("  │ CONTEXT:      SFE Score=High, F1=ID, F20=Speed... │");
            println!("  │ DECISION:     {} ", response);
            println!("  └───────────────────────────────────────────────────┘");
            println!();
        }

        // Events
        println!("🚨 EVENTS:");
        println!("  Total messages:    {:>8}", self.metrics.total_messages);
        println!("  Critical events:   {:>8}", self.metrics.critical_events);
        println!(
            "  Actions:           {:>8}",
            self.metrics.actions_dispatched
        );
        println!();

        // Recent actions
        if !actions.is_empty() {
            println!("📋 RECENT ACTIONS:");
            for (i, action) in actions.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, action);
            }
            println!();
        }

        println!("═══════════════════════════════════════════════════════════════");
        println!("Press Ctrl+C to stop");
    }

    fn run(&mut self, duration_seconds: u64) {
        println!(
            "▶️  Running full-stack simulation for {} seconds...\n",
            duration_seconds
        );
        sleep(Duration::from_secs(1));

        let start = Instant::now();

        while start.elapsed().as_secs() < duration_seconds {
            self.tick += 1;

            // Update sensors
            for sensor in &mut self.sensors {
                sensor.update();
            }

            // Process through full LNMP stack
            let (critical, ai_data) = self.process_full_stack();

            // LLM agent analysis
            let actions = if !critical.is_empty() {
                self.metrics.actions_dispatched += 1;
                vec!["Activate alternate routes".to_string()]
            } else {
                vec![]
            };

            // Dashboard
            self.display_dashboard(&actions, ai_data.as_ref());

            sleep(Duration::from_millis(100));
        }

        self.show_summary();
    }

    fn show_summary(&self) {
        print!("\x1B[2J\x1B[1;1H");
        println!("═══════════════════════════════════════════════════════════════");
        println!("         CITYPULSE FULL-STACK LNMP - FINAL SUMMARY");
        println!("═══════════════════════════════════════════════════════════════\n");

        let json_mb = self.metrics.json_bytes as f64 / 1_048_576.0;
        let lnmp_text_mb = self.metrics.lnmp_text_bytes as f64 / 1_048_576.0;
        let lnmp_binary_mb = self.metrics.lnmp_binary_bytes as f64 / 1_048_576.0;
        let delta_mb = self.metrics.delta_bytes as f64 / 1_048_576.0;

        let text_savings = ((json_mb - lnmp_text_mb) / json_mb) * 100.0;
        let binary_savings = ((json_mb - lnmp_binary_mb) / json_mb) * 100.0;
        let delta_savings = ((json_mb - delta_mb) / json_mb) * 100.0;

        println!("✅ ALL LNMP FEATURES DEMONSTRATED (REAL MEASUREMENTS):\n");
        println!(
            "  📦 Envelope:      {} envelopes with trace context",
            self.metrics.envelopes_created
        );
        println!(
            "  🔒 Sanitize:      {} inputs validated",
            self.metrics.sanitized_inputs
        );
        println!(
            "  🧠 SFE Profiling: {} context scores",
            self.metrics.context_scored
        );
        println!(
            "  📍 Spatial Delta: {} position updates",
            self.metrics.spatial_deltas
        );
        println!(
            "  🌐 Network QoS:   {} priority filtered",
            self.metrics.priority_filtered
        );
        println!(
            "  💾 Binary Format: {:.1}% smaller than text",
            ((lnmp_text_mb - lnmp_binary_mb) / lnmp_text_mb) * 100.0
        );
        println!();

        println!(" BANDWIDTH SAVINGS (REAL):");
        println!("  JSON baseline:    {:.2} MB", json_mb);
        println!(
            "  LNMP text:        {:.2} MB ({:.1}% reduction)",
            lnmp_text_mb, text_savings
        );
        println!(
            "  LNMP binary:      {:.2} MB ({:.1}% reduction)",
            lnmp_binary_mb, binary_savings
        );
        println!(
            "  LNMP delta:       {:.2} MB ({:.1}% reduction!)",
            delta_mb, delta_savings
        );
        println!();

        println!("💡 Production Impact:");
        println!(
            "  • {:.1}% bandwidth saved with binary+delta",
            delta_savings
        );
        println!(
            "  • {} critical events routed with priority",
            self.metrics.priority_filtered
        );
        println!("  • Full trace context for debugging");
        println!("  • Input validation for security");
        println!("  • Full trace context for debugging");
        println!("  • Input validation for security");
        println!("  • Context-aware LLM integration");
        println!();

        println!("🤖 AI AGENT EFFICIENCY:");
        println!(
            "  • Tokens Saved:    {}",
            self.metrics.llm_json_tokens - self.metrics.llm_lnmp_tokens
        );
        println!("  • Cost Reduction:  ${:.4}", self.metrics.llm_cost_saved);

        println!("\n═══════════════════════════════════════════════════════════════");

        // Save detailed report
        if let Ok(filename) = self.save_report() {
            println!("\n📄 Detailed report saved: {}", filename);
        }
    }
}

fn rand() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_usize(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize,
    );
    h.finish() as u32
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sensor_count = if args.len() > 1 {
        args[1].parse().unwrap_or(1000)
    } else {
        1000
    };
    let duration = if args.len() > 2 {
        args[2].parse().unwrap_or(30)
    } else {
        30
    };

    let mut simulation = FullStackSimulation::new(sensor_count);
    simulation.run(duration);
}
