//! Built-in decoders for the native programs every transaction leans on
//! (System, SPL Token / Token-2022, Compute Budget). Anything else is left
//! undecoded and named from its logs instead.

use crate::events::TransferKind;
use serde_json::json;

pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const COMPUTE_BUDGET_PROGRAM: &str = "ComputeBudget111111111111111111111111111111";

pub fn known_name(program_id: &str) -> Option<&'static str> {
    Some(match program_id {
        SYSTEM_PROGRAM => "System Program",
        TOKEN_PROGRAM => "SPL Token",
        TOKEN_2022_PROGRAM => "Token-2022",
        COMPUTE_BUDGET_PROGRAM => "Compute Budget",
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => "Associated Token Account",
        "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr" => "Memo v2",
        "Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo" => "Memo v1",
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => "Jupiter v6",
        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => "Pump.fun",
        "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA" => "PumpSwap AMM",
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => "Raydium AMM v4",
        "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK" => "Raydium CLMM",
        "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C" => "Raydium CPMM",
        "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj" => "Raydium Launchpad",
        "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => "Orca Whirlpool",
        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo" => "Meteora DLMM",
        "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG" => "Meteora DAMM v2",
        "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN" => "Meteora DBC",
        "PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY" => "Phoenix",
        "T1pyyaTNZsKv2WcRAB8oVnk93mLJw2XzjtVYqCsaHqt" => "Jito Tip Payment",
        "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD" => "Kamino Lend",
        "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH" => "Drift v2",
        "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD" => "Marinade",
        "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy" => "SPL Stake Pool",
        "Stake11111111111111111111111111111111111111" => "Stake Program",
        "Vote111111111111111111111111111111111111111" => "Vote Program",
        "AddressLookupTab1e1111111111111111111111111" => "Address Lookup Table",
        _ => return None,
    })
}

pub fn known_mint_symbol(mint: &str) -> Option<&'static str> {
    Some(match mint {
        "So11111111111111111111111111111111111111112" => "wSOL",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => "JUP",
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "BONK",
        "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So" => "mSOL",
        "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn" => "JitoSOL",
        "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo" => "PYUSD",
        _ => return None,
    })
}

/// A value movement found by decoding an instruction. Account fields are
/// indices into the instruction's own account list.
pub struct Movement {
    pub kind: TransferKind,
    pub from: Option<usize>,
    pub to: Option<usize>,
    pub authority: Option<usize>,
    pub mint: Option<usize>,
    pub amount: u64,
    pub decimals: Option<u8>,
}

pub struct Decoded {
    pub name: &'static str,
    pub parsed: serde_json::Value,
    pub movement: Option<Movement>,
    pub compute_unit_limit: Option<u32>,
    pub compute_unit_price: Option<u64>,
}

impl Decoded {
    fn named(name: &'static str) -> Self {
        Self {
            name,
            parsed: serde_json::Value::Null,
            movement: None,
            compute_unit_limit: None,
            compute_unit_price: None,
        }
    }
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(data: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(data.get(at..at + 8)?.try_into().ok()?))
}

pub fn decode(program_id: &str, data: &[u8]) -> Option<Decoded> {
    match program_id {
        SYSTEM_PROGRAM => decode_system(data),
        TOKEN_PROGRAM | TOKEN_2022_PROGRAM => decode_token(data),
        COMPUTE_BUDGET_PROGRAM => decode_compute_budget(data),
        _ => None,
    }
}

fn decode_system(data: &[u8]) -> Option<Decoded> {
    let tag = u32_at(data, 0)?;
    let sol = |name, from, to, lamports: u64| Decoded {
        parsed: json!({ "lamports": lamports }),
        movement: Some(Movement {
            kind: TransferKind::Sol,
            from: Some(from),
            to: Some(to),
            authority: None,
            mint: None,
            amount: lamports,
            decimals: Some(9),
        }),
        ..Decoded::named(name)
    };
    Some(match tag {
        0 => sol("CreateAccount", 0, 1, u64_at(data, 4)?),
        1 => Decoded::named("Assign"),
        2 => sol("Transfer", 0, 1, u64_at(data, 4)?),
        3 => Decoded::named("CreateAccountWithSeed"),
        4 => Decoded::named("AdvanceNonceAccount"),
        8 => Decoded::named("Allocate"),
        11 => sol("TransferWithSeed", 0, 2, u64_at(data, 4)?),
        _ => return None,
    })
}

fn decode_token(data: &[u8]) -> Option<Decoded> {
    let tag = *data.first()?;
    let movement = |kind, from, to, authority, mint, decimals| -> Option<Movement> {
        Some(Movement {
            kind,
            from,
            to,
            authority,
            mint,
            amount: u64_at(data, 1)?,
            decimals,
        })
    };
    let with = |name, movement: Option<Movement>| {
        let parsed = movement
            .as_ref()
            .map(|m| json!({ "amount": m.amount }))
            .unwrap_or(serde_json::Value::Null);
        Decoded {
            parsed,
            movement,
            ..Decoded::named(name)
        }
    };
    Some(match tag {
        1 | 16 | 18 => Decoded::named("InitializeAccount"),
        3 => with(
            "Transfer",
            movement(TransferKind::Token, Some(0), Some(1), Some(2), None, None),
        ),
        4 => Decoded::named("Approve"),
        7 => with(
            "MintTo",
            movement(TransferKind::Mint, None, Some(1), Some(2), Some(0), None),
        ),
        8 => with(
            "Burn",
            movement(TransferKind::Burn, Some(0), None, Some(2), Some(1), None),
        ),
        9 => Decoded::named("CloseAccount"),
        12 => with(
            "TransferChecked",
            movement(
                TransferKind::Token,
                Some(0),
                Some(2),
                Some(3),
                Some(1),
                data.get(9).copied(),
            ),
        ),
        14 => with(
            "MintToChecked",
            movement(
                TransferKind::Mint,
                None,
                Some(1),
                Some(2),
                Some(0),
                data.get(9).copied(),
            ),
        ),
        15 => with(
            "BurnChecked",
            movement(
                TransferKind::Burn,
                Some(0),
                None,
                Some(2),
                Some(1),
                data.get(9).copied(),
            ),
        ),
        17 => Decoded::named("SyncNative"),
        _ => return None,
    })
}

fn decode_compute_budget(data: &[u8]) -> Option<Decoded> {
    Some(match *data.first()? {
        2 => {
            let units = u32_at(data, 1)?;
            Decoded {
                parsed: json!({ "units": units }),
                compute_unit_limit: Some(units),
                ..Decoded::named("SetComputeUnitLimit")
            }
        }
        3 => {
            let price = u64_at(data, 1)?;
            Decoded {
                parsed: json!({ "micro_lamports": price }),
                compute_unit_price: Some(price),
                ..Decoded::named("SetComputeUnitPrice")
            }
        }
        _ => return None,
    })
}
