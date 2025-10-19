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
}

pub mod client {
    //! High-level client interface for Teranode

    use crate::proto::blockchain_api::{
        blockchain_api_client::BlockchainApiClient, GetBlockHeaderResponse,
    };
    use anyhow::{Context, Result};
    use tonic::transport::Channel;

    /// Main client for interacting with Teranode
    pub struct TeranodeClient {
        blockchain_client: BlockchainApiClient<Channel>,
    }

    impl TeranodeClient {
        /// Create a new Teranode client
        ///
        /// # Arguments
        /// * `endpoint` - The gRPC endpoint (e.g., "http://127.0.0.1:50051")
        pub async fn connect(endpoint: impl AsRef<str>) -> Result<Self> {
            let endpoint_str = endpoint.as_ref();
            let channel = Channel::from_shared(endpoint_str.to_string())
                .context("Invalid endpoint URL")?
                .connect()
                .await
                .context("Failed to connect to Teranode")?;

            let blockchain_client = BlockchainApiClient::new(channel);

            Ok(Self { blockchain_client })
        }

        /// Get the best (tip) block header
        ///
        /// # Returns
        /// The header of the current best block in the blockchain
        pub async fn get_best_block_header(&mut self) -> Result<GetBlockHeaderResponse> {
            let response = self
                .blockchain_client
                .get_best_block_header(())
                .await
                .context("Failed to get best block header")?;

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
