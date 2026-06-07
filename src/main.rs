use solana_client::rpc_client::RpcClient;

#[tokio::main]
async fn main() {
    let client = RpcClient::new("https://api.mainnet.solana.com");
    let blockhash = client.get_latest_blockhash().unwrap();
    println!("Latest blockhash: {:?}", blockhash);
}
