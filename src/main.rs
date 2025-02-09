use anyhow::Result;
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod core;
mod models;
mod utils;

use crate::core::Indexer;

fn init_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!(
        event = "startup",
        message = "Starting eth-indexer",
        version = env!("CARGO_PKG_VERSION"),
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first thing
    init_logging()?;
    
    // Log config loading attempt
    info!(event = "config_loading", message = "Loading configuration");
    
    // Load configuration
    let config = config::Config::from_env()?;
    info!(
        event = "config_loaded",
        message = "Configuration loaded successfully",
        config = ?config,
    );
    
    // Create and run indexer
    let indexer = Indexer::new(config).await?;  // Removed mut as it's not needed
    indexer.run().await?;
    
    Ok(())
}