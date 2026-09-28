//! Reconstructs the program invocation tree from runtime log messages.
//!
//! The runtime emits a fixed grammar (`Program X invoke [n]`, `Program X
//! consumed a of b compute units`, `Program X success|failed: ...`), which lets
//! us recover per-program compute, instruction names (Anchor logs
//! `Instruction: Name`) and the failing frame without an IDL.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Invocation {
    pub program_id: String,
    /// 1 for top-level instructions, 2+ for CPIs.
    pub depth: u32,
    pub parent: Option<usize>,
    pub instruction: Option<String>,
    pub compute_consumed: Option<u64>,
    pub compute_budget: Option<u64>,
    pub success: Option<bool>,
    pub failure: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgramError {
    pub name: String,
    pub code: Option<u32>,
    pub message: Option<String>,
}

pub struct ParsedLogs {
    pub invocations: Vec<Invocation>,
    pub truncated: bool,
    pub program_error: Option<ProgramError>,
}

pub fn parse(logs: &[String]) -> ParsedLogs {
    let mut invocations: Vec<Invocation> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut truncated = false;
    let mut program_error = None;

    for line in logs {
        if line.starts_with("Log truncated") {
            truncated = true;
            continue;
        }

        if let Some(rest) = line.strip_prefix("Program ") {
            if let Some((program, depth)) = parse_invoke(rest) {
                invocations.push(Invocation {
                    program_id: program.to_string(),
                    depth,
                    parent: stack.last().copied(),
                    ..Default::default()
                });
                stack.push(invocations.len() - 1);
                continue;
            }
            if let Some((program, consumed, budget)) = parse_consumed(rest) {
                if let Some(inv) = top_matching(&mut invocations, &stack, program) {
                    inv.compute_consumed = Some(consumed);
                    inv.compute_budget = Some(budget);
                }
                continue;
            }
            if let Some(program) = rest.strip_suffix(" success") {
                if let Some(inv) = top_matching(&mut invocations, &stack, program) {
                    inv.success = Some(true);
                }
                stack.pop();
                continue;
            }
            if let Some((program, reason)) = rest.split_once(" failed: ") {
                if let Some(inv) = top_matching(&mut invocations, &stack, program) {
                    inv.success = Some(false);
                    inv.failure = Some(reason.to_string());
                }
                stack.pop();
                continue;
            }
        }

        let Some(&current) = stack.last() else { continue };
        let inv = &mut invocations[current];

        if let Some(name) = line.strip_prefix("Program log: Instruction: ") {
            if inv.instruction.is_none() {
                inv.instruction = Some(name.trim().to_string());
            }
        }
        if line.contains("AnchorError") || line.contains("Error Code: ") {
            if let Some(err) = parse_anchor_error(line) {
                program_error = Some(err);
            }
        }
        if inv.logs.len() < 64 {
            inv.logs.push(line.clone());
        }
    }

    ParsedLogs {
        invocations,
        truncated,
        program_error,
    }
}

fn parse_invoke(rest: &str) -> Option<(&str, u32)> {
    let (program, tail) = rest.split_once(" invoke [")?;
    let depth = tail.strip_suffix(']')?.parse().ok()?;
    Some((program, depth))
}

fn parse_consumed(rest: &str) -> Option<(&str, u64, u64)> {
    let (program, tail) = rest.split_once(" consumed ")?;
    let (consumed, tail) = tail.split_once(" of ")?;
    let budget = tail.strip_suffix(" compute units")?;
    Some((program, consumed.parse().ok()?, budget.parse().ok()?))
}

fn top_matching<'a>(
    invocations: &'a mut [Invocation],
    stack: &[usize],
    program: &str,
) -> Option<&'a mut Invocation> {
    let idx = *stack.last()?;
    let inv = &mut invocations[idx];
    (inv.program_id == program).then_some(inv)
}

/// Parses Anchor's `Error Code: X. Error Number: N. Error Message: M.` log line.
fn parse_anchor_error(line: &str) -> Option<ProgramError> {
    let (_, after_code) = line.split_once("Error Code: ")?;
    let (name, rest) = after_code.split_once('.')?;
    let code = rest
        .split_once("Error Number: ")
        .and_then(|(_, r)| r.split_once('.'))
        .and_then(|(n, _)| n.trim().parse().ok());
    let message = rest
        .split_once("Error Message: ")
        .map(|(_, m)| m.trim().trim_end_matches('.').to_string());
    Some(ProgramError {
        name: name.trim().to_string(),
        code,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn builds_invocation_tree_with_compute_and_names() {
        let parsed = parse(&lines(&[
            "Program ComputeBudget111111111111111111111111111111 invoke [1]",
            "Program ComputeBudget111111111111111111111111111111 success",
            "Program Prog111 invoke [1]",
            "Program log: Instruction: Buy",
            "Program Tokenkeg invoke [2]",
            "Program log: Instruction: Transfer",
            "Program Tokenkeg consumed 4645 of 180000 compute units",
            "Program Tokenkeg success",
            "Program Prog111 consumed 30000 of 199850 compute units",
            "Program Prog111 success",
        ]));
        assert_eq!(parsed.invocations.len(), 3);
        let buy = &parsed.invocations[1];
        assert_eq!(buy.instruction.as_deref(), Some("Buy"));
        assert_eq!(buy.compute_consumed, Some(30000));
        assert_eq!(buy.success, Some(true));
        let cpi = &parsed.invocations[2];
        assert_eq!(cpi.depth, 2);
        assert_eq!(cpi.parent, Some(1));
        assert_eq!(cpi.instruction.as_deref(), Some("Transfer"));
    }

    #[test]
    fn captures_anchor_error_and_failure() {
        let parsed = parse(&lines(&[
            "Program Prog111 invoke [1]",
            "Program log: Instruction: Sell",
            "Program log: AnchorError thrown in programs/pump/src/lib.rs:512. Error Code: TooLittleSolReceived. Error Number: 6003. Error Message: Slippage: Too little SOL received to sell the given amount of tokens..",
            "Program Prog111 consumed 21000 of 200000 compute units",
            "Program Prog111 failed: custom program error: 0x1773",
        ]));
        let err = parsed.program_error.unwrap();
        assert_eq!(err.name, "TooLittleSolReceived");
        assert_eq!(err.code, Some(6003));
        assert_eq!(parsed.invocations[0].success, Some(false));
        assert_eq!(
            parsed.invocations[0].failure.as_deref(),
            Some("custom program error: 0x1773")
        );
    }
}
