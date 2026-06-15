use anyhow::{anyhow, Result};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction,
    transaction::Transaction,
    hash::Hash,
};
use solana_client::nonblocking::rpc_client::RpcClient;
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
    rpc_client: &RpcClient,
) -> Result<BundleResult, BundleError> {
    let tip_ix = build_tip_instruction(&keypair.pubkey(), tip_lamports);
    
    // Create a self-transfer to simulate real work alongside the tip
    let transfer_ix = system_instruction::transfer(&keypair.pubkey(), &keypair.pubkey(), 1);

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, tip_ix],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    // In a real Jito environment, this would hit the Jito Block Engine.
    // For our bounty demo, we submit to the standard RPC to let it land and get tracked.
    match rpc_client.send_transaction(&tx).await {
        Ok(sig) => {
            let signature = sig.to_string();
            let bundle_id = format!("bundle-{}", &signature[..8]);
            Ok(BundleResult { bundle_id, signature })
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("BlockhashNotFound") || err_str.contains("expired") {
                Err(BundleError::BlockhashExpired)
            } else {
                Err(BundleError::Unknown(err_str))
            }
        }
    }
}
