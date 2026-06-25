use anyhow::{Context, Result};
use std::env;
use tracing::info;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::tonic::transport::ClientTlsConfig;

pub async fn connect() -> Result<GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>> {
    let endpoint =
        env::var("YELLOWSTONE_ENDPOINT").context("YELLOWSTONE_ENDPOINT must be set in .env")?;
    let token = env::var("YELLOWSTONE_TOKEN").context("YELLOWSTONE_TOKEN must be set in .env")?;

    info!(endpoint = %endpoint, "Connecting to Yellowstone gRPC...");

    let mut builder = GeyserGrpcClient::build_from_shared(endpoint.clone())?
        .x_token(Some(token))?;
        
    if endpoint.starts_with("https://") {
        builder = builder.tls_config(ClientTlsConfig::new())?;
    }

    let client = builder
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(10))
        .connect()
        .await
        .context("Failed to connect to Yellowstone gRPC endpoint")?;

    info!("Yellowstone gRPC connected successfully");
    Ok(client)
}
