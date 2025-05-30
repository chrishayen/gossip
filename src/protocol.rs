use std::net::SocketAddr;
use std::ops::DerefMut;

use crate::config::GossipConfig;
use crate::error::GossipError;
use crate::message::GossipMessage;
use crate::node::{Node, NodeStatus};
use async_trait::async_trait;

use log::{error, info};
use tokio::time::interval;
// use rand::prelude::IndexedRandom;

pub const MAX_PAYLOAD_SIZE: usize = 1024;

pub struct GossipProtocol {
    config: GossipConfig,
    local_node: Node,
    peers: Vec<Node>,
    transport: Box<dyn GossipTransport>,
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
            peers: seed_peers,
            transport,
        }
    }

    pub fn add_peer(&mut self, peer: Node) {
        self.peers.push(peer);
    }

    pub async fn start_heartbeat(&mut self) -> Result<(), GossipError> {
        info!("starting heartbeat");
        let mut interval = interval(self.config.heartbeat_interval);

        loop {
            self.send_heartbeat().await?;
            interval.tick().await;
        }
    }

    pub async fn send_heartbeat(&mut self) -> Result<(), GossipError> {
        let msg = GossipMessage::Heartbeat {
            sender_id: self.local_node.id,
        };
        info!("sending a heartbeart from and to {}", self.local_node.id);

        let len = self
            .transport
            .write(&msg.serialize()?, self.local_node.addr.to_string())
            .await?;

        info!("sent {}", len);
        Ok(())
    }

    pub async fn start_receive(&mut self) -> Result<(), GossipError> {
        info!("starting receive");
        loop {
            self.receive().await?;
        }
    }

    pub async fn receive(&mut self) -> Result<(), GossipError> {
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
                            self.handle_message(msg, src);
                        }
                        Err(e) => {
                            error!("Deserialization error: {}", e)
                        }
                    }
                }
                Err(e) => {
                    return Err(GossipError::NetworkError(e.to_string()));
                }
            }
        }
    }

    // pub fn gossip(&mut self) -> Result<(), GossipError> {
    //     let mut rng = rand::rng();
    //     let targets = self
    //         .peers
    //         .choose_multiple(&mut rng, self.config.fanout)
    //         .collect::<Vec<_>>();

    //     for peer in targets {
    //         if !peer.is_offline(self.config.offline_timeout) {
    //             let msg = GossipMessage::Update {
    //                 node: self.local_node.clone(),
    //             };
    //             let buf = msg.serialize()?;
    //             // self.socket.send_to(&buf, peer.addr)?;
    //             self.sender(msg, peer.addr);
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

    fn handle_message(
        &mut self,
        msg: GossipMessage,
        src: std::net::SocketAddr,
    ) {
        match msg {
            GossipMessage::Heartbeat { sender_id } => {
                if let Some(node) =
                    self.peers.iter_mut().find(|n| n.id == sender_id)
                {
                    node.update_heartbeat();
                    info!("Heartbeat from node {}", sender_id);
                }
            }
            GossipMessage::Update { node } => {
                if let Some(existing) =
                    self.peers.iter_mut().find(|n| n.id == node.id)
                {
                    existing.addr = node.addr;
                    existing.status = node.status.clone();
                    existing.update_heartbeat();
                } else {
                    self.peers.push(node.clone());
                }
                info!("Updated node {} from {}", node.id, src);
            }
        }
    }

    pub fn check_offline_nodes(&mut self) {
        for node in &mut self.peers {
            if node.is_offline(self.config.offline_timeout) {
                node.status = NodeStatus::Offline;
                info!("Marked node {} as offline", node.id);
            }
        }
    }
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
}
