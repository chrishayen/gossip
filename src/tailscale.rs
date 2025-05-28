use std::{path::PathBuf, sync::Arc};

use tailscale_api::Tailscale;
use tokio::{sync::Mutex, task::JoinError};
use tsnet::{ConfigBuilder, TSNet};

use crate::util::make_id;

#[derive(Debug)]
pub enum NetworkError {
    TSNetError(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetworkError::TSNetError(e) => write!(f, "TSNet error: {}", e),
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<JoinError> for NetworkError {
    fn from(err: JoinError) -> Self {
        NetworkError::TSNetError(err.to_string())
    }
}

pub struct Peer {
    ts: Arc<Mutex<TSNet>>,
    api: Arc<Mutex<Tailscale>>,
}

impl Peer {
    pub fn new(state_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let id = make_id();
        let default_config = ConfigBuilder::new().hostname(&id);

        let config = if let Some(state_dir) = state_dir {
            default_config.dir(state_dir.to_str().unwrap())
        } else {
            default_config
        };

        let config = config.build()?;
        let api = Tailscale::new_from_env();
        let api = Arc::new(Mutex::new(api));
        let ts = Arc::new(Mutex::new(TSNet::new(config)?));

        Ok(Self { ts, api })
    }

    pub async fn join_network(&self) -> Result<(), NetworkError> {
        let mut ts = self.ts.lock().await;
        ts.up().map_err(|e| NetworkError::TSNetError(e.to_string()))
    }

    pub async fn get_peers(&self) -> Result<Vec<String>, NetworkError> {
        let api = self.api.lock().await;
        let devices = api
            .list_devices()
            .await
            .map_err(|e| NetworkError::TSNetError(e.to_string()))?;
        Ok(devices.into_iter().map(|d| d.hostname.clone()).collect())
    }
}
