use std::fmt::Debug;

use crate::node::Node;
use heapless::Vec;
use postcard;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GossipMessage {
    pub from_id: u32,
    pub ttl: u8,
    pub msg_type: String,
    pub payload: Vec<u8, 1024>,
}

impl GossipMessage {
    pub fn serialize(&self) -> Result<Vec<u8, 1024>, postcard::Error> {
        postcard::to_vec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    pub fn heartbeat(from_id: u32) -> GossipMessage {
        GossipMessage {
            from_id,
            ttl: 3,
            msg_type: "heartbeat".to_string(),
            payload: postcard::to_vec("").unwrap(),
        }
    }

    pub fn update(node: Node) -> GossipMessage {
        GossipMessage {
            from_id: node.id,
            ttl: 1,
            msg_type: "update".to_string(),
            payload: postcard::to_vec(&node).unwrap(),
        }
    }

    pub fn serialize_payload<T: Serialize>(
        msg: T,
    ) -> Result<Vec<u8, 1024>, postcard::Error> {
        postcard::to_vec(&msg)
    }

    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(
        msg: &[u8],
    ) -> Result<T, postcard::Error> {
        postcard::from_bytes(msg)
    }
}
