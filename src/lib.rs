mod config;
mod error;
mod message;
mod node;
mod protocol;
mod retry;
mod tailscale;
mod util;
use log::error;
use retry::retry;
use std::{path::PathBuf, process::exit, time::Duration};

pub async fn start(state_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let peer = tailscale::Peer::new(state_dir)?;
    let rs = retry(
        || peer.join_network(),
        Some(3),
        Some(Duration::from_secs(10)),
    )
    .await;

    if rs.is_err() {
        error!("Failed to join network: {:?}", rs.err());
        exit(1)
    }

    println!("Joined network {}", rs.unwrap());
    peer.listen().await?;

    Ok(())
}
