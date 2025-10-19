# p2p

Command-line tool for interacting with the Teranode P2P network.

## Installation

From the workspace root:

```bash
cargo build --release --bin p2p
```

The binary will be at `target/release/p2p`.

## Usage

### List Discovered Peers

Join the network and list discovered peers:

```bash
p2p \
  --network mainnet \
  --listen /ip4/0.0.0.0/tcp/9005 \
  --bootstrap /ip4/1.2.3.4/tcp/9005/p2p/12D3KooW... \
  list-peers \
  --duration 30 \
  --interval 5
```

### Options

#### Global Options

- `-n, --network <NETWORK>`: Network to connect to (mainnet, testnet, regtest) [default: mainnet]
- `-l, --listen <LISTEN>`: Listen address [default: /ip4/0.0.0.0/tcp/9005]
- `-b, --bootstrap <BOOTSTRAP>`: Bootstrap peer addresses (can specify multiple)
- `-k, --key-file <KEY_FILE>`: Path to private key file
- `--no-mdns`: Disable mDNS local peer discovery
- `--kad-mode <KAD_MODE>`: Kademlia mode: server or client [default: server]
- `-v, --verbose`: Enable verbose logging

#### list-peers Command

- `-c, --connected`: Only show connected peers
- `-t, --teranode`: Only show Teranode-compatible peers
- `-d, --duration <DURATION>`: Run duration in seconds (0 = run indefinitely) [default: 30]
- `-i, --interval <INTERVAL>`: Update interval in seconds [default: 5]

### Examples

#### Connect to mainnet and list peers

```bash
p2p --network mainnet \
    --bootstrap /ip4/seed1.example.com/tcp/9005/p2p/12D3KooW... \
    list-peers
```

#### Use a specific private key

```bash
p2p --key-file ~/.teranode/p2p.key \
    list-peers --duration 60
```

#### Connect to testnet

```bash
p2p --network testnet \
    --listen /ip4/0.0.0.0/tcp/9006 \
    list-peers
```

#### Verbose logging with custom interval

```bash
p2p -v \
    --bootstrap /ip4/1.2.3.4/tcp/9005/p2p/12D3KooW... \
    list-peers --interval 10
```

#### Show only connected Teranode peers

```bash
p2p list-peers --connected --teranode --duration 0
```

### Environment Variables

You can also configure the tool using environment variables:

- `TERANODE_NETWORK`: Network name
- `P2P_LISTEN_ADDR`: Listen address
- `P2P_BOOTSTRAP_PEERS`: Bootstrap peer addresses
- `P2P_KEY_FILE`: Path to key file

Example:

```bash
export TERANODE_NETWORK=mainnet
export P2P_BOOTSTRAP_PEERS=/ip4/1.2.3.4/tcp/9005/p2p/12D3KooW...
p2p list-peers
```

## Output

The tool will output information about:

- Local peer ID
- Discovered peers
- Connection status
- Teranode protocol compatibility
- Network events (with `-v` flag)

Example output:

```
2025-10-19T12:00:00Z INFO p2p_protocol: Initializing P2P client for network: mainnet
2025-10-19T12:00:00Z INFO p2p_protocol: Local peer ID: 12D3KooWABC...
2025-10-19T12:00:00Z INFO p2p_protocol: Listening on: /ip4/0.0.0.0/tcp/9005
2025-10-19T12:00:01Z INFO p2p_protocol: Bootstrap succeeded with 0 remaining jobs
2025-10-19T12:00:02Z INFO p2p_protocol: Connection established with peer: 12D3KooWDEF...
2025-10-19T12:00:02Z INFO p2p_protocol: Peer 12D3KooWDEF... supports Teranode protocol
```

## Key Management

If no key file is specified, the tool will:

1. Generate a new Ed25519 keypair
2. Save it to the specified location (or a temporary file)
3. Reuse the same key on subsequent runs

This ensures your peer ID remains consistent across sessions.

## Multiaddr Format

Bootstrap peers and listen addresses use the multiaddr format:

```
/ip4/<IP>/tcp/<PORT>/p2p/<PEER_ID>
```

Examples:
- `/ip4/127.0.0.1/tcp/9005/p2p/12D3KooWABC...`
- `/ip4/192.168.1.100/tcp/9005/p2p/12D3KooWDEF...`
- `/dns4/seed.example.com/tcp/9005/p2p/12D3KooWGHI...`

## Troubleshooting

### No peers discovered

- Check that bootstrap peers are reachable
- Verify the network name matches the bootstrap peers' network
- Enable verbose logging with `-v` to see connection attempts

### Connection refused

- Ensure the listen port is not already in use
- Check firewall settings
- Verify the listen address is valid

### Key file errors

- Ensure the key file path is writable
- Check the key file contains a valid 64-byte hex string
- Try removing the key file and letting the tool generate a new one

## See Also

- [p2p-protocol library](../p2p-protocol/README.md)
- [libp2p documentation](https://docs.libp2p.io/)
- [Teranode documentation](https://github.com/bsv-blockchain/teranode)
