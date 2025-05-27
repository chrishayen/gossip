use std::{path::PathBuf, sync::Arc};

use tailscale_api::Tailscale;
use tokio::{sync::Mutex, task};
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

pub struct Network {
    ts: Arc<Mutex<TSNet>>,
    api: Arc<Mutex<Tailscale>>,
}

impl Network {
    pub fn new(state_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let id = make_id();
        let config = ConfigBuilder::new().hostname(&id);
        let config_with_dir = if let Some(state_dir) = state_dir {
            config.dir(state_dir.to_str().unwrap())
        } else {
            config
        };

        let config = config_with_dir.build()?;
        let api = Tailscale::new_from_env();
        let api = Arc::new(Mutex::new(api));

        Ok(Self {
            ts: Arc::new(Mutex::new(TSNet::new(config)?)),
            api,
        })
    }

    pub async fn join(&self) -> Result<(), NetworkError> {
        let ts = self.ts.clone();
        task::spawn_blocking(move || {
            let mut ts = ts.blocking_lock();
            ts.up().unwrap();
        })
        .await
        .map_err(|e| NetworkError::TSNetError(e.to_string()))
    }

    pub async fn get_peers(&self) -> Result<Vec<String>, NetworkError> {
        let api = self.api.clone();
        let devices = {
            let api = api.lock().await;
            api.list_devices().await.unwrap()
        };
        Ok(devices.iter().map(|d| d.hostname.clone()).collect())
    }
}
