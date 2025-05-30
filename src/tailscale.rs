use crate::error::GossipError;

use crate::GossipConfig;
use crate::node::Node;
use crate::protocol::GossipTransport;

use crate::util::{extract_ipv4, hash_node_name, make_id};
use async_trait::async_trait;
use log::info;
use std::net::{SocketAddr, UdpSocket};
use std::{os::fd::FromRawFd, path::PathBuf, sync::Arc};
use tailscale_api::Tailscale as TailscaleApi;
use tokio::sync::Mutex;
use tsnet::{ConfigBuilder, TSNet};

pub struct Tailscale {
    ts: TSNet,
    api: Arc<Mutex<TailscaleApi>>,
    id: String,
    gossip_config: GossipConfig,
}

#[async_trait]
impl GossipTransport for Tailscale {
    async fn write_udp(
        &self,
        buf: &[u8],
        addr: String,
    ) -> Result<usize, GossipError> {
        let fd = self
            .ts
            .dial("udp", addr.as_str())
            .map_err(GossipError::NetworkError)?;
        let socket = unsafe { UdpSocket::from_raw_fd(fd) };
        info!("sending to {} {:?}", addr, buf);
        socket.send(buf).map_err(GossipError::Io)
    }
}
// impl GossipSocket for Tailscale {
//     fn recv_from(
//         &self,
//         listener: i32,
//         buf: &mut [u8],
//     ) -> Result<(usize, std::net::SocketAddr), GossipError> {
//         let ts = self.ts.blocking_lock();

//         let conn = ts
//             .accept(listener)
//             .map_err(|e| GossipError::NetworkError(e))?;

//         let socket = unsafe { UdpSocket::from_raw_fd(conn) };
//         let remote_addr = ts
//             .get_remote_addr(conn, listener)
//             .map_err(|e| GossipError::NetworkError(e))?;
//         let remote_addr = extract_ipv4(&remote_addr)?;
//         let remote_addr = SocketAddr::from((remote_addr, 42069));

//         match socket.recv(buf) {
//             Ok(len) => Ok((len, remote_addr)),
//             Err(e) => Err(GossipError::NetworkError(e.to_string())),
//         }
//     }

//     fn send_to(
//         &self,
//         buf: &[u8],
//         addr: std::net::SocketAddr,
//     ) -> Result<(), GossipError> {
//         info!("Sending to {} {:?}", addr, buf);
//         Ok(())
//     }
// }

impl Tailscale {
    pub fn new(
        gossip_config: GossipConfig,
        state_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let id = make_id();
        let default_config = ConfigBuilder::new().hostname(&id);

        let config = if let Some(state_dir) = state_dir {
            default_config.dir(state_dir.to_str().unwrap())
        } else {
            default_config
        };

        let config = config.build()?;
        let api = TailscaleApi::new_from_env();
        let api = Arc::new(Mutex::new(api));
        let ts = TSNet::new(config)?;

        Ok(Self {
            ts,
            api,
            id,
            gossip_config,
        })
    }

    pub async fn join_network(&mut self) -> Result<String, GossipError> {
        self.ts.up().map_err(|e| GossipError::NetworkError(e))?;
        Ok(self.id.clone())
    }

    pub async fn get_peers(&self) -> Result<Vec<Node>, GossipError> {
        let api = self.api.lock().await;
        let devices = api
            .list_devices()
            .await
            .map_err(|e| GossipError::PeerError(e))?;

        let nodes: Vec<Node> = devices
            .into_iter()
            .filter_map(|d| {
                let name = d.hostname;
                let node_id = hash_node_name(&name);
                let ip_str = d.addresses.first()?;
                let ip = extract_ipv4(ip_str).ok()?;
                let addr = SocketAddr::from((ip, 42069));
                Some(Node::new(node_id, addr))
            })
            .collect();

        Ok(nodes)
    }

    pub async fn get_ip(&self) -> Result<String, GossipError> {
        let ip = self
            .ts
            .get_ips(None)
            .map_err(|e| GossipError::NetworkError(e))?;

        Ok(ip)
    }

    pub async fn listen(&self) -> Result<i32, GossipError> {
        self.ts
            .listen("udp", &format!(":{}", self.gossip_config.gossip_port))
            .map_err(|e| GossipError::NetworkError(e))
    }
}
