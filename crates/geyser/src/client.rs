use anyhow::Result;
use std::env;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::tonic::transport::ClientTlsConfig;

pub async fn connect() -> Result<GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>> {
    let endpoint = env::var("YELLOWSTONE_ENDPOINT")
        .expect("YELLOWSTONE_ENDPOINT must be set");
    let token = env::var("YELLOWSTONE_TOKEN")
        .expect("YELLOWSTONE_TOKEN must be set");

    let client = GeyserGrpcClient::build_from_shared(endpoint)?
        .x_token(Some(token))?
        .tls_config(ClientTlsConfig::new())?
        .connect()
        .await?;

    Ok(client)
}
