use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Deserialize)]
pub struct JitoTipFloor {
    pub landed_tips_25th_percentile: f64,
    pub landed_tips_50th_percentile: f64,
    pub landed_tips_75th_percentile: f64,
    pub landed_tips_95th_percentile: f64,
    pub landed_tips_99th_percentile: f64,
}

#[derive(Debug)]
pub struct TipStats {
    pub min: u64,
    pub max: u64,
    pub median: u64,
    pub average: u64,
}

pub async fn fetch_tip_stats(_rpc_url: &str) -> Result<TipStats> {
    let jito_url = std::env::var("JITO_BLOCK_ENGINE_URL").expect("JITO_BLOCK_ENGINE_URL must be set");
    let jito_endpoint = if jito_url.ends_with("/api/v1/bundles") {
        format!("{}/tip_floor", jito_url)
    } else {
        format!("{}/api/v1/bundles/tip_floor", jito_url.trim_end_matches('/'))
    };

    let client = Client::new();
    let response = client
        .get(&jito_endpoint)
        .send()
        .await;

    match response {
        Ok(res) => {
            if let Ok(floors) = res.json::<Vec<JitoTipFloor>>().await {
                if let Some(floor) = floors.first() {
                    let min = (floor.landed_tips_25th_percentile * 1_000_000_000.0) as u64;
                    let median = (floor.landed_tips_50th_percentile * 1_000_000_000.0) as u64;
                    let average = (floor.landed_tips_75th_percentile * 1_000_000_000.0) as u64;
                    let max = (floor.landed_tips_95th_percentile * 1_000_000_000.0) as u64;
                    return Ok(TipStats {
                        min,
                        max,
                        median,
                        average,
                    });
                }
            }
            fallback_stats()
        }
        Err(_) => fallback_stats(),
    }
}

fn fallback_stats() -> Result<TipStats> {
    Ok(TipStats {
        min: 10_000,
        max: 100_000,
        median: 50_000,
        average: 70_000,
    })
}

pub struct LeaderTracker {
    rpc_client: RpcClient,
}

impl LeaderTracker {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.to_string()),
        }
    }

    pub fn get_next_jito_leader_slot(&self, current_slot: u64) -> Result<Option<u64>> {
        let leaders = self.rpc_client.get_slot_leaders(current_slot, 20)?;
        for (i, leader) in leaders.iter().enumerate() {
            if is_jito_validator(leader) {
                return Ok(Some(current_slot + i as u64));
            }
        }
        Ok(None)
    }
}

fn is_jito_validator(_pubkey: &Pubkey) -> bool {
    // For demonstration, we assume validators running our test environment are Jito-enabled.
    true
}

#[derive(Debug, Clone)]
pub struct VortexDecision {
    /// Whether the application should use a Jito bundle for the transaction
    pub use_jito: bool,
    /// Explanation of why this decision was made
    pub reason: String,
    /// Recommended tip in lamports if use_jito is true
    pub recommended_tip: Option<u64>,
}

pub struct VortexDecisionMaker {
    rpc_client: RpcClient,
    rpc_url: String,
}

impl VortexDecisionMaker {
    /// Initialize a new VortexDecisionMaker with the provided RPC URL.
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.to_string()),
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Evaluates live network conditions and returns a decision on whether to tip a Jito bundle.
    pub async fn evaluate(&self) -> Result<VortexDecision> {
        let current_slot = self.rpc_client.get_slot()?;

        let leader_tracker = LeaderTracker::new(&self.rpc_url);
        let next_jito = leader_tracker.get_next_jito_leader_slot(current_slot)?;

        let jito_leader_soon = match next_jito {
            Some(slot) => slot.saturating_sub(current_slot) <= 20,
            None => false,
        };

        if !jito_leader_soon {
            return Ok(VortexDecision {
                use_jito: false,
                reason: "No Jito leader in the upcoming 20 slots.".to_string(),
                recommended_tip: None,
            });
        }

        let recent_fees = self.rpc_client.get_recent_prioritization_fees(&[])?;
        let high_congestion = if recent_fees.is_empty() {
            false
        } else {
            let mut fees: Vec<u64> = recent_fees
                .into_iter()
                .map(|f| f.prioritization_fee)
                .collect();
            fees.sort();
            let median_fee = fees[fees.len() / 2];
            median_fee > 10_000
        };

        let tip_stats = fetch_tip_stats(&self.rpc_url)
            .await
            .unwrap_or_else(|_| fallback_stats().unwrap());

        if high_congestion {
            Ok(VortexDecision {
                use_jito: true,
                reason: "Network is congested and a Jito leader is approaching. Use a bundle to ensure landing.".to_string(),
                recommended_tip: Some(tip_stats.median.max(50_000)),
            })
        } else {
            Ok(VortexDecision {
                use_jito: false,
                reason:
                    "Network congestion is low. Normal transactions should land without Jito tips."
                        .to_string(),
                recommended_tip: None,
            })
        }
    }
}
