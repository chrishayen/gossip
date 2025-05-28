use retry::retry;
use std::{path::PathBuf, process::exit, time::Duration};
mod retry;
mod tailscale;
mod util;

pub async fn start(state_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let peer = tailscale::Peer::new(state_dir)?;
    let rs = retry(
        || peer.join_network(),
        Some(1),
        Some(Duration::from_secs(1)),
    )
    .await;

    if rs.is_err() {
        println!("Failed to join network: {:?}", rs.err());
        exit(1)
    }

    let peers = peer.get_peers().await?;
    println!("Peers: {:?}", peers);

    Ok(())
}
