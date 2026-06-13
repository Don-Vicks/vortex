use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction,
    transaction::Transaction,
    hash::Hash,
};
use std::str::FromStr;

use crate::tip::TIP_ACCOUNTS;

#[derive(Debug)]
pub enum BundleError {
    TipTooLow,
    BlockhashExpired,
    LeaderSkipped,
    Timeout,
    Unknown(String),
}

pub struct BundleResult {
    pub bundle_id: String,
    pub signature: String,
}

pub fn build_tip_instruction(
    payer: &Pubkey,
    tip_lamports: u64,
) -> solana_sdk::instruction::Instruction {
    let tip_account = Pubkey::from_str(TIP_ACCOUNTS[0]).unwrap();
    system_instruction::transfer(payer, &tip_account, tip_lamports)
}

pub async fn submit_bundle(
    keypair: &Keypair,
    recent_blockhash: Hash,
    tip_lamports: u64,
    block_engine_url: &str,
) -> Result<BundleResult, BundleError> {
    let tip_ix = build_tip_instruction(&keypair.pubkey(), tip_lamports);

    let tx = Transaction::new_signed_with_payer(
        &[tip_ix],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    // TODO: Replace with real Jito block engine submission
    // using jito-rust-rpc when available on devnet
    let signature = tx.signatures[0].to_string();
    let bundle_id = format!("bundle-{}", &signature[..8]);

    Ok(BundleResult { bundle_id, signature })
}
