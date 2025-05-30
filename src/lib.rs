mod config;
mod error;
pub mod message;
mod node;
mod protocol;
mod retry;
pub mod tailscale;
mod util;

use error::GossipError;
use node::Node;
use protocol::GossipTransport;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock};

pub use config::GossipConfig;
use log::info;
use tokio::{select, time::interval};

pub async fn start(
    gossip_config: GossipConfig,
    transport: Box<dyn GossipTransport>,
    seed_peers: Vec<Node>,
) -> Result<(), GossipError> {
    let local_node = Node::new(
        0,
        SocketAddr::from((Ipv4Addr::LOCALHOST, gossip_config.gossip_port)),
    );

    let mut protocol = Arc::new(RwLock::new(protocol::GossipProtocol::new(
        gossip_config,
        local_node,
        seed_peers,
        transport,
    )));

    let _ = tokio::task::spawn({
        let protocol = Arc::clone(&protocol);
        async move { protocol.read().await.start_heartbeat().await }
    });

    let _ = tokio::task::spawn(async move {
        protocol.write().await.start_receive().await
    });

    tokio::time::sleep(Duration::from_secs(6)).await;

    Ok(())
}
