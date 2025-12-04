//! Enhanced Terminal Dashboard for City Pulse OS with Performance Tracking

use crate::agents::AgentSystem;
use crate::pipeline::PipelineStats;
use crate::scenarios::Scenario;
use std::collections::VecDeque;
use std::io::{self, Write};

const HISTORY_SIZE: usize = 20;

pub struct Dashboard {
    log_buffer: Vec<String>,
    llm_log_buffer: Vec<String>,
    bandwidth_history: VecDeque<f32>,
    critical_events_history: VecDeque<usize>,
    agents_active_history: VecDeque<usize>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            log_buffer: Vec::new(),
            llm_log_buffer: Vec::new(),
            bandwidth_history: VecDeque::with_capacity(HISTORY_SIZE),
            critical_events_history: VecDeque::with_capacity(HISTORY_SIZE),
            agents_active_history: VecDeque::with_capacity(HISTORY_SIZE),
        }
    }

    pub fn update(
        &mut self,
        tick: usize,
        scenario: &dyn Scenario,
        stats: &PipelineStats,
        agents: &AgentSystem,
        recent_events: &[String],
        llm_actions: &[String],
    ) {
        // Update logs
        for event in recent_events {
            self.log_buffer.push(event.clone());
            if self.log_buffer.len() > 10 {
                self.log_buffer.remove(0);
            }
        }

        for action in llm_actions {
            self.llm_log_buffer.push(action.clone());
            if self.llm_log_buffer.len() > 5 {
                self.llm_log_buffer.remove(0);
            }
        }

        // Update performance history
        self.bandwidth_history.push_back(stats.bandwidth_reduction);
        if self.bandwidth_history.len() > HISTORY_SIZE {
            self.bandwidth_history.pop_front();
        }

        self.critical_events_history
            .push_back(stats.critical_events);
        if self.critical_events_history.len() > HISTORY_SIZE {
            self.critical_events_history.pop_front();
        }

        let busy = agents.busy_agent_count();
        self.agents_active_history.push_back(busy);
        if self.agents_active_history.len() > HISTORY_SIZE {
            self.agents_active_history.pop_front();
        }

        // Clear screen and render
        print!("\x1B[2J\x1B[1;1H");

        println!("🏙️  TOKYO SMART CITY OS - COMMAND CENTER");
        println!("═══════════════════════════════════════════════════════════════════════════════");

        // 1. Status Bar
        println!(
            "⏱️  TICK: {:<5} | 🚨 SCENARIO: {:<30} | 👥 AGENTS: {}",
            tick,
            scenario.status(),
            agents.active_agent_count()
        );
        println!("───────────────────────────────────────────────────────────────────────────────");

        // 2. Pipeline Metrics with Visual Bar
        println!("📊 PIPELINE METRICS");
        println!("   Input Rate:      {:<6} events/tick", stats.stage1_input);
        println!(
            "   Filtered:        {:<6} events (Stage 1)",
            stats.stage1_output
        );
        println!(
            "   Critical:        {:<6} events (Stage 2)",
            stats.stage2_output
        );

        // Bandwidth savings bar
        let bw_bar = self.create_bar(stats.bandwidth_reduction, 100.0, 30);
        println!(
            "   Bandwidth:       \x1B[32m{:<6.2}%\x1B[0m {}",
            stats.bandwidth_reduction, bw_bar
        );

        // LLM cost savings bar
        let llm_bar = self.create_bar(stats.llm_cost_reduction, 100.0, 30);
        println!(
            "   LLM Cost Saved:  \x1B[32m{:<6.2}%\x1B[0m {}",
            stats.llm_cost_reduction, llm_bar
        );
        println!("───────────────────────────────────────────────────────────────────────────────");

        // 3. Performance Trends
        println!(
            "📈 PERFORMANCE TRENDS (Last {} ticks)",
            self.bandwidth_history.len()
        );
        println!("   Bandwidth Savings:");
        let bw_vec: Vec<f32> = self.bandwidth_history.iter().copied().collect();
        println!("   {}", self.create_sparkline(&bw_vec, 100.0));
        println!("   Critical Events:");
        let ce_vec: Vec<f32> = self
            .critical_events_history
            .iter()
            .map(|&x| x as f32)
            .collect();
        println!("   {}", self.create_sparkline(&ce_vec, 50.0));
        println!("───────────────────────────────────────────────────────────────────────────────");

        // 4. Agent Status
        println!("👮 AGENT STATUS");
        let busy = agents.busy_agent_count();
        let total = agents.active_agent_count();
        let agent_bar = self.create_bar(busy as f32, total as f32, 20);
        println!("   Active Units: {} / {} {}", busy, total, agent_bar);
        println!("───────────────────────────────────────────────────────────────────────────────");

        // 5. AI Decisions
        println!("🧠 AI DECISIONS (LLM Bridge)");
        if self.llm_log_buffer.is_empty() {
            println!("   (No recent AI actions)");
        } else {
            for action in &self.llm_log_buffer {
                println!("   ➤ {}", action);
            }
        }
        println!("───────────────────────────────────────────────────────────────────────────────");

        // 6. Critical Event Log
        println!("📡 CRITICAL EVENT LOG");
        if self.log_buffer.is_empty() {
            println!("   (No critical events)");
        } else {
            for log in &self.log_buffer {
                println!("   • {}", log);
            }
        }

        println!("═══════════════════════════════════════════════════════════════════════════════");
        io::stdout().flush().unwrap();
    }

    fn create_bar(&self, value: f32, max: f32, width: usize) -> String {
        let ratio = (value / max).clamp(0.0, 1.0);
        let filled = (ratio * width as f32) as usize;
        let empty = width - filled;

        format!(
            "\x1B[32m{}\x1B[90m{}\x1B[0m",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }

    fn create_sparkline(&self, data: &[f32], max: f32) -> String {
        if data.is_empty() {
            return "   (no data)".to_string();
        }

        let bars = "▁▂▃▄▅▆▇█";
        let result: String = data
            .iter()
            .map(|&value| {
                let ratio = (value / max).clamp(0.0, 1.0);
                let index = (ratio * (bars.chars().count() - 1) as f32) as usize;
                bars.chars().nth(index).unwrap_or('▁')
            })
            .collect();

        format!("   {}", result)
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}
