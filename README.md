# Teranode Experiments

A collection of Rust libraries and command-line utilities for querying and testing [BSV Blockchain Association's Teranode](https://github.com/bsv-blockchain/teranode.git) installations.

## Project Structure

This is a Rust workspace containing multiple crates:

### gRPC Client Libraries

- **teranode-client** - Core library providing gRPC client functionality for Teranode services
- **tnode-lab** - Command-line tool (`tnode`) for querying Teranode blockchain service

### P2P Network Libraries

- **p2p-protocol** - Native libp2p implementation of the Teranode P2P protocol
- **p2p-lab** - Command-line tool (`p2p`) for joining the Teranode P2P network and discovering peers

## Prerequisites

- Rust 1.70 or later
- A running Teranode instance (for testing)
- Protobuf files from the Teranode repository

## Setup

### 1. Clone and build

```bash
git clone <this-repo>
cd tnode-experiments
cargo build
```

### 2. Add Teranode protobuf definitions

Copy the `.proto` files from the Teranode repository to the proto directory:

```bash
cp /path/to/teranode/proto/*.proto crates/teranode-client/proto/
```

Then rebuild to generate the gRPC client code:

```bash
cargo build
```

## Usage

### Blockchain gRPC Client (`tnode`)

The `tnode` CLI provides commands for interacting with the Teranode blockchain service via gRPC.

**Important**: The endpoint should point to the **blockchain service component** of a full Teranode system (default port: 8087).

#### Configuration

You can configure the endpoint in three ways (in order of precedence):

1. Command-line argument: `--endpoint`
2. Environment variable: `ENDPOINT`
3. `.env` file in project root

```bash
# Copy the example .env file and customize it
cp .env.example .env
# Edit .env to set your blockchain service endpoint
```

#### Commands

```bash
# Build the CLI
cargo build --release

# Get the best (tip) block header (uses default endpoint 127.0.0.1:8087)
./target/release/tnode getbestblock

# Specify a custom endpoint (IP:port format)
./target/release/tnode --endpoint 192.168.1.100:8087 getbestblock

# Or using environment variable
ENDPOINT=192.168.1.100:8087 ./target/release/tnode getbestblock

# Enable verbose logging
./target/release/tnode --verbose getbestblock

# Use alias command format
./target/release/tnode get-best-block
```

### P2P Network Client (`p2p`)

The `p2p` CLI tool allows you to join the Teranode P2P network directly using libp2p, without needing a full Teranode installation.

#### Quick Start

```bash
# Build the P2P client
cargo build --release --bin p2p

# List discovered peers (requires bootstrap peers)
./target/release/p2p \
  --network mainnet \
  --bootstrap /ip4/BOOTSTRAP_IP/tcp/9005/p2p/PEER_ID \
  list-peers --duration 30

# Enable verbose logging to see network events
./target/release/p2p -v \
  --bootstrap /ip4/BOOTSTRAP_IP/tcp/9005/p2p/PEER_ID \
  list-peers
```

#### Features

- **Kademlia DHT**: Discover peers through the distributed hash table
- **GossipSub**: Subscribe to Teranode pub/sub topics (blocks, subtrees, etc.)
- **mDNS**: Discover local peers automatically
- **Identify Protocol**: Learn about peer capabilities and protocol support
- **Key Management**: Ed25519 keys compatible with Teranode format

See [crates/p2p-lab/README.md](crates/p2p-lab/README.md) for detailed usage and examples.

### Library Usage

#### Blockchain gRPC Client Library

Add to your `Cargo.toml`:

```toml
[dependencies]
teranode-client = { path = "crates/teranode-client" }
tokio = { version = "1", features = ["full"] }
```

Example usage:

```rust
use teranode_client::TeranodeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to the Teranode blockchain service
    let mut client = TeranodeClient::connect("http://127.0.0.1:8087").await?;

    // Get the best block header
    let response = client.get_best_block_header().await?;
    println!("Best block height: {}", response.height);

    Ok(())
}
```

#### P2P Protocol Library

Add to your `Cargo.toml`:

```toml
[dependencies]
p2p-protocol = { path = "crates/p2p-protocol" }
tokio = { version = "1", features = ["full"] }
```

Example usage:

```rust
use p2p_protocol::{P2PConfig, P2PClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the P2P client for mainnet
    let config = P2PConfig::new("mainnet".to_string())
        .with_listen_addresses(vec!["/ip4/0.0.0.0/tcp/9005".parse()?])
        .with_bootstrap_peers(vec![
            "/ip4/BOOTSTRAP_IP/tcp/9005/p2p/PEER_ID".parse()?,
        ]);

    // Create and start the client
    let mut client = P2PClient::new(config).await?;
    client.start().await?;

    println!("Local peer ID: {}", client.local_peer_id());

    // Get discovered peers
    let peers = client.get_teranode_peers();
    println!("Found {} Teranode-compatible peers", peers.len());

    Ok(())
}
```

See [crates/p2p-protocol/README.md](crates/p2p-protocol/README.md) for detailed API documentation.

## Development

### Running tests

```bash
cargo test
```

### Code formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Architecture

- **Async Runtime**: Uses Tokio for async I/O
- **gRPC**: Tonic for gRPC client implementation
- **Protobuf**: Prost for Protocol Buffers serialization
- **Error Handling**: Anyhow and thiserror for ergonomic error handling
- **Logging**: Tracing for structured logging

## License

MIT OR Apache-2.0
