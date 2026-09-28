//! Decodes Yellowstone `SubscribeUpdateTransaction` frames into the
//! transport-agnostic `events::VortexTransaction` model.

use crate::events::{
    logs, programs, AccountRef, Instruction, TokenBalanceChange, Transfer, TxError,
    VortexTransaction,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;
use std::collections::HashMap;
use yellowstone_grpc_proto::convert_from;
use yellowstone_grpc_proto::prelude::{
    Message, SubscribeUpdateTransaction, TokenBalance, TransactionStatusMeta,
};

pub fn decode_transaction(
    update: SubscribeUpdateTransaction,
    filters: Vec<String>,
) -> Result<VortexTransaction> {
    let info = update.transaction.ok_or_else(|| anyhow!("update without transaction"))?;
    let signature = bs58::encode(&info.signature).into_string();
    let tx = info.transaction.ok_or_else(|| anyhow!("missing transaction body"))?;
    let message = tx.message.ok_or_else(|| anyhow!("missing message"))?;
    let meta = info.meta.ok_or_else(|| anyhow!("missing meta"))?;

    let accounts = account_refs(&message, &meta);
    let keys: Vec<&str> = accounts.iter().map(|a| a.pubkey.as_str()).collect();

    let token_accounts = token_account_index(&meta, &keys);
    let token_balances = token_balance_changes(&meta, &keys);

    let parsed_logs = logs::parse(&meta.log_messages);

    let mut instructions = Vec::new();
    let mut transfers = Vec::new();
    let mut cu_limit = None;
    let mut cu_price = None;

    let mut inner_by_parent: HashMap<u32, _> = meta
        .inner_instructions
        .iter()
        .map(|group| (group.index, &group.instructions))
        .collect();

    for (top, ix) in message.instructions.iter().enumerate() {
        let top = top as u16;
        let entry = build_instruction(
            &keys,
            ix.program_id_index,
            &ix.accounts,
            &ix.data,
            top.to_string(),
            top,
            None,
            1,
        );
        if let Some(decoded) = programs::decode(&entry.program_id, &ix.data) {
            cu_limit = cu_limit.or(decoded.compute_unit_limit);
            cu_price = cu_price.or(decoded.compute_unit_price);
            push_transfer(&mut transfers, &decoded, &entry, &token_accounts);
        }
        instructions.push(entry);

        if let Some(inner) = inner_by_parent.remove(&(top as u32)) {
            for (i, inner_ix) in inner.iter().enumerate() {
                let entry = build_instruction(
                    &keys,
                    inner_ix.program_id_index,
                    &inner_ix.accounts,
                    &inner_ix.data,
                    format!("{top}.{i}"),
                    top,
                    Some(i as u16),
                    inner_ix.stack_height.unwrap_or(2),
                );
                if let Some(decoded) = programs::decode(&entry.program_id, &inner_ix.data) {
                    push_transfer(&mut transfers, &decoded, &entry, &token_accounts);
                }
                instructions.push(entry);
            }
        }
    }

    name_from_invocations(&mut instructions, &parsed_logs.invocations);

    let error = convert_from::create_tx_error(meta.err.as_ref())
        .ok()
        .flatten()
        .map(|err| tx_error(err, &instructions, parsed_logs.program_error));

    Ok(VortexTransaction {
        signature,
        slot: update.slot,
        index: info.index,
        received_at: Utc::now(),
        success: error.is_none(),
        error,
        fee: meta.fee,
        compute_units: meta.compute_units_consumed,
        compute_unit_limit: cu_limit,
        compute_unit_price: cu_price,
        accounts,
        instructions,
        invocations: parsed_logs.invocations,
        logs: meta.log_messages,
        logs_truncated: parsed_logs.truncated,
        token_balances,
        transfers,
        filters,
    })
}

fn account_refs(message: &Message, meta: &TransactionStatusMeta) -> Vec<AccountRef> {
    let header = message.header.clone().unwrap_or_default();
    let n_static = message.account_keys.len();
    let n_signers = header.num_required_signatures as usize;
    let writable_signers = n_signers.saturating_sub(header.num_readonly_signed_accounts as usize);
    let writable_unsigned_end =
        n_static.saturating_sub(header.num_readonly_unsigned_accounts as usize);

    let static_keys = message.account_keys.iter().enumerate().map(|(i, key)| {
        let signer = i < n_signers;
        let writable = if signer {
            i < writable_signers
        } else {
            i < writable_unsigned_end
        };
        (key, signer, writable, false)
    });
    let loaded_w = meta.loaded_writable_addresses.iter().map(|k| (k, false, true, true));
    let loaded_r = meta.loaded_readonly_addresses.iter().map(|k| (k, false, false, true));

    static_keys
        .chain(loaded_w)
        .chain(loaded_r)
        .enumerate()
        .map(|(i, (key, signer, writable, from_lookup_table))| AccountRef {
            pubkey: bs58::encode(key).into_string(),
            signer,
            writable,
            from_lookup_table,
            pre_lamports: meta.pre_balances.get(i).copied().unwrap_or(0),
            post_lamports: meta.post_balances.get(i).copied().unwrap_or(0),
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_instruction(
    keys: &[&str],
    program_idx: u32,
    account_idx: &[u8],
    data: &[u8],
    path: String,
    top_index: u16,
    inner_index: Option<u16>,
    stack_height: u32,
) -> Instruction {
    let program_id = keys
        .get(program_idx as usize)
        .map(|s| s.to_string())
        .unwrap_or_default();
    let decoded = programs::decode(&program_id, data);
    Instruction {
        path,
        top_index,
        inner_index,
        stack_height,
        program_name: programs::known_name(&program_id).map(str::to_string),
        name: decoded.as_ref().map(|d| d.name.to_string()),
        parsed: decoded.map(|d| d.parsed).filter(|p| !p.is_null()),
        accounts: account_idx
            .iter()
            .filter_map(|&i| keys.get(i as usize).map(|s| s.to_string()))
            .collect(),
        data: bs58::encode(data).into_string(),
        program_id,
    }
}

struct TokenAccountInfo {
    mint: String,
    owner: Option<String>,
    decimals: u8,
}

fn token_account_index(
    meta: &TransactionStatusMeta,
    keys: &[&str],
) -> HashMap<String, TokenAccountInfo> {
    meta.pre_token_balances
        .iter()
        .chain(meta.post_token_balances.iter())
        .filter_map(|b| {
            let key = keys.get(b.account_index as usize)?;
            Some((
                key.to_string(),
                TokenAccountInfo {
                    mint: b.mint.clone(),
                    owner: (!b.owner.is_empty()).then(|| b.owner.clone()),
                    decimals: b.ui_token_amount.as_ref().map(|a| a.decimals as u8).unwrap_or(0),
                },
            ))
        })
        .collect()
}

fn ui_amount(b: &TokenBalance) -> f64 {
    b.ui_token_amount
        .as_ref()
        .and_then(|a| a.ui_amount_string.parse().ok())
        .unwrap_or(0.0)
}

fn token_balance_changes(meta: &TransactionStatusMeta, keys: &[&str]) -> Vec<TokenBalanceChange> {
    let mut by_account: HashMap<u32, (Option<&TokenBalance>, Option<&TokenBalance>)> =
        HashMap::new();
    for b in &meta.pre_token_balances {
        by_account.entry(b.account_index).or_default().0 = Some(b);
    }
    for b in &meta.post_token_balances {
        by_account.entry(b.account_index).or_default().1 = Some(b);
    }

    let mut changes: Vec<TokenBalanceChange> = by_account
        .into_iter()
        .filter_map(|(idx, (pre, post))| {
            let any = post.or(pre)?;
            let pre_amt = pre.map(ui_amount).unwrap_or(0.0);
            let post_amt = post.map(ui_amount).unwrap_or(0.0);
            Some(TokenBalanceChange {
                account: keys.get(idx as usize)?.to_string(),
                owner: (!any.owner.is_empty()).then(|| any.owner.clone()),
                mint: any.mint.clone(),
                decimals: any.ui_token_amount.as_ref().map(|a| a.decimals as u8).unwrap_or(0),
                pre: pre_amt,
                post: post_amt,
                delta: post_amt - pre_amt,
            })
        })
        .collect();
    changes.sort_by(|a, b| a.account.cmp(&b.account));
    changes
}

fn push_transfer(
    out: &mut Vec<Transfer>,
    decoded: &programs::Decoded,
    ix: &Instruction,
    token_accounts: &HashMap<String, TokenAccountInfo>,
) {
    let Some(m) = &decoded.movement else { return };
    let acct = |i: Option<usize>| i.and_then(|i| ix.accounts.get(i).cloned());
    let from = acct(m.from);
    let to = acct(m.to);
    let is_sol = m.kind == crate::events::TransferKind::Sol;

    let info = |a: &Option<String>| a.as_ref().and_then(|a| token_accounts.get(a));
    let token_info = info(&from).or_else(|| info(&to));
    let mint = if is_sol {
        None
    } else {
        acct(m.mint).or_else(|| token_info.map(|t| t.mint.clone()))
    };
    let decimals = m
        .decimals
        .or_else(|| token_info.map(|t| t.decimals))
        .unwrap_or(if is_sol { 9 } else { 0 });

    let owner_of = |a: &Option<String>| {
        if is_sol {
            a.clone()
        } else {
            info(a).and_then(|t| t.owner.clone())
        }
    };

    out.push(Transfer {
        kind: m.kind.clone(),
        mint,
        from_owner: owner_of(&from),
        to_owner: owner_of(&to),
        from,
        to,
        authority: acct(m.authority),
        amount_raw: m.amount,
        decimals,
        amount: m.amount as f64 / 10f64.powi(decimals as i32),
        instruction: ix.path.clone(),
    });
}

/// Invocations from logs appear in the same order as instructions executed,
/// so a zip names program instructions that have no built-in decoder.
fn name_from_invocations(instructions: &mut [Instruction], invocations: &[logs::Invocation]) {
    let mut inv_iter = invocations.iter().peekable();
    for ix in instructions.iter_mut() {
        // Precompiles don't log an invoke line; skip them without consuming.
        let Some(inv) = inv_iter.next_if(|inv| inv.program_id == ix.program_id) else {
            continue;
        };
        if ix.name.is_none() {
            ix.name = inv.instruction.clone();
        }
    }
}

fn tx_error(
    err: TransactionError,
    instructions: &[Instruction],
    program_error: Option<logs::ProgramError>,
) -> TxError {
    let message = format!("{err:?}");
    let (instruction_index, custom_code) = match &err {
        TransactionError::InstructionError(idx, InstructionError::Custom(code)) => {
            (Some(*idx), Some(*code))
        }
        TransactionError::InstructionError(idx, _) => (Some(*idx), None),
        _ => (None, None),
    };
    let program_id = instruction_index.and_then(|idx| {
        instructions
            .iter()
            .find(|ix| ix.inner_index.is_none() && ix.top_index == idx as u16)
            .map(|ix| ix.program_id.clone())
    });
    let name = program_error
        .map(|e| e.name)
        .or_else(|| match &err {
            TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
                Some(format!("Custom({code})"))
            }
            TransactionError::InstructionError(_, e) => Some(format!("{e:?}")),
            other => Some(format!("{other:?}")),
        });
    let class = format!("{:?}", crate::failures::classifier::classify(&message));
    TxError {
        message,
        instruction_index,
        custom_code,
        program_id,
        name,
        class,
    }
}
