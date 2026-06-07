use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub mod logger;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TxStatus {
    Pending,
    Processed,
    Confirmed,
    Finalized,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FailureType {
    ExpiredBlockhash,
    FeeTooLow,
    ComputeExceeded,
    BundleFailure,
    LeaderSkipped,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailureInfo {
    pub error_type: FailureType,
    pub raw_error: String,
    pub retry_count: u32,
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LifecycleEvent {
    pub id: String,
    pub bundle_id: String,
    pub signature: String,
    pub tip_lamports: u64,
    pub tip_reasoning: String,
    pub submitted_at: DateTime<Utc>,
    pub submitted_slot: u64,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_slot: Option<u64>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub confirmed_slot: Option<u64>,
    pub finalized_at: Option<DateTime<Utc>>,
    pub finalized_slot: Option<u64>,
    pub latency_to_processed_ms: Option<i64>,
    pub latency_to_confirmed_ms: Option<i64>,
    pub latency_to_finalized_ms: Option<i64>,
    pub status: TxStatus,
    pub failure: Option<FailureInfo>,
}
