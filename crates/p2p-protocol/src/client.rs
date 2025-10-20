use crate::error::Result as P2PResult;
use crate::{config::KadMode, P2PConfig, P2PError, PeerInfo};
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub, identify, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol, Swarm, Transport,
};
use libp2p_identity::Keypair;
use std::{collections::HashMap, fs, time::Duration};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// A GossipSub message event
#[derive(Clone, Debug)]
pub struct GossipMessage {
    pub topic: String,
    pub data: Vec<u8>,
    pub source: PeerId,
}

/// Main P2P client for joining the Teranode network
pub struct P2PClient {
    swarm: Swarm<TeranodeBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
    config: P2PConfig,
    message_tx: broadcast::Sender<GossipMessage>,
}

/// Combined network behavior for Teranode P2P
#[derive(NetworkBehaviour)]
struct TeranodeBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

impl P2PClient {
    /// Create a new P2P client with the given configuration
    pub async fn new(config: P2PConfig) -> P2PResult<Self> {
        info!("Initializing P2P client for network: {}", config.network);

        // Load or generate keypair
        let keypair = Self::load_or_generate_keypair(&config)?;
        let peer_id = PeerId::from(keypair.public());
        info!("Local peer ID: {}", peer_id);

        // Build the transport
        let transport =
            tcp::tokio::Transport::default()
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::Config::new(&keypair).map_err(|e| {
                    P2PError::Network(format!("Failed to create noise config: {}", e))
                })?)
                .multiplex(yamux::Config::default())
                .boxed();

        // Create Kademlia DHT
        let store = kad::store::MemoryStore::new(peer_id);
        let protocol_name = StreamProtocol::try_from_owned(config.protocol_id())
            .map_err(|e| P2PError::InvalidConfig(format!("Invalid protocol ID: {}", e)))?;
        let kad_config = kad::Config::new(protocol_name);
        let mut kademlia = kad::Behaviour::with_config(peer_id, store, kad_config);

        // Set Kademlia mode
        match config.kad_mode {
            KadMode::Server => {
                kademlia.set_mode(Some(kad::Mode::Server));
                info!("Kademlia DHT mode: Server");
            }
            KadMode::Client => {
                kademlia.set_mode(Some(kad::Mode::Client));
                info!("Kademlia DHT mode: Client");
            }
        }

        // Add bootstrap peers to Kademlia
        for addr in &config.bootstrap_peers {
            if let Some(peer_id) = addr.iter().find_map(|p| match p {
                libp2p::multiaddr::Protocol::P2p(peer_id) => Some(peer_id),
                _ => None,
            }) {
                kademlia.add_address(&peer_id, addr.clone());
                info!("Added bootstrap peer: {}", peer_id);
            }
        }

        // Create GossipSub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| P2PError::InvalidConfig(e.to_string()))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| P2PError::Network(e.to_string()))?;

        // Subscribe to Teranode topics
        let topics = vec!["blocks", "subtrees", "rejected_tx", "node_status"];
        for topic_name in topics {
            let topic =
                gossipsub::IdentTopic::new(format!("{}/{}", config.protocol_id(), topic_name));
            gossipsub.subscribe(&topic).map_err(|e| {
                P2PError::Network(format!(
                    "Failed to subscribe to topic {}: {}",
                    topic_name, e
                ))
            })?;
            info!("Subscribed to topic: {}", topic_name);
        }

        // Create Identify protocol
        let identify_config = identify::Config::new(config.protocol_id(), keypair.public())
            .with_agent_version(format!("teranode-rust/{}", env!("CARGO_PKG_VERSION")));
        let identify = identify::Behaviour::new(identify_config);

        // Create mDNS (if enabled)
        let mdns = if config.enable_mdns {
            info!("mDNS enabled for local peer discovery");
            mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
                .map_err(|e| P2PError::Network(format!("Failed to create mDNS: {}", e)))?
        } else {
            info!("mDNS disabled");
            mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
                .map_err(|e| P2PError::Network(format!("Failed to create mDNS: {}", e)))?
        };

        // Create the combined behaviour
        let behaviour = TeranodeBehaviour {
            kademlia,
            gossipsub,
            identify,
            mdns,
        };

        // Create the swarm
        let swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(60)),
        );

        // Create broadcast channel for gossipsub messages
        let (message_tx, _) = broadcast::channel(256);

        Ok(Self {
            swarm,
            peers: HashMap::new(),
            config,
            message_tx,
        })
    }

    /// Start the P2P client and listen on configured addresses
    pub async fn start(&mut self) -> P2PResult<()> {
        info!("Starting P2P client...");

        // Listen on configured addresses
        for addr in &self.config.listen_addresses {
            self.swarm.listen_on(addr.clone())?;
            info!("Listening on: {}", addr);
        }

        // Bootstrap the DHT
        if !self.config.bootstrap_peers.is_empty() {
            info!("Bootstrapping Kademlia DHT...");
            self.swarm
                .behaviour_mut()
                .kademlia
                .bootstrap()
                .map_err(|e| P2PError::Network(format!("Bootstrap failed: {}", e)))?;
        }

        Ok(())
    }

    /// Run the event loop for the P2P client
    pub async fn run(&mut self) -> P2PResult<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    self.handle_behaviour_event(event).await;
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    info!(
                        "Connection established with peer: {} at {}",
                        peer_id,
                        endpoint.get_remote_address()
                    );
                    self.peers
                        .entry(peer_id)
                        .or_insert_with(|| PeerInfo::new(peer_id))
                        .set_connected(true);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    debug!(
                        "Connection closed with peer: {} (cause: {:?})",
                        peer_id, cause
                    );
                    if let Some(peer) = self.peers.get_mut(&peer_id) {
                        peer.set_connected(false);
                    }
                }
                SwarmEvent::IncomingConnection {
                    local_addr,
                    send_back_addr,
                    ..
                } => {
                    debug!(
                        "Incoming connection from {} to {}",
                        send_back_addr, local_addr
                    );
                }
                SwarmEvent::OutgoingConnectionError {
                    peer_id: Some(peer_id),
                    error,
                    ..
                } => {
                    warn!("Outgoing connection error to {}: {}", peer_id, error);
                    if let Some(peer) = self.peers.get_mut(&peer_id) {
                        peer.increment_attempts();
                    }
                }
                SwarmEvent::OutgoingConnectionError {
                    peer_id: None,
                    error,
                    ..
                } => {
                    warn!("Outgoing connection error (unknown peer): {}", error);
                }
                SwarmEvent::IncomingConnectionError {
                    local_addr,
                    send_back_addr,
                    error,
                    ..
                } => {
                    warn!(
                        "Incoming connection error from {} to {}: {}",
                        send_back_addr, local_addr, error
                    );
                }
                _ => {}
            }
        }
    }

    /// Handle behavior-specific events
    async fn handle_behaviour_event(&mut self, event: TeranodeBehaviourEvent) {
        match event {
            TeranodeBehaviourEvent::Kademlia(kad_event) => {
                self.handle_kad_event(kad_event);
            }
            TeranodeBehaviourEvent::Gossipsub(gossip_event) => {
                self.handle_gossipsub_event(gossip_event);
            }
            TeranodeBehaviourEvent::Identify(identify_event) => {
                self.handle_identify_event(identify_event);
            }
            TeranodeBehaviourEvent::Mdns(mdns_event) => {
                self.handle_mdns_event(mdns_event);
            }
        }
    }

    /// Handle Kademlia DHT events
    fn handle_kad_event(&mut self, event: kad::Event) {
        match event {
            kad::Event::RoutingUpdated { peer, .. } => {
                debug!("Routing updated for peer: {}", peer);
                let _peer_info = self
                    .peers
                    .entry(peer)
                    .or_insert_with(|| PeerInfo::new(peer));
            }
            kad::Event::UnroutablePeer { peer } => {
                debug!("Unroutable peer: {}", peer);
            }
            kad::Event::RoutablePeer { peer, address } => {
                debug!("Routable peer discovered: {} at {}", peer, address);
                let peer_info = self
                    .peers
                    .entry(peer)
                    .or_insert_with(|| PeerInfo::new(peer));
                peer_info.add_address(address);
            }
            kad::Event::OutboundQueryProgressed { result, .. } => match result {
                kad::QueryResult::GetProviders(Ok(_ok)) => {
                    debug!("GetProviders query succeeded");
                }
                kad::QueryResult::GetProviders(Err(e)) => {
                    debug!("GetProviders query failed: {:?}", e);
                }
                kad::QueryResult::Bootstrap(Ok(ok)) => {
                    info!(
                        "Bootstrap succeeded with {} remaining jobs",
                        ok.num_remaining
                    );
                }
                kad::QueryResult::Bootstrap(Err(e)) => {
                    warn!("Bootstrap failed: {:?}", e);
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Handle GossipSub events
    fn handle_gossipsub_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            } => {
                debug!(
                    "Received message from {}: {:?} on topic {:?}",
                    propagation_source, message_id, message.topic
                );

                // Broadcast the message to subscribers
                let msg = GossipMessage {
                    topic: message.topic.to_string(),
                    data: message.data.clone(),
                    source: propagation_source,
                };

                // Ignore send error if no receivers
                let _ = self.message_tx.send(msg);
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                info!("Peer {} subscribed to topic: {:?}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                debug!("Peer {} unsubscribed from topic: {:?}", peer_id, topic);
            }
            _ => {}
        }
    }

    /// Handle Identify protocol events
    fn handle_identify_event(&mut self, event: identify::Event) {
        match event {
            identify::Event::Received {
                peer_id,
                info,
                connection_id: _,
            } => {
                debug!(
                    "Received identify info from {}: {:?}",
                    peer_id, info.protocol_version
                );
                let supports_teranode = info
                    .protocols
                    .iter()
                    .any(|p| p.as_ref().starts_with("/teranode/bitcoin/"));

                let peer_info = self
                    .peers
                    .entry(peer_id)
                    .or_insert_with(|| PeerInfo::new(peer_id));
                peer_info.update_from_identify(
                    info.agent_version,
                    info.protocol_version,
                    supports_teranode,
                );

                // Add observed addresses
                for addr in info.listen_addrs {
                    peer_info.add_address(addr);
                }

                if supports_teranode {
                    info!("Peer {} supports Teranode protocol", peer_id);
                }
            }
            identify::Event::Sent {
                peer_id,
                connection_id: _,
            } => {
                debug!("Sent identify info to {}", peer_id);
            }
            identify::Event::Pushed {
                peer_id,
                info,
                connection_id: _,
            } => {
                debug!(
                    "Pushed identify update to {}: {:?}",
                    peer_id, info.protocol_version
                );
            }
            identify::Event::Error {
                peer_id,
                error,
                connection_id: _,
            } => {
                warn!("Identify error with peer {}: {}", peer_id, error);
            }
        }
    }

    /// Handle mDNS events
    fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    info!("Discovered local peer via mDNS: {} at {}", peer_id, addr);
                    let peer_info = self
                        .peers
                        .entry(peer_id)
                        .or_insert_with(|| PeerInfo::new(peer_id));
                    peer_info.add_address(addr);
                }
            }
            mdns::Event::Expired(peers) => {
                for (peer_id, addr) in peers {
                    debug!("mDNS peer expired: {} at {}", peer_id, addr);
                }
            }
        }
    }

    /// Get all discovered peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.values().cloned().collect()
    }

    /// Get connected peers only
    pub fn get_connected_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.connected)
            .cloned()
            .collect()
    }

    /// Get peers that support the Teranode protocol
    pub fn get_teranode_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.supports_teranode)
            .cloned()
            .collect()
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        self.swarm.local_peer_id()
    }

    /// Subscribe to all gossipsub messages
    pub fn subscribe_to_messages(&self) -> broadcast::Receiver<GossipMessage> {
        self.message_tx.subscribe()
    }

    /// Load or generate a keypair based on configuration
    fn load_or_generate_keypair(config: &P2PConfig) -> P2PResult<Keypair> {
        // Try to load from hex string first
        if let Some(hex_key) = &config.private_key_hex {
            return Self::keypair_from_hex(hex_key);
        }

        // Try to load from file
        if let Some(key_file) = &config.key_file {
            if key_file.exists() {
                info!("Loading private key from file: {:?}", key_file);
                let hex_key = fs::read_to_string(key_file).map_err(P2PError::Io)?;
                return Self::keypair_from_hex(hex_key.trim());
            }
        }

        // Generate new key
        info!("Generating new Ed25519 keypair");
        let keypair = Keypair::generate_ed25519();

        // Save to file if path is provided
        if let Some(key_file) = &config.key_file {
            let hex_key = Self::keypair_to_hex(&keypair)?;
            if let Some(parent) = key_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(key_file, hex_key)?;
            info!("Saved new private key to file: {:?}", key_file);
        }

        Ok(keypair)
    }

    /// Convert hex string (64 bytes) to keypair
    fn keypair_from_hex(hex: &str) -> P2PResult<Keypair> {
        let bytes = hex::decode(hex).map_err(|e| P2PError::KeyDecode(e.to_string()))?;

        if bytes.len() != 64 {
            return Err(P2PError::KeyDecode(format!(
                "Expected 64 bytes, got {}",
                bytes.len()
            )));
        }

        // Ed25519 keypair is 64 bytes: 32 bytes secret key + 32 bytes public key
        let keypair =
            Keypair::ed25519_from_bytes(bytes).map_err(|e| P2PError::KeyDecode(e.to_string()))?;

        Ok(keypair)
    }

    /// Convert keypair to hex string
    fn keypair_to_hex(keypair: &Keypair) -> P2PResult<String> {
        // Try to convert to Ed25519 keypair
        if let Ok(ed_keypair) = keypair.clone().try_into_ed25519() {
            let bytes = ed_keypair.to_bytes();
            Ok(hex::encode(bytes))
        } else {
            Err(P2PError::KeyDecode(
                "Only Ed25519 keys are supported".to_string(),
            ))
        }
    }
}
