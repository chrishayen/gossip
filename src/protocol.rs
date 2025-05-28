use log::{error, info};
use rand::seq::SliceRandom;
use std::net::UdpSocket;
use std::time::Duration;
use thiserror::Error;

use crate::config::GossipConfig;
use crate::message::GossipMessage;
use crate::node::{Node, NodeStatus};

#[derive(Error, Debug)]
pub enum GossipError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
}

pub trait GossipHandler {
    fn handle_message(&mut self, msg: GossipMessage, sender: std::net::SocketAddr);
}

pub struct GossipProtocol<H: GossipHandler> {
    socket: UdpSocket,
    config: GossipConfig,
    local_node: Node,
    peers: Vec<Node>,
    handler: H,
}

impl<H: GossipHandler> GossipProtocol<H> {
    pub fn new(local_node: Node, socket: UdpSocket, config: GossipConfig, handler: H) -> Self {
        socket
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");
        GossipProtocol {
            socket,
            config,
            local_node,
            peers: Vec::new(),
            handler,
        }
    }

    pub fn add_peer(&mut self, peer: Node) {
        self.peers.push(peer);
    }

    pub fn send_heartbeat(&self) -> Result<(), GossipError> {
        let msg = GossipMessage::Heartbeat {
            sender_id: self.local_node.id,
        };
        self.broadcast(&msg)?;
        Ok(())
    }

    pub fn gossip(&mut self) -> Result<(), GossipError> {
        let mut rng = rand::thread_rng();
        let targets = self
            .peers
            .choose_multiple(&mut rng, self.config.fanout)
            .collect::<Vec<_>>();

        for peer in targets {
            if !peer.is_offline(self.config.offline_timeout) {
                let msg = GossipMessage::Update {
                    node: self.local_node.clone(),
                };
                self.send_to(&msg, peer.addr)?;
            }
        }
        Ok(())
    }

    pub fn receive(&mut self) -> Result<(), GossipError> {
        let mut buf = vec![0; self.config.max_payload_size];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if amt > self.config.max_payload_size {
                        error!("Received packet exceeds MTU: {} bytes", amt);
                        continue;
                    }
                    match GossipMessage::deserialize(&buf[..amt]) {
                        Ok(msg) => {
                            self.handle_message(&msg, src);
                            self.handler.handle_message(msg, src);
                        }
                        Err(e) => error!("Deserialization error: {}", e),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(GossipError::Io(e)),
            }
        }
        Ok(())
    }

    fn send_to(&self, msg: &GossipMessage, addr: std::net::SocketAddr) -> Result<(), GossipError> {
        let buf = msg.serialize()?;
        if buf.len() > self.config.max_payload_size {
            return Err(GossipError::Serialization(
                postcard::Error::SerializeBufferTooSmall,
            ));
        }
        self.socket.send_to(&buf, addr)?;
        Ok(())
    }

    fn broadcast(&self, msg: &GossipMessage) -> Result<(), GossipError> {
        for peer in &self.peers {
            if !peer.is_offline(self.config.offline_timeout) {
                self.send_to(msg, peer.addr)?;
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, msg: &GossipMessage, src: std::net::SocketAddr) {
        match msg {
            GossipMessage::Heartbeat { sender_id } => {
                if let Some(node) = self.peers.iter_mut().find(|n| n.id == *sender_id) {
                    node.update_heartbeat();
                    info!("Heartbeat from node {}", sender_id);
                }
            }
            GossipMessage::Update { node } => {
                if let Some(existing) = self.peers.iter_mut().find(|n| n.id == node.id) {
                    existing.addr = node.addr;
                    existing.status = node.status.clone();
                    existing.update_heartbeat();
                } else {
                    self.peers.push(node.clone());
                }
                info!("Updated node {} from {}", node.id, src);
            }
            GossipMessage::Application { .. } => {
                // Handled by custom handler
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
