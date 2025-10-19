//! Teranode gRPC client library
//!
//! This library provides a Rust client for interacting with BSV Teranode instances
//! via gRPC. It includes type-safe bindings generated from protobuf definitions.

pub mod proto {
    //! Generated protobuf types and gRPC client stubs
    //!
    //! The actual modules will be included here once proto files are added
    //! and compiled via build.rs
}

pub mod client {
    //! High-level client interface for Teranode

    use anyhow::Result;

    /// Main client for interacting with Teranode
    pub struct TeranodeClient {
        // gRPC client connections will be added here
    }

    impl TeranodeClient {
        /// Create a new Teranode client
        pub async fn connect(_endpoint: impl AsRef<str>) -> Result<Self> {
            // Implementation will be added once proto files are available
            todo!("Implement client connection")
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
