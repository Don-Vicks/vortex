use chrono::Utc;
use lifecycle::{LifecycleEvent, TxStatus, logger::append_event};
use jito::fetch_tip_stats;

fn main() {
    println!("Fetching Jito tip stats from devnet...");
    
    // Using devnet RPC for testing as requested
    let devnet_rpc = "https://api.devnet.solana.com";
    
    match fetch_tip_stats(devnet_rpc) {
        Ok(stats) => {
            println!("Jito Tip Stats (in lamports):");
            println!("  Min:    {}", stats.min);
            println!("  Max:    {}", stats.max);
            println!("  Median: {}", stats.median);
            println!("  Average:{}", stats.average);
        }
        Err(e) => {
            eprintln!("Error fetching Jito tip stats: {}", e);
        }
    }

    let event = LifecycleEvent {
        id: uuid::Uuid::new_v4().to_string(),
        bundle_id: "test-bundle-001".to_string(),
        signature: "test-signature-abc".to_string(),
        tip_lamports: 5000,
        tip_reasoning: "Test entry — network looked calm".to_string(),
        submitted_at: Utc::now(),
        submitted_slot: 280000000,
        processed_at: None,
        processed_slot: None,
        confirmed_at: None,
        confirmed_slot: None,
        finalized_at: None,
        finalized_slot: None,
        latency_to_processed_ms: None,
        latency_to_confirmed_ms: None,
        latency_to_finalized_ms: None,
        status: TxStatus::Pending,
        failure: None,
    };

    // Ensure logs directory exists
    std::fs::create_dir_all("./logs").unwrap_or_default();

    match append_event("./logs/lifecycle.json", &event) {
        Ok(_) => println!("Event written successfully"),
        Err(e) => eprintln!("Failed to write event: {}", e),
    }
}
