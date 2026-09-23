# solana-vortex 🌪️

`solana-vortex` is a Rust library for transaction routing and MEV tip optimization on Solana. It evaluates live network conditions (such as Jito tip floors, recent failure rates, and upcoming slot leaders) and uses AI-driven heuristics with strict programmatic guardrails to dynamically compute Jito tip amounts and manage execution paths.

## Key Features

- **Dynamic Tip Engine**: Evaluates live network state and queries AI models (Anthropic Claude, OpenAI GPT, Google Gemini, xAI Grok) with hard programmatic floor/ceiling clamping.
- **Geyser Stream Integration**: Native Yellowstone gRPC stream handlers for slot updates and confirmation tracking with RPC fallback polling.
- **Native Jito gRPC Integration**: Hand-compiled Jito Protobuf schemas (`tonic`/`prost`) for direct searcher gRPC submission (`send_bundle_no_wait`).
- **Dual-Send Execution**: Broadcasts transactions simultaneously over Jito gRPC and standard RPC for execution redundancy.
- **Failure Classification & Recovery**: Error categorization (`ExpiredBlockhash`, `FeeTooLow`, `LeaderSkipped`) with automated retry heuristics.

## Installation

Add `solana-vortex` to your `Cargo.toml`:

```toml
[dependencies]
solana-vortex = "0.1.3"
```

## Quick Start Example

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use vortex::agent::{AgentConfig, AiProvider, NetworkState, decisions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(rpc_url.to_string());
    
    // 1. Configure AI Agent
    let agent_config = AgentConfig {
        provider: AiProvider::Anthropic,
        model: "claude-3-5-sonnet-20241022".to_string(),
    };

    // 2. Supply Network Conditions
    let network_state = NetworkState {
        current_slot: 284_900_100,
        tip_min: 10_000,
        tip_max: 5_000_000,
        tip_median: 200_000,
        tip_average: 350_000,
        recent_failure_rate: 0.0,
        time_since_last_success_secs: 10,
    };

    // 3. Obtain Clamped Tip Decision
    let tip_decision = decisions::decide_tip(&agent_config, &network_state).await?;

    println!("🤖 Agent Reasoning: {}", tip_decision.reasoning);
    println!("💰 Recommended Tip: {} lamports", tip_decision.recommended_lamports);

    Ok(())
}
```

## Module Overview

- `vortex::agent` — LLM decision interface, prompt builders, bounds enforcers, and fallback calculators.
- `vortex::geyser` — Yellowstone gRPC subscription stream handlers and RPC slot polling task.
- `vortex::jito` — Native gRPC searcher client, tip floor fetcher, bundle builder, and dual-send executor.
- `vortex::failures` — Error string classifier and retry decision matrix.
- `vortex::lifecycle` — Event models and JSON file append/read logging helpers.
- `vortex::decision` — Network condition evaluator and tip floor stats structs.

## License

Dual-licensed under MIT or Apache-2.0.
