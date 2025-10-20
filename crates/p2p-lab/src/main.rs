use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use p2p_protocol::{KadMode, P2PClient, P2PConfig};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "p2p")]
#[command(about = "Teranode P2P network client", long_about = None)]
struct Cli {
    /// Network to connect to (mainnet, testnet, regtest)
    #[arg(short, long, env = "TERANODE_NETWORK", default_value = "mainnet")]
    network: String,

    /// Listen address (can be specified multiple times)
    #[arg(
        short,
        long,
        env = "P2P_LISTEN_ADDR",
        default_value = "/ip4/0.0.0.0/tcp/9005"
    )]
    listen: Vec<String>,

    /// Bootstrap peer addresses (multiaddr format)
    #[arg(short, long, env = "P2P_BOOTSTRAP_PEERS")]
    bootstrap: Vec<String>,

    /// Path to private key file
    #[arg(short, long, env = "P2P_KEY_FILE")]
    key_file: Option<PathBuf>,

    /// Disable mDNS local peer discovery
    #[arg(long)]
    no_mdns: bool,

    /// Kademlia mode: server or client
    #[arg(long, default_value = "client")]
    kad_mode: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Wait duration in seconds for peer discovery
    #[arg(short, long, default_value = "5")]
    wait: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all discovered peers
    ListPeers {
        /// Only show connected peers
        #[arg(short, long)]
        connected: bool,

        /// Only show Teranode-compatible peers
        #[arg(short, long)]
        teranode: bool,

        /// Run duration in seconds (0 = run indefinitely)
        #[arg(short, long, default_value = "30")]
        duration: u64,

        /// Update interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },
    /// Listen to gossipsub messages
    Listen {
        #[command(subcommand)]
        target: ListenTarget,
    },
    /// Get information about a gossipsub topic
    Topic {
        /// Topic name to query
        topic: String,
    },
    /// Show information about the local node
    Info,
}

#[derive(Subcommand)]
enum ListenTarget {
    /// Listen to block messages
    Blocks {
        /// Run duration in seconds (0 = run indefinitely)
        #[arg(short, long, default_value = "0")]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // Parse listen addresses
    let listen_addresses: Vec<_> = cli
        .listen
        .iter()
        .map(|s| s.parse())
        .collect::<std::result::Result<_, _>>()
        .context("Failed to parse listen address")?;

    // Parse bootstrap peers
    let bootstrap_peers: Vec<_> = cli
        .bootstrap
        .iter()
        .map(|s| s.parse())
        .collect::<std::result::Result<_, _>>()
        .context("Failed to parse bootstrap peer address")?;

    // Parse Kademlia mode
    let kad_mode = match cli.kad_mode.to_lowercase().as_str() {
        "server" => KadMode::Server,
        "client" => KadMode::Client,
        _ => anyhow::bail!("Invalid Kademlia mode. Use 'server' or 'client'"),
    };

    // Build configuration
    let mut config = P2PConfig::new(cli.network)
        .with_listen_addresses(listen_addresses)
        .with_mdns(!cli.no_mdns)
        .with_kad_mode(kad_mode);

    // Only set bootstrap peers if the list is not empty
    if !bootstrap_peers.is_empty() {
        config = config.with_bootstrap_peers(bootstrap_peers);
    }

    if let Some(key_file) = cli.key_file {
        config = config.with_key_file(key_file);
    }

    // Create and start the P2P client
    let (handle, mut client) = P2PClient::new(config)
        .await
        .context("Failed to create P2P client")?;

    info!("P2P client started successfully");
    info!("Local peer ID: {}", client.local_peer_id().await);

    let wait = cli.wait;

    // Execute the command
    match cli.command {
        Commands::ListPeers {
            connected,
            teranode,
            duration,
            interval,
        } => {
            unimplemented!();
            // run_list_peers(client, connected, teranode, duration, interval).await?;
        }
        Commands::Listen { target } => {
            unimplemented!();
            // run_listen(client, target).await?;
        }
        Commands::Topic { topic } => {
            unimplemented!();
            // query_topic(client, &topic, wait).await?;
        }
        Commands::Info => {
            show_info(&client, wait).await;
        }
    }

    client.stop().await;
    let _ = handle.await?;
    Ok(())
}

// async fn run_list_peers(
//     mut client: P2PClient,
//     _connected_only: bool,
//     _teranode_only: bool,
//     duration_secs: u64,
//     interval_secs: u64,
// ) -> Result<()> {
//     let interval_duration = Duration::from_secs(interval_secs);
//     let mut ticker = time::interval(interval_duration);
//
//     // Spawn the event loop in a background task
//     tokio::spawn(async move {
//         if let Err(e) = client.run().await {
//             tracing::error!("P2P client error: {}", e);
//         }
//     });
//
//     // Wait a bit for initial connections
//     info!("Discovering peers...");
//     ticker.tick().await; // First tick completes immediately
//
//     let start = std::time::Instant::now();
//     let run_duration = if duration_secs == 0 {
//         None
//     } else {
//         Some(Duration::from_secs(duration_secs))
//     };
//
//     loop {
//         ticker.tick().await;
//
//         // Check if we should exit
//         if let Some(max_duration) = run_duration {
//             if start.elapsed() >= max_duration {
//                 info!("Run duration completed");
//                 break;
//             }
//         }
//
//         // This won't work because we moved client into the spawn
//         // We need to refactor this to use channels for communication
//         // For now, let's just note this limitation
//     }
//
//     info!("Note: Full peer listing with running event loop requires refactoring to use channels");
//     info!("The current implementation demonstrates the structure but needs async communication");
//
//     Ok(())
// }
//
// async fn run_listen(client: P2PClientActor, target: ListenTarget) -> Result<()> {
//     match target {
//         ListenTarget::Blocks { duration } => {
//             listen_blocks(client, duration).await?;
//         }
//     }
//     Ok(())
// }
//
// async fn listen_blocks(mut client: P2PClientActor, duration_secs: u64) -> Result<()> {
//     info!("Subscribing to block messages...");
//
//     // Subscribe to messages before spawning the client
//     let mut rx = client.subscribe_to_messages();
//
//     // Spawn the event loop in a background task
//     tokio::spawn(async move {
//         if let Err(e) = client.run().await {
//             tracing::error!("P2P client error: {}", e);
//         }
//     });
//
//     info!("Listening for block messages...");
//
//     let start = std::time::Instant::now();
//     let run_duration = if duration_secs == 0 {
//         None
//     } else {
//         Some(Duration::from_secs(duration_secs))
//     };
//
//     loop {
//         // Check if we should exit
//         if let Some(max_duration) = run_duration {
//             if start.elapsed() >= max_duration {
//                 info!("Listen duration completed");
//                 break;
//             }
//         }
//
//         match tokio::time::timeout(Duration::from_secs(1), rx.recv()).await {
//             Ok(Ok(msg)) => {
//                 // Check if this is a blocks message
//                 if msg.topic.contains("blocks") {
//                     println!(
//                         "[{}] {}: {}",
//                         msg.topic,
//                         msg.source,
//                         String::from_utf8_lossy(&msg.data)
//                     );
//                 } else {
//                     println!("received some other message");
//                 }
//             }
//             Ok(Err(_)) => {
//                 // Channel closed
//                 info!("Message channel closed");
//                 break;
//             }
//             Err(_) => {
//                 // Timeout, just continue
//             }
//         }
//     }
//
//     Ok(())
// }
//
// async fn query_topic(client: P2PClientActor, topic: &str, wait_secs: u64) -> Result<()> {
//     // Store info we need before moving client
//     let protocol_id = client.protocol_id();
//     let full_topic = format!("{}/{}", protocol_id, topic);
//
//     info!("Starting P2P network for peer discovery...");
//
//     // Wrap client in Arc<Mutex> for shared access
//     let client_arc = Arc::new(Mutex::new(client));
//     let client_clone = Arc::clone(&client_arc);
//
//     // Spawn the event loop in a background task
//     tokio::spawn(async move {
//         let mut client = client_clone.lock().await;
//         if let Err(e) = client.run().await {
//             tracing::error!("P2P client error: {}", e);
//         }
//     });
//
//     // Poll for peer discovery with periodic updates
//     info!("Discovering peers on the network...");
//     let start = std::time::Instant::now();
//     let mut last_count = 0;
//
//     loop {
//         tokio::time::sleep(Duration::from_millis(500)).await;
//         let elapsed = start.elapsed().as_secs();
//         info!("{} seconds elapsed.", elapsed);
//         if elapsed >= wait_secs {
//             break;
//         }
//
//         {
//             // Check current peer count
//             let client = client_arc.lock().await;
//             info!("got mutex");
//             let peer_count = client.get_topic_peer_count(&full_topic);
//
//             if peer_count != last_count {
//                 info!(
//                     "Found {} peers on topic after {} seconds",
//                     peer_count, elapsed
//                 );
//                 last_count = peer_count;
//             }
//         }
//     }
//
//     // Display final topic information
//     let client = client_arc.lock().await;
//     let peer_count = client.get_topic_peer_count(&full_topic);
//
//     println!("\n=== Topic Information ===");
//     println!("Topic: {}", topic);
//     println!("Full Topic: {}", full_topic);
//     println!("Subscribed Peers: {}", peer_count);
//
//     if peer_count > 0 {
//         let peers = client.get_topic_peers(&full_topic);
//         println!("\nPublisher Peer IDs:");
//         for (i, peer_id) in peers.iter().enumerate() {
//             println!("  {}. {}", i + 1, peer_id);
//         }
//     } else {
//         println!("\n(No peers currently subscribed to this topic)");
//         println!("\nNote: This may indicate:");
//         println!("- No peers are publishing to this topic");
//         println!("- The network has not completed peer discovery yet");
//         println!("- Increase wait time with --wait flag");
//     }
//
//     Ok(())
// }
//

async fn show_info(client: &P2PClient, wait: u64) {
    time::sleep(Duration::from_secs(wait)).await;
    println!("\n=== Local Node Information ===");
    println!("Peer ID: {}", client.local_peer_id().await);
    println!("\nDiscovered Peers:");

    let peers = client.get_peers().await;
    println!("Total: {}", peers.len());

    let connected = client.get_connected_peers().await;
    println!("Connected: {}", connected.len());

    let teranode = client.get_teranode_peers().await;
    println!("Teranode-compatible: {}", teranode.len());
}
