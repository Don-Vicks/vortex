//! The Mirage transport against a local WebSocket server that sends real
//! mainnet transactions as binary Yellowstone frames.

use futures_util::SinkExt;
use prost::Message as _;
use serde_json::Value;
use solana_vortex::geyser::decode::decode_transaction;
use solana_vortex::geyser::mirage::session;
use solana_vortex::geyser::rpc_frame::frame_from_rpc_json;
use solana_vortex::geyser::GeyserEvent;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use yellowstone_grpc_proto::prelude::*;

fn frame(name: &str) -> SubscribeUpdateTransaction {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let resp = &fixture["response"];
    frame_from_rpc_json(fixture["signature"].as_str().unwrap(), resp["slot"].as_u64().unwrap(), resp).unwrap()
}

fn update(tx: SubscribeUpdateTransaction) -> Vec<u8> {
    SubscribeUpdate {
        // Mirage names filters its own way; the transport mustn't depend on ours.
        filters: vec!["whatever_mirage_calls_it".into()],
        update_oneof: Some(subscribe_update::UpdateOneof::Transaction(tx)),
        ..Default::default()
    }
    .encode_to_vec()
}

#[tokio::test]
async fn decodes_frames_like_the_grpc_path() {
    let ok = frame("pump_ok");
    let failed = frame("pump_failed");
    let expected_ok = decode_transaction(ok.clone(), vec![]).unwrap();
    let expected_failed = decode_transaction(failed.clone(), vec![]).unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/mirage/stream/test?api_key=SECRETKEY", listener.local_addr().unwrap());
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        ws.send(Message::Text("hello".into())).await.unwrap(); // ignored
        ws.send(Message::Binary(vec![0xff, 0xff, 0xff, 0x01])).await.unwrap(); // not a frame
        ws.send(Message::Binary(update(ok))).await.unwrap();
        ws.send(Message::Binary(update(failed))).await.unwrap();
        ws.send(Message::Binary(
            SubscribeUpdate {
                update_oneof: Some(subscribe_update::UpdateOneof::Slot(SubscribeUpdateSlot { slot: 42, parent: Some(41), ..Default::default() })),
                ..Default::default()
            }
            .encode_to_vec(),
        ))
        .await
        .unwrap();
        ws.close(None).await.unwrap();
    });

    let (tx, mut rx) = mpsc::channel(16);
    let stopped = session(&url, &tx).await.unwrap();
    assert!(!stopped, "a closed socket asks the caller to reconnect");

    let mut txs = vec![];
    let mut slots = vec![];
    while let Ok(e) = rx.try_recv() {
        match e {
            GeyserEvent::Transaction(t) => txs.push(t),
            GeyserEvent::Slot(s) => slots.push(s.slot),
            _ => {}
        }
    }
    assert_eq!(slots, vec![42]);
    assert_eq!(txs.len(), 2, "bad frames are skipped, real ones decode");
    // Identical to what the gRPC path produces for the same frame.
    assert_eq!(txs[0].signature, expected_ok.signature);
    assert_eq!(txs[0].success, expected_ok.success);
    assert_eq!(txs[0].instructions.len(), expected_ok.instructions.len());
    assert_eq!(txs[1].signature, expected_failed.signature);
    assert!(!txs[1].success);
    assert_eq!(txs[1].error.as_ref().map(|e| e.message.clone()), expected_failed.error.map(|e| e.message));
}

#[tokio::test]
async fn connect_errors_never_leak_the_api_key() {
    let (tx, _rx) = mpsc::channel(1);
    let err = session("ws://127.0.0.1:1/mirage/stream/x?api_key=SECRETKEY", &tx).await.unwrap_err();
    assert!(!format!("{err:#}").contains("SECRETKEY"), "{err:#}");
}
