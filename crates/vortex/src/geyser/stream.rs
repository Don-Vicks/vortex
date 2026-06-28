use crate::geyser::{CommitmentLevel as InternalCommitment, GeyserEvent, SlotInfo, TxConfirmation};
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
    sender: mpsc::Sender<GeyserEvent>,
    wallet_pubkey: Option<String>,
) -> Result<()> {
    let mut reconnect_attempts: u32 = 0;

    loop {
        match try_subscribe(&mut client, &sender, &wallet_pubkey).await {
            Ok(()) => {
                // Stream ended cleanly (server closed connection)
                warn!("Geyser stream ended. Reconnecting...");
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
            error!(
                "Max reconnect attempts ({}) reached. Giving up.",
                MAX_RECONNECT_ATTEMPTS
            );
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
    sender: &mpsc::Sender<GeyserEvent>,
    wallet_pubkey: &Option<String>,
) -> Result<()> {
    let mut transactions_map = HashMap::new();
    if let Some(pubkey) = wallet_pubkey {
        transactions_map.insert(
            "client_txs".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec![pubkey.clone()],
                account_exclude: vec![],
                account_required: vec![],
            },
        );
    }

    let mut slots_map = HashMap::new();
    slots_map.insert(
        "client_slots".to_string(),
        SubscribeRequestFilterSlots {
            filter_by_commitment: Some(true),
        },
    );

    let request = SubscribeRequest {
        slots: slots_map,
        transactions: transactions_map,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };

    let (_, mut stream) = client.subscribe_with_request(Some(request)).await?;
    info!(
        "Geyser subscription active (Processed commitment, tx_tracking={})",
        wallet_pubkey.is_some()
    );

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(update_oneof) = msg.update_oneof {
                    match update_oneof {
                        subscribe_update::UpdateOneof::Slot(slot) => {
                            let slot_info = SlotInfo {
                                slot: slot.slot,
                                parent: slot.parent.unwrap_or(0),
                                timestamp: Utc::now(),
                            };
                            if sender.send(GeyserEvent::Slot(slot_info)).await.is_err() {
                                warn!("Receiver dropped, stopping subscription");
                                return Ok(());
                            }
                        }
                        subscribe_update::UpdateOneof::Transaction(tx) => {
                            let sig = bs58::encode(&tx.transaction.as_ref().unwrap().signature)
                                .into_string();
                            let tx_conf = TxConfirmation {
                                signature: sig,
                                slot: tx.slot,
                                commitment: InternalCommitment::Processed, // Emitted at processed level
                                timestamp: Utc::now(),
                            };
                            if sender.send(GeyserEvent::Tx(tx_conf)).await.is_err() {
                                warn!("Receiver dropped, stopping subscription");
                                return Ok(());
                            }
                        }
                        _ => {} // Ignore other updates like blocks/ping
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "gRPC stream error");
                return Err(e.into());
            }
        }
    }

    Ok(())
}
