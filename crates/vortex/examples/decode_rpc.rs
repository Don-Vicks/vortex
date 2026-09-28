//! Validates `geyser::decode` against real mainnet transactions without a
//! gRPC key: fetches transactions over JSON-RPC, rebuilds the exact
//! Yellowstone `SubscribeUpdateTransaction` frame, and decodes it.
//!
//!     cargo run -p solana-vortex --example decode_rpc -- [PROGRAM_ID] [COUNT]

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use solana_vortex::geyser::decode::decode_transaction;
use solana_vortex::geyser::rpc_frame::frame_from_rpc_json;

fn rpc_url() -> String {
    std::env::var("DECODE_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into())
}

async fn call(client: &reqwest::Client, method: &str, params: Value) -> Result<Value> {
    for attempt in 0..5 {
        let resp: Value = client
            .post(rpc_url())
            .json(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
            .send()
            .await?
            .json()
            .await?;
        if let Some(r) = resp.get("result") {
            return Ok(r.clone());
        }
        tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1))).await;
    }
    Err(anyhow!("{method} kept failing"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let program = args.next().unwrap_or_else(|| "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into());
    let count: usize = args.next().and_then(|c| c.parse().ok()).unwrap_or(12);
    let client = reqwest::Client::new();

    let sigs = call(&client, "getSignaturesForAddress", json!([program, {"limit": 200}])).await?;
    let sigs = sigs.as_array().context("sigs")?;
    // Prefer a mix of failed and successful transactions.
    let failed = sigs.iter().filter(|s| !s["err"].is_null()).take(count / 2);
    let ok = sigs.iter().filter(|s| s["err"].is_null()).take(count - count / 2);
    let (mut decoded, mut mismatches) = (0, 0);

    for s in failed.chain(ok) {
        let sig = s["signature"].as_str().unwrap_or_default();
        let tx = call(&client, "getTransaction", json!([sig, {"encoding": "json", "maxSupportedTransactionVersion": 1}])).await?;
        if tx.is_null() {
            continue;
        }
        let frame = frame_from_rpc_json(sig, tx["slot"].as_u64().unwrap_or(0), &tx)?;
        let t = decode_transaction(frame, vec![])?;
        decoded += 1;

        // Cross-checks against what RPC reports independently.
        let rpc_ok = tx["meta"]["err"].is_null();
        let keys_ok = t.accounts.len() == tx["meta"]["preBalances"].as_array().map(Vec::len).unwrap_or(0);
        let invoked = t.invokes(&program);
        if rpc_ok != t.success || !keys_ok || !invoked || t.signature != sig {
            mismatches += 1;
            println!("MISMATCH {sig}: success {} vs rpc {rpc_ok}, keys_ok {keys_ok}, invokes {invoked}", t.success);
        }

        let names: Vec<_> = t.instruction_names_for(&program).collect();
        println!(
            "\n{} {} slot={} cu={:?} (program {:?}) ixs={} inner={} invocations={} names={:?}",
            if t.success { "OK  " } else { "FAIL" },
            &sig[..20],
            t.slot,
            t.compute_units,
            t.compute_for(&program),
            t.instructions.iter().filter(|i| i.inner_index.is_none()).count(),
            t.instructions.iter().filter(|i| i.inner_index.is_some()).count(),
            t.invocations.len(),
            names,
        );
        if let Some(e) = &t.error {
            println!("     error: {} | name={:?} ix={:?} code={:?} program={:?}", e.message, e.name, e.instruction_index, e.custom_code, e.program_id.as_deref().map(|p| &p[..8]));
        }
        for tr in t.transfers.iter().take(4) {
            println!(
                "     {:?} {:.6} {} {} -> {} (ix {})",
                tr.kind,
                tr.amount,
                tr.mint.as_deref().map(|m| &m[..6]).unwrap_or("SOL"),
                tr.from_owner.as_deref().map(|a| &a[..6]).unwrap_or("?"),
                tr.to_owner.as_deref().map(|a| &a[..6]).unwrap_or("?"),
                tr.instruction
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    println!("\ndecoded {decoded} transactions, {mismatches} mismatches");
    Ok(())
}
