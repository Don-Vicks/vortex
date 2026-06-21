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

use crate::jito::tip::TIP_ACCOUNTS;

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
    
    // Create a dummy transfer to simulate real work alongside the tip
    let recipient = keypair.pubkey(); // Send to self to avoid rent-exemption errors
    let transfer_ix = system_instruction::transfer(&keypair.pubkey(), &recipient, 1);

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, tip_ix],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    let tx_bytes = bincode::serialize(&tx).map_err(|e| BundleError::Unknown(e.to_string()))?;
    let encoded_tx = bs58::encode(&tx_bytes).into_string();

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendBundle",
        "params": [
            [encoded_tx]
        ]
    });

    let jito_url = std::env::var("JITO_BLOCK_ENGINE_URL")
        .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf/api/v1/bundles".to_string());
    
    // Make sure we have the correct path if only the host is provided
    let jito_endpoint = if jito_url.ends_with("/api/v1/bundles") {
        jito_url
    } else {
        format!("{}/api/v1/bundles", jito_url.trim_end_matches('/'))
    };

    match client.post(&jito_endpoint).json(&payload).send().await {
        Ok(res) => {
            if res.status().is_success() {
                let body: serde_json::Value = res.json().await.unwrap_or_default();
                if let Some(bundle_id) = body.get("result").and_then(|v| v.as_str()) {
                    let signature = tx.signatures[0].to_string();
                    Ok(BundleResult {
                        bundle_id: bundle_id.to_string(),
                        signature,
                    })
                } else {
                    let err_msg = body.get("error").map(|e| e.to_string()).unwrap_or_default();
                    if err_msg.contains("BlockhashNotFound") || err_msg.contains("expired") {
                        Err(BundleError::BlockhashExpired)
                    } else {
                        Err(BundleError::Unknown(err_msg))
                    }
                }
            } else {
                let err_text = res.text().await.unwrap_or_default();
                Err(BundleError::Unknown(err_text))
            }
        }
        Err(e) => Err(BundleError::Unknown(e.to_string())),
    }
}

/// The Kora Gasless Feepayer Relayer logic.
/// Accepts a set of instructions from an external SDK, injects the relayer as the `fee_payer`,
/// appends the dynamic Jito tip, and submits it.
pub async fn submit_gasless_bundle(
    relayer_keypair: &Keypair,
    mut instructions: Vec<solana_sdk::instruction::Instruction>,
    recent_blockhash: Hash,
    tip_lamports: u64,
    rpc_client: &RpcClient,
) -> Result<BundleResult, BundleError> {
    // 1. Inject the Jito Tip instruction paid for by the Relayer Treasury
    let tip_ix = build_tip_instruction(&relayer_keypair.pubkey(), tip_lamports);
    instructions.push(tip_ix);

    // 2. Construct the transaction, explicitly setting the relayer as the fee payer
    let mut tx = Transaction::new_with_payer(
        &instructions,
        Some(&relayer_keypair.pubkey()),
    );

    // 3. The relayer signs the transaction to authorize fee payment.
    // (Note: The user's partial signature would normally be attached here before submission)
    tx.partial_sign(&[relayer_keypair], recent_blockhash);

    let tx_bytes = bincode::serialize(&tx).map_err(|e| BundleError::Unknown(e.to_string()))?;
    let encoded_tx = bs58::encode(&tx_bytes).into_string();

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendBundle",
        "params": [
            [encoded_tx]
        ]
    });

    let jito_url = std::env::var("JITO_BLOCK_ENGINE_URL")
        .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf/api/v1/bundles".to_string());
    
    let jito_endpoint = if jito_url.ends_with("/api/v1/bundles") {
        jito_url
    } else {
        format!("{}/api/v1/bundles", jito_url.trim_end_matches('/'))
    };

    match client.post(&jito_endpoint).json(&payload).send().await {
        Ok(res) => {
            if res.status().is_success() {
                let body: serde_json::Value = res.json().await.unwrap_or_default();
                if let Some(bundle_id) = body.get("result").and_then(|v| v.as_str()) {
                    let signature = tx.signatures[0].to_string();
                    Ok(BundleResult {
                        bundle_id: bundle_id.to_string(),
                        signature,
                    })
                } else {
                    let err_msg = body.get("error").map(|e| e.to_string()).unwrap_or_default();
                    if err_msg.contains("BlockhashNotFound") || err_msg.contains("expired") {
                        Err(BundleError::BlockhashExpired)
                    } else {
                        Err(BundleError::Unknown(err_msg))
                    }
                }
            } else {
                let err_text = res.text().await.unwrap_or_default();
                Err(BundleError::Unknown(err_text))
            }
        }
        Err(e) => Err(BundleError::Unknown(e.to_string())),
    }
}

