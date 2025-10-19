use thiserror::Error;

pub type Result<T> = std::result::Result<T, P2PError>;

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("libp2p transport error: {0}")]
    Transport(#[from] libp2p::TransportError<std::io::Error>),

    #[error("Key decode error: {0}")]
    KeyDecode(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
