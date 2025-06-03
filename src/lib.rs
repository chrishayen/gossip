mod config;
pub mod constants;
mod error;
pub mod message;
mod node;
mod protocol;
mod retry;
pub mod tailscale;
pub mod util;

use error::GossipError;
use node::Node;
use protocol::GossipTransport;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
};

pub use config::GossipConfig;

use crate::util::hash_node_name;

pub async fn start(
    gossip_config: GossipConfig,
    transport: Box<dyn GossipTransport>,
    seed_peers: Vec<Node>,
) -> Result<(), GossipError> {
    // get the ip from the transport
    // and define the local node
    let ip = gossip_config.ip_address.parse::<Ipv4Addr>().unwrap();
    let local = Node::new(
        hash_node_name(&gossip_config.node_name),
        SocketAddr::from((ip, gossip_config.gossip_port)),
    );

    // initialize the gossip protocol
    let p = Arc::new(protocol::GossipProtocol::new(
        gossip_config,
        local,
        seed_peers,
        transport,
    ));

    let p1 = Arc::clone(&p);

    let handles = [
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(p1.start_heartbeat());
        }),
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(p.start_receive());
        }),
    ];

    for handle in handles {
        match handle.join() {
            Ok(_) => println!("thread done"),
            Err(e) => println!("Thread panicked: {:?}", e),
        }
    }

    Ok(())
}
