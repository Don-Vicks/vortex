pub mod client;
pub mod decisions;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AiProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Grok,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub provider: AiProvider,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkState {
    pub current_slot: u64,
    pub tip_min: u64,
    pub tip_max: u64,
    pub tip_median: u64,
    pub tip_average: u64,
    pub recent_failure_rate: f64,
    pub time_since_last_success_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TipDecision {
    pub recommended_lamports: u64,
    pub reasoning: String,
    pub confidence: String,
}
