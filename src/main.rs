use anyhow::Result;
use mimalloc::MiMalloc;
use tracing::{info, warn, error, Level};
use tracing_subscriber::fmt::time::ChronoUtc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod core;
mod models;
mod utils;

use crate::core::Indexer;

fn init_logging() -> Result<()> {
    // Start with basic logging first
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Let's write directly to stdout for testing
    println!("Logging initialized");
    println!("CONFIG_PATH={}", std::env::var("CONFIG_PATH").unwrap_or_default());
    
    info!(event = "startup", message = "Starting eth-indexer");
    
    Ok(())
}

#[tokio::main]
async fn main() {
    // Use println! first to ensure we can at least see basic output
    println!("Starting up...");

    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Initialize logging
    match init_logging() {
        Ok(_) => println!("Logging initialized successfully"),
        Err(e) => println!("Failed to initialize logging: {:?}", e),
    }
    
    // Load configuration
    println!("Loading config...");
    let config = config::Config::from_env()?;
    println!("Config loaded: {:?}", config);
    
    // Create and run indexer
    println!("Creating indexer...");
    let mut indexer = Indexer::new(config).await?;
    println!("Running indexer...");
    indexer.run().await?;
    
    Ok(())
}