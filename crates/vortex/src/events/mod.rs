//! Normalized transaction model produced by the Vortex ingestion pipeline.
//!
//! `VortexTransaction` is the public, transport-agnostic representation of a
//! landed Solana transaction. It is built from Yellowstone gRPC updates by
//! `geyser::decode`, and consumed by anything downstream of Vortex (Sentinel,
//! the lifecycle tracker, ...). Consumers should depend on this model, not on
//! the Geyser proto types.

pub mod logs;
pub mod programs;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexTransaction {
    pub signature: String,
    pub slot: u64,
    /// Position of the transaction inside its block.
    pub index: u64,
    /// When Vortex received the update from the stream.
    pub received_at: DateTime<Utc>,
    pub success: bool,
    pub error: Option<TxError>,
    pub fee: u64,
    pub compute_units: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub compute_unit_price: Option<u64>,
    pub accounts: Vec<AccountRef>,
    pub instructions: Vec<Instruction>,
    pub invocations: Vec<logs::Invocation>,
    pub logs: Vec<String>,
    pub logs_truncated: bool,
    pub token_balances: Vec<TokenBalanceChange>,
    pub transfers: Vec<Transfer>,
    /// Subscription filter names that matched this transaction.
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRef {
    pub pubkey: String,
    pub signer: bool,
    pub writable: bool,
    /// Loaded through an address lookup table rather than the static key list.
    pub from_lookup_table: bool,
    pub pre_lamports: u64,
    pub post_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxError {
    /// Debug rendering of the runtime `TransactionError`.
    pub message: String,
    pub instruction_index: Option<u8>,
    pub custom_code: Option<u32>,
    /// Program that owned the failing top-level instruction.
    pub program_id: Option<String>,
    /// Human-readable error name, when the program logged one (Anchor).
    pub name: Option<String>,
    /// Coarse class from `failures::classifier`.
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// "3" for top-level instruction 3, "3.1" for its second inner instruction.
    pub path: String,
    pub top_index: u16,
    pub inner_index: Option<u16>,
    pub stack_height: u32,
    pub program_id: String,
    pub program_name: Option<String>,
    pub accounts: Vec<String>,
    /// Base58-encoded instruction data.
    pub data: String,
    /// Instruction name from a built-in decoder or from program logs.
    pub name: Option<String>,
    pub parsed: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalanceChange {
    pub account: String,
    pub owner: Option<String>,
    pub mint: String,
    pub decimals: u8,
    pub pre: f64,
    pub post: f64,
    pub delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransferKind {
    Sol,
    Token,
    Mint,
    Burn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub kind: TransferKind,
    /// `None` for native SOL.
    pub mint: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// Wallet that owns `from`/`to` when they are token accounts.
    pub from_owner: Option<String>,
    pub to_owner: Option<String>,
    pub authority: Option<String>,
    pub amount_raw: u64,
    pub decimals: u8,
    pub amount: f64,
    pub instruction: String,
}

impl VortexTransaction {
    /// True when `program_id` is invoked anywhere in the transaction.
    pub fn invokes(&self, program_id: &str) -> bool {
        self.instructions.iter().any(|ix| ix.program_id == program_id)
            || self.invocations.iter().any(|inv| inv.program_id == program_id)
    }

    /// True when `pubkey` is referenced by the transaction at all.
    pub fn touches(&self, pubkey: &str) -> bool {
        self.accounts.iter().any(|a| a.pubkey == pubkey)
    }

    pub fn fee_payer(&self) -> Option<&str> {
        self.accounts.first().map(|a| a.pubkey.as_str())
    }

    pub fn signers(&self) -> impl Iterator<Item = &str> {
        self.accounts.iter().filter(|a| a.signer).map(|a| a.pubkey.as_str())
    }

    /// Compute units consumed by invocations of `program_id` at any depth,
    /// falling back to the whole transaction when logs don't say.
    pub fn compute_for(&self, program_id: &str) -> Option<u64> {
        let from_logs: u64 = self
            .invocations
            .iter()
            .filter(|inv| inv.program_id == program_id && inv.depth == 1)
            .filter_map(|inv| inv.compute_consumed)
            .sum();
        if from_logs > 0 {
            Some(from_logs)
        } else {
            self.compute_units
        }
    }

    /// Instruction names invoked on `program_id` (top-level or CPI).
    pub fn instruction_names_for<'a>(&'a self, program_id: &'a str) -> impl Iterator<Item = &'a str> {
        self.invocations
            .iter()
            .filter(move |inv| inv.program_id == program_id)
            .filter_map(|inv| inv.instruction.as_deref())
    }
}
