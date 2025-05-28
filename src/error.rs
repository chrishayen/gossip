use thiserror::Error;

#[derive(Error, Debug)]
pub enum GossipError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Peer errors
    #[error("Peer error: {0}")]
    PeerError(#[from] tailscale_api::APIError),

    /// IP address errors
    #[error("IP address error: {0}")]
    IpAddressError(String),
}
