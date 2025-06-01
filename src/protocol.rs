use std::net::SocketAddr;

use crate::config::GossipConfig;
use crate::error::GossipError;
use crate::message::GossipMessage;
use crate::node::Node;
use async_trait::async_trait;

use log::{error, info};

use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};

pub const MAX_PAYLOAD_SIZE: usize = 1024;

pub struct GossipProtocol {
    config: GossipConfig,
    local_node: Node,
    peers: RwLock<Vec<Node>>,
    transport: Box<dyn GossipTransport>,
    rng: Mutex<StdRng>,
}

impl GossipProtocol {
    pub fn new(
        config: GossipConfig,
        local_node: Node,
        seed_peers: Vec<Node>,
        transport: Box<dyn GossipTransport>,
    ) -> Self {
        GossipProtocol {
            config,
            local_node,
            peers: RwLock::new(seed_peers),
            transport,
            rng: Mutex::new(StdRng::from_os_rng()),
        }
    }

    // pub fn add_peer(&self, peer: Node) {}

    pub async fn start_heartbeat(&self) {
        let mut interval = interval(self.config.heartbeat_interval);

        loop {
            if let Err(e) = self.send_heartbeat().await {
                error!("Error sending heartbeat: {}", e);
            }
            interval.tick().await;
        }
    }

    pub async fn send_heartbeat(&self) -> Result<(), GossipError> {
        let msg = GossipMessage::Heartbeat {
            sender_id: self.local_node.id,
        };

        let mut rng = self.rng.lock().await;
        let peers = self.peers.read().await;
        let targets = peers
            .choose_multiple(&mut rng, self.config.fanout)
            .collect::<Vec<_>>();

        for target in targets {
            self.transport
                .write(&msg.serialize()?, target.addr.to_string())
                .await?;
        }
        Ok(())
    }

    pub async fn start_receive(&self) {
        info!("starting receive");
        let mut buf = vec![0; MAX_PAYLOAD_SIZE];

        loop {
            match self.transport.recv_from(&mut buf).await {
                Ok((amt, src)) => {
                    if amt > MAX_PAYLOAD_SIZE {
                        error!("Received packet exceeds MTU: {} bytes", amt);
                        continue;
                    }
                    match GossipMessage::deserialize(&buf[..amt]) {
                        Ok(msg) => {
                            self.handle_message(msg, src).await;
                        }
                        Err(e) => {
                            error!("Deserialization error: {}", e)
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }

    // pub async fn gossip(&mut self) -> Result<(), GossipError> {
    //     let mut rng = self.rng.lock().await;
    //     let peers = self.peers.read().await;
    //     let targets = peers
    //         .choose_multiple(&mut rng, self.config.fanout)
    //         .collect::<Vec<_>>();

    //     for peer in targets {
    //         if !peer.is_offline(self.config.offline_timeout) {
    //             let msg = GossipMessage::Update {
    //                 node: self.local_node.clone(),
    //             };
    //             let buf = msg.serialize()?;
    //             // self.socket.send_to(&buf, peer.addr)?;
    //             // self.sender(msg, peer.addr);
    //             self.transport.write(&buf, peer.addr.to_string()).await?;
    //         }
    //     }
    //     Ok(())
    // }

    // fn send_to(
    //     &self,
    //     msg: &GossipMessage,
    //     addr: std::net::SocketAddr,
    // ) -> Result<(), GossipError> {
    //     let buf = msg.serialize()?;
    //     if buf.len() > self.config.max_payload_size {
    //         return Err(GossipError::Serialization(
    //             postcard::Error::SerializeBufferFull,
    //         ));
    //     }
    //     self.socket.send_to(&buf, addr)?;
    //     Ok(())
    // }

    // fn broadcast(&self, msg: &GossipMessage) -> Result<(), GossipError> {
    //     for peer in &self.peers {
    //         if !peer.is_offline(self.config.offline_timeout) {
    //             let buf = msg.serialize()?;
    //             let socket = Arc::clone(&self.socket);
    //             socket.send_to(&buf, peer.addr)?;
    //         } else {
    //             info!("Skipping offline node {}", peer.id);
    //         }
    //     }
    //     Ok(())
    // }

    async fn handle_message(
        &self,
        msg: GossipMessage,
        src: std::net::SocketAddr,
    ) {
        let mut peers = self.peers.write().await;

        match msg {
            GossipMessage::Heartbeat { sender_id } => {
                if let Some(node) = peers.iter_mut().find(|n| n.id == sender_id)
                {
                    node.update_heartbeat();
                    info!("Heartbeat from node {}", sender_id);
                }
            }
            GossipMessage::Update { node } => {
                if let Some(existing) =
                    peers.iter_mut().find(|n| n.id == node.id)
                {
                    existing.addr = node.addr;
                    existing.status = node.status.clone();
                    existing.update_heartbeat();
                } else {
                    peers.push(node.clone());
                }
                info!("Updated node {} from {}", node.id, src);
            }
        }
    }

    // pub fn check_offline_nodes(&mut self) {
    //     let peers = self.peers.write().unwrap();

    //     for node in peers {
    //         if node.is_offline(self.config.offline_timeout) {
    //             node.status = NodeStatus::Offline;
    //             info!("Marked node {} as offline", node.id);
    //         }
    //     }
    // }
}

#[async_trait]
pub trait GossipTransport: Send + Sync {
    async fn write(
        &self,
        buf: &heapless::Vec<u8, 1024>,
        addr: String,
    ) -> Result<usize, GossipError>;

    async fn recv_from(
        &self,
        buf: &mut Vec<u8>,
    ) -> Result<(usize, SocketAddr), GossipError>;

    async fn get_ip(&self) -> Result<String, GossipError>;
}
