use bincode::serialize;
use solana_sdk::transaction::VersionedTransaction;

use crate::jito::protos::packet::{Meta as ProtoMeta, Packet as ProtoPacket};

pub fn proto_packet_from_versioned_tx(tx: &VersionedTransaction) -> ProtoPacket {
    let data = serialize(tx).expect("serializes");
    let size = data.len() as u64;
    ProtoPacket {
        data,
        meta: Some(ProtoMeta {
            size,
            addr: "".to_string(),
            port: 0,
            flags: None,
            sender_stake: 0,
        }),
    }
}
