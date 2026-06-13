# Solana Transaction Stack - Bounty Implementation Plan

This document outlines the remaining phases required to bring the `solana-tx-stack` up to full compliance with the SolInfra/Jito bounty requirements.

## Phase 1: Leader Schedule & Jito Window Detection
**Goal:** Submit bundles *only* when a Jito-compatible validator is the upcoming leader, rather than blinding firing at every slot.
- **Fetch Leader Schedule:** Query the RPC (`getLeaderSchedule`) at the start of each epoch.
- **Identify Jito Validators:** Cross-reference leaders with known Jito validators (or assume a strategy for predicting Jito inclusion).
- **Core Loop Integration:** Update the `core` evaluation loop to predict the next Jito leader window and trigger the AI Agent only when the window is approaching.

## Phase 2: Geyser Transaction Streaming (Landing Confirmation)
**Goal:** Replace RPC polling for transaction confirmation with live Yellowstone gRPC stream subscriptions.
- **Geyser Client Update:** Extend `crates/geyser/src/stream.rs` to subscribe to specific transaction signatures or account keys (`subscribe_transaction` / `subscribe_account`).
- **Lifecycle Tracking:** When a transaction is submitted, register its signature with the stream listener.
- **Latency Deltas:** Record exact timestamps for when the transaction hits `Processed`, `Confirmed`, and `Finalized` states as pushed by the stream.

## Phase 3: Transaction Construction & Failure Injection
**Goal:** Construct real, valid Solana transactions and intentionally simulate failures to prove the AI agent's autonomy.
- **Bundle Construction:** Update `crates/jito/src/bundle.rs` to take a funded Keypair, build a basic transfer or memo instruction, append the Jito tip instruction, and sign it.
- **Failure Injection Loop:**
  1. Intentionally fetch an old blockhash.
  2. Submit the bundle.
  3. Geyser stream will detect the `BlockhashNotFound` or expiry failure.
  4. Trigger the AI Agent to reason about the failure ("Blockhash expired").
  5. Agent commands the system to fetch a fresh blockhash, recalculate the tip based on *new* slot congestion, and retry.

## Phase 4: Data Logging & Documentation
**Goal:** Fulfill the deliverable requirements for submission.
- **Lifecycle Log JSON:** Ensure the `lifecycle` crate writes a clean JSON array of 10 submissions (including 2 injected failures) with timestamps and latency deltas.
- **Architecture Document:** Create a Figma/Notion diagram detailing the Geyser stream -> Core -> Agent -> Jito flow.
- **README Answers:** Update `README.md` to formally answer the 3 required questions regarding `processed_at` vs `confirmed_at` deltas, Finalized blockhashes, and Jito leader skips.
