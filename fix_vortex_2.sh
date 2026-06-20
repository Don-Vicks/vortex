#!/bin/bash
set -e

# Fix agent module imports
sed -i '' 's/use crate::{AiProvider, AgentConfig}/use crate::agent::{AiProvider, AgentConfig}/g' crates/vortex/src/agent/client.rs
sed -i '' 's/use crate::{NetworkState, TipDecision, AgentConfig}/use crate::agent::{NetworkState, TipDecision, AgentConfig}/g' crates/vortex/src/agent/decisions.rs
sed -i '' 's/use crate::client/use crate::agent::client/g' crates/vortex/src/agent/decisions.rs

# Fix geyser module imports
sed -i '' 's/use crate::{GeyserEvent/use crate::geyser::{GeyserEvent/g' crates/vortex/src/geyser/stream.rs
sed -i '' 's/use crate::{GeyserEvent/use crate::geyser::{GeyserEvent/g' crates/vortex/src/geyser/rpc_fallback.rs

# Fix jito module imports
sed -i '' 's/use crate::tip/use crate::jito::tip/g' crates/vortex/src/jito/bundle.rs

# Fix lifecycle module imports
sed -i '' 's/use crate::LifecycleEvent/use crate::lifecycle::LifecycleEvent/g' crates/vortex/src/lifecycle/logger.rs

cargo check -p vortex
