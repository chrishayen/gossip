use std::path::PathBuf;

mod discovery;
mod util;

pub async fn start(state_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let discovery = discovery::Network::new(state_dir)?;
    discovery.join().await?;
    let peers = discovery.get_peers().await?;
    println!("Peers: {:?}", peers);

    Ok(())
}
