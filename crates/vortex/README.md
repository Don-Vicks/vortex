# Vortex 🌪️

Vortex is a high-performance, intelligent transaction routing and MEV-protection stack for Solana. It dynamically evaluates live network conditions (like prioritization fees and upcoming slot leaders) and uses AI-driven heuristics to decide whether to submit your transaction through standard RPC or bundle it via Jito to guarantee execution and prevent sandwich attacks.

## Features
- **Dynamic Decision Engine**: Automatically tip Jito only when the network is congested or a Jito leader is approaching.
- **Gasless / Relayer Support**: Built-in support for injecting fee payers for seamless UX.
- **Geyser Stream Integration**: High-speed gRPC slot and transaction confirmation tracking.
- **Autonomous Recovery**: Detects failure reasons and intelligently retries with adjusted tips or fresh blockhashes.

## Installation

Add `vortex` to your `Cargo.toml`:
```toml
[dependencies]
vortex = "0.1.0"
```

## Environment Setup

Vortex relies on a few external services to power its advanced features (Geyser streaming and AI-driven autonomous recovery). You must set the following environment variables (e.g., via a `.env` file) for full functionality:

```env
# 1. Standard Solana RPC
SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"

# 2. Yellowstone gRPC (Required for ultra-low latency transaction tracking)
YELLOWSTONE_GRPC_URL="http://your-geyser-node:20000"
YELLOWSTONE_GRPC_TOKEN="your-auth-token-if-needed"

# 3. AI Agent Setup (Required for autonomous failure analysis)
# Choose your provider: "anthropic", "openai", "gemini", or "grok"
AI_PROVIDER="anthropic"
AI_MODEL="claude-3-sonnet-20240229"

# Provide the API key for your chosen provider
ANTHROPIC_API_KEY="sk-ant-..."
# OPENAI_API_KEY="sk-..."
# GEMINI_API_KEY="AIza..."
```

## Quick Start: Routing and Tracking a Transaction

Here is a quick example of how you can build a transaction, use Vortex to intelligently route it, and then track its lifecycle via Yellowstone Geyser.

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
};
use tokio::sync::mpsc;
use vortex::decision::VortexDecisionMaker;
use vortex::jito::bundle::submit_bundle;
use vortex::geyser::{client::connect as geyser_connect, stream::subscribe_slots};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let rpc_client = RpcClient::new(rpc_url.to_string());
    
    // 1. Initialize Vortex Decision Engine
    let decision_maker = VortexDecisionMaker::new(rpc_url);
    
    // 2. Evaluate Live Network Conditions
    let decision = decision_maker.evaluate().await?;
    println!("Vortex Decision: {}", decision.reason);

    // 3. Build Your Transaction
    let sender = Keypair::new(); // Load your actual keypair here
    let recipient = Keypair::new().pubkey();
    
    let ix = system_instruction::transfer(&sender.pubkey(), &recipient, 1_000_000); // 0.001 SOL
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // 4. Route the Transaction
    if decision.use_jito {
        println!("Routing via Jito with a tip of {} lamports...", decision.recommended_tip.unwrap());
        
        let bundle_result = submit_bundle(
            &sender, 
            recent_blockhash, 
            decision.recommended_tip.unwrap(), 
            &rpc_client
        ).await?;
        
        println!("Bundle Submitted! ID: {}", bundle_result.bundle_id);
    } else {
        println!("Network is quiet. Routing via standard RPC mempool...");
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&sender.pubkey()),
            &[&sender],
            recent_blockhash,
        );
        let sig = rpc_client.send_transaction(&tx)?;
        println!("Transaction sent! Signature: {}", sig);
    }

    // 5. Track the Transaction via Geyser (Ultra-low latency streaming)
    println!("Connecting to Yellowstone gRPC to track transaction lifecycle...");
    // Requires YELLOWSTONE_ENDPOINT and YELLOWSTONE_TOKEN in .env
    let geyser_client = geyser_connect().await?;
    let (tx_sender, mut rx_receiver) = mpsc::channel(100);
    
    // Listen for events related to our sender's pubkey
    let pubkey_str = sender.pubkey().to_string();
    tokio::spawn(async move {
        if let Err(e) = subscribe_slots(geyser_client, tx_sender, Some(pubkey_str)).await {
            eprintln!("Geyser stream error: {}", e);
        }
    });

    while let Some(event) = rx_receiver.recv().await {
        println!("Received Geyser Event: {:?}", event);
        // You can add logic here to break when your specific transaction is Confirmed/Finalized
    }

    Ok(())
}
```

## Modules

- `vortex::agent` - Connects to Anthropic/OpenAI/Gemini to analyze failure logs and dynamically adjust tip strategies.
- `vortex::geyser` - Connects to Yellowstone gRPC for ultra-low latency slot and transaction tracking.
- `vortex::jito` - Utilities for interacting with the Jito Block Engine, constructing bundles, and tracking Jito leaders.
- `vortex::lifecycle` - Event tracking system to log transactions from `Pending` -> `Confirmed` -> `Finalized` or `Failed`.
- `vortex::decision` - The core engine that evaluates network conditions to make routing decisions.
