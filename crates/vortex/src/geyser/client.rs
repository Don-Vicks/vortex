use anyhow::{Context, Result};
use std::env;
use tracing::info;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::tonic::codec::CompressionEncoding;
use yellowstone_grpc_proto::tonic::transport::ClientTlsConfig;

/// HTTP/2 flow-control windows. The 64 KB default caps one stream at window / round-trip time,
/// about 170 KB/s over a 370 ms link, which is far below what a busy program subscription
/// produces. Measured against Solami over such a link, the default delivered 293 tx/s and
/// backed up the server's buffer until it dropped the connection; 16 MB / 32 MB windows with
/// gzip delivered 540 to 640 tx/s.
const STREAM_WINDOW: u32 = 16 * 1024 * 1024;
const CONNECTION_WINDOW: u32 = 32 * 1024 * 1024;

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

    builder = builder
        .initial_stream_window_size(STREAM_WINDOW)
        .initial_connection_window_size(CONNECTION_WINDOW)
        .http2_adaptive_window(true)
        .tcp_nodelay(true)
        .http2_keep_alive_interval(std::time::Duration::from_secs(10))
        .keep_alive_timeout(std::time::Duration::from_secs(10))
        .keep_alive_while_idle(true);
    // Responses are mostly logs and account lists, which compress well. Set
    // YELLOWSTONE_COMPRESSION=none for a server that doesn't support gzip.
    if env::var("YELLOWSTONE_COMPRESSION").map(|v| v != "none").unwrap_or(true) {
        builder = builder.accept_compressed(CompressionEncoding::Gzip);
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
