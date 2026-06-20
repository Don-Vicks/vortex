use crate::geyser::{GeyserEvent, SlotInfo};
use anyhow::{Context, Result};
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Polls the Solana RPC for the current slot at a fixed interval.
/// Use this as a fallback when Yellowstone gRPC is unavailable.
pub async fn poll_slots(
    rpc_url: String,
    sender: mpsc::Sender<GeyserEvent>,
    interval: std::time::Duration,
) -> Result<()> {
    info!(rpc_url = %rpc_url, interval_ms = interval.as_millis(), "Starting RPC slot polling fallback");

    let client = reqwest::Client::new();
    let mut last_slot: u64 = 0;

    loop {
        match fetch_slot(&client, &rpc_url).await {
            Ok(slot) => {
                if slot > last_slot {
                    let slot_info = SlotInfo {
                        slot,
                        parent: last_slot,
                        timestamp: Utc::now(),
                    };
                    last_slot = slot;

                    if sender.send(GeyserEvent::Slot(slot_info)).await.is_err() {
                        warn!("Slot receiver dropped, stopping RPC poller");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "RPC slot fetch failed");
            }
        }

        tokio::time::sleep(interval).await;
    }
}

async fn fetch_slot(client: &reqwest::Client, rpc_url: &str) -> Result<u64> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSlot",
        "params": [{ "commitment": "confirmed" }]
    });

    let resp: serde_json::Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("RPC request failed")?
        .json()
        .await
        .context("Failed to parse RPC response")?;

    resp["result"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("No slot in RPC response: {}", resp))
}
