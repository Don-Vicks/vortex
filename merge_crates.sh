#!/bin/bash
set -e

echo "Starting crate merge into vortex..."

# 1. Prepare vortex src dir
mkdir -p crates/vortex/src/agent
mkdir -p crates/vortex/src/failures
mkdir -p crates/vortex/src/geyser
mkdir -p crates/vortex/src/jito
mkdir -p crates/vortex/src/lifecycle

# 2. Copy source files
cp -r crates/agent/src/* crates/vortex/src/agent/
cp -r crates/failures/src/* crates/vortex/src/failures/
cp -r crates/geyser/src/* crates/vortex/src/geyser/
cp -r crates/jito/src/* crates/vortex/src/jito/
cp -r crates/lifecycle/src/* crates/vortex/src/lifecycle/

# 3. Rename vortex's old lib.rs to decision.rs
mv crates/vortex/src/lib.rs crates/vortex/src/decision.rs

# 4. Create new lib.rs for vortex
cat << 'INNER_EOF' > crates/vortex/src/lib.rs
pub mod agent;
pub mod failures;
pub mod geyser;
pub mod jito;
pub mod lifecycle;
pub mod decision;
INNER_EOF

# 5. Fix internal imports within vortex
# Any "use jito::" -> "use crate::jito::", etc.
find crates/vortex/src -type f -name "*.rs" | xargs sed -i '' 's/use agent::/use crate::agent::/g'
find crates/vortex/src -type f -name "*.rs" | xargs sed -i '' 's/use failures::/use crate::failures::/g'
find crates/vortex/src -type f -name "*.rs" | xargs sed -i '' 's/use geyser::/use crate::geyser::/g'
find crates/vortex/src -type f -name "*.rs" | xargs sed -i '' 's/use jito::/use crate::jito::/g'
find crates/vortex/src -type f -name "*.rs" | xargs sed -i '' 's/use lifecycle::/use crate::lifecycle::/g'

# 6. Merge dependencies into vortex Cargo.toml
# I will just write a comprehensive Cargo.toml for vortex
cat << 'INNER_EOF' > crates/vortex/Cargo.toml
[package]
name = "vortex"
version = "0.1.0"
edition = "2021"
description = "A dynamic decision engine and transaction stack to automatically evaluate Solana network conditions, protect against MEV, and conditionally tip Jito bundles."
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-username/solana-tx-stack"
keywords = ["solana", "jito", "mev", "vortex"]

[dependencies]
solana-sdk.workspace = true
solana-client.workspace = true
anyhow.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
reqwest.workspace = true
tracing.workspace = true
chrono.workspace = true
uuid.workspace = true
futures = "0.3"
yellowstone-grpc-client = "0.6.0"
yellowstone-grpc-proto = "0.6.0"
bs58 = "0.5"
INNER_EOF

# 7. Update core/src/main.rs to use vortex
sed -i '' 's/use lifecycle::/use vortex::lifecycle::/g' crates/core/src/main.rs
sed -i '' 's/use geyser::/use vortex::geyser::/g' crates/core/src/main.rs
sed -i '' 's/use agent::/use vortex::agent::/g' crates/core/src/main.rs
sed -i '' 's/use jito::/use vortex::jito::/g' crates/core/src/main.rs
sed -i '' 's/use failures::/use vortex::failures::/g' crates/core/src/main.rs

sed -i '' 's/ geyser::/ vortex::geyser::/g' crates/core/src/main.rs
sed -i '' 's/ agent::/ vortex::agent::/g' crates/core/src/main.rs
sed -i '' 's/ jito::/ vortex::jito::/g' crates/core/src/main.rs
sed -i '' 's/ lifecycle::/ vortex::lifecycle::/g' crates/core/src/main.rs
sed -i '' 's/ failures::/ vortex::failures::/g' crates/core/src/main.rs

# 8. Update core Cargo.toml
sed -i '' 's/agent =.*/vortex = { path = "..\/vortex" }/g' crates/core/Cargo.toml
sed -i '' '/failures =/d' crates/core/Cargo.toml
sed -i '' '/geyser =/d' crates/core/Cargo.toml
sed -i '' '/jito =/d' crates/core/Cargo.toml
sed -i '' '/lifecycle =/d' crates/core/Cargo.toml

# 9. Update workspace Cargo.toml
cat << 'INNER_EOF' > Cargo.toml
[workspace]
members = [
    "crates/core",
    "crates/vortex",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
solana-sdk = "1.18"
solana-client = "1.18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
dotenv = "0.15"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
INNER_EOF

# 10. Clean up old crates
rm -rf crates/agent crates/failures crates/geyser crates/jito crates/lifecycle

echo "Merge complete. Testing build..."
cargo check -p vortex
cargo check -p core
