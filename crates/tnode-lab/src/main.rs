//! Teranode CLI - Command-line tool for interacting with Teranode instances

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "tnode")]
#[command(about = "CLI tool for BSV Teranode experiments", long_about = None)]
struct Cli {
    /// Teranode endpoint URL
    #[arg(short, long, default_value = "http://localhost:50051")]
    endpoint: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query Teranode information
    Query {
        /// Type of query to perform
        #[arg(value_name = "QUERY_TYPE")]
        query_type: String,
    },

    /// Run tests against Teranode
    Test {
        /// Test suite to run
        #[arg(value_name = "TEST_SUITE")]
        test_suite: Option<String>,
    },

    /// Connect to Teranode and verify connectivity
    Ping,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Teranode CLI starting");
    info!("Connecting to endpoint: {}", cli.endpoint);

    match cli.command {
        Commands::Query { query_type } => {
            info!("Running query: {}", query_type);
            // Implementation will be added once proto definitions are available
            println!("Query functionality will be implemented with proto definitions");
        }
        Commands::Test { test_suite } => {
            let suite = test_suite.unwrap_or_else(|| "default".to_string());
            info!("Running test suite: {}", suite);
            println!("Test functionality will be implemented with proto definitions");
        }
        Commands::Ping => {
            info!("Pinging Teranode instance");
            // let client = teranode_client::TeranodeClient::connect(&cli.endpoint).await?;
            println!("Ping functionality will be implemented with proto definitions");
        }
    }

    Ok(())
}
