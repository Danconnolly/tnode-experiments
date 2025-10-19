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

The `tnode` CLI provides commands for interacting with the Teranode blockchain service.

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
    // Connect to the Teranode blockchain service
    let mut client = TeranodeClient::connect("http://127.0.0.1:8087").await?;

    // Get the best block header
    let response = client.get_best_block_header().await?;
    println!("Best block height: {}", response.height);

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
