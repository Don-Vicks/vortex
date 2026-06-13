use crate::{NetworkState, TipDecision, AgentConfig};
use crate::client::call_agent;
use anyhow::Result;

/// Hard ceiling: never tip more than 5M lamports (0.005 SOL) under normal conditions.
const HARD_CEILING_LAMPORTS: u64 = 5_000_000;
/// Absolute floor: never tip less than 1000 lamports — zero tips guarantee failure.
const HARD_FLOOR_LAMPORTS: u64 = 1_000;
/// When failure rate exceeds this threshold, allow tips up to the max reported by Jito.
const HIGH_FAILURE_THRESHOLD: f64 = 50.0;

pub fn build_prompt(state: &NetworkState) -> String {
    let effective_ceiling = if state.recent_failure_rate > HIGH_FAILURE_THRESHOLD {
        state.tip_max
    } else {
        HARD_CEILING_LAMPORTS.min(state.tip_max)
    };

    format!(
        r#"Given the following Solana network state, return the optimal Jito tip in lamports.

Network state:
- Current slot: {slot}
- Seconds since last successful landing: {since_success}
- Jito tip floor (lamports): min={tip_min}, median={tip_median}, p75={tip_avg}, p95={tip_max}
- Recent tx failure rate: {fail_rate:.1}%

Constraints (HARD — violating any constraint invalidates your output):
- recommended_lamports MUST be >= {floor}
- recommended_lamports MUST be <= {ceiling}
- If failure rate is 0% and time_since_last_success is 0, bid at or near the minimum ({tip_min}).
- Only increase tip proportionally to failure rate.

Respond with ONLY this JSON object, nothing else:
{{"recommended_lamports": <integer>, "reasoning": "<one sentence>", "confidence": "high|medium|low"}}"#,
        slot = state.current_slot,
        since_success = state.time_since_last_success_secs,
        tip_min = state.tip_min,
        tip_median = state.tip_median,
        tip_avg = state.tip_average,
        tip_max = state.tip_max,
        fail_rate = state.recent_failure_rate,
        floor = HARD_FLOOR_LAMPORTS,
        ceiling = effective_ceiling,
    )
}

pub async fn decide_tip(config: &AgentConfig, state: &NetworkState) -> Result<TipDecision> {
    let prompt = build_prompt(state);
    let response = call_agent(config, &prompt).await?;

    let mut decision: TipDecision = serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse agent response '{}': {}", response, e))?;

    // Validate reasoning is present
    if decision.reasoning.is_empty() {
        return Err(anyhow::anyhow!("Agent returned empty reasoning"));
    }

    // Validate confidence field
    match decision.confidence.as_str() {
        "high" | "medium" | "low" => {}
        _ => decision.confidence = "low".to_string(),
    }

    // Clamp to hard bounds — never trust the LLM's number blindly
    let effective_ceiling = if state.recent_failure_rate > HIGH_FAILURE_THRESHOLD {
        state.tip_max
    } else {
        HARD_CEILING_LAMPORTS.min(state.tip_max)
    };
    let original = decision.recommended_lamports;
    decision.recommended_lamports = decision.recommended_lamports.clamp(HARD_FLOOR_LAMPORTS, effective_ceiling);

    if decision.recommended_lamports != original {
        decision.reasoning = format!(
            "{} (clamped from {} to {} lamports)",
            decision.reasoning, original, decision.recommended_lamports
        );
    }

    Ok(decision)
}

pub fn fallback_tip(median: u64) -> TipDecision {
    TipDecision {
        recommended_lamports: median.clamp(HARD_FLOOR_LAMPORTS, HARD_CEILING_LAMPORTS),
        reasoning: "Agent unavailable. Using clamped tip median as fallback.".to_string(),
        confidence: "low".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state(failure_rate: f64, time_since_success: u64) -> NetworkState {
        NetworkState {
            current_slot: 250_000_000,
            tip_min: 1_000_000,
            tip_max: 10_000_000,
            tip_median: 5_000_000,
            tip_average: 7_000_000,
            recent_failure_rate: failure_rate,
            time_since_last_success_secs: time_since_success,
        }
    }

    #[test]
    fn test_prompt_contains_hard_bounds() {
        let state = test_state(0.0, 0);
        let prompt = build_prompt(&state);
        assert!(prompt.contains("MUST be >= 1000"));
        assert!(prompt.contains("MUST be <= 5000000"));
    }

    #[test]
    fn test_prompt_high_failure_raises_ceiling() {
        let state = test_state(60.0, 30);
        let prompt = build_prompt(&state);
        // With >50% failure rate, ceiling should be tip_max (10M), not 5M
        assert!(prompt.contains("MUST be <= 10000000"));
    }

    #[test]
    fn test_fallback_is_clamped() {
        let decision = fallback_tip(100_000_000); // 0.1 SOL — way too high
        assert_eq!(decision.recommended_lamports, HARD_CEILING_LAMPORTS);

        let decision = fallback_tip(0); // zero — too low
        assert_eq!(decision.recommended_lamports, HARD_FLOOR_LAMPORTS);
    }
}
