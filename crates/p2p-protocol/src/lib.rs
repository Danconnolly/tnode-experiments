/// P2P Protocol library for Teranode
///
/// This library provides a rust-libp2p implementation of the Teranode P2P protocol,
/// enabling peer discovery via Kademlia DHT and messaging via GossipSub.
pub mod client;
pub mod config;
pub mod error;
pub mod peer;

pub use client::P2PClient;
pub use config::{KadMode, P2PConfig};
pub use error::{P2PError, Result};
pub use peer::PeerInfo;
