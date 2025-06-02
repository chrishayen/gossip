mod config;
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

    let mut handles = Vec::new();

    handles.push(thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(protocol_clone.start_heartbeat());
    }));

    handles.push(thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(protocol.start_receive());
    }));

    for handle in handles {
        match handle.join() {
            Ok(_) => println!("thread done"),
            Err(e) => println!("Thread panicked: {:?}", e),
        }
    }

    // let handle = thread::spawn(move || {
    //     protocol_clone.start_receive().await
    // });
    // let mut handles = Vec::<JoinHandle<()>>::new();

    // // let heartbeat =
    // handles.push(tokio::task::spawn(async move {
    //     protocol_clone.start_heartbeat().await
    // }));

    // handles.push(tokio::task::spawn(
    //     async move { protocol.start_receive().await },
    // ));

    // let _ = futures::future::select_all(handles).await;

    Ok(())
}
