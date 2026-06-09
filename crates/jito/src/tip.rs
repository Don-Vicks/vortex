use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt13yfRqa8",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TipStats {
    pub min: u64,
    pub max: u64,
    pub median: u64,
    pub average: u64,
}

pub fn fetch_tip_stats(rpc_url: &str) -> Result<TipStats> {
    let client = RpcClient::new(rpc_url.to_string());
    let mut balances = Vec::new();

    for addr in JITO_TIP_ACCOUNTS.iter() {
        let pubkey = Pubkey::from_str(addr)?;
        let balance = client.get_balance(&pubkey)?;
        balances.push(balance);
    }

    if balances.is_empty() {
        return Ok(TipStats::default());
    }

    balances.sort_unstable();

    let min = *balances.first().unwrap_or(&0);
    let max = *balances.last().unwrap_or(&0);
    let sum: u64 = balances.iter().sum();
    let average = sum / balances.len() as u64;

    let median = if balances.len() % 2 == 0 {
        let mid = balances.len() / 2;
        (balances[mid - 1] + balances[mid]) / 2
    } else {
        balances[balances.len() / 2]
    };

    Ok(TipStats {
        min,
        max,
        median,
        average,
    })
}
