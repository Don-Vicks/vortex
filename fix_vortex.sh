#!/bin/bash
set -e

# Fix lifecycle imports in tracker.rs
sed -i '' 's/use crate::{FailureInfo/use crate::lifecycle::{FailureInfo/g' crates/vortex/src/lifecycle/tracker.rs

# Fix agent decisions.rs returning str instead of String? Wait, let's look at call_agent signature
# If call_agent is `async fn call_agent(...) -> Result<String>`, maybe `let response: String = ...` ?
# Let's check the code for call_agent in crates/vortex/src/agent/mod.rs
# Wait, maybe it's `let response = ...` and the compiler infers `str` because of some formatting later?
sed -i '' 's/let response = call_agent/let response: String = call_agent/g' crates/vortex/src/agent/decisions.rs

# Fix num_milliseconds
sed -i '' 's/(now - event.submitted_at).num_milliseconds()/(now.signed_duration_since(event.submitted_at)).num_milliseconds()/g' crates/vortex/src/lifecycle/tracker.rs

# Fix Jito imports in core/main.rs (if I missed any)
# I'll just run cargo check on vortex
cargo check -p vortex
