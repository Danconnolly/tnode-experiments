# Teranode Experiments

A collection of Rust libraries and command-line utilities for querying and testing [BSV Blockchain Association's Teranode](https://github.com/bsv-blockchain/teranode.git) installations.

## Project Structure

This is a Rust workspace containing multiple crates:

- **teranode-client** - Core library providing gRPC client functionality for Teranode
- **tnode-lab** - Experimental command-line tool (`tnode`) for interacting with Teranode instances

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

### CLI Tool

The `tnode` CLI provides various commands for interacting with Teranode:

```bash
# Build the CLI
cargo build --release

# Run with default endpoint (localhost:50051)
./target/release/tnode ping

# Specify a custom endpoint
./target/release/tnode --endpoint http://teranode.example.com:50051 ping

# Run queries
./target/release/tnode query <QUERY_TYPE>

# Run tests
./target/release/tnode test [TEST_SUITE]

# Enable verbose logging
./target/release/tnode --verbose ping
```

### Library Usage

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
    let client = TeranodeClient::connect("http://localhost:50051").await?;
    // Use client to interact with Teranode
    Ok(())
}
```

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
