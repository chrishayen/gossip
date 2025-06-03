use crate::GossipConfig;
use crate::constants::MAX_PAYLOAD_SIZE;
use crate::error::GossipError;
use crate::node::Node;
use crate::protocol::GossipTransport;

use crate::util::{extract_ipv4, hash_node_name, make_id};
use async_trait::async_trait;

use log::info;
use std::net::{SocketAddr, UdpSocket};

use std::os::fd::AsFd;
use std::path::PathBuf;
use tailscale_api::Tailscale as TailscaleApi;
use tsnet::{ConfigBuilder, TSNet, TailscaleListener};

pub struct Tailscale {
    pub id: String,
    gossip_config: GossipConfig,
    ts: TSNet,
    api: TailscaleApi,
    listener: Option<TailscaleListener>,
}

impl Tailscale {
    pub fn new(
        gossip_config: GossipConfig,
        state_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let node_id = make_id(&gossip_config.prefix);
        let api = TailscaleApi::new_from_env();

        let ts_config = if let Some(state_dir) = state_dir {
            ConfigBuilder::new()
                .hostname(&node_id)
                .dir(state_dir.to_str().unwrap())
        } else {
            ConfigBuilder::new().hostname(&node_id)
        };
        let ts_config = ts_config.build()?;
        let ts = TSNet::new(ts_config)?;

        Ok(Self {
            id: node_id,
            gossip_config,
            ts,
            api,
            listener: None,
        })
    }

    pub async fn listen(&mut self) -> Result<(), GossipError> {
        if self.listener.is_none() {
            let ip_addr = self.get_ip().await?;
            let ip_addr = extract_ipv4(&ip_addr)?;
            let ip_addr = ip_addr.to_string();
            let listen_addr =
                format!("{}:{}", ip_addr, self.gossip_config.gossip_port);

            self.listener = Some(
                self.ts
                    .listen("udp", &listen_addr)
                    .map_err(|e| GossipError::NetworkError(e))?,
            );
        }

        Ok(())
    }

    pub async fn join_network(&mut self) -> Result<String, GossipError> {
        self.ts.up().map_err(|e| GossipError::NetworkError(e))?;
        Ok(self.id.clone())
    }

    pub async fn get_ip(&self) -> Result<String, GossipError> {
        let ip = self
            .ts
            .get_ips(None)
            .map_err(|e| GossipError::NetworkError(e))?;

        Ok(ip)
    }

    pub async fn get_peers(&self) -> Result<Vec<Node>, GossipError> {
        let devices = self
            .api
            .list_devices()
            .await
            .map_err(|e| GossipError::PeerError(e))?;

        let prefix = format!("{}-", self.gossip_config.prefix);

        let nodes: Vec<Node> = devices
            .into_iter()
            .filter(|d| d.hostname.starts_with(&prefix))
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

    // pub async fn listen(&self) -> Result<i32, GossipError> {
    //     self.ts
    //         .listen("udp", &format!(":{}", self.gossip_config.gossip_port))
    //         .map_err(|e| GossipError::NetworkError(e))
    // }
}

#[async_trait]
impl GossipTransport for Tailscale {
    async fn write(
        &self,
        buf: &heapless::Vec<u8, MAX_PAYLOAD_SIZE>,
        addr: String,
    ) -> Result<usize, GossipError> {
        let fd = self
            .ts
            .dial("udp", addr.as_str())
            .map_err(GossipError::NetworkError)?;
        let socket = UdpSocket::from(fd);
        socket.send(buf).map_err(GossipError::Io)
    }

    async fn recv_from(
        &self,
        buf: &mut Vec<u8>,
    ) -> Result<(usize, SocketAddr), GossipError> {
        if self.listener.is_none() {
            return Err(GossipError::NetworkError(
                "you must call listen first".to_string(),
            ));
        }

        let listener = self.listener.as_ref().unwrap();

        let conn = self
            .ts
            .accept(listener.as_fd())
            .map_err(|e| GossipError::NetworkError(e))?;

        info!("accepted connection");

        let remote_addr = self
            .ts
            .get_remote_addr(conn.as_fd(), listener.as_fd())
            .map_err(|e| GossipError::NetworkError(e))?;

        let socket = UdpSocket::from(conn);

        let remote_addr = extract_ipv4(&remote_addr)?;
        let remote_addr =
            SocketAddr::from((remote_addr, self.gossip_config.gossip_port));

        let len = socket
            .recv(buf)
            .map_err(|e| GossipError::NetworkError(e.to_string()))?;

        Ok((len, remote_addr))
    }

    async fn get_ip(&self) -> Result<String, GossipError> {
        self.get_ip().await
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
