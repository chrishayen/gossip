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
    time::Duration,
};

pub use config::GossipConfig;
use log::info;
use tokio::{select, time::interval};

pub async fn start(
    gossip_config: GossipConfig,
    transport: Box<dyn GossipTransport>,
) -> Result<(), GossipError> {
    // TODO: loop on outgoing gossip
    // TODO: loop on incoming gossip
    let local_node = Node::new(
        0,
        SocketAddr::from((Ipv4Addr::LOCALHOST, gossip_config.gossip_port)),
    );
    let mut heartbeat_interval = interval(gossip_config.heartbeat_interval);

    let mut protocol =
        protocol::GossipProtocol::new(gossip_config, local_node, transport);

    let _ = tokio::task::spawn(async move {
        loop {
            select! {
                _ = heartbeat_interval.tick() => {
                    info!("sending heartbeat");
                    let res = protocol.send_heartbeat().await;

                    if let Err(e) = res {
                        info!("error sending heartbeat: {}", e);
                    }
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(6)).await;

    Ok(())
}
