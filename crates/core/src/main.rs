use anyhow::Result;
use chrono::Utc;
use dotenv::dotenv;
use vortex::lifecycle::{
    logger::{append_event, read_events},
    LifecycleEvent, TxStatus,
};
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let rpc_url = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let log_path = env::var("LOG_FILE_PATH")
        .unwrap_or_else(|_| "./logs/lifecycle.json".to_string());

    // Auto-create log directory
    if let Some(parent) = std::path::Path::new(&log_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Shared slot tracker — written by geyser/rpc task, read by main loop
    let current_slot = Arc::new(AtomicU64::new(0));

    // Set up Geyser event streaming channel
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<vortex::geyser::GeyserEvent>(100);

    // Spawn event updater: reads from channel, writes to AtomicU64 and processes Tx confirmations
    let slot_writer = Arc::clone(&current_slot);
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                vortex::geyser::GeyserEvent::Slot(slot_info) => {
                    slot_writer.store(slot_info.slot, Ordering::Relaxed);
                }
                vortex::geyser::GeyserEvent::Tx(tx_conf) => {
                    tracing::info!(
                        signature = tx_conf.signature,
                        slot = tx_conf.slot,
                        commitment = ?tx_conf.commitment,
                        "Transaction Stream Event Received"
                    );
                    // In Phase 4, we will append this exact timestamp to the lifecycle log to calculate processing latency.
                }
            }
        }
    });

    // Try Yellowstone gRPC first, fall back to RPC polling
    let event_tx_clone = event_tx.clone();
    let rpc_url_clone = rpc_url.clone();
    
    // For Phase 2, we fetch the sender's pubkey so Geyser can filter our transactions
    let wallet_pubkey = env::var("SENDER_PUBKEY").ok();

    tokio::spawn(async move {
        let geyser_failed = match vortex::geyser::client::connect().await {
            Ok(client) => {
                tracing::info!("Yellowstone gRPC connected — using live geyser stream");
                match vortex::geyser::stream::subscribe_slots(client, event_tx_clone.clone(), wallet_pubkey).await {
                    Ok(()) => false,
                    Err(e) => {
                        tracing::warn!("Geyser stream terminated: {}. Falling back to RPC polling.", e);
                        true
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Yellowstone gRPC unavailable: {}. Falling back to RPC polling.", e);
                true
            }
        };

        if geyser_failed {
            if let Err(e) = vortex::geyser::rpc_fallback::poll_slots(
                rpc_url_clone,
                event_tx_clone,
                std::time::Duration::from_millis(400),
            )
            .await
            {
                tracing::error!("RPC slot polling also terminated: {}", e);
            }
        }
    });

    // Wait for first slot with a generous timeout
    tracing::info!("Waiting for first slot update (up to 15s)...");
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if current_slot.load(Ordering::Relaxed) > 0 {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!("Slot timeout — proceeding with slot 0. Tip decisions may be degraded.");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let first_slot = current_slot.load(Ordering::Relaxed);
    tracing::info!(slot = first_slot, "Slot tracking active");

    // Set up graceful shutdown
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_notify = Arc::new(tokio::sync::Notify::new());
    let flag_clone = Arc::clone(&shutdown_flag);
    let notify_clone = Arc::clone(&shutdown_notify);
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        tracing::info!("Shutdown signal received. Finishing current cycle...");
        flag_clone.store(true, Ordering::Relaxed);
        notify_clone.notify_one();
    });

    // Build agent config
    let provider = env::var("AI_PROVIDER")
        .unwrap_or_else(|_| "anthropic".to_string())
        .to_lowercase();
    let provider = match provider.as_str() {
        "openai" => vortex::agent::AiProvider::OpenAI,
        "gemini" => vortex::agent::AiProvider::Gemini,
        "grok" => vortex::agent::AiProvider::Grok,
        _ => vortex::agent::AiProvider::Anthropic,
    };
    let model = env::var("AI_MODEL").unwrap_or_else(|_| match provider {
        vortex::agent::AiProvider::Anthropic => "claude-3-sonnet-20240229".to_string(),
        vortex::agent::AiProvider::OpenAI => "gpt-4-turbo-preview".to_string(),
        vortex::agent::AiProvider::Gemini => "gemini-2.0-flash".to_string(),
        vortex::agent::AiProvider::Grok => "grok-1".to_string(),
    });
    let agent_config = vortex::agent::AgentConfig { provider, model };

    // Initialize RPC Client and Keypair
    let rpc_client = RpcClient::new(rpc_url.clone());
    let keypair_path = env::var("WALLET_KEYPAIR_PATH").unwrap_or_else(|_| "./keypair.json".to_string());
    let keypair = read_keypair_file(&keypair_path).expect("Failed to read wallet keypair. Make sure keypair.json exists.");
    let mut iteration_count = 0;

    // Initialize Leader Tracker
    let leader_tracker = vortex::jito::leader::LeaderTracker::new(&rpc_url);

    // === Main Loop ===
    tracing::info!("Entering main evaluation loop. Press Ctrl+C to stop.");

    loop {
        // Check for shutdown
        if shutdown_flag.load(Ordering::Relaxed) {
            tracing::info!("Shutting down gracefully.");
            break;
        }

        let live_slot = current_slot.load(Ordering::Relaxed);

        // Phase 1: Detect Jito Leader Window
        let next_jito_slot = match leader_tracker.get_next_jito_leader_slot(live_slot) {
            Ok(Some(slot)) => slot,
            Ok(None) => {
                tracing::warn!("No Jito leaders found in the next 20 slots. Waiting...");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
            Err(e) => {
                tracing::error!("Failed to fetch leader schedule: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        if next_jito_slot > live_slot + 5 {
            // Jito leader is too far away, sleep for a bit to avoid burning RPC limits
            tracing::debug!("Next Jito leader is at slot {}, sleeping...", next_jito_slot);
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            continue;
        }

        tracing::info!("Jito leader window detected upcoming at slot: {}", next_jito_slot);

        // Fetch tip stats from Jito
        let tip_stats = match vortex::jito::tip::fetch_tip_stats(&rpc_url).await {
            Ok(stats) => stats,
            Err(e) => {
                tracing::error!("Failed to fetch tip stats: {}. Retrying in 5s...", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        tracing::info!(
            min = tip_stats.min,
            median = tip_stats.median,
            avg = tip_stats.average,
            max = tip_stats.max,
            "Tip floor stats"
        );

        // Read recent events for failure rate
        let recent_events = read_events(&log_path).unwrap_or_default();
        let recent_10: Vec<_> = recent_events.iter().rev().take(10).collect();
        let failure_count = recent_10
            .iter()
            .filter(|e| matches!(e.status, TxStatus::Failed))
            .count();
        let failure_rate = if recent_10.is_empty() {
            0.0
        } else {
            (failure_count as f64 / recent_10.len() as f64) * 100.0
        };
        let time_since_last_success = recent_events
            .iter()
            .rev()
            .find(|e| matches!(e.status, TxStatus::Finalized))
            .map(|e| (Utc::now() - e.submitted_at).num_seconds().max(0) as u64)
            .unwrap_or(0);

        // Build network state with LIVE slot
        let network_state = vortex::agent::NetworkState {
            current_slot: live_slot,
            tip_min: tip_stats.min,
            tip_max: tip_stats.max,
            tip_median: tip_stats.median,
            tip_average: tip_stats.average,
            recent_failure_rate: failure_rate,
            time_since_last_success_secs: time_since_last_success,
        };

        // Get agent tip decision
        let tip_decision =
            match vortex::agent::decisions::decide_tip(&agent_config, &network_state).await {
                Ok(decision) => {
                    tracing::info!(
                        lamports = decision.recommended_lamports,
                        confidence = %decision.confidence,
                        reasoning = %decision.reasoning,
                        "Agent tip decision"
                    );
                    decision
                }
                Err(e) => {
                    tracing::warn!("Agent failed: {}. Using fallback.", e);
                    vortex::agent::decisions::fallback_tip(tip_stats.median)
                }
            };

        // Fetch real blockhash and submit bundle
        iteration_count += 1;
        let mut recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap_or_default();
        
        // Phase 3: Failure Injection Loop
        if iteration_count % 3 == 0 {
            tracing::warn!("FAILURE INJECTION: Intentionally using a dummy blockhash for bundle submission!");
            recent_blockhash = solana_sdk::hash::Hash::default();
        }

        let bundle_result = vortex::jito::bundle::submit_bundle(
            &keypair,
            recent_blockhash,
            tip_decision.recommended_lamports,
            &rpc_client
        ).await;

        let (bundle_id, signature, status, failure) = match bundle_result {
            Ok(res) => (
                res.bundle_id,
                res.signature,
                TxStatus::Pending,
                None
            ),
            Err(e) => {
                let error_type = vortex::failures::classifier::classify(&format!("{:?}", e));
                let fail_info = vortex::lifecycle::FailureInfo {
                    error_type,
                    raw_error: format!("{:?}", e),
                    retry_count: 0,
                    resolved: false,
                };
                (
                    format!("failed-bundle-{}", &Uuid::new_v4().to_string()[..8]),
                    format!("failed-sig-{}", &Uuid::new_v4().to_string()[..8]),
                    TxStatus::Failed,
                    Some(fail_info)
                )
            }
        };

        // Create and log lifecycle event
        let mut event = LifecycleEvent {
            id: Uuid::new_v4().to_string(),
            bundle_id,
            signature,
            tip_lamports: tip_decision.recommended_lamports,
            tip_reasoning: tip_decision.reasoning,
            submitted_at: Utc::now(),
            submitted_slot: live_slot,
            processed_at: None,
            processed_slot: None,
            confirmed_at: None,
            confirmed_slot: None,
            finalized_at: None,
            finalized_slot: None,
            latency_to_processed_ms: None,
            latency_to_confirmed_ms: None,
            latency_to_finalized_ms: None,
            status: status.clone(),
            failure: failure.clone(),
        };

        if let Err(e) = append_event(&log_path, &event) {
            tracing::error!("Failed to log event: {}", e);
        } else {
            tracing::info!(id = %event.id, slot = live_slot, "Event logged");
        }

        // Autonomous retry loop (Phase 3)
        if matches!(status, TxStatus::Failed) {
            if let Some(f) = failure {
                tracing::warn!("Agent analyzing failure autonomously: {}", f.raw_error);
                if let Ok(analysis) = vortex::agent::decisions::analyze_failure(&agent_config, &f.raw_error).await {
                    tracing::info!(cause = %analysis.cause, action = %analysis.action, "Agent failure analysis complete");
                    
                    if analysis.action == "refresh_blockhash" || analysis.action == "increase_tip" {
                        let new_tip = (event.tip_lamports as f64 * analysis.suggested_tip_multiplier) as u64;
                        let fresh_blockhash = rpc_client.get_latest_blockhash().await.unwrap_or_default();
                        
                        tracing::info!("Agent autonomously resubmitting with fresh blockhash and tip {}", new_tip);
                        let retry_res = vortex::jito::bundle::submit_bundle(
                            &keypair,
                            fresh_blockhash,
                            new_tip,
                            &rpc_client
                        ).await;
                        
                        let mut retry_event = event.clone();
                        retry_event.id = Uuid::new_v4().to_string();
                        retry_event.submitted_at = Utc::now();
                        retry_event.tip_lamports = new_tip;
                        retry_event.tip_reasoning = format!("Autonomous recovery. Action: {}, Cause: {}", analysis.action, analysis.cause);
                        
                        match retry_res {
                            Ok(res) => {
                                retry_event.bundle_id = res.bundle_id;
                                retry_event.signature = res.signature;
                                retry_event.status = TxStatus::Pending;
                                retry_event.failure = None;
                                tracing::info!("Autonomous retry successful!");
                            }
                            Err(e) => {
                                retry_event.status = TxStatus::Failed;
                                let error_type = vortex::failures::classifier::classify(&format!("{:?}", e));
                                retry_event.failure = Some(vortex::lifecycle::FailureInfo {
                                    error_type,
                                    raw_error: format!("{:?}", e),
                                    retry_count: 1,
                                    resolved: false,
                                });
                                tracing::error!("Autonomous retry failed again: {:?}", e);
                            }
                        }
                        if let Err(e) = append_event(&log_path, &retry_event) {
                            tracing::error!("Failed to log retry event: {}", e);
                        }
                    }
                }
            }
        }

        // Wait before next cycle (or until shutdown)
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {},
            _ = shutdown_notify.notified() => {
                tracing::info!("Shutting down gracefully.");
                break;
            }
        }
    }

    tracing::info!("Solana TX Stack stopped.");
    Ok(())
}
