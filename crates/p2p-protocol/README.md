# p2p-protocol

A Rust implementation of the Teranode P2P protocol using libp2p.

## Overview

This library provides a native libp2p implementation that joins the Teranode P2P network for peer discovery and messaging. It implements the same protocol used by the BSV Blockchain Association's Teranode implementation.

## Features

- **Peer Discovery**: Kademlia DHT for distributed peer discovery
- **Messaging**: GossipSub for pub/sub messaging on Teranode topics
- **Identification**: Identify protocol for peer information exchange
- **Local Discovery**: mDNS for local network peer discovery
- **Ed25519 Keys**: Compatible with Teranode's key format (64-byte hex)

## Protocol Compatibility

The implementation is compatible with Teranode's P2P protocol:

- **Protocol ID**: `/teranode/bitcoin/<network>/<version>`
  - Example: `/teranode/bitcoin/mainnet/1.0.0`
- **Key Format**: Ed25519, 64 bytes hex-encoded (32 private + 32 public)
- **Discovery**: Kademlia DHT (same as go-p2p-message-bus)
- **Pub/Sub**: GossipSub for topic-based messaging
- **Topics**: blocks, subtrees, rejected_tx, node_status, invalid_blocks, invalid_subtrees

## Usage

### Basic Example

```rust
use p2p_protocol::{P2PConfig, P2PClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the P2P client
    let config = P2PConfig::new("mainnet".to_string())
        .with_listen_addresses(vec![
            "/ip4/0.0.0.0/tcp/9005".parse()?,
        ])
        .with_bootstrap_peers(vec![
            "/ip4/1.2.3.4/tcp/9005/p2p/12D3KooW...".parse()?,
        ]);

    // Create and start the client
    let mut client = P2PClient::new(config).await?;
    client.start().await?;

    println!("Local peer ID: {}", client.local_peer_id());

    // Run the event loop
    client.run().await?;

    Ok(())
}
```

### Listing Discovered Peers

```rust
// Get all discovered peers
let all_peers = client.get_peers();
for peer in all_peers {
    println!("Peer: {}", peer.peer_id);
    println!("  Connected: {}", peer.connected);
    println!("  Addresses: {:?}", peer.addresses);
    if peer.supports_teranode {
        println!("  Supports Teranode protocol");
    }
}

// Get only connected peers
let connected = client.get_connected_peers();
println!("Connected peers: {}", connected.len());

// Get Teranode-compatible peers
let teranode_peers = client.get_teranode_peers();
println!("Teranode peers: {}", teranode_peers.len());
```

## Configuration

### P2PConfig Options

- `network`: Network name ("mainnet", "testnet", "regtest")
- `protocol_version`: Protocol version (default: "1.0.0")
- `listen_addresses`: Addresses to listen on
- `bootstrap_peers`: Initial peers to connect to
- `key_file`: Path to store/load Ed25519 private key
- `private_key_hex`: Hex-encoded private key (takes precedence over file)
- `enable_mdns`: Enable mDNS for local discovery (default: true)
- `kad_mode`: Kademlia mode (Server or Client)

### Kademlia Modes

- **Server Mode**: Responds to DHT queries and stores records (default)
- **Client Mode**: Only queries the DHT, doesn't respond to queries

## Key Management

The library supports multiple ways to manage the Ed25519 keypair:

1. **Provide hex key directly**: Use `with_private_key_hex()`
2. **Load from file**: Use `with_key_file()` - will load if exists
3. **Auto-generate**: If neither is provided, a new key is generated

Generated keys are automatically saved to the configured key file for reuse.

### Key Format

Keys are 64-byte hex strings compatible with Teranode:
- First 32 bytes: Ed25519 private key
- Last 32 bytes: Ed25519 public key

Example: `a1b2c3d4...` (128 hex characters total)

## Architecture

### Components

- **P2PClient**: Main client for managing the P2P network
- **TeranodeBehaviour**: Combined libp2p network behavior
  - Kademlia DHT
  - GossipSub
  - Identify
  - mDNS
- **PeerInfo**: Information about discovered peers
- **P2PConfig**: Configuration for the P2P client

### Network Stack

```
Application
    ↓
P2PClient (this library)
    ↓
libp2p Swarm
    ├── Kademlia DHT (peer discovery)
    ├── GossipSub (pub/sub messaging)
    ├── Identify (peer info exchange)
    └── mDNS (local discovery)
    ↓
Transport Layer (TCP + Noise + Yamux)
```

## Comparison with Teranode Go Implementation

This Rust implementation mirrors the Go implementation found in `bsv-blockchain/teranode/services/p2p`:

| Feature | Go (go-p2p-message-bus) | Rust (this library) |
|---------|-------------------------|---------------------|
| DHT | Kademlia (libp2p v0.43) | Kademlia (libp2p v0.54) |
| Pub/Sub | GossipSub | GossipSub |
| Transport | TCP + Noise + Yamux | TCP + Noise + Yamux |
| Keys | Ed25519 (64-byte hex) | Ed25519 (64-byte hex) |
| Protocol ID | `/teranode/bitcoin/<net>/1.0.0` | Same |
| Topics | blocks, subtrees, etc. | Same |

## License

MIT OR Apache-2.0
