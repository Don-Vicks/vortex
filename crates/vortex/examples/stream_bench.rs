//! Measures what a Yellowstone subscription delivers under different HTTP/2 settings:
//! `cargo run --release --example stream_bench -- <default|windows|gzip> [seconds] [program,...]`
//! Needs YELLOWSTONE_ENDPOINT and YELLOWSTONE_TOKEN. Prints messages/s, MB/s, the largest
//! message, and whether the server closed the stream.

use futures_util::StreamExt;
use prost::Message;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;
use yellowstone_grpc_proto::tonic::codec::CompressionEncoding;
use yellowstone_grpc_proto::tonic::transport::ClientTlsConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "default".into());
    let secs: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(45);
    let programs: Vec<String> = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P,JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".into())
        .split(',')
        .map(String::from)
        .collect();

    let endpoint = std::env::var("YELLOWSTONE_ENDPOINT")?;
    let token = std::env::var("YELLOWSTONE_TOKEN")?;
    let mut b = GeyserGrpcClient::build_from_shared(endpoint)?
        .x_token(Some(token))?
        .tls_config(ClientTlsConfig::new())?
        .connect_timeout(Duration::from_secs(10));
    match mode.as_str() {
        "windows" | "gzip" => {
            b = b
                .initial_stream_window_size(16 * 1024 * 1024)
                .initial_connection_window_size(32 * 1024 * 1024)
                .http2_adaptive_window(true);
        }
        _ => {}
    }
    match mode.as_str() {
        "gzip" => b = b.accept_compressed(CompressionEncoding::Gzip),
        _ => {}
    }
    let mut client = b.connect().await?;

    let mut txs = HashMap::new();
    txs.insert(
        "bench".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: None,
            signature: None,
            account_include: programs,
            account_exclude: vec![],
            account_required: vec![],
        },
    );
    let mut slots = HashMap::new();
    slots.insert("s".to_string(), SubscribeRequestFilterSlots { filter_by_commitment: Some(true) });
    let (_sink, mut stream) = client
        .subscribe_with_request(Some(SubscribeRequest {
            slots,
            transactions: txs,
            commitment: Some(CommitmentLevel::Processed as i32),
            ..Default::default()
        }))
        .await?;

    let start = Instant::now();
    let (mut n_tx, mut n_slot, mut bytes, mut biggest) = (0u64, 0u64, 0u64, 0usize);
    let mut closed = "still open";
    let mut newest_slot = 0u64;
    let mut longest_gap = Duration::ZERO;
    let mut last = Instant::now();
    while start.elapsed() < Duration::from_secs(secs) {
        match tokio::time::timeout(Duration::from_secs(10), stream.next()).await {
            Err(_) => {
                closed = "silent for 10s";
                break;
            }
            Ok(None) => {
                closed = "closed by server";
                break;
            }
            Ok(Some(Err(e))) => {
                println!("error: {e}");
                closed = "error";
                break;
            }
            Ok(Some(Ok(m))) => {
                longest_gap = longest_gap.max(last.elapsed());
                last = Instant::now();
                let len = m.encoded_len();
                bytes += len as u64;
                biggest = biggest.max(len);
                match m.update_oneof {
                    Some(subscribe_update::UpdateOneof::Transaction(t)) => {
                        n_tx += 1;
                        newest_slot = newest_slot.max(t.slot);
                    }
                    Some(subscribe_update::UpdateOneof::Slot(sl)) => {
                        n_slot += 1;
                        newest_slot = newest_slot.max(sl.slot);
                    }
                    _ => {}
                }
            }
        }
    }
    let t = start.elapsed().as_secs_f64();
    // Freshness: how far the newest streamed slot is behind the chain right now.
    let tip = client.get_slot(Some(CommitmentLevel::Processed)).await.map(|r| r.slot).unwrap_or(0);
    let behind = tip.saturating_sub(newest_slot);
    println!(
        "{mode:>8}: {:>5.0} tx/s  {:>4.1} slots/s  {:>5.2} MB/s  avg {:>5} B/tx  max {:>6} B  longest gap {:>4}ms  BEHIND CHAIN {:>4} slots (~{:.1}s)  after {:>4.1}s: {closed}",
        n_tx as f64 / t,
        n_slot as f64 / t,
        bytes as f64 / t / 1e6,
        if n_tx > 0 { bytes / (n_tx + n_slot).max(1) } else { 0 },
        biggest,
        longest_gap.as_millis(),
        behind,
        behind as f64 * 0.4,
        t
    );
    Ok(())
}
