mod config;
mod error;
pub mod message;
mod node;
mod protocol;
mod retry;
mod tailscale;
mod util;

use log::info;
use node::Node;
pub use protocol::GossipHandler;
use retry::retry;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use util::{extract_ipv4, hash_node_name};

pub async fn start(
    handler: impl GossipHandler,
    state_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::GossipConfig::default();
    let ts = tailscale::Tailscale::new(state_dir)?;
    let name = retry(
        || ts.join_network(),
        Some(3),
        Some(Duration::from_secs(10)),
    )
    .await?;

    info!("joined network {}", name);

    let peers = retry(
        || ts.get_peers(),
        Some(3),
        Some(Duration::from_secs(10)),
    )
    .await?;

    info!("potential peers: {:?}", peers);

    let ip4 =
        retry(|| ts.get_ip(), Some(3), Some(Duration::from_secs(10)))
            .await?;

    let ip4 = extract_ipv4(&ip4)?;
    let ip4 = SocketAddr::from((ip4, 42069));
    info!("ip: {:?}", ip4);

    let node_id = hash_node_name(&name);
    let me = Node::new(node_id, ip4);

    let udp_fd = ts.listen().await?;

    let mut protocol = protocol::GossipProtocol::new(
        me, udp_fd, config, handler, ts,
    );

    for peer in peers {
        protocol.add_peer(peer);
    }

    protocol.send_heartbeat()?;
    // protocol.gossip().await?;

    // let handle = tokio::spawn(async move { peer.listen().await });

    // info!("peers: {:?}", peers);

    Ok(())
}
