//! Stand-in for a Solami Mirage stream: replays two real mainnet Pump.fun
//! transactions as binary Yellowstone frames, so the transport can be tried
//! without an API key. `cargo run --example mirage_mock` then point
//! `MIRAGE_STREAM_URL` at `ws://127.0.0.1:9911/mirage/stream/mock?api_key=x`.

use futures_util::SinkExt;
use prost::Message as _;
use serde_json::Value;
use solana_vortex::geyser::rpc_frame::frame_from_rpc_json;
use tokio_tungstenite::tungstenite::Message;
use yellowstone_grpc_proto::prelude::*;

fn frame(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let resp = &fixture["response"];
    let tx = frame_from_rpc_json(fixture["signature"].as_str().unwrap(), resp["slot"].as_u64().unwrap(), resp).unwrap();
    SubscribeUpdate { update_oneof: Some(subscribe_update::UpdateOneof::Transaction(tx)), ..Default::default() }.encode_to_vec()
}

#[tokio::main]
async fn main() {
    let frames = [frame("pump_ok"), frame("pump_failed")];
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9911").await.unwrap();
    println!("mock Mirage on ws://127.0.0.1:9911");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let frames = frames.clone();
        tokio::spawn(async move {
            let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else { return };
            let mut i = 0usize;
            loop {
                if ws.send(Message::Binary(frames[i % 2].clone())).await.is_err() {
                    return;
                }
                i += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }
}
