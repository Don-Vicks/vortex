use crate::{NetworkState, TipDecision, AgentConfig};
use crate::client::call_agent;
use anyhow::Result;

pub fn build_prompt(state: &NetworkState) -> String {
    format!(
        r#"You are a Solana transaction tip optimizer. Your goal is EXTREME cost-efficiency.

Current network state (LIVE via Geyser):
- Current slot: {}
- Time since last success: {} seconds
- Jito Tip Floor stats (lamports):
  - Min: {}
  - Median: {}
  - Average: {}
  - Max: {}
- Recent failure rate: {}%

Rules:
1. NEVER suggest a tip over 5,000,000 lamports (0.005 SOL) unless failure rate is > 50%.
2. If time since last success is 0, the network is fast; bid as close to the MINIMUM as possible.
3. Be aggressive about saving money. 0.2 dollars is too much for a calm network.

Respond ONLY in this JSON format:
{{
  "reasoning": "short reasoning based on slot and speed",
  "recommended_lamports": 1000000,
  "confidence": "high"
}}"#,
        state.current_slot,
        state.time_since_last_success_secs,
        state.tip_min,
        state.tip_median,
        state.tip_average,
        state.tip_max,
        state.recent_failure_rate,
    )
}

pub async fn decide_tip(config: &AgentConfig, state: &NetworkState) -> Result<TipDecision> {
    let prompt = build_prompt(state);
    let response = call_agent(config, &prompt).await?;

    let decision: TipDecision = serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse agent response: {}", e))?;

    if decision.reasoning.is_empty() {
        return Err(anyhow::anyhow!("Agent returned empty reasoning"));
    }

    Ok(decision)
}

pub fn fallback_tip(median: u64) -> TipDecision {
    TipDecision {
        recommended_lamports: median,
        reasoning: "Agent unavailable. Using tip median as fallback.".to_string(),
        confidence: "low".to_string(),
    }
}
