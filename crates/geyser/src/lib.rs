pub mod client;
pub mod stream;
pub mod rpc_fallback;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot: u64,
    pub parent: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitmentLevel {
    Processed,
    Confirmed,
    Finalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxConfirmation {
    pub signature: String,
    pub slot: u64,
    pub commitment: CommitmentLevel,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeyserEvent {
    Slot(SlotInfo),
    Tx(TxConfirmation),
}
