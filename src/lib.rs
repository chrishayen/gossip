mod config;
mod error;
pub mod message;
mod node;
mod protocol;
mod retry;
pub mod tailscale;
pub mod util;

use error::GossipError;
use log::info;
use node::Node;
use protocol::GossipTransport;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

pub use config::GossipConfig;

pub async fn start(
    gossip_config: GossipConfig,
    transport: Box<dyn GossipTransport>,
    seed_peers: Vec<Node>,
) -> Result<(), GossipError> {
    // get the ip from the transport
    // and define the local node
    let ip_addr = gossip_config.ip_address.parse::<Ipv4Addr>().unwrap();

    let local_node =
        Node::new(0, SocketAddr::from((ip_addr, gossip_config.gossip_port)));

    // initialize the gossip protocol
    let protocol = Arc::new(protocol::GossipProtocol::new(
        gossip_config,
        local_node,
        seed_peers,
        transport,
    ));

    let protocol_clone = Arc::clone(&protocol);
    let heartbeat =
        tokio::task::spawn(
            async move { protocol_clone.start_heartbeat().await },
        );
    let receive =
        tokio::task::spawn(async move { protocol.start_receive().await });

    let _ = tokio::join!(receive, heartbeat);

    Ok(())
}
