//! Teranode gRPC client library
//!
//! This library provides a Rust client for interacting with BSV Teranode instances
//! via gRPC. It includes type-safe bindings generated from protobuf definitions.

pub mod proto {
    //! Generated protobuf types and gRPC client stubs

    // Re-export commonly used types from model
    pub mod model {
        include!("proto/model.rs");
    }

    // Re-export blockchain API types
    pub mod blockchain_api {
        include!("proto/blockchain_api.rs");
    }

    // Re-export P2P API types
    pub mod p2p_api {
        include!("proto/p2p_api.rs");
    }
}

pub mod client {
    //! High-level client interface for Teranode

    use crate::proto::blockchain_api::{
        blockchain_api_client::BlockchainApiClient, GetBlockHeaderResponse,
    };
    use crate::proto::p2p_api::{peer_service_client::PeerServiceClient, GetPeersResponse};
    use anyhow::{Context, Result};
    use tonic::transport::Channel;

    /// Main client for interacting with Teranode
    pub struct TeranodeClient {
        blockchain_client: Option<BlockchainApiClient<Channel>>,
        peer_client: Option<PeerServiceClient<Channel>>,
    }

    impl TeranodeClient {
        /// Create a new Teranode client with blockchain endpoint
        ///
        /// # Arguments
        /// * `endpoint` - The gRPC endpoint (e.g., "http://127.0.0.1:8087")
        pub async fn connect(endpoint: impl AsRef<str>) -> Result<Self> {
            let endpoint_str = endpoint.as_ref();
            let channel = Channel::from_shared(endpoint_str.to_string())
                .context("Invalid endpoint URL")?
                .connect()
                .await
                .context("Failed to connect to Teranode")?;

            let blockchain_client = BlockchainApiClient::new(channel);

            Ok(Self {
                blockchain_client: Some(blockchain_client),
                peer_client: None,
            })
        }

        /// Create a new Teranode client with both blockchain and peer endpoints
        ///
        /// # Arguments
        /// * `blockchain_endpoint` - The blockchain service gRPC endpoint
        /// * `peer_endpoint` - The peer service gRPC endpoint
        pub async fn connect_with_endpoints(
            blockchain_endpoint: Option<impl AsRef<str>>,
            peer_endpoint: Option<impl AsRef<str>>,
        ) -> Result<Self> {
            let blockchain_client = if let Some(endpoint) = blockchain_endpoint {
                let channel = Channel::from_shared(endpoint.as_ref().to_string())
                    .context("Invalid blockchain endpoint URL")?
                    .connect()
                    .await
                    .context("Failed to connect to blockchain service")?;
                Some(BlockchainApiClient::new(channel))
            } else {
                None
            };

            let peer_client = if let Some(endpoint) = peer_endpoint {
                let channel = Channel::from_shared(endpoint.as_ref().to_string())
                    .context("Invalid peer endpoint URL")?
                    .connect()
                    .await
                    .context("Failed to connect to peer service")?;
                Some(PeerServiceClient::new(channel))
            } else {
                None
            };

            Ok(Self {
                blockchain_client,
                peer_client,
            })
        }

        /// Get the best (tip) block header
        ///
        /// # Returns
        /// The header of the current best block in the blockchain
        pub async fn get_best_block_header(&mut self) -> Result<GetBlockHeaderResponse> {
            let blockchain_client = self
                .blockchain_client
                .as_mut()
                .context("Blockchain client not initialized")?;

            let response = blockchain_client
                .get_best_block_header(())
                .await
                .context("Failed to get best block header")?;

            Ok(response.into_inner())
        }

        /// Get the list of peers
        ///
        /// # Returns
        /// A response containing the list of connected peers
        pub async fn get_peers(&mut self) -> Result<GetPeersResponse> {
            let peer_client = self
                .peer_client
                .as_mut()
                .context("Peer client not initialized")?;

            let response = peer_client
                .get_peers(())
                .await
                .context("Failed to get peers")?;

            Ok(response.into_inner())
        }
    }
}

pub mod error {
    //! Error types for Teranode client operations

    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum TeranodeError {
        #[error("gRPC connection error: {0}")]
        ConnectionError(#[from] tonic::transport::Error),

        #[error("gRPC status error: {0}")]
        GrpcError(#[from] tonic::Status),

        #[error("Invalid configuration: {0}")]
        ConfigError(String),
    }
}

// Re-export commonly used types
pub use client::TeranodeClient;
pub use error::TeranodeError;
