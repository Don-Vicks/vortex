use chrono::Utc;
use lifecycle::{FailureInfo, FailureType, LifecycleEvent, TxStatus, logger::append_event};

fn main() {
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

    append_event("./logs/lifecycle.json", &event).unwrap();
    println!("Event written successfully");
}
