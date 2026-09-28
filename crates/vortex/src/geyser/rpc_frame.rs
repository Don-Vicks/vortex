//! Rebuilds Yellowstone `SubscribeUpdateTransaction` frames from JSON-RPC
//! `getTransaction` responses, so transactions fetched on demand go through
//! the same decoder as the live stream. JSON encoding covers legacy, v0 and
//! v1 messages.

use crate::events::VortexTransaction;
use crate::geyser::decode::decode_transaction;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use solana_sdk::transaction::TransactionError;
use yellowstone_grpc_proto::prelude as proto;

/// Fetches and decodes one transaction. `Ok(None)` if the RPC doesn't have it.
pub async fn fetch_transaction(rpc_url: &str, signature: &str) -> Result<Option<VortexTransaction>> {
    let resp: Value = reqwest::Client::new()
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "getTransaction",
            "params": [signature, {"encoding": "json", "maxSupportedTransactionVersion": 1, "commitment": "confirmed"}]
        }))
        .send()
        .await?
        .json()
        .await?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("getTransaction: {}", err["message"].as_str().unwrap_or("error"));
    }
    let tx = &resp["result"];
    if tx.is_null() {
        return Ok(None);
    }
    let frame = frame_from_rpc_json(signature, tx["slot"].as_u64().unwrap_or(0), tx)?;
    let mut decoded = decode_transaction(frame, vec!["rpc".into()])?;
    if let Some(t) = tx["blockTime"].as_i64().and_then(|t| chrono::DateTime::from_timestamp(t, 0)) {
        decoded.received_at = t;
    }
    Ok(Some(decoded))
}

fn b58(s: &Value) -> Vec<u8> {
    bs58::decode(s.as_str().unwrap_or_default()).into_vec().unwrap_or_default()
}

fn token_balances(v: &Value) -> Vec<proto::TokenBalance> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|b| proto::TokenBalance {
            account_index: b["accountIndex"].as_u64().unwrap_or(0) as u32,
            mint: b["mint"].as_str().unwrap_or_default().into(),
            owner: b["owner"].as_str().unwrap_or_default().into(),
            program_id: b["programId"].as_str().unwrap_or_default().into(),
            ui_token_amount: Some(proto::UiTokenAmount {
                ui_amount: b["uiTokenAmount"]["uiAmount"].as_f64().unwrap_or(0.0),
                decimals: b["uiTokenAmount"]["decimals"].as_u64().unwrap_or(0) as u32,
                amount: b["uiTokenAmount"]["amount"].as_str().unwrap_or("0").into(),
                ui_amount_string: b["uiTokenAmount"]["uiAmountString"].as_str().unwrap_or("0").into(),
            }),
        })
        .collect()
}

/// Rebuilds the Yellowstone frame from a `getTransaction` (json encoding)
/// response. JSON works for legacy, v0 and v1 messages alike.
pub fn frame_from_rpc_json(sig: &str, slot: u64, tx: &Value) -> Result<proto::SubscribeUpdateTransaction> {
    let jm = &tx["transaction"]["message"];
    let h = &jm["header"];
    let n = |v: &Value| v.as_u64().unwrap_or(0) as u32;
    let lookups = jm["addressTableLookups"].as_array().cloned().unwrap_or_default();
    let message = proto::Message {
        header: Some(proto::MessageHeader {
            num_required_signatures: n(&h["numRequiredSignatures"]),
            num_readonly_signed_accounts: n(&h["numReadonlySignedAccounts"]),
            num_readonly_unsigned_accounts: n(&h["numReadonlyUnsignedAccounts"]),
        }),
        account_keys: jm["accountKeys"].as_array().context("keys")?.iter().map(b58).collect(),
        recent_blockhash: b58(&jm["recentBlockhash"]),
        instructions: jm["instructions"]
            .as_array()
            .context("instructions")?
            .iter()
            .map(|ix| proto::CompiledInstruction {
                program_id_index: n(&ix["programIdIndex"]),
                accounts: ix["accounts"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|a| a as u8).collect(),
                data: b58(&ix["data"]),
            })
            .collect(),
        versioned: !tx["version"].is_string(),
        address_table_lookups: lookups
            .iter()
            .map(|l| proto::MessageAddressTableLookup {
                account_key: b58(&l["accountKey"]),
                writable_indexes: l["writableIndexes"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|a| a as u8).collect(),
                readonly_indexes: l["readonlyIndexes"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|a| a as u8).collect(),
            })
            .collect(),
    };
    let signatures = tx["transaction"]["signatures"].as_array().into_iter().flatten().map(b58).collect();

    let m = &tx["meta"];
    let err = if m["err"].is_null() {
        None
    } else {
        let e: TransactionError = serde_json::from_value(m["err"].clone())?;
        Some(proto::TransactionError { err: bincode::serialize(&e)? })
    };
    let u64s = |v: &Value| v.as_array().into_iter().flatten().filter_map(Value::as_u64).collect::<Vec<_>>();
    let meta = proto::TransactionStatusMeta {
        err,
        fee: m["fee"].as_u64().unwrap_or(0),
        pre_balances: u64s(&m["preBalances"]),
        post_balances: u64s(&m["postBalances"]),
        inner_instructions: m["innerInstructions"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|g| proto::InnerInstructions {
                index: g["index"].as_u64().unwrap_or(0) as u32,
                instructions: g["instructions"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|ix| proto::InnerInstruction {
                        program_id_index: ix["programIdIndex"].as_u64().unwrap_or(0) as u32,
                        accounts: u64s(&ix["accounts"]).into_iter().map(|a| a as u8).collect(),
                        data: b58(&ix["data"]),
                        stack_height: ix["stackHeight"].as_u64().map(|s| s as u32),
                    })
                    .collect(),
            })
            .collect(),
        inner_instructions_none: false,
        log_messages: m["logMessages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|l| l.as_str().map(str::to_string))
            .collect(),
        log_messages_none: false,
        pre_token_balances: token_balances(&m["preTokenBalances"]),
        post_token_balances: token_balances(&m["postTokenBalances"]),
        rewards: vec![],
        loaded_writable_addresses: m["loadedAddresses"]["writable"].as_array().into_iter().flatten().map(b58).collect(),
        loaded_readonly_addresses: m["loadedAddresses"]["readonly"].as_array().into_iter().flatten().map(b58).collect(),
        return_data: None,
        return_data_none: true,
        compute_units_consumed: m["computeUnitsConsumed"].as_u64(),
    };

    Ok(proto::SubscribeUpdateTransaction {
        transaction: Some(proto::SubscribeUpdateTransactionInfo {
            signature: bs58::decode(sig).into_vec()?,
            is_vote: false,
            transaction: Some(proto::Transaction {
                signatures,
                message: Some(message),
            }),
            meta: Some(meta),
            index: 0,
        }),
        slot,
    })
}

