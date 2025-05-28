use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Online,
    Offline,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub id: u32,
    pub addr: SocketAddr,
    pub status: NodeStatus,
    pub last_heartbeat: SystemTime,
}

impl Node {
    pub fn new(id: u32, addr: SocketAddr) -> Self {
        Node {
            id,
            addr,
            status: NodeStatus::Online,
            last_heartbeat: SystemTime::now(),
        }
    }

    pub fn is_offline(&self, timeout: Duration) -> bool {
        matches!(self.status, NodeStatus::Offline)
            || SystemTime::now()
                .duration_since(self.last_heartbeat)
                .unwrap_or(Duration::from_secs(0))
                > timeout
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now();
        self.status = NodeStatus::Online;
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.addr == other.addr
    }
}
