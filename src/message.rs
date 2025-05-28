use crate::node::Node;
use heapless::Vec;
use postcard;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum GossipMessage {
    Heartbeat { sender_id: u32 },
    Update { node: Node },
}

impl GossipMessage {
    pub fn serialize(&self) -> Result<Vec<u8, 1024>, postcard::Error> {
        postcard::to_vec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }
}
