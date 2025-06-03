use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GossipConfig {
    /// Time between heartbeats
    pub heartbeat_interval: Duration,
    /// Time between gossip rounds
    pub gossip_interval: Duration,
    /// Timeout to mark node as offline
    pub offline_timeout: Duration,
    /// Number of peers to gossip with per round
    pub fanout: usize,
    /// Port to listen on for gossip
    pub gossip_port: u16,
    /// Prefix for node IDs
    pub prefix: String,
    /// IP address to listen on for gossip
    pub ip_address: String,
    /// Node name
    pub node_name: String,
}

impl Default for GossipConfig {
    /// Default gossip configuration
    /// heartbeat_interval: 1s
    /// gossip_interval: 2s
    /// offline_timeout: 10s
    /// fanout: 4
    fn default() -> Self {
        GossipConfig {
            gossip_port: 42069,
            heartbeat_interval: Duration::from_secs(1),
            gossip_interval: Duration::from_secs(2),
            offline_timeout: Duration::from_secs(10),
            fanout: 4,
            prefix: "ht".to_string(),
            ip_address: "127.0.0.1".to_string(),
            node_name: "".to_string(),
        }
    }
}
