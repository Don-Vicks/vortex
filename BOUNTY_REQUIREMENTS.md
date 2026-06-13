# Solana Transaction Infrastructure Bounty

## Overview
On Solana, sending a transaction is only one small part of the story. There are different lifecycles in the network before a transaction lands; leader scheduling, TPU ingestion, block production, shred propagation, and multiple commitment stages. Production systems need to understand this entire flow, react to failures correctly, and make smart decisions under changing network conditions.

This bounty focuses on building a real transaction infrastructure stack powered by:
- Jito bundles
- Live Yellowstone/Geyser streaming
- Transaction lifecycle tracking
- AI-assisted decision making

You will build a smart transaction stack that observes the network in real time, submits transactions intelligently, tracks outcomes across commitment levels, and uses an AI agent to make one meaningful operational decision autonomously.

---

## Requirements

### 1. Architecture Design Document
Your submission must include a public architecture document hosted separately from your GitHub repository.
- **Accepted formats:** Figma, Notion, Google Docs, Any public URL
- **Your document should explain:**
  - The system architecture
  - Key components
  - Data flow between services
  - Infrastructure decisions
  - Failure handling strategy
  - AI agent responsibilities
  - Diagrams are strongly encouraged.

### 2. The Transaction Stack
Build a working smart transaction stack that can:
- **Monitor live slot and leader data using:**
  - Yellowstone gRPC (or any compatible Geyser stream provider)
  - Detect the correct leader window for submission
- **Construct and submit Jito bundles**
- **Calculate bundle tips dynamically using:**
  - Real recent tip account data
  - Current network conditions
  - *No hardcoded tip values*
- **Track transaction lifecycle stages:**
  - Submitted -> Processed -> Confirmed -> Finalized
  - Capture Timestamps, Slot numbers, Latency deltas between stages
- **Detect and classify failures:**
  - Expired blockhash, Fee too low, Compute exceeded, Bundle failure
- **Confirm landing using stream subscriptions**
  - *RPC polling alone is not sufficient*
- **Handle retries automatically**
  - Including blockhash refresh on expiry

### 3. Lifecycle Log
Your submission must include a lifecycle log from at least:
- **10 real bundle submissions** (Including at least 2 failure cases)
- **Each log entry should contain:**
  - Slot numbers
  - Commitment progression
  - Timestamps
  - Tip amounts
  - Failure classification (if applicable)

### 4. AI Agent Demonstration
You must build an AI agent that owns one real operational decision inside your stack. *A simple wrapper that calls functions sequentially without reasoning will not qualify.*

**(We have chosen: Tip Intelligence & Autonomous Retry)**
- **Autonomous Retry with Fault Injection:**
  - Your stack must simulate at least one blockhash expiry failure.
  - The agent must detect the failure, reason about the cause, refresh the blockhash, recalculate the tip, and resubmit autonomously. No hardcoded retry flow allowed.
- **Tip Intelligence:**
  - The agent analyzes recent tip account data, monitors current slot conditions, and decides how much to tip for each bundle.

### 5. README Questions
Your README should explain the observations, tradeoffs, and lessons from your running system. Your README must answer these three questions:
1. What does the delta between `processed_at` and `confirmed_at` tell you about network health at the time of submission?
2. Why should you never use finalized commitment when fetching a blockhash for a time-sensitive transaction?
3. What happens to your bundle if the Jito leader skips their slot?

### 6. General Requirements
- Open-source code with clear setup instructions.
- Working prototype on Devnet or Mainnet.
- Clean separation between AI layer and Core transaction stack.
- Proper use of Jito and streaming infrastructure (Correct commitment handling, No hardcoded shortcuts).
- **Failure handling is required. Happy-path-only submissions will not score well.**

---

## Resources
- [Jito TypeScript SDK](https://github.com/jito-labs/jito-ts)
- [Jito Rust JSON-RPC SDK](https://github.com/jito-labs/jito-rust-rpc)
- [Jito Documentation](https://docs.jito.wtf)
- [Yellowstone gRPC](https://github.com/rpcpool/yellowstone-grpc)
- [Triton Yellowstone Documentation](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions)
- [Solana JSON RPC API](https://solana.com/docs/rpc)
