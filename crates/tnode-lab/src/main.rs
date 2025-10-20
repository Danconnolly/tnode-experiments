//! Teranode CLI - Command-line tool for interacting with Teranode instances

use anyhow::Result;
use bitcoinsv::bitcoin::BlockHeader;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "tnode")]
#[command(about = "CLI tool for BSV Teranode experiments", long_about = None)]
struct Cli {
    /// Teranode blockchain service endpoint (IP:port format, e.g., "127.0.0.1:8087")
    /// Note: This is the blockchain service component of a full Teranode system
    /// Can be set via BLOCKCHAIN_ENDPOINT environment variable or .env file
    #[arg(
        short = 'b',
        long,
        env = "BLOCKCHAIN_ENDPOINT",
        default_value = "127.0.0.1:8087"
    )]
    blockchain_endpoint: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the best (tip) block header from the blockchain
    #[command(alias = "getbestblock")]
    GetBestBlock,
}

/// Parse endpoint and add default port 8087 if not specified
fn parse_endpoint(endpoint: &str) -> String {
    const DEFAULT_PORT: &str = "8087";

    // If it already has a protocol, return as-is
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return endpoint.to_string();
    }

    // Check if port is specified (contains ':')
    if endpoint.contains(':') {
        endpoint.to_string()
    } else {
        // No port specified, add default
        format!("{}:{}", endpoint, DEFAULT_PORT)
    }
}

/// Format Unix timestamp to human-readable date
fn format_timestamp(timestamp: u32) -> String {
    use std::time::UNIX_EPOCH;

    let duration = std::time::Duration::from_secs(timestamp as u64);
    let datetime = UNIX_EPOCH + duration;

    match datetime.duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let total_secs = d.as_secs();
            let total_days = total_secs / 86400;
            let hours = (total_secs % 86400) / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;

            // Calculate year, month, and day
            let (year, month, day) = days_since_epoch_to_date(total_days);

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                year, month, day, hours, minutes, seconds
            )
        }
        Err(_) => "Invalid timestamp".to_string(),
    }
}

/// Convert days since Unix epoch to (year, month, day)
fn days_since_epoch_to_date(mut days: u64) -> (u32, u32, u32) {
    let mut year = 1970;

    // Count full years
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    // Days in each month (non-leap year)
    let mut days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    // Adjust February for leap year
    if is_leap_year(year) {
        days_in_months[1] = 29;
    }

    // Find the month
    let mut month = 0;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if days < days_in_month as u64 {
            month = i + 1;
            break;
        }
        days -= days_in_month as u64;
    }

    let day = days + 1; // Days are 1-indexed

    (year, month as u32, day as u32)
}

/// Check if a year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists (doesn't error if missing)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::WARN
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse endpoint and add default port if not specified
    let endpoint = parse_endpoint(&cli.blockchain_endpoint);

    // Convert IP:port to URL format for gRPC
    let endpoint_url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.clone()
    } else {
        format!("http://{}", endpoint)
    };

    info!("Teranode CLI starting");
    info!("Connecting to endpoint: {}", endpoint_url);

    match cli.command {
        Commands::GetBestBlock => {
            let mut client = teranode_client::TeranodeClient::connect(&endpoint_url).await?;
            let response = client.get_best_block_header().await?;

            // Parse the block header
            let block_header = if !response.block_header.is_empty() {
                Some(BlockHeader::from_slice(&response.block_header))
            } else {
                None
            };

            // Display the block header information
            println!("Best Block Header:");
            println!("  Block ID: {}", response.id);
            println!("  Height: {}", response.height);
            println!("  Transaction Count: {}", response.tx_count);
            println!("  Size (bytes): {}", response.size_in_bytes);
            println!(
                "  Block Time: {} ({})",
                response.block_time,
                format_timestamp(response.block_time)
            );
            println!(
                "  Timestamp: {} ({})",
                response.timestamp,
                format_timestamp(response.timestamp)
            );
            println!("  Miner: {}", response.miner);
            println!("  Peer ID: {}", response.peer_id);
            println!("  Mined Set: {}", response.mined_set);
            println!("  Subtrees Set: {}", response.subtrees_set);
            println!("  Invalid: {}", response.invalid);
            // Display chain work
            if !response.chain_work.is_empty() {
                let chain_work_hex = hex::encode(&response.chain_work);
                println!("  Chain Work: 0x{}", chain_work_hex);
            }

            if let Some(processed_at) = response.processed_at {
                println!("  Processed At: {:?}", processed_at);
            }

            // Display parsed BlockHeader
            if let Some(header) = block_header {
                println!("\nParsed Block Header:");
                println!("  Block Hash: {}", header.hash());
                println!("  Version: {}", header.version());
                println!("  Previous Block Hash: {}", header.prev_hash());
                println!("  Merkle Root: {}", header.merkle_root());
                println!(
                    "  Time: {} ({})",
                    header.timestamp(),
                    format_timestamp(header.timestamp())
                );
                println!("  Bits: 0x{:08x}", header.bits());
                println!("  Nonce: {}", header.nonce());
            }
        }
    }

    Ok(())
}
