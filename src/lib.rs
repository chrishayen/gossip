mod config;
mod error;
pub mod message;
mod node;
mod protocol;
mod retry;
mod tailscale;
mod util;

use std::path::PathBuf;

use log::{error, info};
use protocol::GossipHandler;

pub async fn start(
    handler: Box<dyn GossipHandler>,
    state_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    //
    //
    let config = config::GossipConfig::default();
    let ts = tailscale::Tailscale::new(state_dir)?;

    Ok(())
}
