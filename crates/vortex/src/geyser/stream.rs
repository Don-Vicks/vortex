use crate::geyser::{
    decode, CommitmentLevel as InternalCommitment, GeyserEvent, SlotInfo, TxConfirmation,
};
use anyhow::Result;
use chrono::Utc;
use futures_util::SinkExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const INITIAL_BACKOFF_MS: u64 = 500;
/// A session that stayed up this long counts as healthy and resets the backoff.
const HEALTHY_SESSION_SECS: u64 = 30;

pub const WALLET_FILTER: &str = "client_txs";
pub const PROGRAM_FILTER: &str = "vortex_programs";

/// What the Geyser subscription should stream. Updating the watch channel
/// re-sends the request over the open subscription without reconnecting.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StreamFilters {
    /// Track Vortex's own submissions (signature-level confirmations).
    pub wallet_pubkey: Option<String>,
    /// Stream fully-decoded transactions touching any of these programs,
    /// including failed ones.
    pub programs: Vec<String>,
}

/// Original entry point: slots plus the sender wallet's confirmations.
pub async fn subscribe_slots(
    client: GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: mpsc::Sender<GeyserEvent>,
    wallet_pubkey: Option<String>,
) -> Result<()> {
    let (_tx, filters) = watch::channel(StreamFilters {
        wallet_pubkey,
        programs: vec![],
    });
    subscribe(client, sender, filters).await
}

/// Slots, wallet confirmations and program transactions on one stream, with
/// filters that can change while it runs.
pub async fn subscribe(
    mut client: GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: mpsc::Sender<GeyserEvent>,
    mut filters: watch::Receiver<StreamFilters>,
) -> Result<()> {
    let mut reconnect_attempts: u32 = 0;

    loop {
        let started = std::time::Instant::now();
        let result = try_subscribe(&mut client, &sender, &mut filters).await;
        if started.elapsed().as_secs() >= HEALTHY_SESSION_SECS {
            reconnect_attempts = 0;
        }

        match result {
            Ok(true) => return Ok(()),
            Ok(false) => {
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

fn build_request(filters: &StreamFilters) -> SubscribeRequest {
    let mut transactions_map = HashMap::new();
    if let Some(pubkey) = &filters.wallet_pubkey {
        transactions_map.insert(
            WALLET_FILTER.to_string(),
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
    if !filters.programs.is_empty() {
        transactions_map.insert(
            PROGRAM_FILTER.to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: None,
                signature: None,
                account_include: filters.programs.clone(),
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

    SubscribeRequest {
        slots: slots_map,
        transactions: transactions_map,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    }
}

/// Returns `Ok(true)` when the receiver is gone and streaming should stop.
async fn try_subscribe(
    client: &mut GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: &mpsc::Sender<GeyserEvent>,
    filters: &mut watch::Receiver<StreamFilters>,
) -> Result<bool> {
    let mut current = filters.borrow_and_update().clone();
    let mut filters_open = true;
    let (mut sink, mut stream) = client
        .subscribe_with_request(Some(build_request(&current)))
        .await?;
    info!(
        tx_tracking = current.wallet_pubkey.is_some(),
        programs = current.programs.len(),
        "Geyser subscription active (Processed commitment)"
    );

    loop {
        tokio::select! {
            changed = filters.changed(), if filters_open => {
                if changed.is_err() {
                    // Filter owner dropped; keep streaming with the last filters.
                    filters_open = false;
                    continue;
                }
                current = filters.borrow_and_update().clone();
                info!(programs = ?current.programs, "Updating Geyser filters");
                sink.send(build_request(&current)).await?;
            }
            message = stream.next() => {
                let Some(message) = message else { return Ok(false) };
                let msg = message.map_err(|e| {
                    error!(error = %e, "gRPC stream error");
                    e
                })?;
                let Some(update) = msg.update_oneof else { continue };
                match update {
                    subscribe_update::UpdateOneof::Slot(slot) => {
                        let slot_info = SlotInfo {
                            slot: slot.slot,
                            parent: slot.parent.unwrap_or(0),
                            timestamp: Utc::now(),
                        };
                        if sender.send(GeyserEvent::Slot(slot_info)).await.is_err() {
                            warn!("Receiver dropped, stopping subscription");
                            return Ok(true);
                        }
                    }
                    subscribe_update::UpdateOneof::Transaction(tx) => {
                        if !forward_transaction(sender, tx, msg.filters).await {
                            warn!("Receiver dropped, stopping subscription");
                            return Ok(true);
                        }
                    }
                    subscribe_update::UpdateOneof::Ping(_) => {
                        // Keep load balancers from closing an idle stream.
                        sink.send(SubscribeRequest {
                            ping: Some(SubscribeRequestPing { id: 1 }),
                            ..Default::default()
                        })
                        .await?;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn forward_transaction(
    sender: &mpsc::Sender<GeyserEvent>,
    tx: SubscribeUpdateTransaction,
    filters: Vec<String>,
) -> bool {
    let for_wallet = filters.iter().any(|f| f == WALLET_FILTER);
    let for_programs = filters.iter().any(|f| f == PROGRAM_FILTER);

    if for_wallet {
        if let Some(info) = tx.transaction.as_ref() {
            let tx_conf = TxConfirmation {
                signature: bs58::encode(&info.signature).into_string(),
                slot: tx.slot,
                commitment: InternalCommitment::Processed, // Emitted at processed level
                timestamp: Utc::now(),
            };
            if sender.send(GeyserEvent::Tx(tx_conf)).await.is_err() {
                return false;
            }
        }
    }

    if for_programs {
        match decode::decode_transaction(tx, filters) {
            Ok(decoded) => {
                if sender
                    .send(GeyserEvent::Transaction(Arc::new(decoded)))
                    .await
                    .is_err()
                {
                    return false;
                }
            }
            Err(e) => debug!(error = %e, "Skipping undecodable transaction"),
        }
    }
    true
}
