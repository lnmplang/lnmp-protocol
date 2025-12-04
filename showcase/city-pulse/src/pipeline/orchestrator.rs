//! LNMP Pipeline Orchestrator - Full Stack Integration

use super::llb_converter::{LLBConverter, LLMPrompt, LLMResponse};
use super::network_filter::NetworkFilter;
use super::sfe_engine::{SFEEngine, ScoredEvent};
use crate::llm::OpenAIClient;
use lnmp::prelude::*;

/// Pipeline statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub stage1_input: usize,
    pub stage1_output: usize,
    pub stage2_output: usize,
    pub critical_events: usize,
    pub bandwidth_reduction: f32,
    pub llm_cost_reduction: f32,
}

/// Pipeline output
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub critical_events: Vec<ScoredEvent>,
    pub llm_prompt: LLMPrompt,
    pub stats: PipelineStats,
}

/// Full LNMP Pipeline orchestrator
pub struct LNMPPipeline {
    network_filter: NetworkFilter,
    sfe_engine: SFEEngine,
    llb_converter: LLBConverter,
    openai_client: Option<OpenAIClient>,
    pub stats: PipelineStats,
}

impl LNMPPipeline {
    pub fn new() -> Self {
        // Try to initialize OpenAI client
        let openai_client = OpenAIClient::new().ok();

        if openai_client.is_some() {
            println!("🤖 OpenAI API Enabled - Using GPT-4o-mini for real AI decisions");
        } else {
            println!("⚠️  OpenAI API Key not found - Using simulated responses");
        }

        Self {
            network_filter: NetworkFilter::new(),
            sfe_engine: SFEEngine::new(),
            llb_converter: LLBConverter::new(),
            openai_client,
            stats: PipelineStats::default(),
        }
    }

    pub fn process(&mut self, events: Vec<LnmpRecord>) -> PipelineOutput {
        self.stats.stage1_input = events.len();

        // Stage 1: Network filtering
        let filtered = self.network_filter.filter(events);
        self.stats.stage1_output = filtered.len();

        // Stage 2: SFE scoring
        let scored = self.sfe_engine.score(filtered);
        let top_events = self.sfe_engine.top_k(scored, 200);
        self.stats.stage2_output = top_events.len();
        self.stats.critical_events = self.sfe_engine.stats.critical_events;

        // Stage 3: LLB conversion
        let llm_prompt = self.llb_converter.to_natural_language(&top_events);

        // Calculate metrics
        self.stats.bandwidth_reduction = if self.stats.stage1_input > 0 {
            (self.stats.stage1_input - self.stats.stage1_output) as f32
                / self.stats.stage1_input as f32
                * 100.0
        } else {
            0.0
        };

        self.stats.llm_cost_reduction = if self.stats.stage1_input > 0 {
            (self.stats.stage1_input - self.stats.stage2_output) as f32
                / self.stats.stage1_input as f32
                * 100.0
        } else {
            0.0
        };

        PipelineOutput {
            critical_events: top_events,
            llm_prompt,
            stats: self.stats.clone(),
        }
    }

    /// Process LLM response and return actions (with real API call if available)
    pub fn process_llm_response(&self, prompt: &str) -> LLMResponse {
        if let Some(client) = &self.openai_client {
            // Real API call
            match client.chat(
                "You are the AI coordinator for Tokyo Smart City Emergency Response System. \
                Analyze critical events and provide clear, actionable dispatch commands. \
                Be concise and direct. Use commands like:\n\
                - DISPATCH POLICE to [location]\n\
                - DISPATCH FIRE UNITS to [location]\n\
                - DISPATCH AMBULANCE to [location]\n\
                - DISPATCH TRAFFIC CONTROL to [location]\n\
                - INITIATE EVACUATION in [area]",
                prompt,
            ) {
                Ok(response) => {
                    println!("\n🤖 REAL AI DECISION: {}", response);
                    self.llb_converter.from_natural_language(&response)
                }
                Err(e) => {
                    eprintln!("⚠️  OpenAI API Error: {}", e);
                    eprintln!("   Falling back to simulated response");
                    self.llb_converter.from_natural_language(prompt)
                }
            }
        } else {
            // Simulated response
            self.llb_converter.from_natural_language(prompt)
        }
    }
}

impl Default for LNMPPipeline {
    fn default() -> Self {
        Self::new()
    }
}
