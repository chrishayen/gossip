use std::time::Duration;

#[derive(Clone)]
pub struct GossipConfig {
    /// Max payload size (e.g., 1024 for 1280 MTU)
    pub max_payload_size: usize,
    /// Time between heartbeats
    pub heartbeat_interval: Duration,
    /// Time between gossip rounds
    pub gossip_interval: Duration,
    /// Timeout to mark node as offline
    pub offline_timeout: Duration,
    /// Number of peers to gossip with per round
    pub fanout: usize,
}

impl Default for GossipConfig {
    /// Default gossip configuration
    /// max_payload_size: 1024 (1280 MTU)
    /// heartbeat_interval: 1s
    /// gossip_interval: 2s
    /// offline_timeout: 10s
    /// fanout: 4
    fn default() -> Self {
        GossipConfig {
            max_payload_size: 1252,
            heartbeat_interval: Duration::from_secs(1),
            gossip_interval: Duration::from_secs(2),
            offline_timeout: Duration::from_secs(10),
            fanout: 4,
        }
    }
}
