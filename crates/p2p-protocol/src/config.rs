use libp2p::Multiaddr;
use std::path::PathBuf;

/// Configuration for the P2P client
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Network name (e.g., "mainnet", "testnet", "regtest")
    pub network: String,

    /// Protocol version (default: "1.0.0")
    pub protocol_version: String,

    /// Listen addresses for the libp2p node
    pub listen_addresses: Vec<Multiaddr>,

    /// Bootstrap peers to connect to initially
    pub bootstrap_peers: Vec<Multiaddr>,

    /// Path to store/load the private key (optional)
    /// If not provided or file doesn't exist, a new key will be generated
    pub key_file: Option<PathBuf>,

    /// Hex-encoded Ed25519 private key (64 bytes: 32 private + 32 public)
    /// If provided, this takes precedence over key_file
    pub private_key_hex: Option<String>,

    /// Enable mDNS for local peer discovery (default: true)
    pub enable_mdns: bool,

    /// Kademlia DHT mode: "server" or "client" (default: "server")
    pub kad_mode: KadMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KadMode {
    /// Server mode: responds to DHT queries and stores records
    Server,
    /// Client mode: only queries the DHT, doesn't respond to queries
    Client,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            protocol_version: "1.0.0".to_string(),
            listen_addresses: vec![],
            bootstrap_peers: P2PConfig::default_bootstrap_peers(),
            key_file: None,
            private_key_hex: None,
            enable_mdns: true,
            kad_mode: KadMode::Server,
        }
    }
}

impl P2PConfig {
    /// Get the default libp2p bootstrap peers
    fn default_bootstrap_peers() -> Vec<Multiaddr> {
        vec![
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"
                .parse()
                .expect("invalid bootstrap peer address"),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa"
                .parse()
                .expect("invalid bootstrap peer address"),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb"
                .parse()
                .expect("invalid bootstrap peer address"),
            "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"
                .parse()
                .expect("invalid bootstrap peer address"),
            "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"
                .parse()
                .expect("invalid bootstrap peer address"),
            "/ip4/104.131.131.82/udp/4001/quic/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"
                .parse()
                .expect("invalid bootstrap peer address"),
        ]
    }

    /// Create a new configuration with required parameters
    pub fn new(network: String) -> Self {
        Self {
            network,
            ..Default::default()
        }
    }

    /// Set listen addresses
    pub fn with_listen_addresses(mut self, addrs: Vec<Multiaddr>) -> Self {
        self.listen_addresses = addrs;
        self
    }

    /// Set bootstrap peers
    pub fn with_bootstrap_peers(mut self, peers: Vec<Multiaddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    /// Set key file path
    pub fn with_key_file(mut self, path: PathBuf) -> Self {
        self.key_file = Some(path);
        self
    }

    /// Set private key hex
    pub fn with_private_key_hex(mut self, hex: String) -> Self {
        self.private_key_hex = Some(hex);
        self
    }

    /// Enable or disable mDNS
    pub fn with_mdns(mut self, enable: bool) -> Self {
        self.enable_mdns = enable;
        self
    }

    /// Set Kademlia mode
    pub fn with_kad_mode(mut self, mode: KadMode) -> Self {
        self.kad_mode = mode;
        self
    }

    /// Get the full protocol ID string
    /// Format: /teranode/bitcoin/<network>/<version>
    pub fn protocol_id(&self) -> String {
        format!(
            "/teranode/bitcoin/{}/{}",
            self.network, self.protocol_version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_id() {
        let config = P2PConfig::new("mainnet".to_string());
        assert_eq!(config.protocol_id(), "/teranode/bitcoin/mainnet/1.0.0");

        let config = P2PConfig::new("testnet".to_string());
        assert_eq!(config.protocol_id(), "/teranode/bitcoin/testnet/1.0.0");
    }

    #[test]
    fn test_builder_pattern() {
        let config = P2PConfig::new("regtest".to_string())
            .with_mdns(false)
            .with_kad_mode(KadMode::Client);

        assert_eq!(config.network, "regtest");
        assert!(!config.enable_mdns);
        assert_eq!(config.kad_mode, KadMode::Client);
    }

    #[test]
    fn test_default_bootstrap_peers() {
        let config = P2PConfig::default();

        // Should have 6 default bootstrap peers
        assert_eq!(config.bootstrap_peers.len(), 6);

        // Verify we have both DNS and direct IP bootstrap peers
        let has_dnsaddr = config
            .bootstrap_peers
            .iter()
            .any(|addr| addr.to_string().contains("dnsaddr"));
        let has_direct_ip = config
            .bootstrap_peers
            .iter()
            .any(|addr| addr.to_string().contains("104.131.131.82"));

        assert!(has_dnsaddr, "should have DNS bootstrap peers");
        assert!(has_direct_ip, "should have direct IP bootstrap peers");
    }
}
