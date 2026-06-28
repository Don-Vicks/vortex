# 🚀 Vortex: Solana Smart Transaction Stack

<div align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Solana-14F195?style=for-the-badge&logo=solana&logoColor=black" alt="Solana" />
  <img src="https://img.shields.io/badge/AI_Agent-Multi--Model_Support-blue?style=for-the-badge" alt="AI Agent" />
  <img src="https://img.shields.io/badge/Geyser-Yellowstone-orange?style=for-the-badge" alt="Geyser" />
  <img src="https://img.shields.io/badge/Status-Production_Ready-success?style=for-the-badge" alt="Status" />
</div>

<br/>

I built **Vortex** to tackle the Solana Smart Transaction Infrastructure Bounty. 

The bounty challenge asked for a solution to a massive pain point for anyone building bots or relayers on Solana: the traditional "spray and pray" approach where transactions drop randomly, public APIs throttle you, and missed slots cost real money. 

To solve the bounty's requirements, I designed Vortex to dynamically evaluate network congestion, predict slot leaders, and use an AI agent to calculate the exact tip needed to guarantee inclusion. I also built a live React frontend dashboard to visually prove the system works in real time.

---

## 📖 Table of Contents

- [The Challenge & My Approach](#-the-challenge--my-approach)
- [Key Features I Built](#-key-features-i-built)
- [Project Structure](#-project-structure)
- [Prerequisites](#-prerequisites)
- [Getting Started](#-getting-started)
- [Configuration & Modes](#-configuration--modes)
- [Bounty Requirements & Mechanics](#-bounty-requirements--mechanics)

---

## 🧠 The Challenge & My Approach

When I started this project, getting a transaction to consistently land on-chain was brutally hard.

Initially, I relied on Jito's public `sendBundle` REST endpoints. But I quickly noticed that during network congestion, my unauthenticated searcher bundles were silently dropped. I could send a perfectly valid transaction, pay the tip, and still watch it disappear into the void. To make matters worse, integrating heavy pre-compiled gRPC crates often forced me into dependency hell, especially since I needed to maintain strict compatibility with `solana-client v1.18.x`.

I realized I couldn't rely on REST APIs for mission-critical execution. I needed a stack that was as close to the bare metal of the network as possible.

### How I Fixed It: Native gRPC, Dual-Send & AI Tipping

1. **Yellowstone Geyser**: Instead of fighting with standard HTTP limits and slow polling, I rebuilt the network layer to operate exclusively via real-time streams, subscribing directly to TPU confirmations.
2. **Native `.proto` Compilation**: To bypass the dependency conflicts holding me back from Jito's gRPC Block Engine, I ripped out the bloated crates. I directly downloaded Jito's raw `.proto` schemas and compiled them into native Rust within my own workspace.
3. **Dual-Send Strategy**: To guarantee absolute liveness, I designed Vortex to simultaneously blast every bundle via native gRPC to the Jito Block Engine _and_ redundantly fire it to a standard premium RPC. If Jito drops it, the standard RPC catches it.
4. **AI-Driven Tipping**: Static tipping is a surefire way to lose money. I integrated an AI Agent (supporting Claude, OpenAI, Gemini, Grok) that analyzes live tip percentiles and my failure rates to dynamically calculate my exact tip.

> **Deep Dive**: For a full look at my thought process and the system's data flow, check out the [ARCHITECTURE.md](./ARCHITECTURE.md) I wrote.

---

## ✨ Key Features I Built

- 🎯 **Jito Leader Window Prediction**: By tracking the live slot against the epoch's leader schedule, I ensure Vortex only fires transactions when a Jito validator is up next.
- ⚡ **Zero-Latency Streaming**: I completely replaced sluggish `getSignatureStatuses` polling with Yellowstone gRPC streaming.
- 🛡️ **Autonomous Recovery**: I built a daemon capable of detecting dropped blocks and expired blockhashes. When I artificially inject a failure, the AI immediately steps in, recalculates the tip, and fetches a fresh blockhash to retry.
- 📊 **Real-Time React Dashboard**: I built a Vite/React dashboard that streams transaction states (`submitted`, `processed`, `confirmed`, `finalized`) with millisecond precision latency deltas directly to my browser.

---

## 📂 Project Structure

This is a full-stack monorepo I structured for high performance:

```text
solana-tx-stack/
├── crates/
│   ├── core/                # Main entrypoint, HTTP Relayer API, and Daemon loop
│   └── vortex/              # My core engine: AI logic, Geyser streaming, and Jito bundling
│       └── proto/           # Official Jito .proto schemas I compiled for native gRPC
├── frontend/                # React/Vite dashboard for live tracking
├── logs/                    # JSON data stores for precision latency tracking
├── .env.example             # Template for RPCs and API Keys
├── ARCHITECTURE.md          # My in-depth system design documentation
└── README.md                # Project documentation
```

---

## 🛠 Prerequisites

To run what I've built, you'll need:

- **Rust** (Edition 2021)
- **Node.js** (v18+) & **npm/pnpm**
- **Solana CLI** (`v1.18.x`)

---

## 🚀 Getting Started

### 1. Environment Setup

Clone my repository and prep the environment variables:

```bash
git clone https://github.com/Don-Vicks/solana-tx-stack.git
cd solana-tx-stack
cp .env.example .env
```

**Required `.env` Variables:**

- `SOLANA_RPC_URL`: Your standard HTTP RPC (e.g., Helius, Quicknode).
- `YELLOWSTONE_GRPC_URL` & `YELLOWSTONE_GRPC_TOKEN`: Required for the Geyser streaming.
- `AI_PROVIDER`: Choose `anthropic`, `openai`, `gemini`, or `grok`.
- `[PROVIDER]_API_KEY`: API key for your chosen AI.
- `WALLET_KEYPAIR_PATH`: Path to your Solana local keypair.

### 2. Running the Backend Engine

I designed the backend to run in two modes. For testing the frontend, run it in **API Mode**.

```bash
cargo build --release
cargo run --release
```

_(If you want to watch my engine run autonomously in the background, set `RUN_DAEMON=true` in your `.env`)_

### 3. Running the Frontend Dashboard

In a new terminal window, start the React dashboard I built to visualize the engine in real-time.

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173`. You can use the **Vortex Transaction Sender** widget to execute real SOL transfers and watch the engine take over.

---

## 📦 Using My `crates.io` Package

I didn't just build a standalone script; I decoupled the core engine and published it as a modular library on `crates.io` so you can use my infrastructure in your own projects. 

*Note: The repository you are currently looking at is mainly the demo. For a full working example of how to implement the package, you can view the reference demo repo here: [solana-vortex-demo](https://github.com/Don-Vicks/solana-vortex-demo).*

Add `solana-vortex` to your `Cargo.toml`:

```bash
cargo add solana-vortex
```

You can initialize my network state evaluator and prompt the AI Agent to decide the optimal tip for your bundle before submission:

```rust
use vortex::agent::{AgentConfig, AiProvider, NetworkState, decisions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent_config = AgentConfig {
        provider: AiProvider::Anthropic,
        model: "claude-3-5-sonnet-20241022".to_string(),
    };

    // Supplying Live Network Conditions
    let network_state = NetworkState {
        current_slot: 284_900_100,
        tip_min: 10_000,
        tip_max: 5_000_000,
        tip_median: 200_000,
        tip_average: 350_000,
        recent_failure_rate: 15.5,
        time_since_last_success_secs: 45,
    };

    let tip_decision = decisions::decide_tip(&agent_config, &network_state).await?;

    println!("🤖 Agent Reasoning: {}", tip_decision.reasoning);
    println!("💰 Recommended Tip: {} lamports", tip_decision.recommended_lamports);

    Ok(())
}
```

---

## 🏆 Bounty Requirements & Mechanics

Here are my thoughts on the specific bounty questions based on what I learned while building this:

### 1. What does the delta between `processed_at` and `confirmed_at` tell me about network health?

Basically, it tells me how fast the cluster is reaching consensus. When the delta is super low (like 400-800ms), it means the network is humming along—validators are voting fast and there's no congestion. When that delta spikes, it usually means the network is struggling with heavy fork switching or high CPU load, and blocks are taking longer to get voted on.

### 2. Why did I avoid using finalized commitment when fetching a blockhash?

Because it wastes way too much time. A blockhash is only good for about 150 slots (roughly a minute). If I wait for a blockhash to hit `finalized` status, I'm burning about 32 slots just waiting for confirmation blocks to pile up on top of it. That's basically 25-30% of the blockhash's lifespan gone before I even sign the transaction. For anything time-sensitive, I always grab a `processed` or `confirmed` blockhash to give my bundle the biggest possible window to land before it expires.

### 3. What happens to my bundle if the Jito leader skips their slot?

If the Jito leader I targeted skips their slot, the Jito Block Engine just drops the bundle for that slot. Technically Jito might try to forward it if the blockhash is still alive, but I don't like leaving that to chance. The best way I found to handle this—and how I built the Vortex daemon to work—is to watch the Geyser stream like a hawk. The second I see a `LeaderSkipped` or `Timeout`, I have my engine pull a fresh blockhash, recalculate the tip, and immediately fire a retry at the next Jito leader.
