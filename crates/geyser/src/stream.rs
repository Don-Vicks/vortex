use crate::SlotInfo;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const INITIAL_BACKOFF_MS: u64 = 500;

pub async fn subscribe_slots(
    mut client: GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: mpsc::Sender<SlotInfo>,
) -> Result<()> {
    let mut reconnect_attempts: u32 = 0;

    loop {
        match try_subscribe(&mut client, &sender).await {
            Ok(()) => {
                // Stream ended cleanly (server closed connection)
                warn!("Geyser slot stream ended. Reconnecting...");
                reconnect_attempts += 1;
            }
            Err(e) => {
                reconnect_attempts += 1;
                error!(
                    attempt = reconnect_attempts,
                    max = MAX_RECONNECT_ATTEMPTS,
                    error = %e,
                    "Geyser stream error"
                );
            }
        }

        if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            error!("Max reconnect attempts ({}) reached. Giving up.", MAX_RECONNECT_ATTEMPTS);
            return Err(anyhow::anyhow!(
                "Geyser stream failed after {} reconnect attempts",
                MAX_RECONNECT_ATTEMPTS
            ));
        }

        let backoff = INITIAL_BACKOFF_MS * 2u64.saturating_pow(reconnect_attempts.min(6));
        warn!(backoff_ms = backoff, "Backing off before reconnect");
        tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
    }
}

async fn try_subscribe(
    client: &mut GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: &mpsc::Sender<SlotInfo>,
) -> Result<()> {
    let mut slots_map = HashMap::new();
    slots_map.insert(
        "client".to_string(),
        SubscribeRequestFilterSlots {
            // Filter to Confirmed commitment — avoids noisy Processed updates
            filter_by_commitment: Some(true),
        },
    );

    let request = SubscribeRequest {
        slots: slots_map,
        // Subscribe at Confirmed commitment level
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    let (_, mut stream) = client.subscribe_with_request(Some(request)).await?;
    info!("Geyser slot subscription active (Confirmed commitment)");

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(subscribe_update::UpdateOneof::Slot(slot)) = msg.update_oneof {
                    let slot_info = SlotInfo {
                        slot: slot.slot,
                        parent: slot.parent.unwrap_or(0),
                        timestamp: Utc::now(),
                    };
                    info!(slot = slot_info.slot, parent = slot_info.parent, "New confirmed slot");

                    if sender.send(slot_info).await.is_err() {
                        warn!("Slot receiver dropped, stopping subscription");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "gRPC stream error (auth failure? rate limit?)");
                return Err(e.into());
            }
        }
    }

    Ok(())
}
