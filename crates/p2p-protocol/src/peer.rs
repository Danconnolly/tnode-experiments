use libp2p::{Multiaddr, PeerId};
use std::time::SystemTime;

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The peer's ID
    pub peer_id: PeerId,

    /// Known addresses for this peer
    pub addresses: Vec<Multiaddr>,

    /// Whether we are currently connected to this peer
    pub connected: bool,

    /// Agent version reported by the peer (from Identify protocol)
    pub agent_version: Option<String>,

    /// Protocol version reported by the peer
    pub protocol_version: Option<String>,

    /// When we first discovered this peer
    pub discovered_at: SystemTime,

    /// When we last saw this peer (connection or DHT activity)
    pub last_seen: SystemTime,

    /// Number of connection attempts
    pub connection_attempts: u32,

    /// Whether this peer supports the Teranode protocol
    pub supports_teranode: bool,
}

impl PeerInfo {
    /// Create a new PeerInfo for a discovered peer
    pub fn new(peer_id: PeerId) -> Self {
        let now = SystemTime::now();
        Self {
            peer_id,
            addresses: Vec::new(),
            connected: false,
            agent_version: None,
            protocol_version: None,
            discovered_at: now,
            last_seen: now,
            connection_attempts: 0,
            supports_teranode: false,
        }
    }

    /// Add an address to this peer's known addresses
    pub fn add_address(&mut self, addr: Multiaddr) {
        if !self.addresses.contains(&addr) {
            self.addresses.push(addr);
        }
        self.last_seen = SystemTime::now();
    }

    /// Mark this peer as connected
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if connected {
            self.last_seen = SystemTime::now();
        }
    }

    /// Update peer information from Identify protocol
    pub fn update_from_identify(
        &mut self,
        agent_version: String,
        protocol_version: String,
        supports_teranode: bool,
    ) {
        self.agent_version = Some(agent_version);
        self.protocol_version = Some(protocol_version);
        self.supports_teranode = supports_teranode;
        self.last_seen = SystemTime::now();
    }

    /// Increment connection attempts
    pub fn increment_attempts(&mut self) {
        self.connection_attempts += 1;
    }
}
