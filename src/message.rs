use serde::{Deserialize, Serialize};

use crate::node::{Node, NodeStatus};

#[derive(Serialize, Deserialize, Debug)]
pub enum GossipMessage {
    Heartbeat { sender_id: u32 },
    Update { node: Node },
    Application { sender_id: u32, data: Vec<u8> }, // Custom app data
}

impl GossipMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }
}
