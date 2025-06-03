use std::fmt::Debug;

use crate::constants::MAX_PAYLOAD_SIZE;
use crate::node::Node;
use heapless::Vec;
use postcard;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GossipMessage {
    pub from_id: u32,
    pub ttl: u8,
    pub msg_type: String,
    pub payload: Vec<u8, MAX_PAYLOAD_SIZE>,
}

impl GossipMessage {
    pub fn serialize(
        msg: &GossipMessage,
    ) -> Result<Vec<u8, MAX_PAYLOAD_SIZE>, postcard::Error> {
        postcard::to_vec(msg)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    pub fn heartbeat(from_id: u32, ttl: Option<u8>) -> GossipMessage {
        GossipMessage {
            from_id,
            ttl: ttl.unwrap_or(3),
            msg_type: "heartbeat".to_string(),
            payload: postcard::to_vec("").unwrap(),
        }
    }

    pub fn update(node: Node, ttl: Option<u8>) -> GossipMessage {
        GossipMessage {
            from_id: node.id,
            ttl: ttl.unwrap_or(3),
            msg_type: "update".to_string(),
            payload: postcard::to_vec(&node).unwrap(),
        }
    }
}
