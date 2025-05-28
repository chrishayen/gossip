mod config;
mod message;
mod node;
mod protocol;
mod retry;
mod tailscale;
mod util;

use retry::retry;
use std::{path::PathBuf, process::exit, time::Duration};

pub async fn start(state_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

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

    println!("Joined network {}", rs.unwrap());
    peer.listen().await?;

    Ok(())
}
