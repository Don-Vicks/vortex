use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

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

pub const TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt13yfRqa8",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

pub async fn fetch_tip_stats(_rpc_url: &str) -> Result<TipStats> {
    let client = Client::new();
    let response = client
        .get("https://mainnet.block-engine.jito.wtf/api/v1/bundles/tip_floor")
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
    // Sane defaults for high-performance Jito tips
    // Min: 0.00001 SOL, Median: 0.00005 SOL, Avg: 0.00007 SOL, Max: 0.0001 SOL
    Ok(TipStats {
        min: 10_000,
        max: 100_000,
        median: 50_000,
        average: 70_000,
    })
}
