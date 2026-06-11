use crate::SlotInfo;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::info;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

pub async fn subscribe_slots(
    mut client: GeyserGrpcClient<impl yellowstone_grpc_client::Interceptor>,
    sender: mpsc::Sender<SlotInfo>,
) -> Result<()> {
    let mut slots_map = HashMap::new();
    slots_map.insert("client".to_string(), SubscribeRequestFilterSlots {
        filter_by_commitment: None,
    });

    let request = SubscribeRequest {
        slots: slots_map,
        ..Default::default()
    };

    let (_, mut stream) = client.subscribe_with_request(Some(request)).await?;

    while let Some(message) = stream.next().await {
        if let Ok(msg) = message {
            if let Some(subscribe_update::UpdateOneof::Slot(slot)) = msg.update_oneof {
                let slot_info = SlotInfo {
                    slot: slot.slot,
                    parent: slot.parent.unwrap_or(0),
                    timestamp: Utc::now(),
                };
                info!("New slot: {}", slot_info.slot);
                sender.send(slot_info).await?;
            }
        }
    }

    Ok(())
}
