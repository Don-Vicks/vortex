//! Decoder regression tests against real mainnet Pump.fun transactions
//! (captured with `getTransaction`, rebuilt into Yellowstone frames).

use serde_json::Value;
use solana_vortex::events::{TransferKind, VortexTransaction};
use solana_vortex::geyser::decode::decode_transaction;
use solana_vortex::geyser::rpc_frame::frame_from_rpc_json;

const PUMP: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

fn load(name: &str) -> (Value, VortexTransaction) {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sig = fixture["signature"].as_str().unwrap();
    let resp = &fixture["response"];
    let frame = frame_from_rpc_json(sig, resp["slot"].as_u64().unwrap(), resp).unwrap();
    let tx = decode_transaction(frame, vec![]).unwrap();
    (fixture, tx)
}

fn check_common(fixture: &Value, tx: &VortexTransaction) {
    let meta = &fixture["response"]["meta"];
    assert_eq!(tx.signature, fixture["signature"].as_str().unwrap());
    assert_eq!(tx.success, meta["err"].is_null());
    assert_eq!(tx.accounts.len(), meta["preBalances"].as_array().unwrap().len());
    assert_eq!(tx.compute_units, meta["computeUnitsConsumed"].as_u64());
    assert_eq!(tx.fee, meta["fee"].as_u64().unwrap());
    assert!(tx.invokes(PUMP));
    assert_eq!(tx.fee_payer(), fixture["response"]["transaction"]["message"]["accountKeys"][0].as_str());
    // Every log `invoke` line becomes exactly one invocation.
    let invokes = meta["logMessages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l.as_str().unwrap().contains(" invoke ["))
        .count();
    assert_eq!(tx.invocations.len(), invokes);
    // Top-level + inner instruction counts match the message and meta.
    let top = fixture["response"]["transaction"]["message"]["instructions"].as_array().unwrap().len();
    let inner: usize = meta["innerInstructions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["instructions"].as_array().unwrap().len())
        .sum();
    assert_eq!(tx.instructions.len(), top + inner);
}

#[test]
fn failed_buy_is_attributed_to_pump_with_anchor_error() {
    let (fixture, tx) = load("pump_failed");
    check_common(&fixture, &tx);
    let err = tx.error.as_ref().unwrap();
    assert_eq!(err.custom_code, Some(6002));
    assert_eq!(err.name.as_deref(), Some("TooMuchSolRequired"));
    let deepest = tx
        .invocations
        .iter()
        .filter(|i| i.success == Some(false))
        .max_by_key(|i| i.depth)
        .unwrap();
    assert_eq!(deepest.program_id, PUMP);
    assert_eq!(deepest.instruction.as_deref(), Some("Buy"));
}

#[test]
fn successful_trade_has_token_and_sol_movement() {
    let (fixture, tx) = load("pump_ok");
    check_common(&fixture, &tx);
    assert!(tx.error.is_none());
    assert!(tx.transfers.iter().any(|t| t.kind == TransferKind::Token && t.amount > 0.0));
    assert!(tx.transfers.iter().any(|t| t.kind == TransferKind::Sol && t.amount > 0.0));
    // Token movements resolve token accounts to their owning wallets.
    assert!(tx
        .transfers
        .iter()
        .filter(|t| t.kind == TransferKind::Token)
        .all(|t| t.from_owner.is_some() || t.to_owner.is_some()));
    assert!(!tx.token_balances.is_empty());
}

#[test]
fn version_1_transaction_decodes() {
    let (fixture, tx) = load("pump_v1");
    assert_eq!(fixture["response"]["version"], 1);
    check_common(&fixture, &tx);
}
