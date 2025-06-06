use std::collections::HashMap;

use env_logger::Env;
use gossip::{GossipConfig, start, tailscale, util};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let tmp_dir = tempdir::TempDir::new("gossip-test").unwrap();

    let mut gossip_config = GossipConfig::default();

    // connect to tailscale
    let mut ts = tailscale::Tailscale::new(
        gossip_config.clone(),
        Some(tmp_dir.path().to_path_buf()),
    )
    .unwrap();

    ts.join_network().await.unwrap();
    ts.listen().await.unwrap();

    let seed_peers = ts.get_peers().await.unwrap();
    let seed_peers = seed_peers
        .iter()
        .map(|p| (p.id, p.clone()))
        .collect::<HashMap<_, _>>();

    let ip = ts.get_ip().await.unwrap();
    let ip = util::extract_ipv4(ip.as_str()).unwrap();

    info!("ip: {}", ip);

    gossip_config.ip_address = ip.to_string();
    gossip_config.node_name = ts.id.clone();

    start(gossip_config, Box::new(ts), seed_peers)
        .await
        .unwrap();
}
