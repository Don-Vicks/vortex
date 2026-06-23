use crate::lifecycle::{FailureInfo, LifecycleEvent, TxStatus};
use chrono::Utc;

pub fn mark_processed(event: &mut LifecycleEvent, slot: u64) {
    let now = Utc::now();
    event.processed_at = Some(now);
    event.processed_slot = Some(slot);
    event.latency_to_processed_ms =
        Some((now.signed_duration_since(event.submitted_at)).num_milliseconds());
    event.status = TxStatus::Processed;
}

pub fn mark_confirmed(event: &mut LifecycleEvent, slot: u64) {
    let now = Utc::now();
    event.confirmed_at = Some(now);
    event.confirmed_slot = Some(slot);
    event.latency_to_confirmed_ms =
        Some((now.signed_duration_since(event.submitted_at)).num_milliseconds());
    event.status = TxStatus::Confirmed;
}

pub fn mark_finalized(event: &mut LifecycleEvent, slot: u64) {
    let now = Utc::now();
    event.finalized_at = Some(now);
    event.finalized_slot = Some(slot);
    event.latency_to_finalized_ms =
        Some((now.signed_duration_since(event.submitted_at)).num_milliseconds());
    event.status = TxStatus::Finalized;
}

pub fn mark_failed(event: &mut LifecycleEvent, failure: FailureInfo) {
    event.status = TxStatus::Failed;
    event.failure = Some(failure);
}
