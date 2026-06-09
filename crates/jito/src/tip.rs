use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::fmt;

/// The 8 official Jito tip account addresses used for transaction bundling.
/// These are validated at compile-time using the `pubkey!` macro.
pub const JITO_TIP_ACCOUNTS: [Pubkey; 8] = [
    pubkey!("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"),
    pubkey!("HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe"),
    pubkey!("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY"),
    pubkey!("ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt13yfRqa8"),
    pubkey!("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh"),
    pubkey!("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"),
    pubkey!("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
    pubkey!("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT"),
];

/// Statistics for Jito tips, measured in lamports.
/// Includes percentiles for granular competitive analysis.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct TipStats {
    pub min: u64,
    pub max: u64,
    pub average: u64,
    pub p25: u64,
    pub p50: u64, // Median
    pub p75: u64,
    pub p95: u64,
    pub sample_size: usize,
}

impl TipStats {
    /// Utility to convert lamports to SOL.
    pub fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }

    /// Returns the average tip in SOL.
    pub fn average_sol(&self) -> f64 {
        Self::lamports_to_sol(self.average)
    }

    /// Calculates statistics from a raw list of balances.
    pub fn from_balances(mut balances: Vec<u64>) -> Result<Self> {
        let sample_size = balances.len();
        if sample_size == 0 {
            return Err(anyhow!("Cannot calculate stats from empty balances"));
        }

        balances.sort_unstable();

        let min = balances[0];
        let max = balances[sample_size - 1];
        let sum: u64 = balances.iter().sum();
        let average = sum / sample_size as u64;

        // Percentile helper: index = (p/100) * (N-1)
        let get_p = |p: f64| -> u64 {
            let idx = (p / 100.0 * (sample_size - 1) as f64).round() as usize;
            balances[idx.min(sample_size - 1)]
        };

        Ok(Self {
            min,
            max,
            average,
            p25: get_p(25.0),
            p50: get_p(50.0),
            p75: get_p(75.0),
            p95: get_p(95.0),
            sample_size,
        })
    }
}

impl fmt::Display for TipStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Jito Tip Stats (N={}):\n\
               Min:     {:>15} ({:.6} SOL)\n\
               P25:     {:>15} ({:.6} SOL)\n\
               Median:  {:>15} ({:.6} SOL)\n\
               P75:     {:>15} ({:.6} SOL)\n\
               P95:     {:>15} ({:.6} SOL)\n\
               Max:     {:>15} ({:.6} SOL)\n\
               Average: {:>15} ({:.6} SOL)",
            self.sample_size,
            self.min, Self::lamports_to_sol(self.min),
            self.p25, Self::lamports_to_sol(self.p25),
            self.p50, Self::lamports_to_sol(self.p50),
            self.p75, Self::lamports_to_sol(self.p75),
            self.p95, Self::lamports_to_sol(self.p95),
            self.max, Self::lamports_to_sol(self.max),
            self.average, Self::lamports_to_sol(self.average)
        )
    }
}

/// Fetches current balances for Jito tip accounts using the provided RPC client.
/// Uses a single batch request for efficiency.
pub fn fetch_tip_stats(client: &RpcClient) -> Result<TipStats> {
    let accounts = client
        .get_multiple_accounts(&JITO_TIP_ACCOUNTS)
        .map_err(|e| anyhow!("Failed to fetch tip accounts: {}", e))?;

    let balances: Vec<u64> = accounts
        .into_iter()
        .flatten()
        .map(|acc| acc.lamports)
        .collect();

    TipStats::from_balances(balances)
}

/// Helper to fetch stats by RPC URL (creates a temporary client).
pub fn fetch_tip_stats_from_url(rpc_url: &str) -> Result<TipStats> {
    let client = RpcClient::new(rpc_url.to_string());
    fetch_tip_stats(&client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_calculation() {
        let balances = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let stats = TipStats::from_balances(balances).unwrap();
        
        assert_eq!(stats.min, 10);
        assert_eq!(stats.max, 100);
        assert_eq!(stats.average, 55);
        assert_eq!(stats.sample_size, 10);
        // p25: idx = 0.25 * 9 = 2.25 -> 2. balances[2] = 30
        assert_eq!(stats.p25, 30);
        // p50: idx = 0.50 * 9 = 4.5 -> 5. balances[5] = 60
        assert_eq!(stats.p50, 60);
        // p75: idx = 0.75 * 9 = 6.75 -> 7. balances[7] = 80
        assert_eq!(stats.p75, 80);
    }

    #[test]
    fn test_empty_balances() {
        let res = TipStats::from_balances(vec![]);
        assert!(res.is_err());
    }
}
