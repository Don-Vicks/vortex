//! Streams decoded transactions for one program through the Vortex pipeline.
//!
//!     cargo run -p solana-vortex --example watch_program -- <PROGRAM_ID> [seconds]

use std::sync::Arc;
use tokio::sync::mpsc;
use solana_vortex::geyser::GeyserEvent;
use solana_vortex::hub::VortexHub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let mut args = std::env::args().skip(1);
    let program = args
        .next()
        .unwrap_or_else(|| "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string());
    let secs: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(10);

    let hub = VortexHub::new(None);
    hub.set_programs("example", [program.clone()]);
    let (tx, mut rx) = mpsc::channel(10_000);
    let client = solana_vortex::geyser::client::connect().await?;
    let filters = hub.stream_filters();
    tokio::spawn(async move { solana_vortex::geyser::stream::subscribe(client, tx, filters).await });

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
    let (mut n, mut failed) = (0u64, 0u64);
    while let Ok(Some(event)) = tokio::time::timeout_at(deadline, rx.recv()).await {
        if let GeyserEvent::Transaction(t) = event {
            n += 1;
            if !t.success {
                failed += 1;
            }
            if n <= 5 || !t.success && failed <= 3 {
                let names: Vec<_> = t.instruction_names_for(&program).collect();
                println!(
                    "{} slot={} ok={} cu={:?} ixs={:?} transfers={} err={:?}",
                    &t.signature[..16],
                    t.slot,
                    t.success,
                    t.compute_for(&program),
                    names,
                    t.transfers.len(),
                    t.error.as_ref().and_then(|e| e.name.clone())
                );
            }
            hub.publish(Arc::clone(&t));
        }
    }
    println!("{n} transactions ({failed} failed) in {secs}s");
    Ok(())
}
