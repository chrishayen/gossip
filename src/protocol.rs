use std::collections::HashMap;
use std::{net::SocketAddr, time::Duration};

use crate::constants::MAX_PAYLOAD_SIZE;
use crate::error::GossipError;
// use crate::message::GossipMessage;
use crate::node::Node;
use crate::{config::GossipConfig, message::GossipMessage};
use async_trait::async_trait;

use log::{debug, error, info};

use rand::{SeedableRng, rngs::StdRng, seq::index::sample};

use tokio::{
    sync::{Mutex, RwLock},
    time::{interval, sleep},
};

pub struct GossipProtocol {
    config: GossipConfig,
    local_node: Node,
    nodes: RwLock<HashMap<u32, Node>>,
    transport: Box<dyn GossipTransport>,
    rng: Mutex<StdRng>,
}

impl GossipProtocol {
    pub fn new(
        config: GossipConfig,
        local_node: Node,
        seed_peers: HashMap<u32, Node>,
        transport: Box<dyn GossipTransport>,
    ) -> Self {
        GossipProtocol {
            config,
            local_node,
            nodes: RwLock::new(seed_peers),
            transport,
            rng: Mutex::new(StdRng::from_os_rng()),
        }
    }

    pub async fn start_heartbeat(&self) {
        let mut interval = interval(self.config.heartbeat_interval);

        loop {
            let msg = GossipMessage::heartbeat(
                self.local_node.id,
                Some(self.config.message_ttl),
            );

            if let Err(e) = self.gossip(msg, None).await {
                error!("Error sending heartbeat: {}", e);
            }

            interval.tick().await;
        }
    }

    /// Start the receive loop.
    /// This will receive messages from the network and handle them,
    /// forwarding non-system messages to the user's handler.
    pub async fn start_receive(&self) {
        info!("starting receive");
        let mut buf = vec![0; MAX_PAYLOAD_SIZE];

        loop {
            match self.transport.recv_from(&mut buf).await {
                Ok((amt, src)) => {
                    info!("received packet from {}", src);

                    if amt > MAX_PAYLOAD_SIZE {
                        error!("Received packet exceeds MTU: {} bytes", amt);
                        continue;
                    }

                    if amt == 0 {
                        debug!("received empty packet");
                        continue;
                    }

                    match GossipMessage::deserialize(&buf[..amt]) {
                        Ok(msg) => {
                            self.handle_message(msg, src).await;
                        }
                        Err(e) => {
                            debug!(
                                "Deserialization error: size:{} e:{}",
                                amt, e,
                            )
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }

    async fn handle_message(
        &self,
        msg: GossipMessage,
        src: std::net::SocketAddr,
    ) {
        info!("Received message from {}", src);

        // update the nodes list if needed
        self.update_nodes(msg.from_id, src).await;

        match msg.msg_type.as_str() {
            "heartbeat" => {
                self.update_heartbeat(msg.from_id).await;
            }
            _ => {
                error!("Unknown message type: {}", msg.msg_type);
            }
        }

        self.forward(msg).await;
    }

    async fn forward(&self, mut msg: GossipMessage) {
        let exclude_id = Some(msg.from_id);
        msg.ttl -= 1;

        if msg.ttl > 0 {
            let _ = self.gossip(msg, exclude_id).await;
        }
    }

    async fn update_nodes(&self, node_id: u32, src: std::net::SocketAddr) {
        let mut nodes = self.nodes.write().await;

        // if the node is not in the peers list, add it
        if !nodes.keys().any(|n| *n == node_id) {
            info!("new node {}", node_id);
            nodes.insert(node_id, Node::new(node_id, src));
        }
    }

    async fn update_heartbeat(&self, node_id: u32) {
        let mut nodes = self.nodes.write().await;
        error!("nodes: {}", nodes.len());
        if let Some(node) = nodes.get_mut(&node_id) {
            node.update_heartbeat();
        }
    }

    async fn gossip_addresses(
        &self,
        exclude_id: Option<u32>,
    ) -> Vec<SocketAddr> {
        let mut rng = self.rng.lock().await;
        let peers = self.nodes.read().await;

        let valid_peers = peers
            .values()
            .filter(|n| !n.is_offline(self.config.offline_timeout))
            .filter(|n| n.id != self.local_node.id)
            .filter(|n| exclude_id.map(|id| n.id != id).unwrap_or(true))
            .collect::<Vec<_>>();

        let fanout = valid_peers.len().min(self.config.fanout);

        let mut addresses = sample(&mut rng, valid_peers.len(), fanout)
            .iter()
            .map(|i| valid_peers[i].addr)
            .collect::<Vec<_>>();

        let offline_peers = peers
            .values()
            .filter(|n| n.is_offline(self.config.offline_timeout))
            .filter(|n| exclude_id.map(|id| n.id != id).unwrap_or(true))
            .collect::<Vec<_>>();

        // add an offline peer if there is one
        let rescue_peers =
            sample(&mut rng, offline_peers.len(), 1.min(offline_peers.len()))
                .iter()
                .map(|i| offline_peers[i].addr)
                .collect::<Vec<_>>();

        if !rescue_peers.is_empty() {
            addresses.extend(rescue_peers);
        }

        if addresses.is_empty() {
            error!("no addresses to gossip to");
        }

        addresses
    }

    pub async fn gossip(
        &self,
        msg: GossipMessage,
        exclude_id: Option<u32>,
    ) -> Result<(), GossipError> {
        let addresses = self.gossip_addresses(exclude_id).await;
        info!("gossiping to {:?}", addresses);

        for addr in addresses {
            let buf = GossipMessage::serialize(&msg)?;
            self.transport.write(&buf, addr.to_string()).await?;
            sleep(Duration::from_millis(1)).await;
        }

        Ok(())
    }
}

#[async_trait]
pub trait GossipTransport: Send + Sync {
    async fn write(
        &self,
        buf: &heapless::Vec<u8, MAX_PAYLOAD_SIZE>,
        addr: String,
    ) -> Result<usize, GossipError>;

    async fn recv_from(
        &self,
        buf: &mut Vec<u8>,
    ) -> Result<(usize, SocketAddr), GossipError>;

    async fn get_ip(&self) -> Result<String, GossipError>;
}
