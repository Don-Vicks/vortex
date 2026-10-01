//! Solami Mirage: the Yellowstone stream over a plain WebSocket.
//!
//! Mirage sends the same `SubscribeUpdate` frames as gRPC, byte for byte, so
//! they go through the same decoder as the gRPC transport. The filter lives
//! server-side in a saved Mirage subscription (`account_include`, `failed`,
//! `commitment`), so unlike gRPC it can't follow the hub's live filter changes.
//! Use it as a failover for the programs that subscription names.

use crate::geyser::stream::{forward_transaction, PROGRAM_FILTER};
use crate::geyser::{GeyserEvent, SlotInfo};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures_util::StreamExt;
use prost::Message as _;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};
use yellowstone_grpc_proto::prelude::*;

/// Connects to a Mirage stream URL (`wss://ws.solami.dev/mirage/stream/{id}?api_key=…`)
/// and forwards decoded transactions and slots until the socket closes.
/// Returns `Ok(true)` when the receiver is gone and streaming should stop.
pub async fn session(url: &str, sender: &mpsc::Sender<GeyserEvent>) -> Result<bool> {
    // The URL carries the API key; keep it out of logs and error text.
    let host = url.split("://").nth(1).and_then(|r| r.split(['/', '?']).next()).unwrap_or("mirage");
    let (mut socket, _) = connect_async(url)
        .await
        .map_err(|e| anyhow!("Mirage connect to {host} failed: {}", scrub(&e.to_string(), url)))?;
    info!(host, "Mirage stream connected");

    while let Some(message) = socket.next().await {
        match message.context("Mirage socket error")? {
            Message::Binary(bytes) => match SubscribeUpdate::decode(bytes.as_slice()) {
                Ok(update) => {
                    if !handle(update, sender).await {
                        return Ok(true);
                    }
                }
                Err(e) => debug!(error = %e, "Skipping undecodable Mirage frame"),
            },
            Message::Close(frame) => {
                // 4029 = concurrent-stream limit, 4002 = out of bandwidth and balance, 1001 = node restarting.
                warn!(?frame, "Mirage closed the stream");
                return Ok(false);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn scrub(text: &str, url: &str) -> String {
    match url.split("api_key=").nth(1).map(|k| k.split('&').next().unwrap_or(k)) {
        Some(key) if !key.is_empty() => text.replace(key, "***"),
        _ => text.to_string(),
    }
}

/// `false` when the receiver has gone.
async fn handle(update: SubscribeUpdate, sender: &mpsc::Sender<GeyserEvent>) -> bool {
    match update.update_oneof {
        Some(subscribe_update::UpdateOneof::Slot(slot)) => sender
            .send(GeyserEvent::Slot(SlotInfo {
                slot: slot.slot,
                parent: slot.parent.unwrap_or(0),
                timestamp: Utc::now(),
            }))
            .await
            .is_ok(),
        // Mirage names its own filters; everything it sends is a subscribed transaction.
        Some(subscribe_update::UpdateOneof::Transaction(tx)) => {
            forward_transaction(sender, tx, vec![PROGRAM_FILTER.to_string()]).await
        }
        _ => true,
    }
}
